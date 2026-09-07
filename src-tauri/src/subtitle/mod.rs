//! Altyazı — dosyanın yanındakileri bulmak ve mpv'ye vermek.
//!
//! Gömülü izler için burada iş yok: mpv onları `track-list` içinde zaten
//! veriyor (`mpv::izleri_oku`). Bu modülün tek konusu **harici** dosyalar.
//!
//! mpv'nin kendi `sub-auto` taraması KAPALI (`mpv/gercek.rs`). Sebebi:
//! `sub-auto=exact` yalnız `film.srt`i alıyor, `sub-auto=fuzzy` ise aynı
//! klasördeki *bütün* altyazıları — on filmlik bir klasörde her filme on
//! altyazı ekliyor. Aradaki doğru davranış "aynı adla başlayanlar", ve o
//! mpv'de yok.

use std::path::{Path, PathBuf};

use serde::Serialize;

/// Aradığımız uzantılar.
///
/// `idx`/`sub` (VobSub) çifti listede yok: ikisi birlikte çalışıyor ve
/// yalnız `.sub` eklemek mpv'de sessizce boş bir iz üretiyor. mpv `.idx`
/// verildiğinde ikisini birden alıyor, o yüzden `idx` listede.
pub const ALTYAZI_UZANTILARI: &[&str] = &["srt", "ass", "ssa", "vtt", "sub", "idx"];

/// Video dosyasının yanında bulunan bir altyazı.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NearbySubtitle {
    pub path: String,
    /// Arayüzde gösterilecek ad: `film.tr.srt` → `tr · srt`, yoksa dosya adı.
    pub label: String,
    pub lang: Option<String>,
}

/// Video dosyasının yanındaki altyazıları bulur.
///
/// Kural: aynı klasörde, adı videonun adıyla BAŞLAYAN ve uzantısı altyazı
/// olan dosyalar. `film.mkv` için `film.srt`, `film.tr.srt`, `film.en.ass`
/// geliyor; `film2.srt` gelmiyor (ad eşleşmesi tam kök üstünden, ardından
/// nokta ya da dize sonu aranıyor).
///
/// Klasör tek seferde okunuyor. `docs/Subtitles.md`'deki ilk taslak her
/// uzantı için ayrı bir `read_dir` yapıyordu ve tam eşleşen dosyayı listeye
/// iki kez koyuyordu — arayüzde aynı altyazı iki satır olarak görünürdü.
pub fn yanindakiler(video: &Path) -> Vec<NearbySubtitle> {
    let Some(kok) = video.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
        return Vec::new();
    };
    let Some(klasor) = video.parent() else {
        return Vec::new();
    };
    let Ok(girdiler) = std::fs::read_dir(klasor) else {
        return Vec::new();
    };

    let mut bulunan: Vec<NearbySubtitle> = Vec::new();

    for girdi in girdiler.flatten() {
        let yol = girdi.path();
        if !yol.is_file() {
            continue;
        }

        let Some(uzanti) = yol.extension().map(|u| u.to_string_lossy().to_lowercase()) else {
            continue;
        };
        if !ALTYAZI_UZANTILARI.contains(&uzanti.as_str()) {
            continue;
        }

        let Some(ad_kok) = yol.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
            continue;
        };
        let Some(ek) = eslesme_eki(&ad_kok, &kok) else {
            continue;
        };

        bulunan.push(NearbySubtitle {
            path: yol.to_string_lossy().into_owned(),
            label: etiket(&ek, &uzanti, &ad_kok),
            lang: dil_kodu(&ek),
        });
    }

    // Sıra kararlı olmalı: `read_dir` işletim sistemine göre değişiyor ve
    // ilk sıradaki altyazı otomatik seçildiği için sıranın değişmesi
    // "her açışta başka dil geliyor" demek olurdu.
    bulunan.sort_by(|a, b| a.path.cmp(&b.path));
    bulunan
}

/// `ad_kok` videonun köküyle başlıyorsa geri kalanını döner.
///
/// `film` + `film.tr` → `tr`; `film` + `film` → `""`; `film` + `film2` →
/// `None` (kökten sonra nokta gelmediği için eşleşme değil).
///
/// Karşılaştırma BÜYÜK/KÜÇÜK HARF AYIRMIYOR. Windows ve macOS dosya
/// sistemleri de ayırmıyor: kullanıcının klasöründe `Film.mkv` ile
/// `film.tr.srt` yan yana durabiliyor ve bunlar aynı filmin dosyaları.
/// Harfe duyarlı karşılaştırma o altyazıyı sessizce ıskalıyordu — hata da
/// vermiyordu, altyazı iz listesinde hiç görünmüyordu.
///
/// Dönen ek ORİJİNAL yazımıyla veriliyor: etikette ve dil kodunda
/// kullanıcının dosyaya yazdığı hâl görünmeli.
fn eslesme_eki(ad_kok: &str, video_kok: &str) -> Option<String> {
    let kalan = onek_kirp(ad_kok, video_kok)?;
    if kalan.is_empty() {
        Some(String::new())
    } else {
        kalan.strip_prefix('.').map(|s| s.to_string())
    }
}

/// `ad`ın başındaki `onek`i harf ayrımı gözetmeden kırpar.
///
/// Karşılaştırma KARAKTER karakter yürüyor, iki dizeyi küçültüp
/// karşılaştırarak değil. Sebebi Unicode: küçültme uzunluğu
/// değiştirebiliyor (`'İ'.to_lowercase()` iki kod noktası veriyor) ve
/// küçültülmüş dizede bulunan eşleşme uzunluğu ORİJİNAL dizede başka bir
/// yere düşüyor. Kesme noktasını orijinalin kendi indekslerinden almak,
/// ekin yanlış yerden kesilmesini imkânsız kılıyor.
fn onek_kirp<'a>(ad: &'a str, onek: &str) -> Option<&'a str> {
    let mut adimlar = ad.char_indices();
    for beklenen in onek.chars() {
        let (_, gelen) = adimlar.next()?;
        if !ayni_harf(gelen, beklenen) {
            return None;
        }
    }
    // Ön ek bittikten sonraki ilk karakterin başlangıcı; ad tam orada
    // bitmişse dizenin sonu.
    let kalan_baslangic = adimlar.next().map(|(i, _)| i).unwrap_or(ad.len());
    Some(&ad[kalan_baslangic..])
}

/// İki karakter aynı harfin büyüğü/küçüğü mü.
///
/// Türkçe çiftlerini (Ö/ö, Ç/ç, Ş/ş, Ğ/ğ, Ü/ü) Unicode küçültmesi doğru
/// eşliyor. `İ` ile `i` eşleşmiyor ve bu burada DOĞRU: Türkçede `İ`nin
/// küçüğü `i`, `I`nın küçüğü `ı` ve Unicode'un dilden bağımsız tablosu bunu
/// bilmiyor. Dosya adı eşleştirmesinde yanlış eşleme ıskalanandan kötü —
/// başka bir filmin altyazısını yüklerdi.
fn ayni_harf(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

/// Ekten iki/üç harfli dil kodu çıkarır.
///
/// `tr`, `en`, `tur` gibi. `forced` ya da `sdh` gibi ekler dil değil, `None`
/// dönüyor ve etikette ham hâliyle görünüyorlar.
fn dil_kodu(ek: &str) -> Option<String> {
    let ilk = ek.split('.').next()?.to_lowercase();
    let uzunluk = ilk.chars().count();
    if (2..=3).contains(&uzunluk) && ilk.chars().all(|c| c.is_ascii_alphabetic()) {
        Some(ilk)
    } else {
        None
    }
}

fn etiket(ek: &str, uzanti: &str, ad_kok: &str) -> String {
    if ek.is_empty() {
        format!("{ad_kok}.{uzanti}")
    } else {
        format!("{ek} · {uzanti}")
    }
}

/// Bulunan altyazıları mpv'ye ekler.
///
/// `auto` bayrağı: ekle, ama zaten seçili bir altyazı varsa seçme. İlk
/// eklenen seçiliyor — sıra [`yanindakiler`] içinde kararlı, yani "hangi dil
/// açılıyor" sorusunun yanıtı her açılışta aynı.
///
/// Hata YUTULUYOR: bozuk bir `.srt` yüzünden dosyanın kendisi açılmasın
/// istemiyoruz. Eklenemeyen altyazı iz listesinde görünmüyor, o kadar.
pub fn otomatik_yukle(motor: &crate::mpv::Motor, video_yolu: &str) {
    for altyazi in yanindakiler(Path::new(video_yolu)) {
        let _ = motor.komut("sub-add", &[&altyazi.path, "auto"]);
    }
}

/// Kullanıcının elle seçtiği bir altyazı dosyası. `select`: hemen görünsün,
/// çünkü kullanıcı onu görmek için seçti.
pub fn elle_ekle(motor: &crate::mpv::Motor, yol: &str) -> crate::hata::Sonuc<()> {
    if !PathBuf::from(yol).is_file() {
        return Err(crate::hata::Hata::yeni(format!(
            "altyazı dosyası bulunamadı: {yol}"
        )));
    }
    motor.komut("sub-add", &[yol, "select"])
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn eslesme_noktadan_sonrasini_alir() {
        assert_eq!(eslesme_eki("film", "film"), Some(String::new()));
        assert_eq!(eslesme_eki("film.tr", "film"), Some("tr".into()));
        assert_eq!(
            eslesme_eki("film.tr.forced", "film"),
            Some("tr.forced".into())
        );
    }

    #[test]
    fn buyuk_kucuk_harf_ayirmiyor() {
        // Windows ve macOS'ta bunlar aynı filmin dosyaları.
        assert_eq!(eslesme_eki("Film.tr", "film"), Some("tr".into()));
        assert_eq!(eslesme_eki("film.EN", "FILM"), Some("EN".into()));
        // Eşleşse de ek orijinal yazımıyla dönüyor: etiket kullanıcının
        // dosyaya yazdığını göstermeli.
        assert_eq!(
            eslesme_eki("BOLUM1.Tur.Forced", "bolum1"),
            Some("Tur.Forced".into())
        );
        // Ayrım kalkarken "başka dosya" kuralı bozulmamalı.
        assert_eq!(eslesme_eki("FILM2", "film"), None);
    }

    /// [`yanindakiler`] gerçek bir klasör üzerinde.
    ///
    /// Ayrı bir test çünkü [`eslesme_eki`] saf ve dosya sistemini görmüyor:
    /// uzantı süzgeci, klasör okuma ve sıralama ancak burada sınanıyor.
    /// Asıl korunan şey büyük/küçük harf ayrımının KALKTIĞInın uçtan uca
    /// doğrulanması — birim testi geçip bütün yol yine de ıskalayabilirdi.
    #[test]
    fn yanindakiler_gercek_klasorde_calisiyor() {
        use std::fs;

        // `env::temp_dir` + süreç kimliği: testler paralel çalışıyor ve sabit
        // bir ad iki testin aynı klasörü silmesi demek olurdu.
        let klasor = std::env::temp_dir().join(format!("muiply-altyazi-{}", std::process::id()));
        let _ = fs::remove_dir_all(&klasor);
        fs::create_dir_all(&klasor).expect("geçici klasör açılamadı");

        let video = klasor.join("Deneme Filmi.mkv");
        fs::write(&video, b"x").unwrap();
        // Adı küçük harfle yazılmış altyazı: Windows ve macOS'ta bu ikisi
        // kullanıcı için aynı filmin dosyaları.
        fs::write(klasor.join("deneme filmi.tr.srt"), b"x").unwrap();
        fs::write(klasor.join("Deneme Filmi.en.ass"), b"x").unwrap();
        // Başka bir filmin altyazısı ve altyazı olmayan bir dosya: ikisi de
        // listeye girmemeli.
        fs::write(klasor.join("Deneme Filmi 2.tr.srt"), b"x").unwrap();
        fs::write(klasor.join("Deneme Filmi.txt"), b"x").unwrap();

        let bulunan = yanindakiler(&video);
        let diller: Vec<Option<String>> = bulunan.iter().map(|a| a.lang.clone()).collect();

        assert_eq!(
            bulunan.len(),
            2,
            "beklenen iki altyazı, bulunan: {:?}",
            bulunan.iter().map(|a| &a.path).collect::<Vec<_>>()
        );
        // Sıra yola göre kararlı: ".en.ass" ".tr.srt"ten önce geliyor.
        assert_eq!(diller, vec![Some("en".to_string()), Some("tr".to_string())]);

        let _ = fs::remove_dir_all(&klasor);
    }

    #[test]
    fn turkce_harflerde_de_ayrim_yok() {
        // Unicode'un doğru eşlediği çiftler: kullanıcının klasöründe
        // "Çıkış.mkv" ile "çıkış.tr.srt" yan yana olabiliyor.
        assert_eq!(eslesme_eki("çıkış.tr", "Çıkış"), Some("tr".into()));
        assert_eq!(eslesme_eki("GÖLGE.en", "gölge"), Some("en".into()));
        assert_eq!(eslesme_eki("şuğüm", "ŞUĞÜM"), Some(String::new()));
    }

    #[test]
    fn nokta_i_ile_duz_i_karistirilmiyor() {
        // Türkçede 'İ'nin küçüğü 'i', 'I'nın küçüğü 'ı'. Unicode bunu
        // bilmiyor, o yüzden bu çiftler eşleşmiyor — ve eşleşmemeleri
        // doğru: yanlış eşleme başka bir filmin altyazısını yüklerdi.
        assert_eq!(eslesme_eki("islem.tr", "İslem"), None);
        // Kesme noktası orijinalden alındığı için ek yine de bozulmuyor:
        // aynı yazımla eşleşen dosya doğru ek veriyor.
        assert_eq!(eslesme_eki("İslem.tr", "İslem"), Some("tr".into()));
    }

    #[test]
    fn baska_filmin_altyazisi_eslesmez() {
        // En sık karışan durum: aynı klasörde "Bolum1" ve "Bolum10".
        assert_eq!(eslesme_eki("Bolum10", "Bolum1"), None);
        assert_eq!(eslesme_eki("film2", "film"), None);
    }

    #[test]
    fn dil_kodu_yalniz_harf_ve_iki_uc_karakter() {
        assert_eq!(dil_kodu("tr"), Some("tr".into()));
        assert_eq!(dil_kodu("tur.forced"), Some("tur".into()));
        // "forced" dil değil: etikette ham kalmalı.
        assert_eq!(dil_kodu("forced"), None);
        assert_eq!(dil_kodu("2"), None);
        assert_eq!(dil_kodu(""), None);
    }

    #[test]
    fn etiket_eksizken_dosya_adini_gosterir() {
        assert_eq!(etiket("", "srt", "film"), "film.srt");
        assert_eq!(etiket("tr", "srt", "film.tr"), "tr · srt");
    }
}
