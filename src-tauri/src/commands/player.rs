//! `player_*` komutları — oynatıcının arayüze açılan yüzü.
//!
//! Hepsi ince sarmalayıcı: iş [`crate::mpv::oynatici`] içinde. Buraya mantık
//! yazılmıyor ki aynı davranış hem komuttan hem mpv kısayolundan geldiğinde
//! ikiye ayrılmasın.

use tauri::State;

use crate::hata::Sonuc;
use crate::mpv::{oynatici, MpvState, PlayerState, Track};
use crate::playlist::surucu;

/// Tek bir dosya açar.
///
/// Künye (süre, izler, başlık) DÖNMÜYOR: mpv dosyayı eşzamansız açıyor,
/// burada beklemek arayüzü dosya çözümlenene kadar dondurmak olurdu. Bilgi
/// `player://file-loaded` olayıyla geliyor (bkz. docs/IPC.md).
///
/// Açma işi [`surucu::dosyayi_ac`]'a devrediliyor: kuyruk hizalaması ve
/// oynatma sayacı orada, tek yerde. Doğrudan `oynatici::ac` çağırmak,
/// kütüphaneden açılan dosyanın "çalınmış" sayılmaması demek olurdu.
#[tauri::command]
pub fn player_open(app: tauri::AppHandle, path: String) -> Sonuc<()> {
    let oge = surucu::oge_uret(&app, &path);
    surucu::dosyayi_ac(&app, oge)
}

#[tauri::command]
pub fn player_play(durum: State<'_, MpvState>) -> Sonuc<()> {
    oynatici::oynat(&durum)
}

#[tauri::command]
pub fn player_pause(durum: State<'_, MpvState>) -> Sonuc<()> {
    oynatici::duraklat(&durum)
}

#[tauri::command]
pub fn player_toggle_pause(durum: State<'_, MpvState>) -> Sonuc<()> {
    oynatici::duraklat_degistir(&durum)
}

#[tauri::command]
pub fn player_seek(durum: State<'_, MpvState>, position: f64) -> Sonuc<()> {
    oynatici::ara(&durum, position)
}

#[tauri::command]
pub fn player_seek_relative(durum: State<'_, MpvState>, delta: f64) -> Sonuc<()> {
    oynatici::goreli_ara(&durum, delta)
}

#[tauri::command]
pub fn player_set_volume(durum: State<'_, MpvState>, volume: i64) -> Sonuc<()> {
    oynatici::ses_ayarla(&durum, volume)
}

#[tauri::command]
pub fn player_set_mute(durum: State<'_, MpvState>, muted: bool) -> Sonuc<()> {
    oynatici::sessiz_ayarla(&durum, muted)
}

#[tauri::command]
pub fn player_stop(app: tauri::AppHandle, durum: State<'_, MpvState>) -> Sonuc<()> {
    oynatici::dur(&app, &durum)
}

#[tauri::command]
pub fn player_set_playback_rate(durum: State<'_, MpvState>, rate: f64) -> Sonuc<()> {
    oynatici::hiz_ayarla(&durum, rate)
}

/// Durumun anlık kopyası. Arayüz açılışta ve pencere odağı döndüğünde
/// çağırıyor; olay kaçırmış olma ihtimaline karşı tek doğrulama noktası.
#[tauri::command]
pub fn player_get_state(durum: State<'_, MpvState>) -> PlayerState {
    durum.anlik()
}

#[tauri::command]
pub fn player_get_tracks(durum: State<'_, MpvState>) -> Sonuc<Vec<Track>> {
    oynatici::izler(&durum)
}

/// Video yüzeyinin duracağı dikdörtgen — arayüzden CSS pikseliyle geliyor.
///
/// Ölçek çarpanı burada UYGULANMIYOR ve bu bilinçli: hangi platformun
/// fiziksel piksel istediğini yüzeyin kendisi biliyor. Windows'ta HWND
/// koordinatları fiziksel, GTK3 ve AppKit'te mantıksal — çarpanı ortak yola
/// koymak, HiDPI bir Linux ya da Mac ekranında videoyu iki katı büyüklükte
/// çizmek olurdu. Ayrıntı `mpv/yuzey.rs` → "Ölçü birimi".
#[tauri::command]
pub fn player_set_video_rect(
    app: tauri::AppHandle,
    durum: State<'_, MpvState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Sonuc<()> {
    durum.yuzey.konumla(&app, x, y, width, height)
}

/// Yüzeyi gizler/gösterir. Kütüphane ızgarasına geçince gizleniyor, yoksa
/// siyah dikdörtgen ızgaranın üstünde asılı kalıyor.
///
/// `app`: Linux ve macOS'ta gizleme ana iş parçacığına gönderiliyor
/// (`mpv/yuzey.rs` → "Ana iş parçacığı").
#[tauri::command]
pub fn player_set_video_visible(
    app: tauri::AppHandle,
    durum: State<'_, MpvState>,
    visible: bool,
) -> Sonuc<()> {
    durum.yuzey.goster(&app, visible)
}

/// Sürüklenip bırakılan yolları açar.
///
/// İş [`crate::acilis::yollari_ac`] içinde: aynı karara açılışta komut
/// satırından ve uygulama açıkken çift tıklanan ikinci dosyadan da
/// geliniyor, üçü tek yerde kalsın diye.
#[tauri::command]
pub fn app_open_paths(app: tauri::AppHandle, paths: Vec<String>) -> Sonuc<()> {
    crate::acilis::yollari_ac(&app, paths)
}
