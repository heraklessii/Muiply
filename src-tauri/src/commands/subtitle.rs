//! `subtitle_*` komutları.
//!
//! Gömülü izler ile harici dosyalar arayüzde TEK bir listede görünüyor:
//! kullanıcı için ikisi de "altyazı". Ayrım yalnızca `Track.external`
//! alanında ve orada da sadece küçük bir rozet olarak görünüyor.

use tauri::State;

use crate::hata::Sonuc;
use crate::mpv::{oynatici, MpvState, Track};
use crate::subtitle::{self, NearbySubtitle};

/// Yalnız altyazı izleri.
#[tauri::command]
pub fn subtitle_get_tracks(durum: State<'_, MpvState>) -> Sonuc<Vec<Track>> {
    Ok(oynatici::izler(&durum)?
        .into_iter()
        .filter(|i| i.kind == "sub")
        .collect())
}

/// İzi seçer. `None` altyazıyı kapatıyor.
///
/// mpv'de kapatmanın karşılığı `sid=no` — sayı değil metin. İki ayrı çağrı
/// olmasının sebebi bu; `sid=0` "sıfır numaralı iz" demek değil, mpv'de iz
/// numaraları 1'den başlıyor.
#[tauri::command]
pub fn subtitle_select(durum: State<'_, MpvState>, track_id: Option<i64>) -> Sonuc<()> {
    match track_id {
        Some(id) => durum.motor.ayar_sayi("sid", id),
        None => durum.motor.ayar_metin("sid", "no"),
    }
}

/// Kullanıcının elle seçtiği altyazı dosyasını ekler ve gösterir.
#[tauri::command]
pub fn subtitle_add_file(durum: State<'_, MpvState>, path: String) -> Sonuc<()> {
    subtitle::elle_ekle(&durum.motor, &path)
}

/// Altyazı gecikmesi (saniye). Artı değer altyazıyı geciktiriyor.
#[tauri::command]
pub fn subtitle_set_delay(durum: State<'_, MpvState>, seconds: f64) -> Sonuc<()> {
    durum.motor.ayar_ondalik("sub-delay", seconds)
}

#[tauri::command]
pub fn subtitle_get_delay(durum: State<'_, MpvState>) -> Sonuc<f64> {
    durum.motor.oku_ondalik("sub-delay")
}

/// Bir video dosyasının yanındaki altyazı dosyaları.
///
/// Oynatma sırasında otomatik olarak zaten yükleniyor; bu komut arayüzün
/// "yanında şunlar var" listesini gösterebilmesi için — dosya açılmadan da
/// sorulabiliyor.
#[tauri::command]
pub fn subtitle_find_nearby(path: String) -> Vec<NearbySubtitle> {
    subtitle::yanindakiler(std::path::Path::new(&path))
}
