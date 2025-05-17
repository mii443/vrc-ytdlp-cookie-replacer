# **注意: 本ソフトを使用すると、GoogleやVRChatのアカウントがbanされる危険性があります。このソフトが何をするか理解している人のみ使用してください。**

## 挙動
YoutubeからのIPブロックにより、VRChat内で使用しているyt-dlpから[動画のURLを取得できない問題](https://github.com/yt-dlp/yt-dlp/issues/12475)を修正するソフトです。
本ソフトは以下の挙動をします。
1. VRChat標準の `yt-dlp.exe` を置き換える。
2. 新しい `yt-dlp.exe` をダウンロードし、`yt-dlp-original.exe` へ保存する。（VRChat標準の `yt-dlp.exe` がCookieに対応していないため）
3. `yt-dlp.exe` の整合性レベルを `Medium` へ変更する (https://github.com/yt-dlp/yt-dlp/issues/12812 への対応)
4. VRChatが `yt-dlp.exe` を呼び出した際に、`--cookies-from-browser firefox --cookies C:\Users\<USERNAME>\AppData\LocalLow\VRChat\VRChat\Tools\cookies.txt` を引数へ追加し、`yt-dlp-original.exe` を呼び出す。

これによって、正しく認証されたGoogleアカウントで動画URLを取得するため、IPブロックを回避できるようになります。

# 使用方法
1. Firefoxで**banされてもいい**Googleアカウントへログインする
2. Youtubeを開く
3. 本ソフトを `cargo build --release` でビルドする
4. ビルド済みの `yt-dlp-cookie.exe` をダブルクリックする
5. VRChat起動中は常に `yt-dlp-cookie.exe` を起動しておく

以上で動画プレイヤーの再生が可能になります。

# ダウンロード
各自ビルドしてください。

# 注意
- **GoogleやVRChatのアカウントがbanされる危険性があります。** (https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies)
- ワールドにjoinした際に、置き換えたyt-dlp.exeが修復されるようです。本ソフトは修復を検知し、対策済みyt-dlp.exeへ自動的に置き換えます。そのため、VRChat起動中は本ソフトを常に起動しておく必要があります。
