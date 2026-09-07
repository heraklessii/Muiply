//! Küçük resimler — izlediğin karenin kendisi.
//!
//! # Neden tarama sırasında değil
//!
//! Alışılmış yol, taramada her dosyanın %10'una atlayıp bir kare almak.
//! libmpv'de bunun bedeli ağır: kare almak bir video çıkışı istiyor, yani
//! tarama için ekrana çizen ikinci bir mpv kurmak gerekiyor. Bin dosyalık bir
//! klasörde bu, kullanıcının hiç açmayacağı dosyalar için dakikalarca kod
//! çözme demek.
//!
//! Bunun yerine kare, dosya ÇALARKEN alınıyor — zaten çizen bir video çıkışı
//! varken, bedava. Sonuç kural olarak da tutarlı: **ızgarada resmi olan
//! kayıtlar, açtıkların.** Hiç açmadıkların soyut bantla duruyor (aynı kural
//! MuiLabs'ta da var: göstermediğimiz şeyi gösteriyormuş gibi yapmıyoruz).
//!
//! # Neden tam çözünürlük
//!
//! mpv'nin `screenshot-to-file` komutunda ölçekleme yok. Küçültmek için bir
//! görüntü kütüphanesi eklemek gerekirdi; 208 piksel genişliğinde çizilen bir
//! bant için tarayıcının kendi ölçeklemesi yeterli. Bedeli dosya başına
//! ~150 KB önbellek.

use std::path::PathBuf;

use tauri::{Emitter, Manager};

use crate::mpv::MpvState;

use super::{LibraryState, OLAY_MEDYA};

/// Karenin alındığı an (saniye).
///
/// Başlangıç jeneriği, siyah kare ve dağıtımcı logosu ilk saniyelerde.
/// 8 saniye, çoğu dosyada içeriğin kendisine düşecek kadar ileri; kullanıcının
/// dosyayı kapatmadan önce geçireceği kadar da erken.
const KARE_ANI: f64 = 8.0;

/// JPEG kalitesi. 75 gözle ayırt edilmiyor, 95'in üçte biri yer tutuyor.
const KALITE: i64 = 75;

/// Yeni bir dosya yüklendi: karesi gerekiyorsa sıraya al.
///
/// Kütüphanede olmayan (kullanıcının sürükleyip attığı) dosyalar için hiçbir
/// şey yapılmıyor — yazacak bir kayıt yok.
pub fn isaretle(app: &tauri::AppHandle, yol: &str) {
    let Some(kutuphane) = app.try_state::<LibraryState>() else {
        return;
    };

    let kayit = super::db::kilit(&kutuphane)
        .ok()
        .and_then(|db| db.yol_ile(yol).ok())
        .flatten();

    let bekleyen = match kayit {
        // Resmi zaten var: bir daha almıyoruz. Kullanıcının gördüğü kare
        // her açışta değişseydi, ızgara her seferinde başka görünürdü.
        Some(k) if k.thumbnail.is_none() && k.media_type == "video" => Some(k.id),
        _ => None,
    };

    // `let-else`, `if let` değil: `if let` bloğunun geçici değeri fonksiyonun
    // sonuna kadar yaşıyor ve `kutuphane`den sonra düşüyor — ödünç denetimi
    // haklı olarak reddediyor.
    let Ok(mut yuva) = kutuphane.bekleyen_kare.lock() else {
        return;
    };
    *yuva = bekleyen;
}

/// Oynatma konumu ilerledi: sıradaki kare alınacak ana geldiyse al.
///
/// Saniyede birkaç kez çağrılıyor, o yüzden ilk satır en ucuz kontrol.
pub fn belki_al(app: &tauri::AppHandle, konum: f64) {
    if konum < KARE_ANI {
        return;
    }

    let Some(kutuphane) = app.try_state::<LibraryState>() else {
        return;
    };

    // Kimliği kilidin İÇİNDE alıp hemen boşaltıyoruz: kare almak mpv'ye bir
    // komut ve dosya yazımı; kilidi o kadar süre tutmak, aynı anda gelen bir
    // kütüphane sorgusunu bekletirdi.
    let kimlik = {
        let Ok(mut yuva) = kutuphane.bekleyen_kare.lock() else {
            return;
        };
        match yuva.take() {
            Some(k) => k,
            None => return,
        }
    };

    let Some(oynatici) = app.try_state::<MpvState>() else {
        return;
    };

    let dizin = kutuphane.kucukresim_dizini();
    if std::fs::create_dir_all(&dizin).is_err() {
        return;
    }
    let hedef: PathBuf = dizin.join(format!("{kimlik}.jpg"));
    let hedef_metin = hedef.to_string_lossy().into_owned();

    let _ = oynatici.motor.ayar_sayi("screenshot-jpeg-quality", KALITE);
    // `video`: altyazısız ve OSD'siz ham kare. `subtitles` kipi altyazıyı da
    // yakalar; ızgarada donmuş bir altyazı satırı görmek istemiyoruz.
    if oynatici
        .motor
        .komut("screenshot-to-file", &[&hedef_metin, "video"])
        .is_err()
    {
        // Alınamadı (kod çözücü kareyi vermedi, disk dolu ...). Kayıt
        // resimsiz kalıyor; bir dahaki açılışta yeniden sıraya girecek.
        return;
    }

    let Ok(db) = super::db::kilit(&kutuphane) else {
        return;
    };
    if db.kucukresim_yaz(&kimlik, &hedef_metin).is_ok() {
        let _ = app.emit(OLAY_MEDYA, serde_json::json!({ "id": kimlik }));
    }
}
