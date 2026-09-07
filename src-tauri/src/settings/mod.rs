//! Ayarlar — kullanıcının kalıcı tercihleri.
//!
//! Beş tane var ve beşi de bilerek az: `hwdec`, ses aygıtı, varsayılan hız,
//! kaldığı yerden devam, medya tuşları. mpv'nin yüzlerce seçeneği arayüze
//! AÇILMIYOR (bkz. `docs/Roadmap.md` → Kapsam Dışı); buradakiler kullanıcının
//! bir kez ayarlayıp unutacağı, ama ayarlamadığında canını yakan şeyler.
//!
//! İkisinin karşılığı mpv'de YOK ve bu ikisi `uygula`dan geçmiyor: `resume`
//! dosya açılışında verilen bir karar (`library/devam.rs`), `media_keys` ise
//! işletim sisteminden bir kısayol kaydı (`tepsi::medya_tuslari`).
//!
//! **Tema burada yok.** O arayüzün `localStorage`'ında (`src/lib/platform.ts`)
//! çünkü ilk boyamadan ÖNCE bilinmesi gerekiyor; backend'e sormak pencerenin
//! bir kare yanlış renkte açılması demek olurdu.
//!
//! Depolama anahtar/değer: `settings` tablosu (`library/db.rs`). Değerler HEP
//! metin, tip bilgisi burada — okuyan taraf zaten ne beklediğini biliyor ve
//! yeni bir ayar eklemek şema geçişi gerektirmiyor.
//!
//! Okuma HİÇ başarısız olmuyor: bozuk ya da eksik bir değer varsayılana
//! düşüyor. Veritabanında elle bozulmuş bir satır yüzünden oynatıcının
//! açılmaması orantısız olurdu.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::hata::Sonuc;
use crate::library::db::LibraryDb;
use crate::mpv::Motor;

/// `hwdec` için arayüzün sunduğu seçenekler.
///
/// mpv'nin listesi çok daha uzun (`d3d11va`, `vaapi`, `nvdec` ...) ama onu
/// kullanıcıya açmak, hangisinin makinesinde çalıştığını ona sordurmak
/// olurdu. Üç seçenek üç niyet: "sen bil", "elinden geleni yap", "hiç deneme".
pub const HWDEC_SECENEKLERI: &[&str] = &["auto-safe", "auto", "no"];

/// Sistem varsayılanı. mpv'nin `audio-device` özelliğinde de aynı dize.
pub const SES_AYGITI_OTOMATIK: &str = "auto";

/// Hız sınırları — [`crate::mpv::oynatici::hiz_ayarla`] ile aynı aralık.
pub const HIZ_ALT: f64 = 0.25;
pub const HIZ_UST: f64 = 4.0;

const A_HWDEC: &str = "hwdec";
const A_SES_AYGITI: &str = "audio_device";
const A_HIZ: &str = "default_rate";
const A_DEVAM: &str = "resume";
const A_MEDYA_TUSLARI: &str = "media_keys";

/// Arayüzle paylaşılan ayarlar. Alan adları İngilizce: `docs/IPC.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub hwdec: String,
    pub audio_device: String,
    pub default_rate: f64,
    pub resume: bool,
    /// Klavyenin medya tuşları Muiply'yi mi yönetsin.
    ///
    /// Ayar olmak zorunda: kayıt GLOBAL, yani açıkken tuş başka bir
    /// oynatıcıya gitmiyor (bkz. `tepsi::medya_tuslari`).
    pub media_keys: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            // `mpv/gercek.rs`'deki başlangıç değeriyle AYNI olmak zorunda:
            // ayara hiç dokunmamış bir kullanıcıda ikisi ayrışırsa, uygulama
            // açılışta kendi varsayılanının üstüne başka bir varsayılan yazar.
            hwdec: "auto-safe".to_string(),
            audio_device: SES_AYGITI_OTOMATIK.to_string(),
            default_rate: 1.0,
            // Varsayılan KAPALI. Açık olsaydı bir filmi bilerek baştan açan
            // kullanıcı kendini ortasında bulurdu ve bunun neden olduğunu
            // ayarları hiç görmeden anlaması gerekirdi.
            resume: false,
            // Varsayılan AÇIK. Bir medya oynatıcısından beklenen bu ve
            // kullanıcı Muiply'yi açtığı anda tuşların çalışmasını bekliyor.
            // Kapatma yolu ayarlarda, tek tık.
            media_keys: true,
        }
    }
}

impl Settings {
    /// Aralık dışı ve tanınmayan değerleri bilinen olana çeker.
    ///
    /// Hata DÖNMÜYOR: ayarları yazan tek yer arayüz ve orada yanlış bir değer
    /// ancak bizim hatamızla oluşur. Kullanıcıya "geçersiz hwdec" demek, onun
    /// düzeltemeyeceği bir şeyi bildirmek olurdu. Düzeltilmiş değer geri
    /// dönüyor (`docs/IPC.md`) ve arayüz gönderdiğini değil onu gösteriyor.
    pub fn duzelt(mut self) -> Self {
        if !HWDEC_SECENEKLERI.contains(&self.hwdec.as_str()) {
            self.hwdec = Settings::default().hwdec;
        }
        if self.audio_device.trim().is_empty() {
            self.audio_device = SES_AYGITI_OTOMATIK.to_string();
        }
        // `clamp` NaN'ı NaN bırakıyor; mpv'ye NaN yazmak oynatmayı sessizce
        // durdururdu, o yüzden önce sonluluk sınanıyor.
        self.default_rate = if self.default_rate.is_finite() {
            self.default_rate.clamp(HIZ_ALT, HIZ_UST)
        } else {
            1.0
        };
        // `resume` ve `media_keys` burada YOK ve olmayacak: bir `bool`un
        // geçersiz değeri yok. Yeni bir ayar eklendiğinde bu fonksiyona
        // dokunulup dokunulmayacağı sorusunun cevabı "tipinde düzeltilecek
        // bir şey var mı" (CLAUDE.md kuralı 11).
        self
    }
}

/// Tauri'nin yönettiği ayar durumu.
///
/// Bellekte bir kopya duruyor çünkü [`crate::library::devam`] her dosya
/// açılışında "devam açık mı" diye soruyor; her seferinde veritabanına gitmek,
/// bilinen bir bayrak için kilit almak olurdu.
pub struct SettingsState(pub Mutex<Settings>);

impl SettingsState {
    pub fn yeni(a: Settings) -> Self {
        SettingsState(Mutex::new(a))
    }

    /// Anlık kopya. Kilit bozuksa varsayılan: ayarları okuyamamak oynatmayı
    /// durdurmamalı.
    pub fn anlik(&self) -> Settings {
        self.0
            .lock()
            .map(|a| a.clone())
            .unwrap_or_else(|_| Settings::default())
    }

    pub fn yerlestir(&self, yeni: Settings) {
        if let Ok(mut a) = self.0.lock() {
            *a = yeni;
        }
    }
}

/// Kayıtlı ayarlar. Eksik ya da bozuk anahtar varsayılana düşüyor.
pub fn oku(db: &LibraryDb) -> Settings {
    let Ok(satirlar) = db.ayarlar() else {
        return Settings::default();
    };
    let varsayilan = Settings::default();
    let al = |anahtar: &str| satirlar.get(anahtar).map(String::as_str);

    Settings {
        hwdec: al(A_HWDEC).unwrap_or(&varsayilan.hwdec).to_string(),
        audio_device: al(A_SES_AYGITI)
            .unwrap_or(&varsayilan.audio_device)
            .to_string(),
        default_rate: al(A_HIZ)
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(varsayilan.default_rate),
        resume: al(A_DEVAM).map(|d| d == "1").unwrap_or(varsayilan.resume),
        media_keys: al(A_MEDYA_TUSLARI)
            .map(|d| d == "1")
            .unwrap_or(varsayilan.media_keys),
    }
    .duzelt()
}

/// Ayarları yazar.
///
/// Hata DÖNÜYOR (okumanın tersine): bu kullanıcının bilerek yaptığı bir
/// işlem ve sessizce kaybolması, ayarı bir daha açtığında eski değeri
/// görmesi demek olurdu.
///
/// Hız noktalı ondalıkla yazılıyor (`to_string`) — geri okuyan
/// `parse::<f64>()` virgüllü ayracı tanımıyor.
pub fn yaz(db: &LibraryDb, a: &Settings) -> Sonuc<()> {
    db.ayar_yaz(A_HWDEC, &a.hwdec)?;
    db.ayar_yaz(A_SES_AYGITI, &a.audio_device)?;
    db.ayar_yaz(A_HIZ, &a.default_rate.to_string())?;
    db.ayar_yaz(A_DEVAM, if a.resume { "1" } else { "0" })?;
    db.ayar_yaz(A_MEDYA_TUSLARI, if a.media_keys { "1" } else { "0" })?;
    Ok(())
}

/// Ayarları mpv'ye uygular.
///
/// `resume` ve `media_keys` burada YOK: ikisinin de karşılığı bir mpv
/// özelliği değil. `resume` dosya açılışında verilen bir karar
/// ([`crate::library::devam`]); `media_keys` işletim sisteminden bir kısayol
/// kaydı ([`crate::tepsi::medya_tuslari::uygula`]).
///
/// Motorsuz derlemede hiçbir şey yapmıyor. Yoksa her çağrı "motor yok"
/// hatası dönerdi ve açılışta, kütüphane kabuğunu kullanmayı hiç
/// engellemeyen bir hatayı kullanıcıya bildirmenin karşılığı yok.
pub fn uygula(motor: &Motor, a: &Settings) -> Sonuc<()> {
    if !Motor::VAR {
        return Ok(());
    }
    motor.ayar_metin("hwdec", &a.hwdec)?;
    // Aygıt kayıtlı ama takılı değilse mpv sesi açamıyor; ayar yine de
    // yazılıyor ve sorun bir sonraki dosyada "ses yok" olarak görünüyor.
    // Reddetmek daha kötü olurdu: kullanıcı aygıtı geri taktığında ayarı
    // yeniden kurmak zorunda kalırdı.
    motor.ayar_metin("audio-device", &a.audio_device)?;
    motor.ayar_ondalik("speed", a.default_rate)?;
    Ok(())
}

/// mpv'nin bildiği bir ses çıkışı.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    /// `audio-device` özelliğine yazılacak ad.
    pub name: String,
    /// Kullanıcıya gösterilen ad. mpv boş bırakırsa `name` kullanılıyor.
    pub description: String,
}

/// Ses aygıtlarını mpv'den okur.
///
/// Okuma [`crate::mpv::izleri_oku`] ile aynı biçimde, indeksli alt
/// özelliklerle: `audio-device-list` bir düğüm dizisi ve onu tek parça olarak
/// almak JSON çözmek demek olurdu.
///
/// Hata BOŞ LİSTE: motorsuz derlemede ve mpv'nin listeyi vermediği durumda
/// "aygıt yok" doğru cevap. Arayüz o zaman yalnız sistem varsayılanını
/// gösteriyor, ki o seçenek her makinede çalışıyor.
pub fn ses_aygitlari(motor: &Motor) -> Vec<AudioDevice> {
    let sayi = motor.oku_sayi("audio-device-list/count").unwrap_or(0);
    let mut aygitlar = Vec::new();

    for i in 0..sayi {
        let Ok(name) = motor.oku_metin(&format!("audio-device-list/{i}/name")) else {
            continue;
        };
        let description = motor
            .oku_metin(&format!("audio-device-list/{i}/description"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| name.clone());
        aygitlar.push(AudioDevice { name, description });
    }

    aygitlar
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn taninmayan_hwdec_varsayilana_dusuyor() {
        let a = Settings {
            hwdec: "d3d11va-copy".into(),
            ..Settings::default()
        }
        .duzelt();
        assert_eq!(a.hwdec, "auto-safe");
    }

    #[test]
    fn bilinen_hwdec_korunuyor() {
        for s in HWDEC_SECENEKLERI {
            let a = Settings {
                hwdec: (*s).into(),
                ..Settings::default()
            }
            .duzelt();
            assert_eq!(&a.hwdec, s);
        }
    }

    #[test]
    fn hiz_sinira_cekiliyor() {
        let hizli = Settings {
            default_rate: 12.0,
            ..Settings::default()
        }
        .duzelt();
        assert_eq!(hizli.default_rate, HIZ_UST);

        let yavas = Settings {
            default_rate: 0.0,
            ..Settings::default()
        }
        .duzelt();
        assert_eq!(yavas.default_rate, HIZ_ALT);
    }

    #[test]
    fn bozuk_hiz_bire_donuyor() {
        let a = Settings {
            default_rate: f64::NAN,
            ..Settings::default()
        }
        .duzelt();
        assert_eq!(a.default_rate, 1.0);
    }

    #[test]
    fn bos_ses_aygiti_otomatige_donuyor() {
        let a = Settings {
            audio_device: "   ".into(),
            ..Settings::default()
        }
        .duzelt();
        assert_eq!(a.audio_device, SES_AYGITI_OTOMATIK);
    }

    /// Diskte kendi klasörü olan bir veritabanı. `LibraryDb::ac` bir dizin
    /// istiyor; testler birbirinin dosyasını görmesin diye ad benzersiz.
    fn bos_db(ad: &str) -> LibraryDb {
        let dizin = std::env::temp_dir().join(format!("muiply-test-ayar-{ad}"));
        let _ = std::fs::remove_dir_all(&dizin);
        LibraryDb::ac(&dizin).expect("veritabanı")
    }

    #[test]
    fn bos_veritabani_varsayilan_veriyor() {
        assert_eq!(oku(&bos_db("bos")), Settings::default());
    }

    #[test]
    fn yazilan_ayar_geri_okunuyor() {
        let db = bos_db("gidis-donus");
        let a = Settings {
            hwdec: "no".into(),
            audio_device: "wasapi/hoparlor".into(),
            default_rate: 1.5,
            resume: true,
            media_keys: false,
        };

        yaz(&db, &a).expect("yazma");

        assert_eq!(oku(&db), a);
    }

    #[test]
    fn bozuk_deger_varsayilana_dusuyor() {
        let db = bos_db("bozuk");
        db.ayar_yaz(A_HIZ, "çok hızlı").expect("yazma");
        db.ayar_yaz(A_HWDEC, "uydurma").expect("yazma");
        db.ayar_yaz(A_DEVAM, "belki").expect("yazma");
        db.ayar_yaz(A_MEDYA_TUSLARI, "belki").expect("yazma");

        let a = oku(&db);

        assert_eq!(a.default_rate, 1.0);
        assert_eq!(a.hwdec, "auto-safe");
        assert!(!a.resume);
        // Bayraklarda "1 değilse kapalı": `media_keys` varsayılanı AÇIK
        // olmasına rağmen bozuk değer kapalıya düşüyor, çünkü satırın kendisi
        // var — eksik anahtar ile bozuk anahtar farklı şeyler.
        assert!(!a.media_keys);
    }

    #[test]
    fn medya_tuslari_varsayilan_acik() {
        assert!(Settings::default().media_keys);
        // Anahtar hiç yokken de açık: yeni bir kurulumda ayar satırı yok.
        assert!(oku(&bos_db("medya-tuslari")).media_keys);
    }

    #[test]
    fn ondalik_nokta_ile_yaziliyor() {
        // Virgüllü ayraç `parse::<f64>()` ile geri okunamaz; ayar bir sonraki
        // açılışta sessizce 1.0'a dönerdi.
        let db = bos_db("ondalik");
        yaz(
            &db,
            &Settings {
                default_rate: 1.25,
                ..Settings::default()
            },
        )
        .expect("yazma");

        let ham: String = db
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = 'default_rate'",
                [],
                |s| s.get(0),
            )
            .expect("okuma");

        assert_eq!(ham, "1.25");
    }
}
