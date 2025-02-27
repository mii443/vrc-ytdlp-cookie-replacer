**注意: 本ソフトを使用すると、GoogleやVRChatのアカウントがbanされる危険性があります。**

YoutubeからのIPブロックにより、VRChat内で使用しているyt-dlpから[動画のURLを取得できない問題](https://github.com/yt-dlp/yt-dlp/issues/12475)を修正するソフトです。
本ソフトは以下の挙動をします。
1. VRChat標準の `yt-dlp.exe` を置き換える。
2. 新しい `yt-dlp.exe` をダウンロードし、`yt-dlp-original.exe` へ保存する。（VRChat標準の `yt-dlp.exe` がCookieに対応していないため）
3. VRChatが `yt-dlp.exe` を呼び出した際に、`--cookies-from-browser firefox --cookies C:\Users\<USERNAME>\AppData\LocalLow\VRChat\VRChat\Tools\cookies.txt` を引数へ追加し、`yt-dlp-original.exe` を呼び出す。

これによって、正しく認証されたGoogleアカウントで動画URLを取得するため、IPブロックを回避できるようになります。

# 使用方法
1. FirefoxでGoogleアカウントへログインする
2. Youtubeを開く
3. 本ソフトを `cargo build --release` でビルドする
4. ビルド済みの `yt-dlp-cookie.exe` をダブルクリックする

以上で動画プレイヤーの再生が可能になります。

# 注意
- GoogleやVRChatのアカウントがbanされる危険性があります。 (https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies)
- ワールドにjoinした際に、置き換えたyt-dlp.exeが修復されるようです。動画プレイヤーが再生できなくなった場合は、再び手順 4 の "ビルド済みの `yt-dlp-cookie.exe` をダブルクリックする" を再実行してください。
