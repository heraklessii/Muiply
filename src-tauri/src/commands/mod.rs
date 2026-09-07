//! Tauri komutları — arayüzün backend'e açılan tek kapısı.
//!
//! Her dosya bir alan: oynatıcı, kütüphane, çalma listesi, altyazı. Komut
//! adları ve imzaları `docs/IPC.md` ile birebir aynı olmak zorunda; orası
//! sözleşme, burası uygulaması.
//!
//! Kural: buraya iş mantığı yazılmıyor. Bir komut ya tek bir modül
//! fonksiyonunu çağırıyor ya da birkaçını sırayla; karar veren kod
//! modüllerde, çünkü aynı karara mpv'nin olay döngüsünden de gelinebiliyor.

pub mod library;
pub mod pencere;
pub mod player;
pub mod playlist;
pub mod settings;
pub mod subtitle;
