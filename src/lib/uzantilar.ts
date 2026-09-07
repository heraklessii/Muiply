/**
 * Dosya seçme diyaloğundaki süzgeç.
 *
 * mpv çok daha fazlasını açıyor ve diyalogda "Bütün dosyalar" seçeneği
 * zaten var; buradaki liste kolaylık. Kaynağı Rust tarafındaki tarama
 * listesi (`library/mod.rs`) ve kurulumun işletim sistemine bildirdiği
 * uzantılar (`tauri.conf.json` > `bundle.fileAssociations`) — üçü birlikte
 * değişiyor.
 */
export const MEDYA_UZANTILARI = [
  // Video — `VIDEO_UZANTILARI` (library/mod.rs) ile aynı sıra, aynı liste.
  "mp4", "mkv", "avi", "mov", "webm", "flv", "ts", "m2ts", "wmv", "3gp", "ogv", "m4v",
  "mpg", "mpeg",
  // Ses — `SES_UZANTILARI` ile aynı.
  "mp3", "flac", "aac", "ogg", "opus", "wav", "m4a", "wma", "ape", "alac", "aiff", "mka",
];
