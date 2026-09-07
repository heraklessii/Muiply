//! Kütüphane — izlenen klasörler, taranan dosyalar, SQLite.
//!
//! Katmanlar:
//!
//! ```text
//!   commands/library.rs   ince sarmalayıcı
//!         │
//!   library/tarayici.rs   klasörü gez, değişeni bul, olay yayınla
//!         │
//!   library/db.rs         SQLite şeması ve sorguları
//! ```
//!
//! Küçük resimler [`kucukresim`] içinde ve tarayıcının işi DEĞİL — sebebi
//! orada yazılı.

pub mod db;
pub mod devam;
pub mod kucukresim;
pub mod tarayici;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Arayüze giden kütüphane olayları. `docs/IPC.md` ile birebir aynı.
pub const OLAY_TARAMA: &str = "library://scan-progress";
pub const OLAY_TARAMA_BITTI: &str = "library://scan-complete";
pub const OLAY_MEDYA: &str = "library://media-updated";

/// libmpv/FFmpeg'in açtığı biçimlerin bizim taradığımız alt kümesi.
///
/// Liste bilerek KISA: mpv çok daha fazlasını açıyor ama kütüphaneye
/// girmesini istediğimiz şey kullanıcının "film/müzik" saydığı dosyalar.
/// Klasördeki her `.bin`i listeye almak, kütüphaneyi çöplük yapar.
pub const VIDEO_UZANTILARI: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "ts", "m2ts", "wmv", "3gp", "ogv", "m4v", "mpg",
    "mpeg",
];

pub const SES_UZANTILARI: &[&str] = &[
    "mp3", "flac", "aac", "ogg", "opus", "wav", "m4a", "wma", "ape", "alac", "aiff", "mka",
];

/// Uzantıdan medya türü. Tanımadığımız uzantı `None` — dosya taranmaz.
pub fn tur(uzanti: &str) -> Option<&'static str> {
    let u = uzanti.to_ascii_lowercase();
    if VIDEO_UZANTILARI.contains(&u.as_str()) {
        Some("video")
    } else if SES_UZANTILARI.contains(&u.as_str()) {
        Some("audio")
    } else {
        None
    }
}

/// Kütüphanedeki bir kayıt. Alan adları İngilizce: `docs/IPC.md` sözleşmesi.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub size: i64,
    pub media_type: String,
    pub extension: String,
    /// Küçük resim dosyasının TAM YOLU. Arayüz bunu `convertFileSrc` ile
    /// `asset:` adresine çeviriyor; ham yol `<img src>` içinde çalışmaz.
    pub thumbnail: Option<String>,
    pub added_at: i64,
    pub last_played: Option<i64>,
    pub play_count: i64,
    /// Yarım bırakılan yer (saniye). Bitmiş ya da hiç açılmamış dosyada
    /// `None`. Karar veren kod [`devam`] içinde.
    pub last_position: Option<f64>,
}

/// `library_get_media` süzgeci.
///
/// Arama BURADA yok: arayüz yazdıkça süzüyor (bkz. `docs/Frontend.md`).
/// Her tuş vuruşunda IPC'ye çıkmak, bellekteki bir diziyi süzmenin yanında
/// bedava değil.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFilter {
    /// `"video"` · `"audio"` · yoksa hepsi.
    pub media_type: Option<String>,
    /// `"title"` · `"added"` · `"played"` · `"duration"` — varsayılan `added`.
    pub sort: Option<String>,
    pub limit: Option<i64>,
}

/// Tauri'nin yönettiği kütüphane durumu.
pub struct LibraryState {
    pub db: Mutex<db::LibraryDb>,
    /// `$APPDATA/muiply` — küçük resim önbelleği de burada.
    pub kok: PathBuf,
    /// Aynı anda iki tarama başlamasın. İki tarama aynı satırları yazıp
    /// ilerleme olaylarını iç içe geçirir; kullanıcı yüzdeyi zıplarken görür.
    pub taraniyor: AtomicBool,
    /// Çalınmaya başlamış ama karesi henüz alınmamış kayıt.
    /// Ayrıntı: [`kucukresim`].
    pub bekleyen_kare: Mutex<Option<String>>,
}

impl LibraryState {
    pub fn kucukresim_dizini(&self) -> PathBuf {
        self.kok.join("kucukresim")
    }
}

/// Yoldan kimlik: SHA-256'nın ilk 16 baytı, onaltılık.
///
/// Yol seçildi çünkü aynı dosyanın iki kopyası kütüphanede iki kayıt olmalı
/// (kullanıcı ikisini de görüyor). İçerik özeti almak her dosyayı baştan
/// sona okumak demekti; tarama saatler sürerdi.
pub fn kimlik(yol: &str) -> String {
    use sha2::{Digest, Sha256};
    let ozet = Sha256::digest(yol.as_bytes());
    hex::encode(&ozet[..16])
}

/// Şu anki unix zamanı (saniye).
pub fn simdi() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod testler {
    use super::{SES_UZANTILARI, VIDEO_UZANTILARI};

    /// `tauri.conf.json` derleme zamanında gömülüyor: test dosyayı diskte
    /// aramıyor, yani çalışma dizininden bağımsız.
    const YAPILANDIRMA: &str = include_str!("../../tauri.conf.json");

    /// Bir dosya ilişkilendirmesinin `ext` listesi.
    fn iliskilendirme(ad: &str) -> Vec<String> {
        let kok: serde_json::Value =
            serde_json::from_str(YAPILANDIRMA).expect("tauri.conf.json okunamadı");
        let listeler = kok["bundle"]["fileAssociations"]
            .as_array()
            .expect("bundle.fileAssociations bir dizi değil");

        let hedef = listeler
            .iter()
            .find(|g| g["name"] == ad)
            .unwrap_or_else(|| panic!("`{ad}` ilişkilendirmesi yok"));

        hedef["ext"]
            .as_array()
            .unwrap_or_else(|| panic!("`{ad}` içinde `ext` dizisi yok"))
            .iter()
            .map(|u| u.as_str().expect("uzantı dize değil").to_string())
            .collect()
    }

    /// Tarama listesi ile işletim sistemine bildirilen liste AYNI olmalı.
    ///
    /// İkisi ayrıştığında belirti kullanıcı tarafında görünüyor ve sebebi
    /// hiç göstermiyor: `tauri.conf.json`da olup burada olmayan bir uzantı,
    /// çift tıklandığında açılan ama kütüphanede hiç görünmeyen bir dosya
    /// demek; tersi ise ızgarada duran ama Muiply'ın "Varsayılan
    /// uygulamalar" listesinde çıkmadığı bir tür.
    ///
    /// Kural CLAUDE.md'de de yazılı (13) ama yazılı bir kural unutulabiliyor;
    /// bu test unutulamıyor.
    #[test]
    fn video_uzantilari_yapilandirmayla_ayni() {
        assert_eq!(iliskilendirme("MuiplyVideo"), VIDEO_UZANTILARI);
    }

    #[test]
    fn ses_uzantilari_yapilandirmayla_ayni() {
        assert_eq!(iliskilendirme("MuiplyAudio"), SES_UZANTILARI);
    }

    /// Aynı uzantı iki listede birden olamaz: [`tur`] ilk eşleşeni döndürüyor
    /// ve çakışan bir uzantı, aynı dosyanın bir yerde video bir yerde ses
    /// sayılması demek olurdu.
    #[test]
    fn iki_liste_kesismiyor() {
        for u in VIDEO_UZANTILARI {
            assert!(
                !SES_UZANTILARI.contains(u),
                "`{u}` hem video hem ses listesinde"
            );
        }
    }
}
