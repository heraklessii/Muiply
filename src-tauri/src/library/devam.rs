//! Kaldığı yerden devam.
//!
//! Üç parçası var ve üçü de ayrı yerlerden çağrılıyor:
//!
//! - [`kaydet`] — mpv'nin olay döngüsü çalarken düzenli aralıkla,
//! - [`temizle`] — dosya sonuna geldiğinde (`EndFile` · `eof`),
//! - [`geri_yukle`] — yeni dosya yüklendiğinde (`FileLoaded`).
//!
//! Karar veren şey [`devam_konumu`] ve o SAF: eşikleri sınamak için ne mpv
//! ne veritabanı gerekiyor. Geri kalanı o kararı taşıyan tesisat.
//!
//! Anahtar dosyanın YOLU. Olay döngüsünün elinde kütüphane kimliği yok, çalan
//! dosyanın yolu var; kimliğe çevirmek için her konum yazımında bir sorgu
//! daha atmak gerekirdi.

use tauri::{AppHandle, Manager};

use crate::library::{db::kilit, LibraryState};
use crate::mpv::{oynatici, MpvState};
use crate::settings::SettingsState;

/// Başa bu kadar yakınsa devam edilmiyor.
///
/// Kullanıcı jeneriği geçmemişken bıraktıysa "kaldığın yer" diye 12. saniyeye
/// atlamak yardım değil, şaşırtma. Sıfırdan başlamak zaten doğru cevap.
pub const BAS_ESIK: f64 = 20.0;

/// Sona bu kadar yakınsa devam edilmiyor: dosya bitmiş sayılıyor.
///
/// Gerekli çünkü kaydı temizleyen şey `EndFile` · `eof` ve kullanıcı son
/// saniyelerde durdurup çıkarsa o olay hiç gelmiyor. O kayıt kalsaydı dosya
/// bir daha açıldığında doğrudan jeneriğe atlardı.
pub const SON_ESIK: f64 = 20.0;

/// Nereden devam edilecek — ya da hiç.
///
/// `sure` sıfır/bilinmiyorsa `None`: sonun nerede olduğunu bilmeden "sona
/// yakın mı" sorusu yanıtlanamaz ve yanlış yanıt kullanıcıyı dosyanın
/// bitişine atmak demek.
pub fn devam_konumu(kayitli: Option<f64>, sure: f64) -> Option<f64> {
    let konum = kayitli?;
    if !(konum.is_finite() && sure.is_finite()) || sure <= 0.0 {
        return None;
    }
    if konum < BAS_ESIK || sure - konum < SON_ESIK {
        return None;
    }
    Some(konum)
}

/// Ayar açık mı. Kapalıysa bu modülün hiçbir işi yok.
fn acik(app: &AppHandle) -> bool {
    app.try_state::<SettingsState>()
        .map(|a| a.anlik().resume)
        .unwrap_or(false)
}

/// Yarım kalma yerini kütüphaneye yazar.
///
/// Ayar KAPALIYSA yazmıyor: kimsenin okumadığı bir değer için oynatma
/// boyunca birkaç saniyede bir veritabanına gitmenin karşılığı yok.
///
/// Hata YUTULUYOR: bu, dakikada birkaç kez çalışan bir yan iş. Veritabanı o
/// an kilitliyse kullanıcıya söylenecek bir şey yok — bir sonraki yazım
/// zaten birkaç saniye sonra.
pub fn kaydet(app: &AppHandle, yol: &str, konum: f64) {
    if !acik(app) {
        return;
    }
    yaz(app, yol, Some(konum));
}

/// Şu an çalan dosyanın bulunduğu yeri yazar.
///
/// Çıkış yollarının ortak adımı. Olay döngüsü zaten beş saniyede bir
/// yazıyor (`mpv/gercek.rs`) ama aradaki o birkaç saniye tam da
/// kullanıcının "kapattığım an" saydığı yer.
///
/// İki çağıranı var ve ikisi de bir çıkış: pencerenin X'i (`lib.rs`) ve
/// tepsi menüsündeki "Çıkış" (`tepsi.rs`). Ayrı bir fonksiyon olmasının
/// sebebi bu — tepsiden çıkan kullanıcı, X'e basandan farklı bir yerde
/// kalmamalı.
pub fn simdiki_konumu_kaydet(app: &AppHandle) {
    let Some(oynatici_durum) = app.try_state::<MpvState>() else {
        return;
    };
    let d = oynatici_durum.anlik();
    if let Some(yol) = d.path {
        kaydet(app, &yol, d.position);
    }
}

/// Kaydı siler: dosya sonuna kadar izlendi, bir dahakine baştan başlasın.
///
/// Ayar kapalıyken de siliyor — sonradan açan kullanıcı, kapalıyken bitirdiği
/// bir filmin ortasına atlamasın.
pub fn temizle(app: &AppHandle, yol: &str) {
    yaz(app, yol, None);
}

fn yaz(app: &AppHandle, yol: &str, konum: Option<f64>) {
    if yol.is_empty() {
        return;
    }
    let Some(kutuphane) = app.try_state::<LibraryState>() else {
        return;
    };
    let Ok(db) = kilit(&kutuphane) else {
        return;
    };
    let _ = db.konum_yaz(yol, konum);
}

/// Yeni yüklenen dosyayı kaldığı yere atlatır.
///
/// `FileLoaded`ten çağrılıyor, `player_open`dan değil: eşik kararı süreyi
/// bilmeyi gerektiriyor ve süre dosya çözümlenene kadar bilinmiyor.
///
/// Ayar kapalıysa hiç okumuyor — kapalı bir özelliğin her dosya açılışında
/// bir sorgu atması gereksiz.
pub fn geri_yukle(app: &AppHandle, yol: &str, sure: f64) {
    if !acik(app) {
        return;
    }

    let kayitli = app
        .try_state::<LibraryState>()
        .and_then(|k| kilit(&k).ok().and_then(|db| db.konum_oku(yol)));

    let Some(konum) = devam_konumu(kayitli, sure) else {
        return;
    };
    let Some(oynatici_durum) = app.try_state::<MpvState>() else {
        return;
    };

    // Hata yutuluyor: atlayamamak dosyanın baştan çalması demek, oynatmanın
    // durması değil.
    let _ = oynatici::ara(&oynatici_durum, konum);
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kayit_yoksa_devam_yok() {
        assert_eq!(devam_konumu(None, 3600.0), None);
    }

    #[test]
    fn ortadan_devam_ediliyor() {
        assert_eq!(devam_konumu(Some(1800.0), 3600.0), Some(1800.0));
    }

    #[test]
    fn basa_yakin_kayit_yok_sayiliyor() {
        assert_eq!(devam_konumu(Some(BAS_ESIK - 0.1), 3600.0), None);
        assert_eq!(devam_konumu(Some(BAS_ESIK), 3600.0), Some(BAS_ESIK));
    }

    #[test]
    fn sona_yakin_kayit_yok_sayiliyor() {
        // Son saniyelerde kapatılan dosya için `eof` hiç gelmiyor; kayıt
        // duruyor ama onu kullanmak kullanıcıyı doğrudan jeneriğe atardı.
        assert_eq!(devam_konumu(Some(3595.0), 3600.0), None);
        assert_eq!(devam_konumu(Some(3580.0), 3600.0), Some(3580.0));
    }

    #[test]
    fn esiklerden_kisa_dosyada_hic_devam_yok() {
        // İki eşik toplamından kısa bir parçada devam edilebilecek aralık
        // yok; kısa bir şarkının ortasına atlamak zaten istenmezdi.
        assert_eq!(devam_konumu(Some(25.0), 30.0), None);
    }

    #[test]
    fn sure_bilinmiyorsa_devam_yok() {
        // Süre 0: mpv dosyayı henüz çözümlememiş ya da akış süresiz.
        assert_eq!(devam_konumu(Some(500.0), 0.0), None);
        assert_eq!(devam_konumu(Some(500.0), -1.0), None);
    }

    #[test]
    fn bozuk_sayilar_devam_ettirmiyor() {
        assert_eq!(devam_konumu(Some(f64::NAN), 3600.0), None);
        assert_eq!(devam_konumu(Some(500.0), f64::INFINITY), None);
    }
}
