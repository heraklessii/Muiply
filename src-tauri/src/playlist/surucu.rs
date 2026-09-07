//! Kuyruğu oynatıcıya bağlayan katman.
//!
//! [`super::kuyruk`] saf: hangi öğeye geçileceğini biliyor ama mpv'yi,
//! veritabanını ve Tauri'yi tanımıyor. Burası o kararı uyguluyor — dosyayı
//! açıyor, oynatma sayacını artırıyor, arayüze haber veriyor.
//!
//! Ayrı dosya olmasının sebebi test: kuyruk kararlarını sınamak için pencere
//! açmak gerekmesin.

use tauri::{AppHandle, Emitter, Manager};

use crate::hata::{Hata, Sonuc};
use crate::library::{db::kilit, LibraryState};
use crate::mpv::{oynatici, MpvState};
use crate::settings::SettingsState;

use super::kuyruk::{QueueItem, QueueState, Repeat};

/// Kuyruk değiştiğinde arayüze giden olay.
///
/// Gerekli çünkü kuyruğu ilerleten her zaman arayüz değil: dosya bitince
/// karar burada veriliyor ve arayüzün bunu öğrenmesinin başka yolu yok.
pub const OLAY_KUYRUK: &str = "playlist://queue";

/// Öğeyi çalar.
///
/// Sıra önemli: önce dosya açılıyor, sonra sayaç. Dosya açılamazsa (silinmiş,
/// sürücü çıkarılmış) "çaldı" sayılmamalı.
///
/// Oynatmanın TEK kapısı burası — kütüphaneden tıklama da, kuyruğun kendi
/// kendine ilerlemesi de buradan geçiyor. Bu yüzden "her dosya şu hızda
/// başlasın" gibi dosya başına kararlar da burada.
pub fn oynat(app: &AppHandle, oge: &QueueItem) -> Sonuc<()> {
    let oynatici_durum = app
        .try_state::<MpvState>()
        .ok_or_else(|| Hata::yeni("oynatıcı hazır değil"))?;

    oynatici::ac(&oynatici_durum, &oge.path)?;

    // Varsayılan hız HER dosyada yeniden uygulanıyor: mpv `speed`i dosyalar
    // arasında koruyor, oysa "varsayılan hız" bir dosyanın BAŞLADIĞI hız
    // demek. Çubuktan yapılan değişiklik o dosya boyunca geçerli, sonrakine
    // taşınmıyor. Hata yutuluyor — hız uygulanamadı diye dosyayı çalmamak
    // orantısız.
    if let Some(ayarlar) = app.try_state::<SettingsState>() {
        let _ = oynatici::hiz_ayarla(&oynatici_durum, ayarlar.anlik().default_rate);
    }

    // Sayaç kütüphane kaydı varsa artıyor; kuyruğa sürükleyip bırakılan,
    // kütüphanede olmayan bir dosya için yazacak satır yok.
    //
    // Yazma BAŞARILIYSA olay da gidiyor: `play_count` ve `last_played`
    // değişti, yani o kaydı gösteren her yer artık eski. Olay olmadan
    // kütüphane penceresindeki "Son çalınanlar" bir oturum boyunca hiç
    // güncellenmiyordu — kullanıcı bir şey çalıyor, listede görünmüyordu.
    if let Some(kutuphane) = app.try_state::<LibraryState>() {
        let yazildi = kilit(&kutuphane)
            .ok()
            .map(|db| db.oynatildi(&oge.id).is_ok())
            .unwrap_or(false);
        if yazildi {
            let _ = app.emit(
                crate::library::OLAY_MEDYA,
                serde_json::json!({ "id": oge.id }),
            );
        }
    }

    yayinla(app);
    Ok(())
}

/// Kuyrukta bir sonrakine geçer.
///
/// `otomatik`: dosya kendiliğinden bittiği için mi çağrıldı. Ayrımın sebebi
/// [`Repeat::One`] — bkz. `kuyruk::sonraki`.
///
/// Sırada bir şey yoksa `Ok(None)`: bu bir hata değil, listenin sonu.
pub fn ilerle(app: &AppHandle, otomatik: bool) -> Sonuc<Option<QueueItem>> {
    let hedef = {
        let durum = app
            .try_state::<QueueState>()
            .ok_or_else(|| Hata::yeni("kuyruk hazır değil"))?;
        let mut kuyruk = durum
            .0
            .lock()
            .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?;
        kuyruk.ilerle(otomatik)
    };

    match hedef {
        Some(oge) => {
            oynat(app, &oge)?;
            Ok(Some(oge))
        }
        None => {
            // Liste bitti. Kuyruk yerinde duruyor (kullanıcı yeniden
            // başlatabilsin) ama mpv'de artık yüklü dosya YOK: `keep-open`
            // kapalı ve mpv biten dosyayı boşalttı. Oynatıcı durumu da
            // boşaltılmazsa arayüz bitmiş videoyu duruyor gibi gösteriyor,
            // sürgü boş mpv'ye `seek` gönderip hata şeridi çıkarıyor.
            crate::mpv::oynatici::bosalt(app);
            yayinla(app);
            Ok(None)
        }
    }
}

pub fn gerile(app: &AppHandle) -> Sonuc<Option<QueueItem>> {
    let hedef = {
        let durum = app
            .try_state::<QueueState>()
            .ok_or_else(|| Hata::yeni("kuyruk hazır değil"))?;
        let mut kuyruk = durum
            .0
            .lock()
            .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?;
        kuyruk.gerile()
    };

    match hedef {
        Some(oge) => {
            oynat(app, &oge)?;
            Ok(Some(oge))
        }
        None => Ok(None),
    }
}

pub fn indeksten_oynat(app: &AppHandle, indeks: usize) -> Sonuc<()> {
    let oge = {
        let durum = app
            .try_state::<QueueState>()
            .ok_or_else(|| Hata::yeni("kuyruk hazır değil"))?;
        let mut kuyruk = durum
            .0
            .lock()
            .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?;
        kuyruk.sec(indeks)
    };

    let oge = oge.ok_or_else(|| Hata::yeni("kuyrukta böyle bir sıra yok"))?;
    oynat(app, &oge)
}

/// Dosya bitti — sıradakine geç.
///
/// `mpv/gercek.rs`'deki olay döngüsünden çağrılıyor. Hata YUTULUYOR: olay
/// döngüsü kullanıcıya diyalog gösteremez ve bir sonraki dosya açılamadı
/// diye döngünün durması, oynatıcının sessizce ölmesi demek olurdu. Hata
/// yerine `player://error` yayınlanıyor.
pub fn dosya_bitti(app: &AppHandle) {
    if let Err(e) = ilerle(app, true) {
        let _ = app.emit(crate::mpv::OLAY_HATA, e.to_string());
    }
}

/// Tek bir dosyayı açar — sürükle-bırak, "Dosya aç", ızgaradan tek tık.
///
/// Kuyruk önce hizalanıyor: dosya kuyruktaysa "çalan" işareti oraya geçiyor
/// ve kuyruk korunuyor. Değilse kuyruk TEK öğeye indiriliyor — yoksa
/// kullanıcının açtığı film bitince, ilgisiz bir çalma listesinden bir parça
/// başlardı.
pub fn dosyayi_ac(app: &AppHandle, oge: QueueItem) -> Sonuc<()> {
    if let Some(durum) = app.try_state::<QueueState>() {
        if let Ok(mut kuyruk) = durum.0.lock() {
            if !kuyruk.ogeler().iter().any(|o| o.path == oge.path) {
                kuyruk.yerlestir(vec![oge.clone()]);
            }
            kuyruk.yolu_isaretle(&oge.path);
        }
    }

    oynat(app, &oge)
}

pub fn repeat_ayarla(app: &AppHandle, r: Repeat) -> Sonuc<()> {
    let durum = app
        .try_state::<QueueState>()
        .ok_or_else(|| Hata::yeni("kuyruk hazır değil"))?;
    durum
        .0
        .lock()
        .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?
        .repeat_ayarla(r);
    yayinla(app);
    Ok(())
}

pub fn shuffle_ayarla(app: &AppHandle, acik: bool) -> Sonuc<()> {
    let durum = app
        .try_state::<QueueState>()
        .ok_or_else(|| Hata::yeni("kuyruk hazır değil"))?;
    durum
        .0
        .lock()
        .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?
        .shuffle_ayarla(acik);
    yayinla(app);
    Ok(())
}

/// Kuyruğun anlık hâlini arayüze gönderir.
pub fn yayinla(app: &AppHandle) {
    let Some(durum) = app.try_state::<QueueState>() else {
        return;
    };
    let Ok(kuyruk) = durum.0.lock() else {
        return;
    };
    let _ = app.emit(OLAY_KUYRUK, kuyruk.anlik());
}

/// Yoldan kuyruk öğesi üretir.
///
/// Dosya kütüphanedeyse künyesi oradan geliyor (başlık, süre, tür). Değilse
/// yoldan kuruluyor: sürüklenip bırakılan ya da üstüne çift tıklanan bir
/// dosya kütüphanede olmak zorunda değil, ama çalabilmeli.
///
/// Burada, `commands/`de değil: aynı öğeyi hem `player_open` hem de
/// dışarıdan gelen yollar (`acilis::yollari_ac`) üretiyor ve iki kopya
/// birbirinden sessizce ayrışırdı.
pub fn oge_uret(app: &AppHandle, yol: &str) -> QueueItem {
    let kayit = app
        .try_state::<LibraryState>()
        .and_then(|k| kilit(&k).ok().and_then(|db| db.yol_ile(yol).ok()).flatten());

    match kayit {
        Some(m) => QueueItem::from(&m),
        None => QueueItem {
            id: crate::library::kimlik(yol),
            path: yol.to_string(),
            title: std::path::Path::new(yol)
                .file_stem()
                .map(|a| a.to_string_lossy().into_owned())
                .unwrap_or_else(|| yol.to_string()),
            duration: 0.0,
            // Tür bilinmiyor; mpv dosyayı açtığında `player://file-loaded`
            // gerçeğini söyleyecek. Buradaki değer yalnız kuyruk satırındaki
            // ikon için ve "video" daha az yanıltıcı bir varsayılan.
            media_type: "video".to_string(),
        },
    }
}
