//! Klasör tarayıcı.
//!
//! Tarama kendi iş parçacığında dönüyor ve ilerlemeyi olay olarak yayınlıyor.
//! Eşzamansız olmasının sebebi süre: bin dosyalık bir klasörde her dosya
//! [`Sonda`] ile açılıyor, bu dakikalar alabiliyor. Komut dönene kadar
//! beklemek arayüzü o kadar süre dondururdu.
//!
//! Tarama üç geçiş:
//!
//! 1. **Say.** Klasörleri gez, aday dosyaları topla. Toplam sayı bilinmeden
//!    ilerleme çubuğu çizilemez, o yüzden ayrı bir geçiş.
//! 2. **Yaz.** Değişmemiş dosyaları (`mtime` aynı) hiç açmadan atla,
//!    diğerlerini ölç ve veritabanına yaz.
//! 3. **Ayıkla.** Veritabanında olup diskte olmayan kayıtları sil.
//!
//! Üçüncü geçiş şart: kullanıcı bir dosyayı silince ızgarada tıklandığında
//! "dosya bulunamadı" diyen bir kart kalıyordu.

use std::path::Path;
use std::sync::atomic::Ordering;

use tauri::{Emitter, Manager};
use walkdir::WalkDir;

use crate::hata::{Hata, Sonuc};
use crate::mpv::sonda::Sonda;

use super::db::{kilit, YeniMedya};
use super::{kimlik, tur, LibraryState, OLAY_TARAMA, OLAY_TARAMA_BITTI};

/// Kaç dosyada bir ilerleme olayı yayınlanacak.
///
/// Her dosyada yayınlamak, hızlı atlanan (değişmemiş) dosyalarda saniyede
/// yüzlerce olay demek; arayüz o kadar sık yeniden çizilmeyi hak etmiyor.
const ADIM: usize = 5;

/// Kaç kayıt bir arada yazılacak.
///
/// Her kaydı tek başına yazmak kayıt başına bir işlem kesinleştirmesi
/// demekti. Yığın küçük çünkü yazma kilidi o süre boyunca tutuluyor ve
/// arayüz tarama sırasında da kütüphaneyi sorabilmeli; 64 kayıt göz açıp
/// kapayana kadar yazılıyor.
const YIGIN: usize = 64;

/// Taramayı başlatır. Hemen dönüyor; iş arka planda.
pub fn baslat(app: tauri::AppHandle) -> Sonuc<()> {
    let durum = app
        .try_state::<LibraryState>()
        .ok_or_else(|| Hata::yeni("kütüphane hazır değil"))?;

    // `swap`: "kapalıysa aç ve bana kapalı olduğunu söyle". İki taramanın
    // arasına sıkışmayı imkânsız kılıyor — `load` + `store` ile arada bir
    // pencere kalırdı.
    if durum.taraniyor.swap(true, Ordering::SeqCst) {
        return Err(Hata::yeni("tarama zaten sürüyor"));
    }

    std::thread::Builder::new()
        .name("muiply-tarama".into())
        .spawn(move || {
            let sonuc = calis(&app);
            if let Some(durum) = app.try_state::<LibraryState>() {
                durum.taraniyor.store(false, Ordering::SeqCst);
            }
            if let Err(e) = sonuc {
                let _ = app.emit(crate::mpv::OLAY_HATA, e.to_string());
            }
        })
        .map_err(|e| Hata::yeni(format!("tarama iş parçacığı başlatılamadı: {e}")))?;

    Ok(())
}

fn calis(app: &tauri::AppHandle) -> Sonuc<()> {
    let durum = app
        .try_state::<LibraryState>()
        .ok_or_else(|| Hata::yeni("kütüphane hazır değil"))?;

    let klasorler = kilit(&durum)?.klasorler()?;

    // 1. geçiş — say
    let mut adaylar: Vec<(String, i64, i64, String)> = Vec::new(); // yol, mtime, boyut, uzantı
    for kok in &klasorler {
        adaylar.extend(adaylari_topla(kok));
    }

    // İç içe kökler (kullanıcı hem `C:\A`yı hem `C:\A\B`yi eklediyse) aynı
    // dosyayı iki kez getiriyor. Tekrarı burada atıyoruz: aynı dosyayı iki
    // kez sondayla açmak taramanın en pahalı işini boşuna tekrarlamak,
    // ayrıca ilerleme çubuğunun toplamını da şişiriyordu.
    adaylar.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    adaylar.dedup_by(|a, b| a.0 == b.0);

    let toplam = adaylar.len();
    ilerleme(app, 0, toplam);

    // Sonda motorsuz derlemede kurulamıyor. Tarama yine de çalışıyor:
    // kayıtlar süresiz/boyutsuz yazılıyor. Kütüphaneyi hiç kurmamaktansa
    // eksik künyeyle kurmak yeğ.
    let sonda = Sonda::yeni().ok();

    // 2. geçiş — yaz
    //
    // Kayıtlı değişme zamanları TEK sorguda alınıyor; ikinci bir taramada
    // dosyaların çoğu değişmemiş oluyor ve o durumda hiç kilit alınmıyor.
    let bilinen = kilit(&durum)?.mtimeler()?;

    let mut eklenen = 0u32;
    let mut guncellenen = 0u32;
    let mut yigin: Vec<YeniMedya> = Vec::with_capacity(YIGIN);

    for (i, (yol, mtime, boyut, uzanti)) in adaylar.iter().enumerate() {
        let onceki = bilinen.get(yol).copied();

        match onceki {
            // Dosya değişmemiş: açmaya değmez.
            Some(m) if m == *mtime => {}
            _ => {
                let yeni = onceki.is_none();
                let olcum = sonda.as_ref().and_then(|s| s.olc(yol)).unwrap_or_default();
                let medya_turu = tur(uzanti).unwrap_or("video").to_string();

                yigin.push(YeniMedya {
                    id: kimlik(yol),
                    path: yol.clone(),
                    // Etiketteki başlık dosya adından daha iyi OLMAYABİLİR:
                    // kırpılmış, "Track 01" gibi ya da yanlış olabiliyor.
                    // Ses için etiket, video için dosya adı tercih ediliyor.
                    title: baslik(&medya_turu, &olcum, yol),
                    artist: olcum.artist.clone(),
                    album: olcum.album.clone(),
                    duration: olcum.duration,
                    width: olcum.width,
                    height: olcum.height,
                    size: *boyut,
                    media_type: medya_turu,
                    extension: uzanti.clone(),
                    mtime: *mtime,
                });

                if yeni {
                    eklenen += 1;
                } else {
                    guncellenen += 1;
                }
            }
        }

        if yigin.len() >= YIGIN {
            kilit(&durum)?.yaz_toplu(&yigin)?;
            yigin.clear();
        }

        if (i + 1) % ADIM == 0 || i + 1 == toplam {
            ilerleme(app, i + 1, toplam);
        }
    }

    // Son yığın: döngü tam YIGIN katında bitmediyse artakalanlar burada.
    kilit(&durum)?.yaz_toplu(&yigin)?;

    // 3. geçiş — ayıkla
    let mut silinen = 0u32;
    for kok in &klasorler {
        let kayitlar = kilit(&durum)?.kokteki_kayitlar(kok)?;
        for (id, yol) in kayitlar {
            if !Path::new(&yol).exists() {
                kilit(&durum)?.sil(&id)?;
                silinen += 1;
            }
        }
    }

    let _ = app.emit(
        OLAY_TARAMA_BITTI,
        serde_json::json!({
            "added": eklenen,
            "updated": guncellenen,
            "removed": silinen,
        }),
    );
    Ok(())
}

/// Başlık seçimi.
///
/// Videoda dosya adı kazanıyor: "Film.2019.1080p.mkv" kullanıcının dosyayı
/// tanıdığı ad, konteynerdeki `title` etiketi ise çoğu zaman ya boş ya da
/// kodlayanın bıraktığı çöp. Seste tersi: etiket doğru, dosya adı "01.mp3".
fn baslik(medya_turu: &str, olcum: &crate::mpv::sonda::Olcum, yol: &str) -> String {
    let dosya = dosya_adi(yol);
    if medya_turu == "audio" {
        olcum.title.clone().unwrap_or(dosya)
    } else {
        dosya
    }
}

fn dosya_adi(yol: &str) -> String {
    Path::new(yol)
        .file_stem()
        .map(|a| a.to_string_lossy().into_owned())
        .unwrap_or_else(|| yol.to_string())
}

/// Bir kökün altındaki medya dosyalarını toplar.
///
/// Sembolik bağlar İZLENMİYOR (`walkdir` varsayılanı): kendine dönen bir bağ
/// taramayı sonsuz döngüye sokardı. Okunamayan girdiler sessizce atlanıyor —
/// izin verilmeyen bir alt klasör yüzünden bütün taramayı durdurmak orantısız.
fn adaylari_topla(kok: &str) -> Vec<(String, i64, i64, String)> {
    let mut bulunan = Vec::new();

    for girdi in WalkDir::new(kok).follow_links(false).into_iter().flatten() {
        if !girdi.file_type().is_file() {
            continue;
        }
        let yol = girdi.path();
        let Some(uzanti) = yol.extension().map(|u| u.to_string_lossy().to_lowercase()) else {
            continue;
        };
        if tur(&uzanti).is_none() {
            continue;
        }

        let Ok(bilgi) = girdi.metadata() else {
            continue;
        };
        let mtime = bilgi
            .modified()
            .ok()
            .and_then(|z| z.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        bulunan.push((
            yol.to_string_lossy().into_owned(),
            mtime,
            bilgi.len() as i64,
            uzanti,
        ));
    }

    bulunan
}

fn ilerleme(app: &tauri::AppHandle, yapilan: usize, toplam: usize) {
    let _ = app.emit(
        OLAY_TARAMA,
        serde_json::json!({ "done": yapilan, "total": toplam }),
    );
}
