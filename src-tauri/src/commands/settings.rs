//! `settings_*` komutları.
//!
//! İnce sarmalayıcı: düzeltme, yazma ve mpv'ye uygulama [`crate::settings`]
//! içinde. Buraya mantık yazılmıyor — aynı ayarlar açılışta da uygulanıyor
//! (`lib.rs`) ve iki kopya birbirinden sessizce ayrışırdı.

use tauri::State;

use crate::hata::Sonuc;
use crate::library::db::kilit;
use crate::library::LibraryState;
use crate::mpv::MpvState;
use crate::settings::{self, AudioDevice, Settings, SettingsState};

/// Ayarların anlık kopyası. `Result` değil: kopya bellekte, okumak başarısız
/// olamaz (bkz. `docs/IPC.md`).
#[tauri::command]
pub fn settings_get(durum: State<'_, SettingsState>) -> Settings {
    durum.anlik()
}

/// Ayarları yazar, mpv'ye uygular ve DÜZELTİLMİŞ hâlini döndürür.
///
/// Sıra önemli: önce diske, sonra mpv'ye, en son belleğe. mpv'ye yazmak
/// başarısız olursa (motorsuz derleme dışında: geçersiz bir aygıt adı)
/// kullanıcı hatayı görüyor ama tercihi kayıtlı kalıyor — bir sonraki
/// açılışta yeniden denenecek, ki aygıt o zaman takılı olabilir.
#[tauri::command]
pub fn settings_set(
    app: tauri::AppHandle,
    ayar_durum: State<'_, SettingsState>,
    kutuphane: State<'_, LibraryState>,
    oynatici: State<'_, MpvState>,
    settings: Settings,
) -> Sonuc<Settings> {
    let duzeltilmis = settings.duzelt();

    let db = kilit(&kutuphane)?;
    settings::yaz(&db, &duzeltilmis)?;
    drop(db);

    ayar_durum.yerlestir(duzeltilmis.clone());

    // Medya tuşları mpv'nin değil işletim sisteminin işi, o yüzden
    // `settings::uygula`dan AYRI. mpv yazımından ÖNCE çünkü kayıt hata
    // dönmüyor: `uygula` bir aygıt adı yüzünden düşse bile tuş tercihinin
    // yürümemesi için sebep yok.
    crate::tepsi::medya_tuslari::uygula(&app, duzeltilmis.media_keys);

    settings::uygula(&oynatici.motor, &duzeltilmis)?;

    Ok(duzeltilmis)
}

/// mpv'nin bildiği ses çıkışları. Motorsuz derlemede boş liste.
#[tauri::command]
pub fn settings_audio_devices(durum: State<'_, MpvState>) -> Vec<AudioDevice> {
    settings::ses_aygitlari(&durum.motor)
}
