use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

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

fn copy_self_to_vrchat_tools_dir(yt_dlp_path: &Path) {
    let src = env::current_exe().unwrap();
    std::fs::copy(src, yt_dlp_path).unwrap();
}

fn download_yt_dlp(yt_dlp_original_path: &Path) {
    if yt_dlp_original_path.exists() {
        std::fs::remove_file(yt_dlp_original_path).unwrap();
    }

    let url = "https://github.com/yt-dlp/yt-dlp/releases/download/2025.02.19/yt-dlp_x86.exe";
    let _ = Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg(yt_dlp_original_path)
        .arg(url)
        .status()
        .unwrap();
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

    let yt_dlp_original_path = local_appdata_low.join("VRChat").join("VRChat").join("Tools").join("yt-dlp-original.exe");
    let yt_dlp_path = local_appdata_low.join("VRChat").join("VRChat").join("Tools").join("yt-dlp.exe");
    let cookie_file = local_appdata_low.join("VRChat").join("VRChat").join("Tools").join("cookies.txt");
    
    if args.is_empty() {
        download_yt_dlp(&yt_dlp_original_path);
        copy_self_to_vrchat_tools_dir(&yt_dlp_path);

        println!("\nyt-dlpの置き換えに成功しました。");

        let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
        return ExitCode::from(0);
    }
    
    let cookie_args = vec!["--cookies-from-browser".to_string(), "firefox".to_string(), "--cookies".to_string(), cookie_file.to_str().unwrap().to_string()];

    let status = match Command::new(&yt_dlp_original_path)
        .args(&args)
        .args(&cookie_args)
        .status() {
            Ok(status) => status,
            Err(e) => {
                eprintln!("実行に失敗しました {}: {}", yt_dlp_original_path.to_string_lossy(), e);
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
        },
        None => ExitCode::from(2),
    }
}
