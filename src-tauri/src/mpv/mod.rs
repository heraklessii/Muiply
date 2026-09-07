//! mpv motoru ve arayüzle paylaşılan oynatma tipleri.
//!
//! Katmanlar:
//!
//! ```text
//!   commands/player.rs        ince sarmalayıcı, Tauri'ye bakan yüz
//!         │
//!   mpv/oynatici.rs           "dosyayı aç", "sonrakine geç" gibi işler
//!         │
//!   mpv/{gercek,yok}.rs       Motor — libmpv'ye giden tek kapı
//!         │
//!   mpv/yuzey.rs              mpv'nin çizdiği native alt pencere
//! ```
//!
//! **Motor bir Cargo özelliği.** `mpv` özelliği kapalı derlendiğinde
//! [`yok::Motor`] devreye giriyor: her çağrı "motor yok" hatası dönüyor,
//! kütüphane ve çalma listesi tarafı çalışmaya devam ediyor. Sebep
//! `docs/Mpv_Integration.md`'de: libmpv bağlama zamanında bağlanıyor, DLL
//! yoksa uygulama hiç açılmıyor — yani "libmpv kurulu mu" sorusunu çalışma
//! zamanında sormak mümkün değil, derleme zamanında sormak gerekiyor.

pub mod oynatici;
pub mod sonda;
pub mod yuzey;

#[cfg(feature = "mpv")]
mod gercek;
#[cfg(feature = "mpv")]
pub use gercek::Motor;

#[cfg(not(feature = "mpv"))]
mod yok;
#[cfg(not(feature = "mpv"))]
pub use yok::Motor;

use std::sync::Mutex;

use serde::Serialize;

use crate::hata::Sonuc;
use yuzey::Yuzey;

/// Arayüze giden olay adları. `docs/IPC.md` ile birebir aynı olmalı.
pub const OLAY_DOSYA: &str = "player://file-loaded";
pub const OLAY_KONUM: &str = "player://time-pos";
pub const OLAY_DURAKLAT: &str = "player://pause-change";
pub const OLAY_SES: &str = "player://volume-change";
pub const OLAY_BITTI: &str = "player://end-file";
/// Sürenin YENİ değeri. mpv çoğu kapsayıcıda süreyi `FileLoaded`dan sonra
/// öğreniyor; olay yayınlanmazsa arayüzdeki süre 0 kalıyor ve arama sürgüsü
/// devre dışı görünüyor — kullanıcı için "sarma çalışmıyor" demek.
pub const OLAY_SURE: &str = "player://duration-change";
/// Oynatma hızının yeni değeri. Her dosya varsayılan hızla başlıyor
/// (`playlist/surucu.rs`); haber verilmezse arayüz bir önceki dosyanın
/// hızını göstermeye devam ediyor.
pub const OLAY_HIZ: &str = "player://rate-change";
pub const OLAY_HATA: &str = "player://error";
/// Oynatıcıda dosya KALMADI: kuyruk bitti ya da durduruldu. Yük yok.
///
/// Gerekli çünkü mpv dosya bitince onu boşaltıyor (`keep-open` kapalı) ama
/// `player://end-file` yalnız "çalmıyor" diyor; hangi dosyanın yüklü
/// olduğunu söylemiyor. Durumu yüklü bırakmak, bitmiş bir videoyu duruyor
/// gibi göstermek ve sürgüye basıldığında boş mpv'ye `seek` göndermek
/// demekti.
pub const OLAY_TEMIZ: &str = "player://cleared";
/// mpv penceresinden gelen kısayol (çift tık, `f`, `n` ...). Yük: eylemin adı.
pub const OLAY_ISTEK: &str = "player://request";

/// `player_get_state` ve `player://` olaylarıyla arayüze giden anlık durum.
///
/// Alan adları İngilizce çünkü `docs/IPC.md`'deki sözleşme öyle; kodun geri
/// kalanı Türkçe. Sınır burada.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub playing: bool,
    pub paused: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: i64,
    pub muted: bool,
    pub rate: f64,
    pub path: Option<String>,
    pub title: Option<String>,
    /// `"video"` ya da `"audio"`. Sahnenin video mu kapak mı göstereceğini
    /// bu belirliyor.
    pub media_type: Option<String>,
    /// Motor derlemeye dahil mi. `false` ise arayüz oynatma denetimlerini
    /// kapatıp sebebini yazıyor — sessizce tıklanan ölü düğme bırakmamak için.
    pub engine: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            playing: false,
            paused: true,
            position: 0.0,
            duration: 0.0,
            // mpv'nin kendi varsayılanı da 100. Arayüz ilk boyamada bu değeri
            // gösteriyor; motor açılınca gerçek değerle üzerine yazılıyor.
            volume: 100,
            muted: false,
            rate: 1.0,
            path: None,
            title: None,
            media_type: None,
            engine: Motor::VAR,
        }
    }
}

/// Bir ses / video / altyazı izi.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: i64,
    /// `"video"` · `"audio"` · `"sub"`
    pub kind: String,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub selected: bool,
    /// Dosyanın içinden mi geldi, yandaki bir dosyadan mı yüklendi.
    pub external: bool,
    pub codec: Option<String>,
}

/// Dosya açıldığında bir kez gönderilen künye.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub title: String,
    pub duration: f64,
    pub media_type: String,
    pub tracks: Vec<Track>,
}

/// Tauri'nin yönettiği oynatıcı durumu.
///
/// [`PlayerState`] burada bir kopya olarak tutuluyor. Sebep: arayüz açılışta
/// ve her görünüm değişiminde durumu soruyor; her seferinde mpv'ye sekiz ayrı
/// özellik sormak, olay döngüsünün zaten bildiği şeyi yeniden sormak olurdu.
/// Kopyayı **yalnızca** olay döngüsü yazıyor (tek yazar), komutlar okuyor.
pub struct MpvState {
    pub motor: Motor,
    pub yuzey: Yuzey,
    durum: Mutex<PlayerState>,
}

impl MpvState {
    pub fn yeni(motor: Motor, yuzey: Yuzey) -> Self {
        MpvState {
            motor,
            yuzey,
            durum: Mutex::new(PlayerState::default()),
        }
    }

    pub fn anlik(&self) -> PlayerState {
        self.durum.lock().expect("oynatıcı durumu kilidi").clone()
    }

    pub fn guncelle(&self, degistir: impl FnOnce(&mut PlayerState)) {
        let mut d = self.durum.lock().expect("oynatıcı durumu kilidi");
        degistir(&mut d);
    }
}

/// Motor kapalı derlemede her komutun döndüğü cümle.
///
/// Tek bir yerde: arayüz bu metni olduğu gibi gösteriyor ve iki farklı yerde
/// iki farklı cümle yazmak, kullanıcıya iki farklı sorun varmış gibi gelirdi.
pub const MOTOR_YOK: &str =
    "Oynatma motoru bu derlemede yok (libmpv olmadan derlendi). Kütüphane ve \
     çalma listeleri çalışıyor; oynatmak için libmpv kurulu bir sürüm gerekiyor.";

/// Saniyeyi mpv'nin beklediği metne çevirir.
///
/// mpv `seek` komutu argümanları metin olarak alıyor. Ayrı bir fonksiyon
/// çünkü biçim yanlış olduğunda hata sessiz: mpv komutu reddediyor, kullanıcı
/// sürgüyü bıraktığında hiçbir şey olmuyor.
pub fn saniye_metni(saniye: f64) -> String {
    format!("{saniye:.3}")
}

/// `track-list` girdisindeki metin alanını okur; boş dize `None` sayılır.
///
/// mpv bulunmayan alan için hata döndürmüyor, boş dize döndürebiliyor;
/// arayüzde `title: ""` bir izin adını "" yapıp satırı boş gösterirdi.
pub fn metin_alan(motor: &Motor, yol: &str) -> Option<String> {
    match motor.oku_metin(yol) {
        Ok(s) if !s.trim().is_empty() => Some(s),
        _ => None,
    }
}

/// mpv'nin `track-list` özelliğinden izleri toplar.
pub fn izleri_oku(motor: &Motor) -> Sonuc<Vec<Track>> {
    let sayi = motor.oku_sayi("track-list/count").unwrap_or(0);
    let mut izler = Vec::new();

    for i in 0..sayi {
        // Tür okunamıyorsa girdi bizim için yok: mpv listeyi dosya yüklenirken
        // değiştirebiliyor ve yarım okunan bir girdiyi listeye koymak, arayüzde
        // "Bilinmeyen iz" satırı üretmekten başka bir işe yaramaz.
        let Ok(kind) = motor.oku_metin(&format!("track-list/{i}/type")) else {
            continue;
        };
        let Ok(id) = motor.oku_sayi(&format!("track-list/{i}/id")) else {
            continue;
        };

        izler.push(Track {
            id,
            kind,
            title: metin_alan(motor, &format!("track-list/{i}/title")),
            lang: metin_alan(motor, &format!("track-list/{i}/lang")),
            selected: motor
                .oku_bayrak(&format!("track-list/{i}/selected"))
                .unwrap_or(false),
            external: motor
                .oku_bayrak(&format!("track-list/{i}/external"))
                .unwrap_or(false),
            codec: metin_alan(motor, &format!("track-list/{i}/codec")),
        });
    }

    Ok(izler)
}
