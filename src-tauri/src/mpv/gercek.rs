//! Gerçek motor — libmpv sarmalayıcısı ve olay döngüsü.
//!
//! Yalnızca `mpv` özelliği açıkken derleniyor. Karşılığı [`super::yok`];
//! ikisinin yüzeyi birebir aynı olmak zorunda, yoksa özelliği kapatan derleme
//! kırılır.

use std::sync::Arc;
use std::time::{Duration, Instant};

use libmpv2::events::{Event, PropertyData};
use libmpv2::{Format, Mpv};
use tauri::{AppHandle, Emitter, Manager};

use crate::hata::{Hata, Sonuc};

use super::{
    izleri_oku, FileInfo, MpvState, PlayerState, OLAY_BITTI, OLAY_DOSYA, OLAY_DURAKLAT, OLAY_HATA,
    OLAY_HIZ, OLAY_ISTEK, OLAY_KONUM, OLAY_SES, OLAY_SURE,
};

/// mpv'ye giden tek kapı.
///
/// `Mpv` kendisi `Send + Sync` (libmpv istemci API'si iş parçacığı güvenli),
/// bu yüzden burada Mutex YOK. Bir kilit koymak, olay döngüsünün her
/// tıklamada komutları bekletmesi demek olurdu.
pub struct Motor {
    mpv: Arc<Mpv>,
}

/// mpv'den bize gelen kısayolların ön eki. `script-message muiply <eylem>`
/// biçimindeki her mesaj [`OLAY_ISTEK`] olarak arayüze düşüyor.
const ISTEK_ONEKI: &str = "muiply";

impl Motor {
    /// Motor derlemeye dahil mi. Arayüz denetimleri buna göre kapatıyor.
    pub const VAR: bool = true;

    /// mpv'yi kurar.
    ///
    /// `yuzey_id`: mpv'nin çizeceği native pencerenin tanıtıcısı (Windows'ta
    /// HWND). `None` ise mpv kendi penceresini açar — bu yalnızca yüzey
    /// oluşturulamadığında oluyor ve videosuz bir oturumdan iyi.
    ///
    /// Ayarların çoğu `with_initializer` içinde, yani `mpv_set_option` ile
    /// veriliyor: `wid`, `vo` ve `input-*` başlatmadan SONRA yazılamaz.
    pub fn kur(yuzey_id: Option<i64>) -> Sonuc<Self> {
        // Hangi özelliğin patladığını dışarı taşıyan hücre. libmpv'nin
        // döndürdüğü tek şey bir sayı (Raw(-11)) ve on beş set_property
        // çağrısından hangisinin ürettiğini söylemiyor; kurulum sıralı
        // olduğu için son denenen ad doğru cevabı veriyor.
        let son = std::cell::Cell::new("");
        let mpv = Mpv::with_initializer(|init| {
            let yaz = |ad: &'static str, deger: &str| {
                son.set(ad);
                init.set_property(ad, deger)
            };

            if let Some(id) = yuzey_id {
                son.set("wid");
                init.set_property("wid", id)?;
            }

            // Dosya bitince ya da hiç yüklenmemişken mpv KAPANMASIN. Bu
            // olmadan uygulama ilk dosyanın sonunda motorunu kaybediyor.
            yaz("idle", "yes")?;
            // Dosya yokken de yüzeyi boya: pencere bir anlığına delinmiş
            // görünmesin.
            yaz("force-window", "yes")?;
            // Zemin `--oyuk` jetonuyla aynı (src/styles.css). İki yerde
            // yazılıyor çünkü biri CSS'te biri mpv'de; renk değişirse ikisi de.
            //
            // İKİ AD deneniyor: mpv 0.38 seçeneği ikiye ayırdı — `background`
            // artık none|color|tiles, renk `background-color`a taşındı. Yeni
            // libmpv'ye eski adla renk yazmak `MPV_ERROR_PROPERTY_ERROR`
            // veriyor.
            //
            // Hata açılışı DURDURMUYOR: yanlış renkte bir zemin, hiç
            // açılmayan bir oynatıcıdan iyi. Zaten dosya oynarken zemin
            // görünmüyor; yalnız boş pencerede ve videonun kenarlarında.
            if yaz("background-color", "#07090C").is_err() {
                let _ = yaz("background", "#07090C");
            }

            // mpv'nin kendi denetimleri kapalı: bizim çubuğumuz var, ikisi
            // birden aynı anda görünürse hangisinin doğru olduğu belirsiz.
            yaz("osc", "no")?;
            son.set("osd-level");
            init.set_property("osd-level", 1i64)?;

            // Yerleşik kısayollar kapalı — `q` uygulamayı kapatırdı. Kendi
            // bağlarımızı `keybind` ile kuruyoruz (bkz. `kisayollari_kur`).
            yaz("input-default-bindings", "no")?;
            yaz("input-vo-keyboard", "yes")?;

            // `auto-safe`: bozuk sürücülerde yeşil kare / çökme üretebilen
            // yöntemleri dışarıda bırakan liste. Düz `auto` daha hızlı ama
            // hatası kullanıcının anlayamayacağı bir görüntü bozukluğu.
            yaz("hwdec", "auto-safe")?;
            yaz("audio-client-name", "muiply")?;

            // Altyazı dosyalarını BİZ buluyoruz (subtitle/mod.rs): mpv'nin
            // kendi taraması yalnız aynı adlı dosyayı alıyor, bizimki dil
            // ekli olanları da.
            yaz("sub-auto", "no")?;

            Ok(())
        })
        .map_err(|e| {
            Hata::yeni(format!(
                "mpv başlatılamadı: `{}` yazılamadı ({e})",
                son.get()
            ))
        })?;

        let motor = Motor { mpv: Arc::new(mpv) };
        motor.kisayollari_kur();
        Ok(motor)
    }

    /// mpv penceresine düşen fare/klavye olaylarını bağlar.
    ///
    /// Neden mpv'de değil de arayüzde değil: mpv'nin alt penceresi webview'in
    /// ÜSTÜNDE duruyor (bkz. `yuzey.rs`), yani video üzerindeyken tuşlar
    /// webview'e hiç ulaşmıyor. Bağlar iki gruba ayrılıyor —
    ///
    /// - mpv'nin kendi başına yapabildikleri (`cycle pause`, `seek`): doğrudan
    ///   mpv komutu. Durum değişikliği zaten izlenen özelliklerden arayüze
    ///   düşüyor, ayrıca haber vermeye gerek yok.
    /// - uygulamanın işi olanlar (tam ekran, sonraki parça): `script-message`
    ///   ile bize geri geliyor, [`OLAY_ISTEK`] olarak arayüze çıkıyor.
    ///
    /// Hata yutuluyor: `keybind` mpv 0.36+ komutu. Eski bir libmpv ile
    /// kısayollar çalışmaz ama oynatıcının kendisi çalışır; bunun için
    /// açılışı durdurmak orantısız.
    fn kisayollari_kur(&self) {
        let baglar: &[(&str, &str)] = &[
            ("SPACE", "cycle pause"),
            ("MBTN_LEFT", "cycle pause"),
            ("RIGHT", "seek 5"),
            ("LEFT", "seek -5"),
            ("Shift+RIGHT", "seek 60"),
            ("Shift+LEFT", "seek -60"),
            ("UP", "add volume 5"),
            ("DOWN", "add volume -5"),
            ("WHEEL_UP", "add volume 5"),
            ("WHEEL_DOWN", "add volume -5"),
            ("m", "cycle mute"),
            //
            ("MBTN_LEFT_DBL", "script-message muiply tam-ekran"),
            ("f", "script-message muiply tam-ekran"),
            ("ESC", "script-message muiply tam-ekran-kapat"),
            ("n", "script-message muiply sonraki"),
            ("p", "script-message muiply onceki"),
            ("s", "script-message muiply altyazi"),
        ];

        for (tus, komut) in baglar {
            let _ = self.komut("keybind", &[tus, komut]);
        }
    }

    pub fn komut(&self, ad: &str, args: &[&str]) -> Sonuc<()> {
        self.mpv
            .command(ad, args)
            .map_err(|e| Hata::yeni(format!("mpv komutu '{ad}' başarısız: {e}")))
    }

    pub fn ayar_bayrak(&self, ad: &str, deger: bool) -> Sonuc<()> {
        self.ayarla(ad, deger)
    }

    pub fn ayar_sayi(&self, ad: &str, deger: i64) -> Sonuc<()> {
        self.ayarla(ad, deger)
    }

    pub fn ayar_ondalik(&self, ad: &str, deger: f64) -> Sonuc<()> {
        self.ayarla(ad, deger)
    }

    pub fn ayar_metin(&self, ad: &str, deger: &str) -> Sonuc<()> {
        self.ayarla(ad, deger)
    }

    fn ayarla<T: libmpv2::SetData>(&self, ad: &str, deger: T) -> Sonuc<()> {
        self.mpv
            .set_property(ad, deger)
            .map_err(|e| Hata::yeni(format!("mpv özelliği '{ad}' yazılamadı: {e}")))
    }

    pub fn oku_bayrak(&self, ad: &str) -> Sonuc<bool> {
        self.oku(ad)
    }

    pub fn oku_sayi(&self, ad: &str) -> Sonuc<i64> {
        self.oku(ad)
    }

    pub fn oku_ondalik(&self, ad: &str) -> Sonuc<f64> {
        self.oku(ad)
    }

    pub fn oku_metin(&self, ad: &str) -> Sonuc<String> {
        self.oku(ad)
    }

    fn oku<T: libmpv2::GetData>(&self, ad: &str) -> Sonuc<T> {
        self.mpv
            .get_property(ad)
            .map_err(|e| Hata::yeni(format!("mpv özelliği '{ad}' okunamadı: {e}")))
    }

    /// Olay döngüsünü kendi iş parçacığında başlatır.
    ///
    /// Döngü **ayrı bir mpv istemcisiyle** çalışıyor (`create_client`): olay
    /// kuyruğu istemci başına, yani komutları çalıştıran kapı ile olayları
    /// bekleyen kapı birbirini bloklamıyor.
    ///
    /// İstemciye AD VERİLMİYOR (`None`). libmpv2 6.0.0'ın adlı yolu bozuk:
    /// `CString::new(name)?.as_ptr()` yazıyor ve geçici `CString` daha
    /// `mpv_create_client` çağrılmadan düşüyor — mpv'ye asılı bir işaretçi
    /// gidiyor. Dönen `NULL` da crate içinde `NonNull::new_unchecked`e
    /// giriyor, yani hata değil doğrudan abort (`0xc0000409`) oluyor ve
    /// yukarıdaki `Err` dalı hiç çalışmıyor. Adın bize faydası yoktu:
    /// yalnızca `mpv_client_name()` döndürüyor, hiçbir yerde okumuyoruz.
    /// Ses aygıtında görünen ad ayrı bir seçenek (`audio-client-name`).
    pub fn olaylari_yayinla(&self, app: AppHandle) {
        let istemci = match self.mpv.create_client(None) {
            Ok(c) => c,
            Err(e) => {
                // Olaysız bir oynatıcı sürgüsü kıpırdamayan, bittiğini
                // söylemeyen bir oynatıcı. Açılışı durdurmuyoruz ama sessiz
                // de kalmıyoruz.
                let _ = app.emit(
                    OLAY_HATA,
                    format!("mpv olay kanalı açılamadı, oynatma durumu güncellenmeyecek: {e}"),
                );
                return;
            }
        };

        std::thread::Builder::new()
            .name("muiply-mpv-olay".into())
            .spawn(move || dongu(istemci, app))
            .expect("mpv olay iş parçacığı başlatılamadı");
    }
}

/// İzlenen özelliklerin kimlikleri. `PropertyChange` olayında ad zaten
/// geliyor; kimlikler yalnızca `unobserve` için anlamlı, o yüzden sabit.
const IZLENEN: &[(&str, Format, u64)] = &[
    ("time-pos", Format::Double, 1),
    ("duration", Format::Double, 2),
    ("pause", Format::Flag, 3),
    ("volume", Format::Int64, 4),
    ("mute", Format::Flag, 5),
    ("speed", Format::Double, 6),
];

/// `time-pos` olaylarının arayüze en sık yayınlanma aralığı.
///
/// mpv bu özelliği KARE HIZINDA bildiriyor — saniyede altmış olay. Durumun
/// kendisi hepsinde güncelleniyor (`player_get_state` doğruyu söylemeli) ama
/// her birini arayüze taşımak saniyede altmış JSON serileştirmesi ve altmış
/// IPC gidişi demek. Arayüz zaten 200 ms'de bir çiziyor
/// (`useOynatici.ts` → `KONUM_ARALIGI`), yani o olayların çoğu yol boyunca
/// taşınıp atılıyordu.
const KONUM_ARALIGI: Duration = Duration::from_millis(100);

/// Kaldığı yerin kütüphaneye yazılma aralığı.
///
/// Yayınlama aralığından (100 ms) ayrı ve ondan çok seyrek: bu bir DİSK
/// yazımı. Beş saniye, "kapattığım yer" ile "kaydedilen yer" arasında en kötü
/// beş saniye fark demek — kullanıcının fark etmeyeceği kadar az, sürekli
/// yazmanın SSD'yi ve veritabanı kilidini yormayacağı kadar seyrek.
/// Pencereyi kapatmak zaten son konumu ayrıca yazıyor (`lib.rs`).
const KONUM_KAYDETME: Duration = Duration::from_secs(5);

fn dongu(istemci: Mpv, app: AppHandle) {
    for (ad, bicim, kimlik) in IZLENEN {
        if let Err(e) = istemci.observe_property(ad, *bicim, *kimlik) {
            let _ = app.emit(OLAY_HATA, format!("'{ad}' izlenemedi: {e}"));
        }
    }

    // Son konum yayınının anı. Döngü tek iş parçacığı, o yüzden düz bir
    // yerel değişken — paylaşılan bir sayaç ve kilidi gerekmiyor.
    let mut son_konum = Instant::now() - KONUM_ARALIGI;
    // Kaldığı yerin son yazılma anı. Açılışta "az önce yazıldı" sayılıyor:
    // ilk time-pos olayı dosyanın başında geliyor ve onu hemen yazmak,
    // kayıtlı konumu sıfırla ezmek olurdu.
    let mut son_kayit = Instant::now();

    loop {
        // 1 saniye: uygulama kapanırken bu iş parçacığının takılıp kalmaması
        // için bir üst sınır. Olay varsa zaten hemen dönüyor.
        let Some(olay) = istemci.wait_event(1.0) else {
            continue;
        };

        let olay = match olay {
            Ok(o) => o,
            Err(e) => {
                let _ = app.emit(OLAY_HATA, format!("mpv olayı okunamadı: {e}"));
                continue;
            }
        };

        match olay {
            Event::FileLoaded => {
                // Sayaç YENİDEN başlıyor. Yoksa önceki dosyadan kalan
                // süre dolmuşken, yeni dosyanın daha başındaki konumu
                // (sıfıra yakın) onun kayıtlı yerinin ÜZERİNE yazılırdı —
                // ve kaldığı yerden devam, tam da devam edilecek dosyada
                // çalışmazdı.
                son_kayit = Instant::now();
                dosya_yuklendi(&app);
            }

            Event::EndFile(sebep) => {
                // Yol daha SONRA okunamaz: `eof` kuyruğu ilerletiyor ve
                // ilerleyen kuyruk durumdaki yolu bir sonraki dosyayla
                // değiştiriyor.
                let yol = app.try_state::<MpvState>().and_then(|d| d.anlik().path);

                let etiket = match sebep {
                    libmpv2::mpv_end_file_reason::Eof => "eof",
                    libmpv2::mpv_end_file_reason::Stop => "stop",
                    libmpv2::mpv_end_file_reason::Quit => "quit",
                    libmpv2::mpv_end_file_reason::Error => "error",
                    // `Redirect`: mpv bir yönlendirmeyi izliyor, dosya
                    // bitmedi. Kuyruğu ilerletirsek yanlış parçaya geçeriz.
                    _ => continue,
                };
                durumu_degistir(&app, |d| {
                    d.playing = false;
                    if etiket != "eof" {
                        d.position = 0.0;
                    }
                });
                let _ = app.emit(OLAY_BITTI, serde_json::json!({ "reason": etiket }));
                // Tepsi ipucu HER bitişte sıfırlanıyor, sebebine bakmadan.
                // `loadfile replace` de buradan geçiyor (sebep `stop`) ve
                // ipucu bir an "Muiply"ye dönüyor — ama hemen ardından gelen
                // `FileLoaded` yeni adı yazıyor. Sebebe göre ayırmak, kuyruk
                // bittiğinde artık çalmayan bir dosyanın adının ipuçunda
                // asılı kalması demekti.
                crate::tepsi::baslik_yaz(&app, None);

                // Kuyruk YALNIZ gerçek bitişte ilerliyor. `loadfile replace`
                // de bir `EndFile` üretiyor ama sebebi `stop`; onu bitiş
                // saymak, her dosya açılışında bir sonrakine atlamak olurdu.
                if etiket == "eof" {
                    // Sonuna kadar izlendi: yarım kalma kaydı artık yanlış.
                    // Silmek İLK iş, çünkü sıradaki dosya açılınca bu yol
                    // durumda kalmıyor.
                    if let Some(y) = &yol {
                        crate::library::devam::temizle(&app, y);
                    }
                    crate::playlist::surucu::dosya_bitti(&app);
                }
            }

            Event::PropertyChange { name, change, .. } => {
                ozellik_degisti(&app, name, change, &mut son_konum, &mut son_kayit);
            }

            Event::ClientMessage(parcalar) => {
                // `script-message muiply <eylem>` → arayüze <eylem>.
                if parcalar.first() == Some(&ISTEK_ONEKI) {
                    if let Some(eylem) = parcalar.get(1) {
                        let _ = app.emit(OLAY_ISTEK, eylem.to_string());
                    }
                }
            }

            Event::Shutdown => break,

            _ => {}
        }
    }
}

/// İzlenen bir özellik değişti: önbelleği güncelle, arayüze haber ver.
///
/// `time-pos` saniyede birkaç kez geliyor ve arayüz onu sürgüde gösteriyor;
/// diğerlerinin kendi olayı var çünkü arayüz onlara farklı tepki veriyor
/// (ses düğmesinin biçimi, oynat/duraklat ikonu).
fn ozellik_degisti(
    app: &AppHandle,
    ad: &str,
    veri: PropertyData<'_>,
    son_konum: &mut Instant,
    son_kayit: &mut Instant,
) {
    match (ad, veri) {
        ("time-pos", PropertyData::Double(v)) => {
            // Önbellek HER olayda güncelleniyor: `player_get_state` sorulduğu
            // anın doğrusunu söylemeli, yayının kısıtlanması onu ilgilendirmez.
            durumu_degistir(app, |d| d.position = v);

            if son_konum.elapsed() < KONUM_ARALIGI {
                return;
            }
            *son_konum = Instant::now();

            let _ = app.emit(OLAY_KONUM, serde_json::json!({ "position": v }));
            // Küçük resim burada alınıyor: video zaten çiziliyorken bir kare
            // istemek bedava. Gerekçe library/kucukresim.rs başında.
            crate::library::kucukresim::belki_al(app, v);

            // Kaldığı yer: aynı olaydan ama çok daha seyrek (KONUM_KAYDETME).
            if son_kayit.elapsed() >= KONUM_KAYDETME {
                *son_kayit = Instant::now();
                if let Some(yol) = app.try_state::<MpvState>().and_then(|d| d.anlik().path) {
                    crate::library::devam::kaydet(app, &yol, v);
                }
            }
        }
        ("duration", PropertyData::Double(v)) => {
            durumu_degistir(app, |d| d.duration = v);
            let _ = app.emit(OLAY_SURE, serde_json::json!({ "duration": v }));
        }
        ("pause", PropertyData::Flag(v)) => {
            durumu_degistir(app, |d| {
                d.paused = v;
                d.playing = !v && d.path.is_some();
            });
            let _ = app.emit(OLAY_DURAKLAT, serde_json::json!({ "paused": v }));
        }
        ("volume", PropertyData::Int64(v)) => {
            durumu_degistir(app, |d| d.volume = v);
            let _ = app.emit(OLAY_SES, serde_json::json!({ "volume": v }));
        }
        ("mute", PropertyData::Flag(v)) => {
            durumu_degistir(app, |d| d.muted = v);
            let _ = app.emit(OLAY_SES, serde_json::json!({ "muted": v }));
        }
        ("speed", PropertyData::Double(v)) => {
            durumu_degistir(app, |d| d.rate = v);
            let _ = app.emit(OLAY_HIZ, serde_json::json!({ "rate": v }));
        }
        // `time-pos` dosya yokken null geliyor (Format::None). Bizim için
        // "konum yok" demek; sıfırlamak sürgüyü başa atar, doğrusu bu.
        ("time-pos", _) => {
            durumu_degistir(app, |d| d.position = 0.0);
        }
        _ => {}
    }
}

fn dosya_yuklendi(app: &AppHandle) {
    let Some(durum) = app.try_state::<MpvState>() else {
        return;
    };
    let motor = &durum.motor;

    let path = motor.oku_metin("path").unwrap_or_default();

    // Yandaki altyazılar izler okunmadan ÖNCE ekleniyor. Sonra eklenirse ilk
    // gönderilen iz listesi eksik kalır ve arayüz bir an "altyazı yok" der.
    crate::subtitle::otomatik_yukle(motor, &path);
    // Kaydın küçük resmi yoksa sıraya al (library/kucukresim.rs).
    crate::library::kucukresim::isaretle(app, &path);

    let duration = motor.oku_ondalik("duration").unwrap_or(0.0);
    let title = motor
        .oku_metin("media-title")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| dosya_adi(&path));

    // Kaldığı yere atlama BURADA, `player_open`da değil: eşik kararı süreyi
    // bilmeyi gerektiriyor ve süre dosya çözümlenene kadar bilinmiyor.
    crate::library::devam::geri_yukle(app, &path, duration);

    let izler = izleri_oku(motor).unwrap_or_default();
    let media_type = tur_belirle(motor, &izler);

    durum.guncelle(|d| {
        d.path = Some(path.clone());
        d.title = Some(title.clone());
        d.duration = duration;
        d.media_type = Some(media_type.clone());
        d.playing = !d.paused;
    });

    // Tepsi ipucu: küçültülmüş bir oynatıcıda "ne çalıyor" sorusunun tek
    // cevabı. Olaydan ÖNCE, çünkü olay arayüzü boyamaya başlatıyor ve ikisi
    // arasında bir sıra bağı yok.
    crate::tepsi::baslik_yaz(app, Some(&title));

    // VİDEO başladıysa oynatıcı penceresi görünür olmalı. Karar burada,
    // arayüzde değil: kuyruk kendi kendine de ilerliyor ve o anda hiçbir
    // pencerenin açık olması gerekmiyor. Gizli bir pencereye çizilen video
    // "ses geliyor ama görüntü yok" demek.
    //
    // `gorunur_yap`, `goster` değil: zaten görünen pencereyi öne fırlatıp
    // kullanıcının kütüphanedeki odağını almıyor. Ses için hiç
    // çağrılmıyor — müzik dinlerken kütüphanede gezinmek doğru davranış.
    if media_type == "video" {
        crate::pencere::gorunur_yap(app, crate::pencere::OYNATICI);
    }

    let _ = app.emit(
        OLAY_DOSYA,
        FileInfo {
            path,
            title,
            duration,
            media_type,
            tracks: izler,
        },
    );
}

/// Dosya video mu ses mi.
///
/// "Video izi var mı" yetmiyor: kapak resmi taşıyan bir MP3'te mpv bir video
/// izi bildiriyor. Ayrım `albumart` bayrağında — o bayrak açıksa sahnede
/// video değil kapak gösterilmeli.
fn tur_belirle(motor: &Motor, izler: &[super::Track]) -> String {
    let kapak = motor
        .oku_bayrak("current-tracks/video/albumart")
        .unwrap_or(false);
    let video_var = izler.iter().any(|i| i.kind == "video");

    if video_var && !kapak {
        "video".to_string()
    } else {
        "audio".to_string()
    }
}

fn dosya_adi(yol: &str) -> String {
    std::path::Path::new(yol)
        .file_name()
        .map(|a| a.to_string_lossy().into_owned())
        .unwrap_or_else(|| yol.to_string())
}

fn durumu_degistir(app: &AppHandle, degistir: impl FnOnce(&mut PlayerState)) {
    if let Some(durum) = app.try_state::<MpvState>() {
        durum.guncelle(degistir);
    }
}
