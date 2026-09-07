//! Sonda — kütüphane taramasının süre/boyut/etiket okuyucusu.
//!
//! # Neden ikinci bir mpv örneği
//!
//! Tarama bir dosyanın süresini öğrenmek için onu açmak zorunda. Bunu ÇALAN
//! örnekle yapmak, kullanıcının izlediği filmi durdurup başka bir dosyayı
//! yüklemek olurdu. Sonda ayrı bir libmpv bağlamı: kendi `vo=null`,
//! `ao=null` yapılandırmasıyla ne ekrana çiziyor ne ses aygıtına dokunuyor.
//!
//! # Neden ayrı bir meta veri kütüphanesi değil
//!
//! `symphonia` sesi okur, videoyu okumaz; ikisi için iki ayrı yol yazmak
//! gerekirdi. libmpv zaten kurulu ve her iki tarafı da aynı sözlükle
//! (`duration`, `width`, `metadata/by-key/...`) anlatıyor.
//!
//! Motor kapalı derlemede sonda da yok: süre `0`, boyut `None` kalıyor ve
//! arayüz süre rozetini hiç çizmiyor. Sahte bir süre üretmek, ızgarada
//! yanlış bilgi göstermek olurdu.

/// Bir dosyadan okunabilenler. Okunamayan alan `None`/`0` — tahmin yok.
#[derive(Debug, Default, Clone)]
pub struct Olcum {
    pub duration: f64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[cfg(feature = "mpv")]
mod ic {
    use std::time::{Duration, Instant};

    use libmpv2::events::Event;
    use libmpv2::Mpv;

    use crate::hata::{Hata, Sonuc};

    use super::Olcum;

    /// Bir dosyanın açılması için beklenen en uzun süre.
    ///
    /// Ağ sürücüsündeki ya da bozuk bir dosya süresiz bekletebilir; tarama
    /// tek bir dosyada asılı kalmamalı. Süre dolarsa kayıt ölçümsüz yazılıyor.
    const ZAMAN_ASIMI: Duration = Duration::from_secs(6);

    pub struct Sonda {
        mpv: Mpv,
    }

    impl Sonda {
        pub fn yeni() -> Sonuc<Self> {
            let mpv = Mpv::with_initializer(|init| {
                // Ne çiz ne çal: sonda yalnızca demuxer'ın söylediğini okuyor.
                init.set_property("vo", "null")?;
                init.set_property("ao", "null")?;
                // Kapak resmi çıkarmaya çalışmasın: yavaş ve bize gereksiz.
                init.set_property("audio-display", "no")?;
                // Yüklendiği anda çalmaya başlamasın; dosyayı açmak yeter.
                init.set_property("pause", "yes")?;
                init.set_property("idle", "yes")?;
                init.set_property("input-default-bindings", "no")?;
                init.set_property("input-vo-keyboard", "no")?;
                init.set_property("sub-auto", "no")?;
                Ok(())
            })
            .map_err(|e| Hata::yeni(format!("sonda başlatılamadı: {e}")))?;

            Ok(Sonda { mpv })
        }

        /// Dosyayı açar, künyesini okur, kapatır.
        ///
        /// `None`: dosya açılamadı ya da zaman aşımına uğradı. Tarama bunu
        /// hata saymıyor — kayıt ölçümsüz yazılıyor, çünkü listede görünmesi
        /// süresinin bilinmesinden önemli.
        pub fn olc(&self, yol: &str) -> Option<Olcum> {
            // Bir önceki ölçümden kalan olaylar BU ölçümün cevabı sanılmasın.
            // `stop` ve `loadfile replace` birer `EndFile` üretiyor ve olay
            // kuyruğu istemci başına duruyor: temizlenmediğinde ikinci dosya
            // kendi `FileLoaded`ını beklerken öncekinin `EndFile`ını görüp
            // "açılamadı" diye dönüyordu. Sonrasındaki dosyalarda daha kötüsü
            // oluyordu — bir öncekinin `FileLoaded`ı bu dosyanınki sanılıp
            // künyesi YANLIŞ kayda yazılabiliyordu.
            self.kuyrugu_bosalt();

            if self.mpv.command("loadfile", &[yol, "replace"]).is_err() {
                return None;
            }

            let baslangic = Instant::now();
            let yuklendi = loop {
                if baslangic.elapsed() > ZAMAN_ASIMI {
                    break false;
                }
                match self.mpv.wait_event(0.5) {
                    Some(Ok(Event::FileLoaded)) => break true,
                    // Açılamadı: bozuk dosya, desteklenmeyen kod çözücü.
                    Some(Ok(Event::EndFile(_))) => break false,
                    _ => continue,
                }
            };

            // Dosyayı her durumda bırak — açılamayanı da. Açık kalan bir
            // demuxer dosyayı kilitli tutuyor (Windows'ta kullanıcı o dosyayı
            // silemez) ve yarım kalan yükleme sonraki ölçüme olay taşıyor.
            let olcum = yuklendi.then(|| Olcum {
                duration: self.mpv.get_property("duration").unwrap_or(0.0),
                width: self.mpv.get_property("width").ok().filter(|w| *w > 0),
                height: self.mpv.get_property("height").ok().filter(|h| *h > 0),
                title: self.etiket("media-title"),
                artist: self.etiket("metadata/by-key/artist"),
                album: self.etiket("metadata/by-key/album"),
            });

            let _ = self.mpv.command("stop", &[]);
            olcum
        }

        /// Kuyrukta bekleyen olayları atar. `wait_event(0.0)` kuyruk boşken
        /// hemen `None` dönüyor, yani bu döngü beklemiyor.
        ///
        /// Üst sınır var çünkü mpv olay üretmeye devam edebilir; sonsuz bir
        /// döngü taramayı kilitlerdi. Sınıra dayanmak zararsız: kalan olay
        /// bir sonraki `olc` çağrısının başında yeniden atılıyor.
        fn kuyrugu_bosalt(&self) {
            for _ in 0..256 {
                if self.mpv.wait_event(0.0).is_none() {
                    return;
                }
            }
        }

        fn etiket(&self, ad: &str) -> Option<String> {
            self.mpv
                .get_property::<String>(ad)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        }
    }
}

#[cfg(not(feature = "mpv"))]
mod ic {
    use crate::hata::{Hata, Sonuc};

    use super::Olcum;

    pub struct Sonda;

    impl Sonda {
        pub fn yeni() -> Sonuc<Self> {
            Err(Hata::yeni(crate::mpv::MOTOR_YOK))
        }

        pub fn olc(&self, _yol: &str) -> Option<Olcum> {
            None
        }
    }
}

pub use ic::Sonda;
