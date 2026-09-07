//! Muiply masaüstü uygulamasının giriş noktası.
//!
//! Bu dosyanın tek işi Tauri'yi kurmak: video yüzeyini açmak, mpv motorunu
//! ve kütüphane veritabanını ayağa kaldırmak, durumları kaydetmek, komutları
//! bağlamak. İş mantığı burada değil — [`mpv`], [`library`], [`playlist`] ve
//! [`subtitle`] içinde.
//!
//! Kurulum sırası ÖNEMLİ ve şöyle: yüzey → motor → durumlar → olay döngüsü.
//! Olay döngüsü ilk olayında `MpvState` ve `LibraryState` arıyor; onlardan
//! önce başlarsa ilk saniyenin güncellemeleri sessizce düşer.

pub mod acilis;
pub mod commands;
pub mod hata;
pub mod library;
pub mod mpv;
pub mod pencere;
pub mod playlist;
pub mod settings;
pub mod subtitle;
pub mod tepsi;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use tauri::Manager;

use library::db::LibraryDb;
use library::LibraryState;
use mpv::yuzey::Yuzey;
use mpv::{Motor, MpvState};
use playlist::kuyruk::{Kuyruk, QueueState};
use settings::SettingsState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let kurucu = tauri::Builder::default();

    // Tekil örnek EN ÖNDE kuruluyor — eklentinin sözleşmesi bu: ikinci
    // süreci Tauri daha pencere açmadan sonlandırıyor.
    //
    // Varsayılan oynatıcı olmanın şartı: kullanıcı bir dosyaya çift
    // tıkladığında Windows Muiply'ı YENİDEN çalıştırıyor. Tekil örnek
    // olmasa her dosya ikinci bir pencere, ikinci bir mpv ve aynı SQLite
    // dosyasına ikinci bir yazar demek olurdu.
    #[cfg(desktop)]
    let kurucu = kurucu.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
        acilis::ikinci_ornek(app, argv);
    }));

    kurucu
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Yüzey OYNATICI penceresine bağlanıyor ve bu geri alınamaz:
            // mpv `wid`i yalnız başlatılırken alıyor. Pencerenin açılışta
            // (gizli de olsa) var olmasının sebebi bu — bkz. `pencere.rs`.
            let pencere = app
                .get_webview_window(pencere::OYNATICI)
                .ok_or("oynatıcı penceresi bulunamadı")?;

            // Yüzey açılamazsa (Windows dışı, ya da beklenmedik bir Win32
            // hatası) durmuyoruz: mpv kendi penceresini açar. Ayrı pencereli
            // çirkin bir oynatıcı, hiç açılmayan bir uygulamadan iyi.
            let yuzey = match Yuzey::olustur(&pencere) {
                Ok(y) => y,
                Err(e) => {
                    eprintln!("muiply: video yüzeyi yok, mpv kendi penceresini açacak ({e})");
                    Yuzey::yok()
                }
            };

            let motor = Motor::kur(yuzey.id())?;

            // Platformun standart dizini. Bulunamazsa çalışma dizinine
            // düşülüyor — açılamamaktansa alışılmadık bir yere yazmak yeğ.
            let kok = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let db = LibraryDb::ac(&kok)?;

            // Ayarlar motor DURUMA KONMADAN uygulanıyor: `manage` motoru
            // içeri taşıyor ve sonrasında ona ancak `State` üzerinden
            // erişilebiliyor. Hata durdurmuyor — `hwdec` yazılamamış bir
            // oynatıcı yavaş çalışır, hiç açılmayan bir oynatıcıdan iyidir.
            let ayarlar = settings::oku(&db);
            // `manage` ayarları içeri taşıyor ve medya tuşları kurulumun
            // sonunda okunuyor; tek bir bayrak burada kopyalanıyor.
            let medya_tuslari_acik = ayarlar.media_keys;
            if let Err(e) = settings::uygula(&motor, &ayarlar) {
                eprintln!("muiply: ayarlar uygulanamadı ({e})");
            }

            app.manage(MpvState::yeni(motor, yuzey));
            app.manage(SettingsState::yeni(ayarlar));
            app.manage(LibraryState {
                db: Mutex::new(db),
                kok,
                taraniyor: AtomicBool::new(false),
                bekleyen_kare: Mutex::new(None),
            });
            app.manage(QueueState(Mutex::new(Kuyruk::default())));

            app.state::<MpvState>()
                .motor
                .olaylari_yayinla(app.handle().clone());

            // Tepsi ve medya tuşları EN SON: ikisi de menüden gelen bir
            // tıklamayla `MpvState`/`QueueState` arıyor ve durumlar
            // yerleşmeden kurulurlarsa ilk tıklama "oynatıcı hazır değil"
            // derdi.
            //
            // Hata durdurmuyor. Tepsi Linux'ta appindicator olmadan
            // kurulamıyor ve kısayol kaydını başka bir uygulama almış
            // olabilir; ikisi de kullanıcının Muiply içinde çözemeyeceği
            // şeyler. Tepsisiz bir oynatıcı, hiç açılmayandan iyi.
            if let Err(e) = tepsi::kur(app.handle()) {
                eprintln!("muiply: sistem tepsisi kurulamadı ({e})");
            }
            if let Err(e) = tepsi::medya_tuslari::eklentiyi_kur(app.handle()) {
                eprintln!("muiply: medya tuşları eklentisi kurulamadı ({e})");
            } else {
                tepsi::medya_tuslari::uygula(app.handle(), medya_tuslari_acik);
            }

            // Komut satırındaki dosya EN SON: açmak kuyruğa, kütüphaneye ve
            // mpv'ye aynı anda dokunuyor, üçü de yukarıda yerleşiyor. Daha
            // erken çağrılsaydı "oynatıcı hazır değil" derdi.
            let dosyayla_acildi = acilis::baslangicta_ac(app.handle());

            // İki pencere de gizli doğuyor; hangisinin görüneceği burada
            // belli oluyor. Dosyaya çift tıklayan kullanıcı oynatıcı
            // istiyor, Başlat menüsünden açan kütüphane. Yanlış tarafa
            // düşmek "video açtım, karşıma kütüphane çıktı" demek.
            pencere::goster(
                app.handle(),
                if dosyayla_acildi {
                    pencere::OYNATICI
                } else {
                    pencere::KUTUPHANE
                },
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::player::player_open,
            commands::player::player_play,
            commands::player::player_pause,
            commands::player::player_toggle_pause,
            commands::player::player_seek,
            commands::player::player_seek_relative,
            commands::player::player_set_volume,
            commands::player::player_set_mute,
            commands::player::player_stop,
            commands::player::player_set_playback_rate,
            commands::player::player_get_state,
            commands::player::player_get_tracks,
            commands::player::player_set_video_rect,
            commands::player::player_set_video_visible,
            commands::player::app_open_paths,
            //
            commands::pencere::window_show_player,
            commands::pencere::window_show_library,
            //
            commands::library::library_add_folder,
            commands::library::library_remove_folder,
            commands::library::library_get_folders,
            commands::library::library_scan,
            commands::library::library_get_media,
            commands::library::library_get_recent,
            commands::library::library_delete_media,
            commands::library::library_get_item,
            //
            commands::playlist::playlist_create,
            commands::playlist::playlist_delete,
            commands::playlist::playlist_rename,
            commands::playlist::playlist_get_all,
            commands::playlist::playlist_get_items,
            commands::playlist::playlist_add_item,
            commands::playlist::playlist_remove_item,
            commands::playlist::playlist_reorder,
            commands::playlist::playlist_play,
            commands::playlist::queue_set,
            commands::playlist::queue_get,
            commands::playlist::queue_play_at,
            commands::playlist::queue_next,
            commands::playlist::queue_prev,
            commands::playlist::queue_set_repeat,
            commands::playlist::queue_set_shuffle,
            //
            commands::subtitle::subtitle_get_tracks,
            commands::subtitle::subtitle_select,
            commands::subtitle::subtitle_add_file,
            commands::subtitle::subtitle_set_delay,
            commands::subtitle::subtitle_get_delay,
            commands::subtitle::subtitle_find_nearby,
            //
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::settings::settings_audio_devices,
        ])
        // Pencere kapanırken kaldığı yer yazılıyor. Olay döngüsü zaten
        // düzenli aralıkla yazıyor (`mpv/gercek.rs`) ama aradaki birkaç
        // saniye tam da kullanıcının "kapattığım an" saydığı yer.
        .on_window_event(|pencere_ref, olay| {
            let tauri::WindowEvent::CloseRequested { api, .. } = olay else {
                return;
            };
            let app = pencere_ref.app_handle();

            library::devam::simdiki_konumu_kaydet(app);

            // Pencere YOK EDİLMİYOR, gizleniyor: yok edilen oynatıcı
            // penceresiyle birlikte mpv'nin çizdiği yüzey de giderdi ve bir
            // daha video açılamazdı. Görünen son pencere kapanınca uygulama
            // gerçekten çıkıyor (`pencere::kapatma_istegi`).
            api.prevent_close();
            pencere::kapatma_istegi(pencere_ref);
        })
        .run(tauri::generate_context!())
        .expect("Muiply penceresi açılamadı");
}
