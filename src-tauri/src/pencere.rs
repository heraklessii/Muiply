//! İki pencere: **oynatıcı** ve **kütüphane**.
//!
//! Muiply tek pencereli değil çünkü bir video oynatırken yan sütun, kütüphane
//! ızgarası ve tepe çubuğu gereksiz — insanlar oynatıcıdan oynatıcı gibi
//! davranmasını bekliyor. Kütüphane, listeler, kuyruk ve ayarlar kendi
//! penceresinde.
//!
//! **İkisi de açılışta yaratılıyor** (`tauri.conf.json` > `app.windows`) ve
//! ikisi de gizli başlıyor; hangisinin görüneceğine backend karar veriyor.
//! Sonradan yaratmak mümkün değil: mpv çizeceği pencereyi (`wid`) yalnız
//! başlatılırken alıyor ve o pencere `oynatici`. Açılışta var olmasaydı
//! videonun gidecek bir yeri hiç olmazdı (bkz. `mpv/yuzey.rs`).
//!
//! Aynı arayüz paketi iki pencerede de çalışıyor; hangisi olduğunu
//! `getCurrentWindow().label` söylüyor (`src/main.tsx`).

use tauri::{AppHandle, Manager, Window};

/// mpv'nin çizdiği pencere. Etiket `yuzey.rs` ve `capabilities/default.json`
/// ile aynı olmak zorunda.
pub const OYNATICI: &str = "oynatici";

/// Kütüphane, çalma listeleri, kuyruk, ayarlar.
pub const KUTUPHANE: &str = "kutuphane";

/// Pencereyi gösterip ÖNE alır — kullanıcı açıkça istediğinde.
///
/// Üç adım da gerekiyor: gizliyse `show`, küçültülmüşse `unminimize`, arkada
/// kalmışsa `set_focus`. Biri eksikken tıklama hiçbir şey yapmamış gibi
/// görünüyor.
pub fn goster(app: &AppHandle, etiket: &str) {
    let Some(pencere) = app.get_webview_window(etiket) else {
        return;
    };
    let _ = pencere.show();
    let _ = pencere.unminimize();
    let _ = pencere.set_focus();
}

/// Gizliyse gösterir, GÖRÜNÜRSE dokunmaz.
///
/// Kendiliğinden olan gösterme buradan geçiyor: kuyruk ilerleyip yeni bir
/// video başladığında pencere gerekiyor, ama kullanıcı o sırada kütüphanede
/// bir şey ararken odağı elinden almak istemiyoruz. Zaten görünen bir
/// pencereyi öne fırlatmanın da anlamı yok.
pub fn gorunur_yap(app: &AppHandle, etiket: &str) {
    let Some(pencere) = app.get_webview_window(etiket) else {
        return;
    };
    if pencere.is_visible().unwrap_or(false) {
        return;
    }
    let _ = pencere.show();
}

/// Pencerenin X'ine basıldığında.
///
/// Pencere yok edilmiyor, GİZLENİYOR: yok edilen pencere geri gelmiyor ve
/// oynatıcı yok edilirse mpv'nin çizdiği yüzey de gidiyor — bir daha video
/// açılamıyor.
///
/// Görünen son pencere kapatıldığında uygulama gerçekten çıkıyor. Tepsiye
/// inen, kapattığını sanan kullanıcının arkasında çalışmaya devam eden bir
/// uygulama bırakmıyoruz (`docs/Roadmap.md`, kapsam dışı).
/// Tür `Window`, `WebviewWindow` değil: `on_window_event` bize bunu veriyor.
pub fn kapatma_istegi(pencere: &Window) {
    let app = pencere.app_handle();
    let _ = pencere.hide();

    let baska_gorunur = [OYNATICI, KUTUPHANE]
        .iter()
        .filter(|e| **e != pencere.label())
        .any(|e| {
            app.get_webview_window(e)
                .and_then(|p| p.is_visible().ok())
                .unwrap_or(false)
        });

    if !baska_gorunur {
        app.exit(0);
    }
}
