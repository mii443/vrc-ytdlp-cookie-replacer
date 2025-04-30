use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use sha2::Digest;

#[derive(Debug)]
enum AppError {
    IoError(io::Error),
    EnvError(env::VarError),
    NotifyError(notify::Error),
    ExeError(String),
    CommandError(String),
    HashError(String),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<env::VarError> for AppError {
    fn from(err: env::VarError) -> Self {
        AppError::EnvError(err)
    }
}

impl From<notify::Error> for AppError {
    fn from(err: notify::Error) -> Self {
        AppError::NotifyError(err)
    }
}

type Result<T> = std::result::Result<T, AppError>;

fn get_localappdata_low() -> Option<PathBuf> {
    if let Ok(user_profile) = env::var("USERPROFILE") {
        let localappdata_low = Path::new(&user_profile).join("AppData").join("LocalLow");
        if localappdata_low.exists() {
            return Some(localappdata_low);
        }
    }

    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let path = Path::new(&local_appdata);
        if let Some(appdata) = path.parent() {
            let localappdata_low = appdata.join("LocalLow");
            if localappdata_low.exists() {
                return Some(localappdata_low);
            }
        }
    }

    None
}

fn get_current_exe() -> Result<PathBuf> {
    const MAX_RETRIES: usize = 3;
    let mut last_error = None;

    for _ in 0..MAX_RETRIES {
        match env::current_exe() {
            Ok(path) => return Ok(path),
            Err(err) => {
                last_error = Some(err);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    Err(AppError::ExeError(format!(
        "現在の実行可能ファイルを取得できませんでした: {:?}",
        last_error.unwrap()
    )))
}

fn copy_with_retry(
    src: &Path,
    dst: &Path,
    max_retries: usize,
    retry_delay: Duration,
) -> Result<()> {
    let mut retries = 0;
    loop {
        match fs::copy(src, dst) {
            Ok(_) => return Ok(()),
            Err(err) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(AppError::IoError(err));
                }
                eprintln!(
                    "コピー失敗 (リトライ {}/{}): {} -> {}: {}",
                    retries,
                    max_retries,
                    src.display(),
                    dst.display(),
                    err
                );
                thread::sleep(retry_delay);
            }
        }
    }
}

fn copy_self_to_vrchat_tools_dir(yt_dlp_path: &Path) -> Result<()> {
    let src = get_current_exe()?;
    copy_with_retry(&src, yt_dlp_path, 10, Duration::from_millis(100))
}

fn download_yt_dlp(yt_dlp_original_path: &Path) -> Result<()> {
    if yt_dlp_original_path.exists() {
        let _ = fs::remove_file(yt_dlp_original_path);
    }

    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

    for attempt in 1..=3 {
        let status = Command::new("curl")
            .arg("-L")
            .arg("-o")
            .arg(yt_dlp_original_path)
            .arg(url)
            .status();

        match status {
            Ok(exit_status) if exit_status.success() => return Ok(()),
            Ok(exit_status) => {
                if attempt < 3 {
                    eprintln!(
                        "ダウンロード失敗 (リトライ {}/3): exit code: {:?}",
                        attempt,
                        exit_status.code()
                    );
                    thread::sleep(Duration::from_secs(1));
                } else {
                    return Err(AppError::CommandError(format!(
                        "curlコマンドが失敗しました: {:?}",
                        exit_status
                    )));
                }
            }
            Err(err) => {
                if attempt < 3 {
                    eprintln!("ダウンロード失敗 (リトライ {}/3): {}", attempt, err);
                    thread::sleep(Duration::from_secs(1));
                } else {
                    return Err(AppError::IoError(err));
                }
            }
        }
    }

    unreachable!("リトライループから抜けられないはず");
}

fn set_integrity_level(path: &Path) -> Result<()> {
    for attempt in 1..=3 {
        let status = Command::new("icacls")
            .arg(path)
            .arg("/setintegritylevel")
            .arg("medium")
            .status();

        match status {
            Ok(exit_status) if exit_status.success() => return Ok(()),
            Ok(exit_status) => {
                if attempt < 3 {
                    eprintln!(
                        "整合性レベル設定失敗 (リトライ {}/3): exit code: {:?}",
                        attempt,
                        exit_status.code()
                    );
                    thread::sleep(Duration::from_secs(1));
                } else {
                    return Err(AppError::CommandError(format!(
                        "icaclsコマンドが失敗しました: {:?}",
                        exit_status
                    )));
                }
            }
            Err(err) => {
                if attempt < 3 {
                    eprintln!("整合性レベル設定失敗 (リトライ {}/3): {}", attempt, err);
                    thread::sleep(Duration::from_secs(1));
                } else {
                    return Err(AppError::IoError(err));
                }
            }
        }
    }

    unreachable!("リトライループから抜けられないはず");
}

fn replace(yt_dlp_original_path: &Path, yt_dlp_path: &Path, bypass_exists: bool) -> Result<()> {
    if !yt_dlp_original_path.exists() || bypass_exists {
        download_yt_dlp(yt_dlp_original_path)?;
    }

    copy_self_to_vrchat_tools_dir(yt_dlp_path)?;
    set_integrity_level(yt_dlp_original_path)?;

    Ok(())
}

fn get_hash(path: &Path) -> Result<String> {
    let mut hasher = sha2::Sha256::new();

    let file = fs::File::options()
        .append(false)
        .create(false)
        .truncate(false)
        .create_new(false)
        .read(true)
        .open(path);

    let mut file = match file {
        Ok(f) => f,
        Err(err) => return Err(AppError::IoError(err)),
    };

    match io::copy(&mut file, &mut hasher) {
        Ok(_) => Ok(format!("{:x}", hasher.finalize())),
        Err(err) => Err(AppError::HashError(format!("ハッシュ計算エラー: {}", err))),
    }
}

fn watch_loop(yt_dlp_original_path: &Path, yt_dlp_path: &Path) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let watcher = notify::recommended_watcher(tx);
    let mut watcher = match watcher {
        Ok(w) => w,
        Err(err) => return Err(AppError::NotifyError(err)),
    };

    let parent = match yt_dlp_path.parent() {
        Some(p) => p,
        None => {
            return Err(AppError::ExeError(
                "yt-dlpのパスに親ディレクトリがありません".to_string(),
            ));
        }
    };

    match watcher.watch(parent, RecursiveMode::NonRecursive) {
        Ok(_) => (),
        Err(err) => return Err(AppError::NotifyError(err)),
    }

    println!("ファイル監視を開始しました。VRChatの実行中はこのウィンドウを閉じないでください。");

    for res in rx {
        match res {
            Ok(_) => {
                thread::sleep(Duration::from_millis(300));

                let yt_dlp_hash = match get_hash(yt_dlp_path) {
                    Ok(hash) => hash,
                    Err(err) => {
                        eprintln!("yt-dlpのハッシュ計算に失敗しました: {:?}", err);
                        continue;
                    }
                };

                let self_exe = match get_current_exe() {
                    Ok(exe) => exe,
                    Err(err) => {
                        eprintln!("実行ファイルのパス取得に失敗: {:?}", err);
                        continue;
                    }
                };

                let self_hash = match get_hash(&self_exe) {
                    Ok(hash) => hash,
                    Err(err) => {
                        eprintln!("自身のハッシュ計算に失敗しました: {:?}", err);
                        continue;
                    }
                };

                if yt_dlp_hash != self_hash {
                    println!("\nyt-dlpが変更されました。置き換えます。");
                    match replace(yt_dlp_original_path, yt_dlp_path, false) {
                        Ok(_) => println!("置き換えが完了しました。"),
                        Err(err) => eprintln!("置き換え失敗: {:?}", err),
                    }
                }
            }
            Err(e) => eprintln!("監視エラー: {:?}", e),
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    let local_appdata_low = get_localappdata_low();
    let args: Vec<String> = env::args().skip(1).collect();

    if local_appdata_low.is_none() {
        eprintln!("LocalLow フォルダが見つかりませんでした");
        if args.is_empty() {
            let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
        }
        return ExitCode::from(1);
    }

    let local_appdata_low = local_appdata_low.unwrap();

    let yt_dlp_original_path = local_appdata_low
        .join("VRChat")
        .join("VRChat")
        .join("Tools")
        .join("yt-dlp-original.exe");
    let yt_dlp_path = local_appdata_low
        .join("VRChat")
        .join("VRChat")
        .join("Tools")
        .join("yt-dlp.exe");
    let cookie_file = local_appdata_low
        .join("VRChat")
        .join("VRChat")
        .join("Tools")
        .join("cookies.txt");
    let latest_args = local_appdata_low
        .join("VRChat")
        .join("VRChat")
        .join("Tools")
        .join("latest_args.txt");

    let tools_dir = yt_dlp_path.parent().unwrap_or(Path::new(""));
    if !tools_dir.exists() {
        if let Err(e) = fs::create_dir_all(tools_dir) {
            eprintln!("Tools ディレクトリの作成に失敗: {}", e);
            return ExitCode::from(1);
        }
    }

    let args_str = args.join(" ");
    if let Err(e) = fs::write(&latest_args, &args_str) {
        eprintln!(
            "引数の書き込みに失敗しました {}: {}",
            latest_args.to_string_lossy(),
            e
        );
    }

    if args.is_empty() {
        println!("置き換え用 yt-dlp.exe をダウンロードしています。");
        match replace(&yt_dlp_original_path, &yt_dlp_path, true) {
            Ok(_) => println!(
                "置き換えが完了しました。自動置き換えをするため、ソフトを起動したままにしてください。"
            ),
            Err(e) => {
                eprintln!("置き換えに失敗しました: {:?}", e);
                let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
                return ExitCode::from(1);
            }
        }

        match watch_loop(&yt_dlp_original_path, &yt_dlp_path) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("監視ループエラー: {:?}", e);
                let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
                return ExitCode::from(1);
            }
        }
    }

    let cookie_args = vec![
        "--cookies-from-browser".to_string(),
        "firefox".to_string(),
        "--cookies".to_string(),
        cookie_file.to_str().unwrap_or_default().to_string(),
    ];

    let status = match Command::new(&yt_dlp_original_path)
        .args(&args)
        .args(&cookie_args)
        .status()
    {
        Ok(status) => status,
        Err(e) => {
            eprintln!(
                "実行に失敗しました {}: {}",
                yt_dlp_original_path.to_string_lossy(),
                e
            );
            return ExitCode::from(1);
        }
    };

    match status.code() {
        Some(code) => {
            if code > 255 {
                ExitCode::from(255)
            } else if code < 0 {
                ExitCode::from(1)
            } else {
                ExitCode::from(code as u8)
            }
        }
        None => ExitCode::from(2),
    }
}
