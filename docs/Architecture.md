# Mimari

## Süreç modeli

```
┌─────────────────────────────────────────────────────────┐
│                   Tauri uygulama süreci                  │
│                                                          │
│  ┌────────────────┐    ┌─────────────────────────────┐  │
│  │  Rust          │◄IPC┤ WebView 1 — "oynatici"      │  │
│  │                │olay│ sahne kutusu + çubuk        │  │
│  │  MpvState      │    └─────────────────────────────┘  │
│  │  LibraryState  │◄IPC┐┌─────────────────────────────┐ │
│  │  QueueState    │olay└┤ WebView 2 — "kutuphane"     │ │
│  └───────┬────────┘     │ yan · ızgara · listeler     │ │
│          │              └─────────────────────────────┘ │
│  ┌───────▼────────┐   ┌──────────────────────────────┐   │
│  │    libmpv      │──►│  native alt pencere (wid)    │   │
│  │  kod çözme     │   │  OYNATICI webview'in ÜSTÜNDE │   │
│  └────────────────┘   └──────────────────────────────┘   │
│                                                          │
│  ┌────────────────┐                                      │
│  │  SQLite (WAL)  │  $APPDATA/com.mui.muiply/library.db  │
│  └────────────────┘                                      │
└─────────────────────────────────────────────────────────┘
```

Video, arayüzün içinde değil **üstünde**. Sebebi ve sonuçları
`docs/Mpv_Integration.md` → "Video nasıl çiziliyor".

## İki pencere

Tek süreç, tek Rust durumu, **iki pencere**: `oynatici` (video + çubuk) ve
`kutuphane` (ızgara, listeler, kuyruk, ayarlar). İkisi de aynı arayüz
paketini yüklüyor; hangisi olduğunu `getCurrentWindow().label` söylüyor
(`src/main.tsx`).

Ayrılmalarının sebebi ürün: bir film izlerken yanda duran kütüphane sütunu,
oynatıcıyı oynatıcı gibi hissettirmiyordu. Çift tıklanan bir dosya doğrudan
oynatıcıyı açıyor, kütüphane hiç görünmüyor.

Üç kural bu yapıyı ayakta tutuyor:

- **İkisi de AÇILIŞTA yaratılıyor** (`tauri.conf.json` > `app.windows`) ve
  gizli başlıyor. Sonradan yaratmak mümkün değil: mpv çizeceği pencereyi
  (`wid`) yalnız başlatılırken alıyor ve o pencere `oynatici`.
- **Hangisinin görüneceğine backend karar veriyor** (`src-tauri/src/pencere.rs`).
  Açılışta: dosyayla açıldıysa oynatıcı, düz açıldıysa kütüphane. Sonra:
  video yüklendiğinde oynatıcı kendiliğinden görünüyor — kuyruk kendi
  kendine de ilerliyor ve o anda hiçbir arayüzün açık olması gerekmiyor.
- **Kapatma pencereyi YOK ETMİYOR, gizliyor.** Yok edilen oynatıcı
  penceresiyle mpv'nin yüzeyi de giderdi. Görünen son pencere kapatıldığında
  uygulama gerçekten çıkıyor.

İki webview aynı olayları alıyor ve ikisi de durumu backend'den okuyor; biri
kapalıyken öbürü eksik çalışmıyor. "Karar veren kod backend'de" kuralının
bedeli burada geri ödeniyor.

## Klasörler

```
muiply/
├── CLAUDE.md
├── docs/
├── src/                      ← arayüz
│   ├── pencereler/           OynaticiPenceresi · KutuphanePenceresi
│   ├── components/           Tepe Yan Sahne Cubuk Surgu Kart Ayarlar
│   │                         Kutuphane ListePaneli KuyrukPaneli
│   │                         Menu Bildirimler Kisayollar TemaDugmesi
│   │                         Birak HataSiniri Ikonlar
│   ├── hooks/                useOynatici useKutuphane useKuyruk
│   │                         useListeler useSahneOlcu useKumanda
│   │                         useBirakma useBildirimler useAyarlar useTema
│   ├── ipc/                  index.ts (sarmalayıcılar) · tipler.ts
│   ├── lib/                  sure suz dil platform uzantilar (+ testleri)
│   ├── assets/fonts/         gömülü Outfit
│   ├── styles.css            Mui jetonları + düzen
│   └── main.tsx              pencere etiketine göre kökü seçiyor
└── src-tauri/
    ├── src/
    │   ├── lib.rs            kurulum ve komut kaydı
    │   ├── hata.rs           tek hata tipi
    │   ├── mpv/
    │   │   ├── mod.rs        MpvState, PlayerState, Track, izleri_oku
    │   │   ├── gercek.rs     libmpv sarmalayıcısı + olay döngüsü
    │   │   ├── yok.rs        motorsuz derlemenin karşılığı
    │   │   ├── yuzey.rs      native alt pencere (Win32 · GDK · NSView)
    │   │   ├── oynatici.rs   aç / duraklat / ara / ses
    │   │   └── sonda.rs      tarama için ikinci mpv örneği
    │   ├── library/
    │   │   ├── mod.rs        LibraryState, MediaItem, uzantılar
    │   │   ├── db.rs         şema, geçişler ve sorgular
    │   │   ├── tarayici.rs   üç geçişli tarama
    │   │   ├── devam.rs      SAF eşik kararı + kaldığı yeri yaz/oku
    │   │   └── kucukresim.rs çalarken kare alma
    │   ├── playlist/
    │   │   ├── mod.rs        liste CRUD
    │   │   ├── kuyruk.rs     SAF gezinme kararları + testler
    │   │   └── surucu.rs     kuyruğu oynatıcıya bağlayan katman
    │   ├── subtitle/mod.rs   yanındaki dosyaları bulma + testler
    │   ├── settings/mod.rs   kalıcı tercihler + mpv'ye uygulama
    │   ├── tepsi.rs          sistem tepsisi + medya tuşları
    │   └── commands/         player library playlist subtitle settings
    ├── libs/                 mpv-2.dll · mpv.lib (depoda YOK)
    ├── tauri.conf.json
    ├── tauri.windows.conf.json   nsis + msi
    ├── tauri.linux.conf.json     deb + bağımlılıklar
    ├── tauri.macos.conf.json     app + dmg
    └── tauri.paketleme.conf.json Windows'a DLL kopyalama
```

Platform yapılandırmalarını Tauri ana dosyayla **kendiliğinden**
birleştiriyor (ad kalıbı `tauri.<platform>.conf.json`); `tauri.paketleme`
öyle değil, elle bindiriliyor — sebebi `docs/Setup.md`'de.

## Katmanlama kuralı

```
commands/*.rs      ince sarmalayıcı — Tauri'ye bakan yüz
      │
modüller           kararlar burada
      │
mpv / rusqlite     dış dünya
```

**Komut dosyalarına iş mantığı yazılmıyor.** Sebebi tek bir cümlede: aynı
karara mpv'nin olay döngüsünden de gelinebiliyor. "Dosya bitti, sıradakine
geç" kararı hem `queue_next` komutundan hem `EndFile(eof)` olayından
tetikleniyor; iki yerde iki kopya olsaydı biri sessizce eskiyecekti.

Aynı sebeple **arayüzde iş mantığı yok**: kuyruk backend'de duruyor, çünkü
dosya bitince ne olacağını arayüz açık olmasa da bilen bir yer olmalı.

`tepsi.rs` bu kuralın en görünür sınavı. Tepsi menüsü ve medya tuşları
oynatmayı arayüz hiç açık değilken yönetiyor; ikisi de kendi "sonraki"sini
yazmıyor, `playlist::surucu`'ya gidiyor. Aynı karar üç kapıdan geliyor —
komut, mpv'nin olay döngüsü, tepsi — ve tek bir yerde duruyor.

## Saf çekirdek

İki modül tamamen saf ve testli — dış dünyaya hiç dokunmuyorlar:

- `playlist/kuyruk.rs` — `sonraki`, `onceki`, `karistir`. Bozulduğunda ortaya
  çıkan hata sessiz türden: "liste sonunda başa dönmüyor", "karışıkta aynı
  parça iki kez çalıyor".
- `subtitle/mod.rs` — ad eşleştirme. Gevşerse başka filmin altyazısı geliyor
  (`Bolum1` / `Bolum10`), sıkışırsa kullanıcının altyazısı hiç görünmüyor.
- `library/devam.rs::devam_konumu` — "kaldığı yerden devam edilecek mi".
  Eşikler yanlışsa hata sinsi: kullanıcı ya baştan başlayan ya da doğrudan
  jeneriğe atlayan bir dosya görüyor ve ikisinin de sebebi görünmüyor.
- `settings/mod.rs::Settings::duzelt` — aralık dışı değerleri sınıra çekme.

Arayüz tarafındaki karşılıkları `src/lib/sure.ts` ve `src/lib/suz.ts`.

## Durumlar

| Durum | Tip | Kim yazıyor |
|---|---|---|
| `MpvState` | `Motor` + `Yuzey` + `Mutex<PlayerState>` | olay döngüsü (tek yazar) |
| `LibraryState` | `Mutex<LibraryDb>` + kök + tarama bayrağı | komutlar ve tarayıcı |
| `QueueState` | `Mutex<Kuyruk>` | komutlar ve sürücü |
| `SettingsState` | `Mutex<Settings>` | `settings_set` komutu |

`Settings` de bir **kopya**: her dosya açılışında "kaldığı yerden devam açık
mı" ve "varsayılan hız ne" soruluyor; her seferinde veritabanına gitmek,
bilinen iki değer için kilit almak olurdu.

`PlayerState` bir **kopya**: arayüz açılışta ve her görünüm değişiminde durumu
soruyor; her seferinde mpv'ye sekiz ayrı özellik sormak, olay döngüsünün zaten
bildiği şeyi yeniden sormak olurdu.

`Motor` içinde Mutex **yok**: libmpv istemci API'si iş parçacığı güvenli
(`Mpv: Send + Sync`) ve kilit koymak, olay döngüsünün her tıklamada komutları
bekletmesi demekti.

## Veri akışı: dosya aç

```
Kullanıcı ızgarada bir karta tıklıyor
  → queue_set(görünen kayıtların kimlikleri, tıklananın indeksi)
  → surucu::indeksten_oynat → surucu::oynat
      → oynatici::ac        → mpv loadfile
      → db.oynatildi(id)    → sayaç
      → playlist://queue    → arayüz kuyruğu tazeliyor
  → mpv FileLoaded
      → subtitle::otomatik_yukle   (yandaki altyazılar)
      → library::kucukresim::isaretle
      → library::devam::geri_yukle (kaldığı yere atla — eşikler tutuyorsa)
      → player://file-loaded  { title, duration, tracks }
```

Kaldığı yere atlama `FileLoaded`te, `player_open`da DEĞİL: eşik kararı süreyi
bilmeyi gerektiriyor ve süre dosya çözümlenene kadar bilinmiyor.

## Veri akışı: dosya bitti

```
mpv EndFile(eof)
  → player://end-file { reason: "eof" }
  → library::devam::temizle  (sonuna kadar izlendi, kayıt silindi)
  → surucu::dosya_bitti → kuyruk::sonraki (SAF karar)
      → varsa: surucu::oynat  → yeni dosya + playlist://queue
      → yoksa: kuyruk yerinde kalıyor, oynatma duruyor
```

`reason` `stop` ise **hiçbir şey olmuyor**: `loadfile replace` de bir `EndFile`
üretiyor ve onu bitiş saymak her dosya açılışında bir sonrakine atlamak
olurdu.

## Motor bir Cargo özelliği

`default = ["mpv"]`. Kapalı derlemede `mpv::yok::Motor` devreye giriyor,
kütüphane ve çalma listeleri çalışmaya devam ediyor. Sebep: libmpv bağlama
zamanında bağlanıyor, yani "kurulu mu" sorusu çalışma zamanında sorulamıyor.
Ayrıntı `docs/Mpv_Integration.md`.

## Dil

Kod, yorumlar, sınıf adları ve dosya adları **Türkçe** (ailenin geri kalanı
gibi). İki istisna:

- **IPC sözleşmesi İngilizce** — komut adları, olay adları, seri hâle
  getirilen alan adları. Sınır `docs/IPC.md`.
- **Dış kütüphanelerin kendi adları** — `Mpv`, `Connection`, `WalkDir`.
