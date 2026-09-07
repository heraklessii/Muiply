//! Kuyruk — o an çalan sıra.
//!
//! Kuyruk backend'de duruyor, arayüzde değil. Sebep tek bir cümlede: dosya
//! bittiğinde sıradakine geçme kararını **mpv'nin olay döngüsü** veriyor
//! (`mpv/gercek.rs` → `EndFile(eof)`), ve o karar arayüzün açık olup
//! olmamasından bağımsız olmalı.
//!
//! Gezinme kararları saf fonksiyonlar ([`sonraki`], [`onceki`], [`karistir`])
//! ve testli. Bozulduğunda ortaya çıkan hata sessiz türden: "liste sonunda
//! başa dönmüyor", "karışık modda aynı parça iki kez çalıyor". Test o
//! sessizliği bozmak için var.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::library::MediaItem;

/// Kuyruktaki bir öğe. [`MediaItem`]'ın oynatma için gereken alt kümesi —
/// kuyruk her değiştiğinde bütün künyeyi arayüze taşımanın karşılığı yok.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: String,
    pub path: String,
    pub title: String,
    pub duration: f64,
    pub media_type: String,
}

impl From<&MediaItem> for QueueItem {
    fn from(m: &MediaItem) -> Self {
        QueueItem {
            id: m.id.clone(),
            path: m.path.clone(),
            title: m.title.clone(),
            duration: m.duration,
            media_type: m.media_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Repeat {
    /// Liste bitince durur.
    #[default]
    Off,
    /// Liste bitince başa döner.
    All,
    /// Aynı dosyayı tekrarlar.
    One,
}

/// Arayüze giden kuyruk görüntüsü.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub items: Vec<QueueItem>,
    pub current_index: Option<usize>,
    pub repeat: Repeat,
    pub shuffle: bool,
}

pub struct Kuyruk {
    ogeler: Vec<QueueItem>,
    /// [`Self::ogeler`] içine indeks.
    simdiki: Option<usize>,
    repeat: Repeat,
    shuffle: bool,
    /// Gezinme sırası: `ogeler` indekslerinin bir permütasyonu. Karışık
    /// kapalıyken kimlik permütasyonu (0,1,2...).
    ///
    /// Ayrı bir dizi tutmanın sebebi: karışık açıldığında listenin KENDİSİ
    /// karışmamalı. Kullanıcı listede gördüğü sırayı kaybetmemeli, yalnız
    /// gezinme sırası değişmeli.
    sira: Vec<usize>,
}

pub struct QueueState(pub Mutex<Kuyruk>);

impl Default for Kuyruk {
    fn default() -> Self {
        Kuyruk {
            ogeler: Vec::new(),
            simdiki: None,
            repeat: Repeat::Off,
            shuffle: false,
            sira: Vec::new(),
        }
    }
}

impl Kuyruk {
    pub fn anlik(&self) -> QueueSnapshot {
        QueueSnapshot {
            items: self.ogeler.clone(),
            current_index: self.simdiki,
            repeat: self.repeat,
            shuffle: self.shuffle,
        }
    }

    /// Kuyruğu baştan kurar. Çalan öğe sıfırlanıyor: yeni bir kuyruk yeni bir
    /// oturum.
    pub fn yerlestir(&mut self, ogeler: Vec<QueueItem>) {
        self.ogeler = ogeler;
        self.simdiki = None;
        self.sirayi_kur();
    }

    pub fn ogeler(&self) -> &[QueueItem] {
        &self.ogeler
    }

    pub fn simdiki(&self) -> Option<&QueueItem> {
        self.simdiki.and_then(|i| self.ogeler.get(i))
    }

    /// Verilen indeksi çalınan yapar ve öğeyi döner.
    pub fn sec(&mut self, indeks: usize) -> Option<QueueItem> {
        let oge = self.ogeler.get(indeks)?.clone();
        self.simdiki = Some(indeks);
        Some(oge)
    }

    /// Kuyruktaki bir yolu çalınan olarak işaretler.
    ///
    /// Kullanıcı kütüphaneden doğrudan bir dosya açtığında çağrılıyor: dosya
    /// zaten kuyruktaysa kuyruk yerinde kalsın, "sıradaki" doğru olsun.
    pub fn yolu_isaretle(&mut self, yol: &str) {
        self.simdiki = self.ogeler.iter().position(|o| o.path == yol);
    }

    pub fn ilerle(&mut self, otomatik: bool) -> Option<QueueItem> {
        let hedef = sonraki(&self.sira, self.simdiki, self.repeat, otomatik)?;
        self.sec(hedef)
    }

    pub fn gerile(&mut self) -> Option<QueueItem> {
        let hedef = onceki(&self.sira, self.simdiki, self.repeat)?;
        self.sec(hedef)
    }

    pub fn repeat_ayarla(&mut self, r: Repeat) {
        self.repeat = r;
    }

    pub fn shuffle_ayarla(&mut self, acik: bool) {
        self.shuffle = acik;
        self.sirayi_kur();
    }

    /// Gezinme sırasını yeniden üretir.
    ///
    /// Karışık açıkken çalan öğe sıranın BAŞINA alınıyor: aksi hâlde karışığı
    /// açmak, o an çalan parçayı "zaten çalınmış" saymayıp listenin ortasında
    /// bırakır ve bir sonraki geçişte tekrar ona dönebilirdi.
    fn sirayi_kur(&mut self) {
        let n = self.ogeler.len();
        if !self.shuffle {
            self.sira = (0..n).collect();
            return;
        }

        let mut sira = karistir(n, tohum());
        if let Some(simdiki) = self.simdiki {
            if let Some(k) = sira.iter().position(|&i| i == simdiki) {
                sira.swap(0, k);
            }
        }
        self.sira = sira;
    }
}

/// Sıradaki öğenin indeksi.
///
/// `otomatik`: karar dosya kendiliğinden bittiği için mi veriliyor, kullanıcı
/// "sonraki" dediği için mi. Ayrım [`Repeat::One`] için: dosya bitince aynı
/// dosya baştan çalmalı, ama kullanıcı "sonraki"ye bastığında gerçekten
/// sonrakine gitmeli — yoksa düğme hiçbir şey yapmıyormuş gibi görünür.
pub fn sonraki(
    sira: &[usize],
    simdiki: Option<usize>,
    repeat: Repeat,
    otomatik: bool,
) -> Option<usize> {
    if sira.is_empty() {
        return None;
    }
    if otomatik && repeat == Repeat::One {
        return simdiki;
    }

    let Some(simdiki) = simdiki else {
        return sira.first().copied();
    };
    // Çalan öğe kuyrukta değilse (kullanıcı kütüphaneden başka bir dosya
    // açtı): kuyruğun başından devam et.
    let Some(konum) = sira.iter().position(|&i| i == simdiki) else {
        return sira.first().copied();
    };

    match sira.get(konum + 1) {
        Some(&sonraki) => Some(sonraki),
        // Liste bitti.
        None if repeat != Repeat::Off => sira.first().copied(),
        None => None,
    }
}

/// Bir önceki öğenin indeksi. Listenin başındayken yalnız tekrar açıksa
/// sona sarıyor.
pub fn onceki(sira: &[usize], simdiki: Option<usize>, repeat: Repeat) -> Option<usize> {
    if sira.is_empty() {
        return None;
    }
    let Some(simdiki) = simdiki else {
        return sira.first().copied();
    };
    let Some(konum) = sira.iter().position(|&i| i == simdiki) else {
        return sira.first().copied();
    };

    if konum > 0 {
        sira.get(konum - 1).copied()
    } else if repeat != Repeat::Off {
        sira.last().copied()
    } else {
        None
    }
}

/// İstenen başlangıcı, süzülmüş bir kuyruktaki karşılığına çevirir.
///
/// `queue_set` arayüzün gönderdiği kimliklerin bir kısmını atabiliyor: kayıt
/// arayüzün listeyi çizmesi ile tıklamanın gelmesi arasında silinmiş
/// olabiliyor (tarama üçüncü geçişinde diskten kalkmış dosyaları temizliyor).
/// Atılan her kayıt sonrasındaki indeksleri bir kaydırıyor, yani arayüzün
/// verdiği sayı kuyruğa OLDUĞU GİBİ uygulanamaz — kullanıcı tıkladığından
/// başka bir dosyayı çalardı ve bu sessiz: hata yok, yanlış film var.
///
/// `bulunan`: kuyruğa giren her öğenin, arayüzün gönderdiği listedeki eski
/// indeksi (artan sırada). Dönen değer `istenen`in kuyruktaki karşılığı —
/// tıklanan kayıt bu arada silinmişse ondan SONRAKİ ilk öğe, ondan sonra
/// hiçbir şey kalmamışsa sonuncusu.
pub fn baslangici_esle(bulunan: &[usize], istenen: usize) -> usize {
    match bulunan.iter().position(|&eski| eski >= istenen) {
        Some(k) => k,
        None => bulunan.len().saturating_sub(1),
    }
}

/// `0..n`in bir permütasyonu (Fisher-Yates).
///
/// Kendi üreteci: `rand` bağımlılığı bir çalma listesini karıştırmak için
/// fazla. Buradaki xorshift64 kriptografik değil ve olması da gerekmiyor —
/// istenen tek şey her seferinde farklı, dengeli bir sıra.
pub fn karistir(n: usize, tohum: u64) -> Vec<usize> {
    let mut sira: Vec<usize> = (0..n).collect();
    let mut durum = tohum | 1; // xorshift sıfır tohumda takılı kalıyor

    for i in (1..n).rev() {
        durum ^= durum << 13;
        durum ^= durum >> 7;
        durum ^= durum << 17;
        let j = (durum % (i as u64 + 1)) as usize;
        sira.swap(i, j);
    }
    sira
}

fn tohum() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x2dd4bf)
}

#[cfg(test)]
mod testler {
    use super::*;

    fn duz(n: usize) -> Vec<usize> {
        (0..n).collect()
    }

    #[test]
    fn bos_kuyrukta_gezinme_yok() {
        assert_eq!(sonraki(&[], None, Repeat::All, false), None);
        assert_eq!(onceki(&[], None, Repeat::All), None);
    }

    #[test]
    fn hicbiri_calmiyorken_bastan_baslar() {
        assert_eq!(sonraki(&duz(3), None, Repeat::Off, false), Some(0));
        assert_eq!(onceki(&duz(3), None, Repeat::Off), Some(0));
    }

    #[test]
    fn tekrar_kapaliyken_sonda_durur() {
        assert_eq!(sonraki(&duz(3), Some(2), Repeat::Off, true), None);
        assert_eq!(onceki(&duz(3), Some(0), Repeat::Off), None);
    }

    #[test]
    fn tekrar_hepsi_basa_sarar() {
        assert_eq!(sonraki(&duz(3), Some(2), Repeat::All, true), Some(0));
        assert_eq!(onceki(&duz(3), Some(0), Repeat::All), Some(2));
    }

    #[test]
    fn tekrar_tek_yalniz_otomatik_geciste_ayni_dosya() {
        // Dosya bitti: aynısı yeniden.
        assert_eq!(sonraki(&duz(3), Some(1), Repeat::One, true), Some(1));
        // Kullanıcı "sonraki"ye bastı: gerçekten sonraki.
        assert_eq!(sonraki(&duz(3), Some(1), Repeat::One, false), Some(2));
        // ... ve sondayken başa sarıyor, çünkü tekrar açık.
        assert_eq!(sonraki(&duz(3), Some(2), Repeat::One, false), Some(0));
    }

    #[test]
    fn karisik_sirada_liste_sirasi_degil_gezinme_sirasi_izleniyor() {
        let sira = vec![2, 0, 1];
        assert_eq!(sonraki(&sira, Some(2), Repeat::Off, false), Some(0));
        assert_eq!(sonraki(&sira, Some(0), Repeat::Off, false), Some(1));
        assert_eq!(sonraki(&sira, Some(1), Repeat::Off, false), None);
        assert_eq!(onceki(&sira, Some(0), Repeat::Off), Some(2));
    }

    #[test]
    fn kuyrukta_olmayan_oge_kuyrugun_basina_dusuyor() {
        // Kullanıcı kütüphaneden kuyruk dışı bir dosya açtı, sonra "sonraki".
        assert_eq!(sonraki(&duz(3), Some(99), Repeat::Off, false), Some(0));
    }

    #[test]
    fn hicbiri_kaybolmamissa_indeks_degismiyor() {
        assert_eq!(baslangici_esle(&duz(5), 0), 0);
        assert_eq!(baslangici_esle(&duz(5), 3), 3);
    }

    #[test]
    fn onceki_kayitlar_kaybolunca_indeks_kayiyor() {
        // Arayüz 5 kayıt gönderdi, 0 ve 2 silinmişti: kuyruk [1, 3, 4].
        // Kullanıcı 3'e tıkladı; kuyrukta o artık 1. sırada.
        let bulunan = vec![1, 3, 4];
        assert_eq!(baslangici_esle(&bulunan, 3), 1);
        assert_eq!(baslangici_esle(&bulunan, 4), 2);
        assert_eq!(baslangici_esle(&bulunan, 1), 0);
    }

    #[test]
    fn tiklanan_kayit_silinmisse_sonraki_calar() {
        // 2 silinmiş; ona tıklamak bir sonraki kayda düşüyor.
        assert_eq!(baslangici_esle(&[0, 1, 3], 2), 2);
        // Baştakiler silinmişse ilk kalan.
        assert_eq!(baslangici_esle(&[2, 3], 0), 0);
    }

    #[test]
    fn sondan_sonra_bir_sey_kalmamissa_sonuncu() {
        assert_eq!(baslangici_esle(&[0, 1], 7), 1);
        // Boş liste `queue_set`e ulaşmıyor (önce hata dönüyor); yine de
        // taşmasın.
        assert_eq!(baslangici_esle(&[], 3), 0);
    }

    #[test]
    fn karistirma_permutasyon_uretir() {
        for tohum in [1u64, 42, 0xdeadbeef, 0x2dd4bf] {
            let mut s = karistir(50, tohum);
            s.sort_unstable();
            assert_eq!(s, duz(50), "tohum {tohum} permütasyon üretmedi");
        }
    }

    #[test]
    fn karistirma_sifir_tohumda_da_calisir() {
        // xorshift sıfır durumunda sonsuza kadar sıfır üretir; tohum
        // `| 1` ile korunuyor. Bu test o korumanın bekçisi.
        let mut s = karistir(20, 0);
        s.sort_unstable();
        assert_eq!(s, duz(20));
    }

    #[test]
    fn tek_ogeli_ve_bos_karistirma() {
        assert_eq!(karistir(0, 7), Vec::<usize>::new());
        assert_eq!(karistir(1, 7), vec![0]);
    }

    #[test]
    fn karisik_acilinca_calan_oge_basa_gelir() {
        let mut k = Kuyruk::default();
        k.yerlestir(
            (0..8)
                .map(|i| QueueItem {
                    id: format!("k{i}"),
                    path: format!("/x/{i}.mp3"),
                    title: format!("{i}"),
                    duration: 1.0,
                    media_type: "audio".into(),
                })
                .collect(),
        );
        k.sec(5);
        k.shuffle_ayarla(true);

        assert_eq!(k.sira.first(), Some(&5));
        let mut kopya = k.sira.clone();
        kopya.sort_unstable();
        assert_eq!(kopya, duz(8));
    }
}
