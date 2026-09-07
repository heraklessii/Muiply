//! Sistem tepsisi ve medya tuşları — pencere kapalıyken de duran denetimler.
//!
//! İkisi aynı dosyada çünkü aynı işi yapıyorlar: **arayüz olmadan** oynatmayı
//! yönetmek. Muiply müzik çalarken pencere küçültülmüş oluyor ve o hâlde
//! "sonrakine geç" demenin iki yolu var — tepsi menüsü ve klavyenin medya
//! tuşu. İkisi de aynı üç eyleme iniyor, bir kez yazılıyorlar.
//!
//! # Eylemler nereye gidiyor
//!
//! [`crate::mpv::oynatici`] ve [`crate::playlist::surucu`]'ya. Buraya oynatma
//! mantığı YAZILMIYOR (CLAUDE.md kuralı 1): "sonraki" kararını dosya bitince
//! mpv'nin olay döngüsü de veriyor ve iki kopya birbirinden sessizce ayrışır.
//!
//! # Hatalar
//!
//! Yutulmuyor ama diyaloğa da çıkmıyor: `player://error` yayınlanıyor. Tepsi
//! menüsünden gelen bir eylem sırasında pencere gizli olabilir, o zaman kimse
//! görmez — ama pencere açıksa duyuru şeridinde beliriyor. Tepsiden gelen bir
//! hata için uygulamayı durdurmak orantısız olurdu.
//!
//! # Kapatma davranışı
//!
//! Pencerenin X'i uygulamayı GERÇEKTEN kapatıyor; tepsiye inmiyor. Tepsi
//! ikonu yalnızca bir kısayol. Sebebi somut: "kapattım ama kapanmadı" bir
//! kullanıcı için hatanın kendisi gibi görünüyor ve tepsi ikonunu fark
//! etmeyen biri uygulamayı bir daha nasıl kapatacağını bilmiyor.

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::hata::{Hata, Sonuc};
use crate::mpv::{oynatici, MpvState};
use crate::playlist::surucu;

/// Tepsi ikonunun kimliği. [`baslik_yaz`] onu bu adla buluyor.
const TEPSI_ID: &str = "muiply-tepsi";

/// Menü girdilerinin kimlikleri. Sabit çünkü olay işleyicisi gelen kimliği
/// metinle karşılaştırıyor; yazım hatası sessiz bir "hiçbir şey olmadı".
const M_DEGISTIR: &str = "tepsi-degistir";
const M_ONCEKI: &str = "tepsi-onceki";
const M_SONRAKI: &str = "tepsi-sonraki";
const M_GOSTER: &str = "tepsi-goster";
const M_CIKIS: &str = "tepsi-cikis";

/// Tepsi ikonunu kurar.
///
/// İkon ana pencerenin ikonundan geliyor (`default_window_icon`): aile
/// ikonunu dördüncü bir dosyaya kopyalamak, biri değiştiğinde diğerinin
/// unutulması demekti — ikon zaten üç yerde (CLAUDE.md).
pub fn kur(app: &AppHandle) -> Sonuc<()> {
    let degistir = MenuItem::with_id(app, M_DEGISTIR, "Oynat / Duraklat", true, None::<&str>)?;
    let onceki = MenuItem::with_id(app, M_ONCEKI, "Önceki", true, None::<&str>)?;
    let sonraki = MenuItem::with_id(app, M_SONRAKI, "Sonraki", true, None::<&str>)?;
    let goster = MenuItem::with_id(app, M_GOSTER, "Pencereyi göster", true, None::<&str>)?;
    let cikis = MenuItem::with_id(app, M_CIKIS, "Çıkış", true, None::<&str>)?;
    let ayrac = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[&degistir, &onceki, &sonraki, &ayrac, &goster, &cikis],
    )?;

    let ikon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| Hata::yeni("uygulama ikonu bulunamadı"))?;

    TrayIconBuilder::with_id(TEPSI_ID)
        .icon(ikon)
        .tooltip("Muiply")
        .menu(&menu)
        // Sol tık menüyü AÇMIYOR: sol tık pencereyi getiriyor (aşağıda),
        // menü sağ tıkta. Windows'ta beklenen davranış bu; ikisini birden
        // sol tıka bağlamak pencereyi getirmeyi imkânsız kılardı.
        .show_menu_on_left_click(false)
        .on_menu_event(menu_olayi)
        .on_tray_icon_event(|tepsi, olay| {
            // Yalnız sol tuşun BIRAKILMASI. Basılma da ayrı bir olay;
            // ikisini de dinlemek pencereyi tek tıkta iki kez getirirdi.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = olay
            {
                pencereyi_getir(tepsi.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Tepsi ipucunu çalan dosyanın adıyla günceller.
///
/// mpv'nin olay döngüsünden çağrılıyor (`mpv/gercek.rs` → `dosya_yuklendi`).
/// Sebebi: küçültülmüş bir oynatıcıda "şu an ne çalıyor" sorusunun tek cevabı
/// bu ipucu.
///
/// Hata yutuluyor: tepsi hiç kurulamamış olabilir (Linux'ta appindicator
/// yokken) ve ipucu yazılamadı diye dosya açılışını bozmanın karşılığı yok.
pub fn baslik_yaz(app: &AppHandle, baslik: Option<&str>) {
    let Some(tepsi) = app.tray_by_id(TEPSI_ID) else {
        return;
    };
    // Ad başa değil SONA ekleniyor ("Muiply — X"): tepsi ipuçları kısa
    // kesiliyor ve baştaki uygulama adı hangi uygulama olduğunu söylüyor.
    let metin = match baslik {
        Some(b) if !b.trim().is_empty() => format!("Muiply — {b}"),
        _ => "Muiply".to_string(),
    };
    let _ = tepsi.set_tooltip(Some(&metin));
}

fn menu_olayi(app: &AppHandle, olay: MenuEvent) {
    match olay.id().as_ref() {
        M_DEGISTIR => bildir(app, duraklat_degistir(app)),
        M_ONCEKI => bildir(app, surucu::gerile(app).map(|_| ())),
        M_SONRAKI => bildir(app, surucu::ilerle(app, false).map(|_| ())),
        M_GOSTER => pencereyi_getir(app),
        // `app.exit`, pencereyi kapatmak değil: tepsiden "Çıkış" demek pencere
        // hiç açılmamışken de kapatabilmeli.
        //
        // Kaldığı yer ÖNCE yazılıyor. `app.exit` pencere olayı üretmiyor,
        // yani `lib.rs`teki `CloseRequested` yolu buradan geçmiyordu:
        // tepsiden çıkan kullanıcı, X'e basandan beş saniyeye kadar geride
        // kalıyordu.
        M_CIKIS => {
            crate::library::devam::simdiki_konumu_kaydet(app);
            app.exit(0);
        }
        _ => {}
    }
}

fn duraklat_degistir(app: &AppHandle) -> Sonuc<()> {
    let durum = app
        .try_state::<MpvState>()
        .ok_or_else(|| Hata::yeni("oynatıcı hazır değil"))?;
    oynatici::duraklat_degistir(&durum)
}

/// Tepsiden çağrılan "göster": OYNATICI penceresi.
///
/// Kütüphane değil, çünkü tepsi ikonu çalan şeyin kısayolu — ipucunda yazan
/// da o. Kütüphaneye oynatıcıdaki düğmeden geçiliyor.
fn pencereyi_getir(app: &AppHandle) {
    crate::pencere::goster(app, crate::pencere::OYNATICI);
}

/// Hatayı arayüze taşır. Bkz. modül başlığı — diyalog yok, olay var.
fn bildir(app: &AppHandle, sonuc: Sonuc<()>) {
    if let Err(e) = sonuc {
        let _ = app.emit(crate::mpv::OLAY_HATA, e.to_string());
    }
}

pub mod medya_tuslari {
    //! Klavyenin Play/Pause · Next · Prev · Stop tuşları.
    //!
    //! **Kayıt GLOBAL.** Tuş, Muiply odakta olmasa da yakalanıyor — medya
    //! tuşunun anlamı zaten bu. Bedeli de var: aynı tuşları isteyen başka bir
    //! oynatıcı (Spotify, tarayıcı) varken işletim sistemi tuşu tek bir
    //! uygulamaya veriyor. Bu yüzden davranış bir AYARA bağlı
    //! ([`crate::settings::Settings`] → `media_keys`) ve kapatılabiliyor;
    //! açık/kapalı geçişi [`uygula`] ile anında yürüyor, yeniden başlatma
    //! gerekmiyor.
    //!
    //! Kayıt başarısızlığı hata DEĞİL: tuşu başka bir uygulama almışsa
    //! işletim sistemi bizi reddediyor ve bu, kullanıcının Muiply içinde
    //! düzeltebileceği bir şey değil. Sessizce geçiliyor, uygulama açılıyor.

    use tauri::{AppHandle, Manager};
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

    use crate::hata::{Hata, Sonuc};
    use crate::mpv::{oynatici, MpvState};
    use crate::playlist::surucu;

    /// Dinlenen tuşlar. Değiştiricisiz (`None`): medya tuşlarının Shift'lisi
    /// yok.
    const TUSLAR: &[Code] = &[
        Code::MediaPlayPause,
        Code::MediaTrackNext,
        Code::MediaTrackPrevious,
        Code::MediaStop,
    ];

    /// Eklentiyi kurar. Tuşları KAYDETMİYOR — onu [`uygula`] yapıyor, çünkü
    /// kayıt ayara bağlı ve ayar sonradan da değişebiliyor.
    pub fn eklentiyi_kur(app: &AppHandle) -> Sonuc<()> {
        app.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, kisayol, olay| {
                    // Yalnız BASILMA. Bırakma da bir olay ve ikisini birden
                    // işlemek her tuşu iki kez çalıştırırdı.
                    if olay.state() != ShortcutState::Pressed {
                        return;
                    }
                    basildi(app, kisayol);
                })
                .build(),
        )?;
        Ok(())
    }

    /// Ayarın söylediğini yapar: açıksa kaydeder, kapalıysa kaydı kaldırır.
    ///
    /// İki yön de gerekiyor. Ayarı kapatan kullanıcıda tuşlar Muiply'de
    /// kalmaya devam ederse, "kapattım ama hâlâ öbür oynatıcı çalışmıyor" ile
    /// karşılaşırdı.
    pub fn uygula(app: &AppHandle, acik: bool) {
        let k = app.global_shortcut();
        for kod in TUSLAR {
            let kisayol = Shortcut::new(None, *kod);
            if acik {
                let _ = k.register(kisayol);
            } else {
                let _ = k.unregister(kisayol);
            }
        }
    }

    fn basildi(app: &AppHandle, kisayol: &Shortcut) {
        let sonuc = match kisayol.key {
            Code::MediaPlayPause => oynatici_ile(app, oynatici::duraklat_degistir),
            Code::MediaStop => oynatici_ile(app, |d| oynatici::dur(app, d)),
            Code::MediaTrackNext => surucu::ilerle(app, false).map(|_| ()),
            Code::MediaTrackPrevious => surucu::gerile(app).map(|_| ()),
            _ => return,
        };
        super::bildir(app, sonuc);
    }

    fn oynatici_ile(app: &AppHandle, is: impl FnOnce(&MpvState) -> Sonuc<()>) -> Sonuc<()> {
        let durum = app
            .try_state::<MpvState>()
            .ok_or_else(|| Hata::yeni("oynatıcı hazır değil"))?;
        is(&durum)
    }
}
