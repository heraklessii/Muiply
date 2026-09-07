//! `window_*` komutları — pencereleri gösteren ince sarmalayıcılar.
//!
//! İş [`crate::pencere`] içinde: aynı karara tepsi menüsünden, açılıştaki
//! komut satırından ve mpv'nin olay döngüsünden de geliniyor.

use crate::hata::Sonuc;
use crate::pencere;

/// Oynatıcı penceresini gösterir — kütüphanedeki "Oynatıcı" düğmesi.
///
/// Video yüklendiğinde pencere zaten kendiliğinden görünüyor (olay
/// döngüsü); bu komut kullanıcının açıkça istediği durum için.
#[tauri::command]
pub fn window_show_player(app: tauri::AppHandle) -> Sonuc<()> {
    pencere::goster(&app, pencere::OYNATICI);
    Ok(())
}

/// Kütüphane penceresini gösterir — oynatıcıdaki "Kütüphane" düğmesi.
#[tauri::command]
pub fn window_show_library(app: tauri::AppHandle) -> Sonuc<()> {
    pencere::goster(&app, pencere::KUTUPHANE);
    Ok(())
}
