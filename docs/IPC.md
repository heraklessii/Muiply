# IPC — Tauri Komutları ve Olayları

Bu dosya **sözleşme**. Rust tarafı (`src-tauri/src/commands/`) ve arayüz
sarmalayıcıları (`src/ipc/index.ts`) buraya uyuyor. Bir komut değişecekse
önce burası değişir — iki uçtan biri sessizce kayarsa hata çalışma zamanına
kadar görünmüyor.

Alan adları **İngilizce**, kodun geri kalanı Türkçe. Sınır burası.

## Oynatıcı komutları

| Komut | Parametre | Döner |
|---|---|---|
| `player_open` | `{ path }` | `Result<()>` |
| `player_play` | — | `Result<()>` |
| `player_pause` | — | `Result<()>` |
| `player_toggle_pause` | — | `Result<()>` |
| `player_seek` | `{ position }` | `Result<()>` |
| `player_seek_relative` | `{ delta }` | `Result<()>` |
| `player_set_volume` | `{ volume }` | `Result<()>` |
| `player_set_mute` | `{ muted }` | `Result<()>` |
| `player_stop` | — | `Result<()>` |
| `player_set_playback_rate` | `{ rate }` | `Result<()>` |
| `player_get_state` | — | `PlayerState` |
| `player_get_tracks` | — | `Result<Track[]>` |
| `player_set_video_rect` | `{ x, y, width, height }` | `Result<()>` |
| `player_set_video_visible` | `{ visible }` | `Result<()>` |
| `app_open_paths` | `{ paths: string[] }` | `Result<()>` |

Notlar:

- **`player_open` künye DÖNMÜYOR.** İlk taslak `Result<FileInfo>` diyordu ama
  mpv dosyayı eşzamansız açıyor; burada beklemek arayüzü dosya çözümlenene
  kadar dondurmak olurdu. Künye `player://file-loaded` ile geliyor.
- **`player_get_state` `Result` değil.** Durumun bir kopyası bellekte
  (`MpvState`), okumak başarısız olamıyor.
- **`player_set_video_rect` CSS pikseli alıyor** ve arayüz onu olduğu gibi
  gönderiyor. Ekran ölçeğine çevirilip çevirilmeyeceğine `mpv/yuzey.rs`in
  platform modülü karar veriyor: Windows'ta HWND koordinatları fiziksel
  piksel, GTK3 ve AppKit'te mantıksal. Çarpanı ortak yola koymak HiDPI bir
  Linux ya da Mac ekranında videoyu iki katı büyüklükte çizerdi.
- **`app_open_paths`** sürükle-bırakın karşılığı. "Klasör mü dosya mı" kararı
  backend'de, çünkü arayüzün dosya sistemine erişimi yok
  (`capabilities/default.json`) ve uzantıya bakıp tahmin etmek "My.Videos"
  adlı bir klasörde yanlış cevap verir.
- **Aynı işe IPC'siz de geliniyor.** Komutun gövdesi `acilis::yollari_ac`;
  açılışta komut satırı (çift tıklanan dosya) ve uygulama açıkken ikinci
  örneğin argümanları aynı yere düşüyor, arayüz hiç yokken bile. Bu yüzden
  karar `commands/`de değil `acilis.rs`de.

## Oynatıcı olayları (backend → arayüz)

| Olay | Yük |
|---|---|
| `player://file-loaded` | `FileInfo` |
| `player://time-pos` | `{ position }` |
| `player://pause-change` | `{ paused }` |
| `player://volume-change` | `{ volume? , muted? }` |
| `player://end-file` | `{ reason: "eof" \| "stop" \| "quit" \| "error" }` |
| `player://duration-change` | `{ duration }` |
| `player://rate-change` | `{ rate }` |
| `player://error` | `string` |
| `player://cleared` | — |
| `player://request` | `"tam-ekran" \| "tam-ekran-kapat" \| "sonraki" \| "onceki" \| "altyazi"` |

- **`duration-change` ŞART, künyedeki süre yetmiyor.** mpv çoğu kapsayıcıda
  (MKV, AVI) süreyi `file-loaded`dan sonra öğreniyor. Yalnız künyeye
  güvenildiğinde arayüzdeki süre 0 kalıyor, arama sürgüsü de 0 uzunlukta bir
  aralık için devre dışı çiziliyor — kullanıcı için "sarma çalışmıyor".
- **`rate-change`** aynı sebepten: hız her dosya başında varsayılana dönüyor
  (`playlist/surucu.rs`) ve bunu yapan arayüz değil. Haber verilmezse çubuk
  bir önceki dosyanın hızını gösteriyor.
- **`volume-change` iki şeyi taşıyor.** Ses düzeyi ve sessizlik ayrı mpv
  özellikleri ama arayüzde tek bir düğme; hangisi değiştiyse o alan dolu
  geliyor.
- **`player://cleared`** oynatıcıda dosya KALMADI: kuyruk bitti ya da
  `player_stop` çağrıldı. `end-file`den ayrı bir olay çünkü `end-file`
  yalnız "çalmıyor" diyor; hangi dosyanın yüklü olduğunu söylemiyor. mpv
  biten dosyayı boşaltıyor (`keep-open` kapalı) ve arayüz bunu bilmezse
  bitmiş videoyu duruyor gibi gösteriyor, sürgü de boş mpv'ye `seek`
  gönderip hata üretiyor. Arayüz dosyaya ait alanları sıfırlıyor; ses ve
  sessizlik kalıyor.
- **`player://request`** mpv penceresinden gelen kısayollar. Video üstündeyken
  tuşlar webview'e ulaşmıyor; mpv `script-message` ile geri gönderiyor.
  Gerekçe `docs/Mpv_Integration.md` → "Kısayollar mpv'de".
- `time-pos` mpv'den **kare hızında** geliyor ama olay olarak 100 ms'de bir
  yayınlanıyor; arayüz onu ayrıca 200 ms'de bir işliyor. Kısıtlama yalnız
  yayında: `player_get_state` her zaman son konumu veriyor.

## Kütüphane komutları

| Komut | Parametre | Döner |
|---|---|---|
| `library_add_folder` | `{ path }` | `Result<()>` |
| `library_remove_folder` | `{ path }` | `Result<()>` |
| `library_get_folders` | — | `Result<string[]>` |
| `library_scan` | — | `Result<()>` |
| `library_get_media` | `{ filter?: MediaFilter }` | `Result<MediaItem[]>` |
| `library_get_recent` | `{ limit }` | `Result<MediaItem[]>` |
| `library_delete_media` | `{ id }` | `Result<()>` |
| `library_get_item` | `{ id }` | `Result<MediaItem \| null>` |

- `library_add_folder` taramayı **kendiliğinden başlatıyor**: klasör ekleyip
  boş bir kütüphane görmek, kullanıcıya bir şeyin bozulduğunu düşündürür.
- `library_scan` hemen dönüyor; iş arka planda, ilerleme olayla.
- `library_delete_media` **dosyayı silmiyor**, kaydı siliyor. Arayüzdeki metin
  de "Kütüphaneden kaldır".
- `MediaFilter`de **arama yok**: arayüz yazdıkça bellekte süzüyor
  (`src/lib/suz.ts`).

## Kütüphane olayları

| Olay | Yük |
|---|---|
| `library://scan-progress` | `{ done, total }` |
| `library://scan-complete` | `{ added, updated, removed }` |
| `library://media-updated` | `{ id }` |

`media-updated` TEK bir kaydın değiştiğini söylüyor ve iki yerden çıkıyor:
küçük resim yazıldığında (`library/kucukresim.rs`) ve kayıt çalınmaya
başladığında (`playlist/surucu.rs` — `play_count` artıyor, `last_played`
yazılıyor).

Arayüz bütün listeyi değil tek satırı tazeliyor: ızgarayı baştan kurmak
kaydırma konumunu sıfırlardı. Tek istisna "Son çalınanlar": onun SIRASINI
`last_played` belirlediği için satırı yerinde güncellemek yetmiyor, liste
yeniden çekiliyor (`library_get_recent`).

İkinci kaynak sonradan eklendi. Yokken kütüphane penceresindeki "Son
çalınanlar" bir oturum boyunca hiç güncellenmiyordu: kullanıcı bir dosya
çalıyor, veritabanı doğruyu yazıyor, arayüzün haberi olmuyordu.

## Çalma listesi komutları

| Komut | Parametre | Döner |
|---|---|---|
| `playlist_create` | `{ name }` | `Result<Playlist>` |
| `playlist_delete` | `{ id }` | `Result<()>` |
| `playlist_rename` | `{ id, name }` | `Result<()>` |
| `playlist_get_all` | — | `Result<Playlist[]>` |
| `playlist_get_items` | `{ id }` | `Result<PlaylistItem[]>` |
| `playlist_add_item` | `{ playlistId, mediaId }` | `Result<()>` |
| `playlist_remove_item` | `{ playlistId, itemId }` | `Result<()>` |
| `playlist_reorder` | `{ playlistId, from, to }` | `Result<()>` |
| `playlist_play` | `{ id, startIndex? }` | `Result<()>` |

**`playlist_remove_item` SATIR KİMLİĞİ alıyor** (`PlaylistItem.itemId`), sıra
değil. Önceki sürüm sıra alıyordu, çünkü `playlist_get_items` `MediaItem`
döndürüyordu ve onun kimliği medyanın kimliği — satırın değil; arayüzün elinde
satır kimliği hiç olmuyordu. Sıranın sorunu, arayüzün listeyi gördüğü an ile
silmenin yürüdüğü an arasında listenin değişebilmesi: araya bir ekleme ya da
bir sürükleme girdiğinde, kullanıcının işaret ettiği satır ile o sıradaki
satır aynı olmuyor ve **yanlış öğe siliniyordu**. `PlaylistItem` iki kimlik
birden taşıyor; satır kimliği değişmiyor.

`playlist_reorder` hâlâ SIRA alıyor ve bu doğru: sürükle-bırakın anlattığı şey
zaten "şu sıradaki, bu sıraya" ve iki uç da aynı boyamadan geliyor.

**`playlist_get_items` `MediaItem` değil `PlaylistItem` döndürüyor.** Aradaki
tek fark `itemId`; medya alanları düz, iç içe değil (Rust tarafında
`#[serde(flatten)]`), yani ızgarada ve listede aynı bileşenler aynı alan
adlarını okuyor. İki kimliğin ikisi de gerekiyor çünkü **aynı kayıt bir
listede iki kez bulunabiliyor** — `media.id` o iki satırı ayırmıyor.

## Kuyruk komutları

| Komut | Parametre | Döner |
|---|---|---|
| `queue_set` | `{ mediaIds, startIndex? }` | `Result<()>` |
| `queue_get` | — | `Result<QueueSnapshot>` |
| `queue_play_at` | `{ index }` | `Result<()>` |
| `queue_next` | — | `Result<bool>` |
| `queue_prev` | — | `Result<bool>` |
| `queue_set_repeat` | `{ repeat }` | `Result<()>` |
| `queue_set_shuffle` | `{ shuffle }` | `Result<()>` |

`queue_next`/`queue_prev` `bool` dönüyor: liste bittiyse `false`. Hata değil,
"yapacak bir şey yoktu".

Kuyruk **backend'de** duruyor. Gerekçe `src-tauri/src/playlist/kuyruk.rs`
başında: dosya bitince sıradakine geçme kararını mpv'nin olay döngüsü veriyor
ve o karar arayüzün açık olup olmamasından bağımsız olmalı.

## Kuyruk olayı

| Olay | Yük |
|---|---|
| `playlist://queue` | `QueueSnapshot` |

## Altyazı komutları

| Komut | Parametre | Döner |
|---|---|---|
| `subtitle_get_tracks` | — | `Result<Track[]>` |
| `subtitle_select` | `{ trackId: number \| null }` | `Result<()>` |
| `subtitle_add_file` | `{ path }` | `Result<()>` |
| `subtitle_set_delay` | `{ seconds }` | `Result<()>` |
| `subtitle_get_delay` | — | `Result<number>` |
| `subtitle_find_nearby` | `{ path }` | `NearbySubtitle[]` |

`subtitle_select(null)` altyazıyı kapatıyor. mpv'de karşılığı `sid=no` — sayı
değil metin; `sid=0` "sıfır numaralı iz" demek değil, mpv'de iz numaraları
1'den başlıyor.

## Pencere komutları

Muiply'ın iki penceresi var ve ikisi de **açılışta yaratılıyor**, ikisi de
başlangıçta gizli: `oynatici` (video + çubuk) ve `kutuphane` (kütüphane,
listeler, kuyruk, ayarlar). Hangisinin görüneceğine backend karar veriyor.

| Komut | Parametre | Döner |
|---|---|---|
| `window_show_player` | — | `Result<()>` |
| `window_show_library` | — | `Result<()>` |

- **İkisi de yaratmıyor, GÖSTERİYOR.** Pencereler zaten var; komut gizli
  olanı gösterip öne alıyor. Sebep mpv: çizeceği pencereyi (`wid`) yalnız
  başlatılırken alıyor, sonradan değiştirilemiyor. Oynatıcı penceresi
  açılışta yoksa videonun gidecek yeri hiç olmuyor.
- **Oynatıcıyı arayüz açmıyor, backend açıyor.** Video yüklendiğinde
  (`player://file-loaded`, `mediaType == "video"`) pencere kendiliğinden
  görünüyor — karar mpv'nin olay döngüsünde, çünkü kuyruk kendi kendine de
  ilerliyor ve o anda arayüzün açık olması gerekmiyor. `window_show_player`
  yalnız kullanıcı açıkça istediğinde (kütüphanedeki "Oynatıcı" düğmesi)
  çağrılıyor.
- **Ses için pencere AÇILMIYOR.** Müzik çalarken kütüphanede gezinmek doğru
  davranış; her parça başında öne fırlayan bir pencere değil.
- **Kapatma gerçekten kapatıyor.** Bir pencerenin X'i onu gizliyor; görünen
  son pencere kapatıldığında uygulama çıkıyor. Tepsiye inen gizli bir
  uygulama bırakmıyoruz (`docs/Roadmap.md`, kapsam dışı).

## Ayar komutları

| Komut | Parametre | Döner |
|---|---|---|
| `settings_get` | — | `Settings` |
| `settings_set` | `{ settings: Settings }` | `Result<Settings>` |
| `settings_audio_devices` | — | `AudioDevice[]` |

- **`settings_get` `Result` değil.** Ayarların bir kopyası bellekte
  (`SettingsState`); okumak başarısız olamaz. Veritabanı açılışta okunuyor,
  okunamayan alan varsayılana düşüyor.
- **`settings_set` DÜZELTİLMİŞ ayarı geri döndürüyor.** Aralık dışı bir hız ya
  da tanınmayan bir `hwdec` hata değil, sessizce sınıra çekiliyor; arayüz
  gönderdiğini değil dönen değeri gösteriyor, yoksa kutuda duran sayı ile
  mpv'nin gerçeği ayrışır.
- **`settings_audio_devices` `Result` değil.** Motorsuz derlemede liste BOŞ
  dönüyor: "aygıt yok" doğru cevap, hata değil. Arayüz o durumda yalnız
  "Sistem varsayılanı" seçeneğini gösteriyor.
- **Tema burada YOK.** Tema arayüzün `localStorage`'ında
  (`src/lib/platform.ts`) çünkü ilk boyamadan ÖNCE bilinmesi gerekiyor;
  backend'e sormak pencerenin bir kare yanlış renkte açılması demek olurdu.

## TypeScript tipleri

Tam liste `src/ipc/tipler.ts` içinde. Özet:

```typescript
interface PlayerState {
  playing: boolean;  paused: boolean;
  position: number;  duration: number;
  volume: number;    muted: boolean;   rate: number;
  path: string | null;
  title: string | null;
  mediaType: "video" | "audio" | null;
  /** Motor derlemeye dahil mi (docs/Mpv_Integration.md). */
  engine: boolean;
}

interface Track {
  id: number;
  kind: "video" | "audio" | "sub";
  title: string | null;  lang: string | null;
  selected: boolean;     external: boolean;
  codec: string | null;
}

interface MediaItem {
  id: string;  path: string;  title: string;
  artist: string | null;  album: string | null;
  duration: number;
  width: number | null;   height: number | null;
  size: number;
  mediaType: "video" | "audio";
  extension: string;
  /** Diskteki TAM yol; <img> için `convertFileSrc` gerekiyor. */
  thumbnail: string | null;
  addedAt: number;  lastPlayed: number | null;  playCount: number;
  /** Yarım bırakılan yer (saniye). Bitmiş ya da hiç açılmamış dosyada `null`. */
  lastPosition: number | null;
}

/** Bir çalma listesinin bir SATIRI. `itemId` satırın, `id` medyanın kimliği;
 *  aynı kayıt listede iki kez bulunabildiği için ikisi de gerekiyor. */
interface PlaylistItem extends MediaItem {
  itemId: number;
}

interface QueueSnapshot {
  items: QueueItem[];
  currentIndex: number | null;
  repeat: "off" | "all" | "one";
  shuffle: boolean;
}

interface Settings {
  /** `"auto-safe"` · `"auto"` · `"no"` */
  hwdec: string;
  /** mpv aygıt adı, ya da sistem varsayılanı için `"auto"`. */
  audioDevice: string;
  /** Her dosyanın başladığı hız. 0.25 - 4. */
  defaultRate: number;
  /** Kaldığı yerden devam edilsin mi. */
  resume: boolean;
  /** Klavyenin medya tuşları Muiply'yi mi yönetsin. */
  mediaKeys: boolean;
}

interface AudioDevice {
  name: string;
  description: string;
}
```

## Hatalar

Rust tarafı tek bir hata tipi kullanıyor (`hata.rs`): düz bir Türkçe cümle.
Bir hata ağacının karşılığı yok — burada üretilen her hata sonunda
kullanıcının duyuru şeridinde gördüğü bir cümleye dönüşüyor.
