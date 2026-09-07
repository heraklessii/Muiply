//! Uygulamaya DIŞARIDAN gelen dosya yolları.
//!
//! Üç kapı aynı yere çıkıyor: pencereye sürükle-bırak (`app_open_paths`),
//! açılışta komut satırı (Windows'ta "birlikte aç" ve varsayılan oynatıcı
//! tam olarak bunu kullanıyor) ve uygulama zaten açıkken çift tıklanan
//! ikinci dosya. Üçünün isteği aynı, bu yüzden karar tek yerde.
//!
//! `commands/` içinde duramazdı: oraya iş mantığı yazılmıyor ve buraya
//! arayüz hiç açılmadan da geliniyor — çift tıklanan dosya, pencere
//! boyanmadan önce kuyruğa giriyor.

use tauri::{AppHandle, Manager};

use crate::hata::{Hata, Sonuc};
use crate::library::db::kilit;
use crate::library::LibraryState;
use crate::playlist::kuyruk::QueueItem;
use crate::playlist::surucu;

/// Klasörleri kütüphaneye alır, dosyaları kuyruğa koyup ilkini çalar.
///
/// "Klasör mü dosya mı" kararı burada çünkü dosya sistemine bakmayı
/// gerektiriyor ve arayüzün dosya sistemine erişimi yok
/// (`capabilities/default.json`). Uzantıya bakıp tahmin etmek "My.Videos"
/// adlı bir klasörde yanlış cevap verirdi.
///
/// Karışık giriş (bir klasör + iki dosya) ikisini de yapıyor: klasör
/// kütüphaneye giriyor, dosyalar kuyruğa alınıp ilki çalıyor.
pub fn yollari_ac(app: &AppHandle, yollar: Vec<String>) -> Sonuc<()> {
    let mut klasor_eklendi = false;
    let mut dosyalar: Vec<String> = Vec::new();

    for yol in yollar {
        let p = std::path::Path::new(&yol);
        if p.is_dir() {
            let kutuphane = app
                .try_state::<LibraryState>()
                .ok_or_else(|| Hata::yeni("kütüphane hazır değil"))?;
            kilit(&kutuphane)?.klasor_ekle(&yol)?;
            klasor_eklendi = true;
        } else if p.is_file() {
            // Uzantı SÜZÜLMÜYOR: kullanıcı bir dosyayı bilerek bıraktıysa
            // (ya da üstüne çift tıkladıysa) açmayı denemek doğru. Tarama
            // listesi (library/mod.rs) dar çünkü orada kimse bir şey
            // istemiyor; burada isteyen var.
            dosyalar.push(yol);
        }
    }

    if klasor_eklendi {
        crate::library::tarayici::baslat(app.clone())?;
    }

    if dosyalar.is_empty() {
        return Ok(());
    }

    let ogeler: Vec<QueueItem> = dosyalar.iter().map(|y| surucu::oge_uret(app, y)).collect();
    crate::commands::playlist::kuyruga_koy(app, ogeler)?;
    surucu::indeksten_oynat(app, 0)
}

/// Komut satırı argümanlarından dosya yollarını süzer.
///
/// Saf ve testli, çünkü buranın yanlışı gözle görünmüyor: bir bayrağı yol
/// sanmak açılışta anlamsız bir hata satırı, bir yolu bayrak sanmak ise
/// kullanıcının çift tıkladığı filmin hiç açılmaması demek.
///
/// - İlk argüman (programın kendi yolu) atlanıyor.
/// - `-` ile başlayan her şey atılıyor: WebView2, GTK ve `tauri dev` kendi
///   bayraklarını ekliyor.
/// - Yolun VAR OLUP OLMADIĞINA bakılmıyor; o karar [`yollari_ac`]'ın, tek
///   yerde kalsın diye.
pub fn dosya_argumanlari<I>(argv: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    argv.into_iter()
        .skip(1)
        .filter(|a| !a.is_empty() && !a.starts_with('-'))
        .collect()
}

/// Açılışta komut satırından gelen dosyaları açar.
///
/// Hata DURDURMUYOR: yol yanlışsa ya da dosya silinmişse Muiply boş açılır.
/// Alternatifi, çift tıklanan bir dosya yüzünden oynatıcının hiç
/// gelmemesiydi.
///
/// Arayüz bu sırada henüz bağlı değil ve `playlist://queue` olayını
/// kaçırıyor. Sorun değil: kancalar ilk boyamada `queue_get` ve
/// `player_get_state` çağırıp durumu backend'den okuyor (`src/hooks/`).
/// Dönen değer: komut satırında açılacak bir şey var mıydı. Açılışta hangi
/// pencerenin gösterileceğini bu belirliyor — dosyayla açıldıysa oynatıcı,
/// düz açıldıysa kütüphane (`lib.rs`).
pub fn baslangicta_ac(app: &AppHandle) -> bool {
    // `args_os` + lossy: argümanı düşürmek yerine bozuk çeviriyi yeğliyoruz,
    // çünkü düşen argüman sıradakini programın yolu sanmak demek olurdu.
    let yollar = dosya_argumanlari(std::env::args_os().map(|a| a.to_string_lossy().into_owned()));
    let var = !yollar.is_empty();
    ac_ve_bildir(app, yollar);
    var
}

/// Uygulama açıkken çift tıklanan ikinci dosya.
///
/// Tekil örnek eklentisi ikinci süreci sonlandırıp argümanlarını buraya
/// veriyor. Pencere öne alınıyor: bir dosyaya çift tıklayan kullanıcı bir
/// şeyin AÇILMASINI bekliyor, arkada sessizce değişen bir kuyruk
/// "tıklamam çalışmadı" gibi görünürdü.
pub fn ikinci_ornek(app: &AppHandle, argv: Vec<String>) {
    crate::pencere::goster(app, crate::pencere::OYNATICI);
    ac_ve_bildir(app, dosya_argumanlari(argv));
}

fn ac_ve_bildir(app: &AppHandle, yollar: Vec<String>) {
    if yollar.is_empty() {
        return;
    }
    if let Err(e) = yollari_ac(app, yollar) {
        eprintln!("muiply: dışarıdan gelen yol açılamadı ({e})");
    }
}

#[cfg(test)]
mod testler {
    use super::dosya_argumanlari;

    fn dizi(ogeler: &[&str]) -> Vec<String> {
        ogeler.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn program_yolu_atlaniyor() {
        let cikan = dosya_argumanlari(dizi(&[r"C:\Muiply\muiply.exe", r"D:\film.mkv"]));
        assert_eq!(cikan, vec![r"D:\film.mkv".to_string()]);
    }

    #[test]
    fn argumansiz_acilis_bos_donuyor() {
        assert!(dosya_argumanlari(dizi(&["muiply.exe"])).is_empty());
        assert!(dosya_argumanlari(Vec::<String>::new()).is_empty());
    }

    #[test]
    fn bayraklar_atiliyor() {
        let cikan = dosya_argumanlari(dizi(&[
            "muiply.exe",
            "--no-sandbox",
            r"D:\film.mkv",
            "-v",
            "",
        ]));
        assert_eq!(cikan, vec![r"D:\film.mkv".to_string()]);
    }

    #[test]
    fn bosluklu_yol_tek_parca_kaliyor() {
        // İşletim sistemi tırnakları zaten çözüyor; burada bölmemek gerek.
        let yol = r"D:\Filmler\Kill Bill Vol. 1.mkv";
        assert_eq!(
            dosya_argumanlari(dizi(&["muiply.exe", yol])),
            vec![yol.to_string()]
        );
    }

    #[test]
    fn birden_cok_dosya_sirasini_koruyor() {
        let cikan = dosya_argumanlari(dizi(&["muiply.exe", "a.mp3", "b.mp3", "c.mp3"]));
        assert_eq!(cikan, dizi(&["a.mp3", "b.mp3", "c.mp3"]));
    }
}
