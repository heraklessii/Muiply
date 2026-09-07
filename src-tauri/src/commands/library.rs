//! `library_*` komutları.

use tauri::State;

use crate::hata::Sonuc;
use crate::library::db::kilit;
use crate::library::{tarayici, LibraryState, MediaFilter, MediaItem};

/// Klasörü izlenenlere ekler ve taramayı başlatır.
///
/// Tarama ayrı bir komut olarak da var ama ekledikten sonra kendiliğinden
/// başlamalı: "klasör ekle" deyip boş bir kütüphane görmek, kullanıcıya bir
/// şeyin bozulduğunu düşündürür.
#[tauri::command]
pub fn library_add_folder(
    app: tauri::AppHandle,
    durum: State<'_, LibraryState>,
    path: String,
) -> Sonuc<()> {
    kilit(&durum)?.klasor_ekle(&path)?;
    tarayici::baslat(app)
}

#[tauri::command]
pub fn library_remove_folder(durum: State<'_, LibraryState>, path: String) -> Sonuc<()> {
    kilit(&durum)?.klasor_sil(&path)
}

#[tauri::command]
pub fn library_get_folders(durum: State<'_, LibraryState>) -> Sonuc<Vec<String>> {
    kilit(&durum)?.klasorler()
}

#[tauri::command]
pub fn library_scan(app: tauri::AppHandle) -> Sonuc<()> {
    tarayici::baslat(app)
}

/// Kütüphanedeki kayıtlar.
///
/// `filter` isteğe bağlı: arayüz ilk açılışta süzgeçsiz çağırıyor.
#[tauri::command]
pub fn library_get_media(
    durum: State<'_, LibraryState>,
    filter: Option<MediaFilter>,
) -> Sonuc<Vec<MediaItem>> {
    kilit(&durum)?.medya(&filter.unwrap_or_default())
}

#[tauri::command]
pub fn library_get_recent(durum: State<'_, LibraryState>, limit: i64) -> Sonuc<Vec<MediaItem>> {
    kilit(&durum)?.son_calinanlar(limit.clamp(1, 200))
}

/// Kaydı kütüphaneden siler. **Dosyayı silmez.**
///
/// Muiply bir dosya yöneticisi değil; diskten silmek geri alınamaz ve bu
/// uygulamanın işi değil. Arayüzdeki metin de "Kütüphaneden kaldır" diyor.
#[tauri::command]
pub fn library_delete_media(durum: State<'_, LibraryState>, id: String) -> Sonuc<()> {
    kilit(&durum)?.sil(&id)
}

#[tauri::command]
pub fn library_get_item(durum: State<'_, LibraryState>, id: String) -> Sonuc<Option<MediaItem>> {
    kilit(&durum)?.medya_bir(&id)
}
