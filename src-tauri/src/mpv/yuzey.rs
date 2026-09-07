//! mpv'nin çizdiği native alt pencere.
//!
//! # Neden alt pencere, neden şeffaf webview değil
//!
//! İlk tasarım (bkz. `docs/Setup.md`'nin eski hâli) şeffaf bir webview'in
//! ALTINA mpv koymayı öngörüyordu. Windows'ta bu güvenilir değil: WebView2
//! kendi DirectComposition ağacında çiziliyor ve şeffaf bir bölgesinden
//! kardeş bir alt pencere düzenli olarak görünmüyor.
//!
//! Bu yüzden yön ters: mpv'nin penceresi webview'in **ÜSTÜNDE**, sahnenin
//! dikdörtgeni kadar. Sonuçları —
//!
//! - Arayüz videonun üstüne bindirilemiyor. Denetimler videonun ALTINDA,
//!   kendi çubuğunda duruyor. Tam ekranda çubuk göründüğünde sahne o kadar
//!   küçülüyor.
//! - Video üstündeki fare/klavye webview'e ulaşmıyor. Kısayollar mpv'nin
//!   kendi `keybind`leriyle kuruluyor (bkz. `gercek.rs::kisayollari_kur`).
//! - Şeffaflık gerekmiyor: pencere normal, işletim sistemi çerçevesi duruyor.
//!   Ailenin geri kalanıyla aynı (Muiwatch, Muiget de çerçeveli).
//!
//! # Üç platform, aynı üç işlem
//!
//! Her platformda bir nesne yaratılıyor, taşınıyor ve gizleniyor. Nesne
//! değişiyor, iş değişmiyor:
//!
//! | Platform | Nesne | mpv'ye verilen `wid` |
//! |---|---|---|
//! | Windows | `WS_CHILD` bir HWND | HWND'nin kendisi |
//! | Linux/BSD | çocuk `GdkWindow` | X11 pencere kimliği (XID) |
//! | macOS | `NSView` alt görünümü | `NSView*` |
//!
//! Linux'ta iki değer AYRI: taşımayı GdkWindow yapıyor ama mpv XID istiyor.
//! Bu yüzden [`Yuzey`] iki tanıtıcı taşıyor.
//!
//! # Ölçü birimi
//!
//! Dikdörtgen buraya **mantıksal** (CSS) pikselle geliyor ve fiziksel piksele
//! çevirme yalnız Windows'ta yapılıyor — çünkü yalnız orada gerekiyor. GTK3
//! pencere koordinatları da AppKit noktaları da zaten mantıksal; ölçekle
//! çarpmak, HiDPI bir ekranda videoyu olması gerekenin iki katı büyüklüğünde
//! çizmek olurdu. Karar platform modülünde, çağıranda değil.
//!
//! # Ana iş parçacığı
//!
//! GTK ve AppKit çağrıları ana iş parçacığından yapılmak zorunda, oysa
//! `player_set_video_rect` bir Tauri komutu ve komutlar havuzdan bir iş
//! parçacığında çalışıyor. Bu yüzden Linux ve macOS `run_on_main_thread`
//! üzerinden geçiyor. Windows'ta gerek yok: `SetWindowPos`/`ShowWindow`
//! mesajı pencerenin kendi kuyruğuna gönderiyor.
//!
//! # Geri düşüş
//!
//! Yüzey açılamazsa (Wayland oturumu, gerçeklenmemiş bir pencere, beklenmedik
//! bir hata) `olustur` hata dönüyor ve `lib.rs` [`Yuzey::yok`]a düşüyor: mpv
//! kendi ayrı penceresini açıyor. Ses, kütüphane, kuyruk ve denetimler orada
//! da çalışıyor — çirkin ama çalışır, ve hiç açılmayan bir uygulamadan iyi.

use std::sync::atomic::{AtomicIsize, Ordering};

use crate::hata::{Hata, Sonuc};

/// mpv'nin çizdiği yüzey.
///
/// İki `AtomicIsize`, `Mutex` değil: yazılmaları bir kez (kurulumda),
/// okunmaları her yeniden boyutlamada. Kilit almanın karşılığı yok. `isize`
/// olmalarının sebebi de bu — ham işaretçiler `Send`/`Sync` değil, oysa
/// [`Yuzey`] `MpvState` içinde iş parçacıkları arasında paylaşılıyor.
pub struct Yuzey {
    /// mpv'ye `wid` olarak verilen değer.
    mpv_id: AtomicIsize,
    /// Taşıyıp gizlediğimiz platform nesnesi. Windows'ta HWND, macOS'ta
    /// `NSView*`, Linux'ta `GdkWindow*`. İlk ikisinde `mpv_id` ile aynı sayı;
    /// Linux'ta farklı.
    yerel: AtomicIsize,
}

impl Yuzey {
    /// Yüzeysiz hâl — mpv kendi penceresini açar.
    pub fn yok() -> Self {
        Yuzey {
            mpv_id: AtomicIsize::new(0),
            yerel: AtomicIsize::new(0),
        }
    }

    /// mpv'ye verilecek `wid`. Yüzey yoksa `None`.
    pub fn id(&self) -> Option<i64> {
        match self.mpv_id.load(Ordering::Relaxed) {
            0 => None,
            t => Some(t as i64),
        }
    }

    /// Platform nesnesinin ham işaretçisi. Yüzey yoksa `None`.
    fn yerel_ptr(&self) -> Option<isize> {
        match self.yerel.load(Ordering::Relaxed) {
            0 => None,
            p => Some(p),
        }
    }
}

/// İşi ana iş parçacığına gönderir.
///
/// Beklemiyor: `run_on_main_thread` olay döngüsüne bir mesaj bırakıp dönüyor.
/// Bu yüzden içerideki hatalar geri gelemiyor — zaten gelseler de yapacak bir
/// şey yok, konumlanamayan bir yüzey için oynatmayı durdurmak orantısız.
/// Ana iş parçacığından çağrılsa bile kilitlenme yok; mesaj sıraya giriyor.
#[cfg(not(target_os = "windows"))]
fn ana_iste(app: &tauri::AppHandle, is: impl FnOnce() + Send + 'static) -> Sonuc<()> {
    app.run_on_main_thread(is)?;
    Ok(())
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    use std::ffi::c_void;

    use tauri::Manager;
    use windows::core::w;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, SetWindowPos, ShowWindow, HWND_TOP, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNA,
        WINDOW_EX_STYLE, WS_CHILD, WS_CLIPCHILDREN, WS_VISIBLE,
    };

    impl Yuzey {
        /// Ana pencerenin içinde mpv için bir alt pencere açar.
        ///
        /// **Ana iş parçacığından çağrılmalı** (Tauri'nin `setup` kancası
        /// öyle): bir pencerenin mesaj kuyruğu onu yaratan iş parçacığına
        /// bağlı, başka bir yerde yaratmak girdi ve boyama yönlendirmesini
        /// bozar. Sonraki `SetWindowPos`/`ShowWindow` çağrıları başka bir iş
        /// parçacığından güvenli — Windows onları pencerenin kuyruğuna
        /// gönderiyor.
        ///
        /// Sınıf olarak `STATIC` kullanılıyor: kendi sınıfımızı kaydetmenin
        /// karşılığı yok, pencere hiçbir mesaj işlemiyor. Boyamanın tamamı
        /// mpv'nin kendi alt penceresinde.
        pub fn olustur(pencere: &tauri::WebviewWindow) -> Sonuc<Yuzey> {
            let ana = pencere
                .hwnd()
                .map_err(|e| Hata::yeni(format!("pencere tanıtıcısı alınamadı: {e}")))?;
            // `.0` sürüme göre `isize` ya da `*mut c_void`; ikisi de `as isize`
            // kabul ediyor. Tauri'nin `windows` sürümü bizimkinden farklı
            // olabilir, tip değil sayı taşınıyor.
            let ana = HWND(ana.0 as isize as *mut c_void);

            // WS_CLIPCHILDREN: mpv kendi alt penceresini bunun içine açıyor;
            // bayrak olmadan üst pencere her boyamada onun üstünü siliyor ve
            // video titriyor.
            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("STATIC"),
                    None,
                    WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
                    0,
                    0,
                    1,
                    1,
                    Some(ana),
                    None,
                    None,
                    None,
                )
            }
            .map_err(|e| Hata::yeni(format!("video yüzeyi oluşturulamadı: {e}")))?;

            let yuzey = Yuzey::yok();
            // mpv'ye verilen değer ile taşıdığımız nesne aynı: HWND.
            yuzey.mpv_id.store(hwnd.0 as isize, Ordering::Relaxed);
            yuzey.yerel.store(hwnd.0 as isize, Ordering::Relaxed);
            Ok(yuzey)
        }

        /// Yüzeyi sahnenin dikdörtgenine taşır. Ölçüler MANTIKSAL piksel.
        ///
        /// Fiziksel piksele çevirme BURADA: HWND koordinatları fiziksel ve
        /// doğru çarpanı pencere biliyor — karışık DPI'lı iki ekran arasında
        /// taşınan bir pencerede `devicePixelRatio` ile ayrışabiliyor.
        pub fn konumla(
            &self,
            app: &tauri::AppHandle,
            x: f64,
            y: f64,
            en: f64,
            boy: f64,
        ) -> Sonuc<()> {
            let Some(t) = self.yerel_ptr() else {
                return Ok(());
            };
            let hwnd = HWND(t as *mut c_void);

            let olcek = app
                .get_webview_window(crate::pencere::OYNATICI)
                .ok_or_else(|| Hata::yeni("oynatıcı penceresi bulunamadı"))?
                .scale_factor()
                .map_err(|e| Hata::yeni(format!("ölçek çarpanı okunamadı: {e}")))?;

            // HWND_TOP: yüzey webview'in üstünde kalmalı, yoksa video
            // kaybolur. SWP_NOACTIVATE: konumlama odağı çalmasın — kullanıcı
            // arama kutusunda yazarken pencere yeniden boyutlanabiliyor.
            unsafe {
                SetWindowPos(
                    hwnd,
                    Some(HWND_TOP),
                    (x * olcek).round() as i32,
                    (y * olcek).round() as i32,
                    ((en * olcek).round() as i32).max(1),
                    ((boy * olcek).round() as i32).max(1),
                    SWP_NOACTIVATE,
                )
            }
            .map_err(|e| Hata::yeni(format!("video yüzeyi konumlanamadı: {e}")))
        }

        /// Yüzeyi gizler/gösterir.
        ///
        /// Gizlemek şart: kütüphane ızgarasındayken yüzey duruyorsa siyah bir
        /// dikdörtgen ızgaranın üstünde asılı kalıyor.
        pub fn goster(&self, _app: &tauri::AppHandle, gorunur: bool) -> Sonuc<()> {
            let Some(t) = self.yerel_ptr() else {
                return Ok(());
            };
            let hwnd = HWND(t as *mut c_void);

            // SW_SHOWNA: göster ama odaklama. Odaklasaydı kütüphaneden bir
            // dosya açmak klavyeyi mpv'ye kaptırırdı.
            let komut = if gorunur { SW_SHOWNA } else { SW_HIDE };

            // Dönüş değeri BAŞARI DEĞİL: pencerenin bir ÖNCEKİ görünürlüğü.
            // Gizli bir pencereyi göstermek `FALSE` döndürüyor, yani `.ok()`
            // ile hataya çevirmek her ilk göstermeyi "başarısız" saymaktı.
            // ShowWindow'un belgelenmiş bir hata dönüşü yok.
            unsafe {
                let _ = ShowWindow(hwnd, komut);
            }
            Ok(())
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use super::*;

    // `gdk` ve `glib` gdkx11'in üzerinden alınıyor: aynı crate'in aynı
    // sürümü olduğuna emin olmanın en kısa yolu. `gtk::prelude` zaten
    // `glib::prelude`i içeriyor — `downcast` (Cast) ve `window()`/`realize()`
    // (WidgetExt) oradan geliyor. gdkx11'in kendi prelude'u YOK.
    use gdkx11::gdk;
    use gdkx11::glib::translate::{from_glib_none, ToGlibPtr};
    use gtk::prelude::*;

    impl Yuzey {
        /// Ana pencerenin GdkWindow'u içinde bir çocuk pencere açar.
        ///
        /// Windows'takiyle aynı fikir: GTK'nın parçacık ağacına HİÇ
        /// dokunmuyoruz, üst pencerenin altına native bir çocuk açıp mpv'ye
        /// onu veriyoruz. Ağaca bir parçacık sokmak (bir `GtkOverlay` kurup
        /// webview'i içine taşımak) Tauri'nin kendi kurduğu düzeni bozardı ve
        /// Tauri o düzeni sürüm sürüm değiştiriyor.
        ///
        /// **Yalnız X11.** Wayland'da `GdkWindow` bir `X11Window` değil ve
        /// dönüşüm başarısız oluyor; mpv o zaman kendi penceresini açıyor.
        /// Wayland'da alt yüzey gömme `wl_subsurface` istiyor ve mpv'nin
        /// `wid` seçeneği onu kabul etmiyor — yani bu bir eksiklik değil,
        /// mpv'nin sınırı. XWayland altında çalıştırıldığında gömme çalışıyor.
        pub fn olustur(pencere: &tauri::WebviewWindow) -> Sonuc<Yuzey> {
            let gtk_pencere = pencere.gtk_window()?;

            // Bir parçacığın GdkWindow'u ancak GERÇEKLENDİKTEN sonra var.
            // `setup` kancasında pencere genelde gerçeklenmiş oluyor ama
            // `"visible": false` ile açılan bir pencerede olmuyor.
            if gtk_pencere.window().is_none() {
                gtk_pencere.realize();
            }
            let ana = gtk_pencere
                .window()
                .ok_or_else(|| Hata::yeni("GTK penceresi gerçeklenemedi"))?;

            let ozellikler = gdk::WindowAttr {
                window_type: gdk::WindowType::Child,
                wclass: gdk::WindowWindowClass::InputOutput,
                x: Some(0),
                y: Some(0),
                width: 1,
                height: 1,
                // Olay maskesi BOŞ: bu pencere hiçbir olay işlemiyor, tıpkı
                // Windows'taki `STATIC` gibi. Girdiyi mpv kendi alt
                // penceresinde alıyor.
                event_mask: gdk::EventMask::empty(),
                ..Default::default()
            };
            let alt = gdk::Window::new(Some(&ana), &ozellikler);

            let xid = alt
                .clone()
                .downcast::<gdkx11::X11Window>()
                .map_err(|_| {
                    Hata::yeni(
                        "X11 dışı bir oturum (büyük olasılıkla Wayland); mpv kendi \
                         penceresini açacak",
                    )
                })?
                .xid();

            alt.show();

            let yuzey = Yuzey::yok();
            yuzey.mpv_id.store(xid as isize, Ordering::Relaxed);
            // `to_glib_full` bir referans EKLİYOR ve o referans bilerek
            // bırakılıyor: `alt` düştüğünde pencere yok olmamalı, uygulama
            // boyunca yaşamalı — Windows'taki HWND gibi.
            let ham: *mut gdk::ffi::GdkWindow = alt.to_glib_full();
            yuzey.yerel.store(ham as isize, Ordering::Relaxed);
            Ok(yuzey)
        }

        /// Ölçüler MANTIKSAL piksel; GTK3 pencere koordinatları da öyle
        /// (HiDPI'ı GDK kendi ölçek çarpanıyla hallediyor). Çarpan
        /// uygulanmıyor — uygulamak videoyu iki katı büyüklükte çizerdi.
        pub fn konumla(
            &self,
            app: &tauri::AppHandle,
            x: f64,
            y: f64,
            en: f64,
            boy: f64,
        ) -> Sonuc<()> {
            let Some(p) = self.yerel_ptr() else {
                return Ok(());
            };
            ana_iste(app, move || {
                let Some(w) = sar(p) else { return };
                w.move_resize(
                    x.round() as i32,
                    y.round() as i32,
                    (en.round() as i32).max(1),
                    (boy.round() as i32).max(1),
                );
                // `raise`: yüzey webview'in üstünde kalmalı. Kardeş native
                // pencereler istifte sıralı ve webview yeniden boyutlandığında
                // öne geçebiliyor.
                w.raise();
            })
        }

        pub fn goster(&self, app: &tauri::AppHandle, gorunur: bool) -> Sonuc<()> {
            let Some(p) = self.yerel_ptr() else {
                return Ok(());
            };
            ana_iste(app, move || {
                let Some(w) = sar(p) else { return };
                if gorunur {
                    w.show();
                    w.raise();
                } else {
                    w.hide();
                }
            })
        }
    }

    /// Ham işaretçiyi geçici bir `gdk::Window`a sarar.
    ///
    /// `from_glib_none`: sarmalayıcı KENDİ referansını alıyor ve düştüğünde
    /// onu bırakıyor. [`Yuzey::olustur`]'da tutulan asıl referansa
    /// dokunmuyor; ona dokunmak pencereyi ilk taşımada yok etmek olurdu.
    ///
    /// **Yalnız ana iş parçacığında çağrılmalı.**
    fn sar(p: isize) -> Option<gdk::Window> {
        if p == 0 {
            return None;
        }
        Some(unsafe { from_glib_none(p as *mut gdk::ffi::GdkWindow) })
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::NSView;
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    impl Yuzey {
        /// Pencerenin içerik görünümüne mpv için bir alt görünüm ekler.
        ///
        /// Windows ve Linux'takiyle aynı fikir, AppKit karşılığıyla: yeni bir
        /// `NSView` yaratılıp işaretçisi mpv'ye `wid` olarak veriliyor (mpv
        /// `--wid`'i macOS'ta `NSView*` olarak okuyor).
        ///
        /// `ns_view()` pencerenin İÇERİK görünümünü döndürüyor, webview'inkini
        /// değil (Tauri onu `Window::ns_view`e devrediyor) — webview de o
        /// içerik görünümünün bir alt görünümü. `addSubview` sona ekliyor,
        /// yani yeni görünüm istifte webview'in ÜSTÜNDE kalıyor; Windows'taki
        /// `HWND_TOP` ile aynı sonuç.
        ///
        /// **Ana iş parçacığından çağrılmalı.** `NSView` AppKit'te ana iş
        /// parçacığına bağlı; `MainThreadMarker::new()` bunu derleme değil
        /// çalışma zamanında sınıyor ve olmadığında hata dönüyor. Tauri'nin
        /// `setup` kancası ana iş parçacığında çalışıyor.
        pub fn olustur(pencere: &tauri::WebviewWindow) -> Sonuc<Yuzey> {
            let mtm = MainThreadMarker::new()
                .ok_or_else(|| Hata::yeni("video yüzeyi ana iş parçacığında açılmalı"))?;

            let ham = pencere.ns_view()?;
            if ham.is_null() {
                return Err(Hata::yeni("pencerenin NSView'ı alınamadı"));
            }
            // Ömrü pencerenin ömrü, yani başvuru almaya gerek yok.
            let ust: &NSView = unsafe { &*(ham as *const NSView) };

            let alt = NSView::initWithFrame(
                NSView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1.0, 1.0)),
            );
            // Katman ŞART: mpv görüntüyü bu görünümün katmanına çiziyor.
            // Katmansız bir görünümde video hiç görünmüyor.
            alt.setWantsLayer(true);
            ust.addSubview(&alt);

            let yuzey = Yuzey::yok();
            // `into_raw` sahipliği bırakıyor ve bu bilerek: görünümü artık
            // üst görünüm tutuyor (`addSubview` başvuru alıyor) ve yüzey
            // uygulama boyunca yaşamalı.
            let p = Retained::into_raw(alt) as isize;
            yuzey.mpv_id.store(p, Ordering::Relaxed);
            yuzey.yerel.store(p, Ordering::Relaxed);
            Ok(yuzey)
        }

        /// Ölçüler MANTIKSAL piksel; AppKit çerçeveleri de NOKTA cinsinden,
        /// yani ölçek çarpanı uygulanmıyor (Retina'da uygulamak videoyu iki
        /// katı büyüklükte çizerdi).
        ///
        /// Y ekseni ÇEVRİLİYOR. Bizim dikdörtgenimizin başlangıcı sol ÜST
        /// (CSS), AppKit'in çevrilmemiş bir görünümde sol ALT. Çevirmeden
        /// verildiğinde video, sahnenin olması gereken yerin dikey aynasında
        /// duruyor.
        pub fn konumla(
            &self,
            app: &tauri::AppHandle,
            x: f64,
            y: f64,
            en: f64,
            boy: f64,
        ) -> Sonuc<()> {
            let Some(p) = self.yerel_ptr() else {
                return Ok(());
            };
            ana_iste(app, move || {
                let gorunum: &NSView = unsafe { &*(p as *const NSView) };
                let Some(ust) = (unsafe { gorunum.superview() }) else {
                    return;
                };
                let en = en.max(1.0);
                let boy = boy.max(1.0);
                let tepe = ust.frame().size.height - y - boy;
                gorunum.setFrame(NSRect::new(NSPoint::new(x, tepe), NSSize::new(en, boy)));
            })
        }

        pub fn goster(&self, app: &tauri::AppHandle, gorunur: bool) -> Sonuc<()> {
            let Some(p) = self.yerel_ptr() else {
                return Ok(());
            };
            ana_iste(app, move || {
                let gorunum: &NSView = unsafe { &*(p as *const NSView) };
                gorunum.setHidden(!gorunur);
            })
        }
    }
}
