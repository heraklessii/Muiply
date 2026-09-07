# mpv Entegrasyonu

## Crate

```toml
libmpv2 = { version = "6", optional = true, default-features = false }
```

`libmpv2` (kohsine/libmpv-rs), eski `mpv 0.2` crate'inin bakımlı devamı. Eski
crate'in `MpvHandler` API'si yok; bu dosyanın ilk taslağı onu anlatıyordu ve
yanlıştı.

`default-features = false`: `render` özelliği (OpenGL bağlamı) kapalı, çünkü
görüntüyü biz çizmiyoruz — mpv kendi penceresine çiziyor (aşağıda).

## Motor bir Cargo özelliği

```toml
[features]
default = ["mpv"]
mpv = ["dep:libmpv2"]
```

`libmpv2-sys` bağlayıcıya `cargo:rustc-link-lib=mpv` basıyor. Bu şu demek:
**libmpv bağlama zamanında bağlanıyor.** DLL/so yoksa uygulama açılmıyor —
"libmpv kurulu mu" sorusu çalışma zamanında sorulamıyor.

Sonuç: soru derleme zamanında soruluyor. Özellik kapalıyken
[`mpv::yok::Motor`](../src-tauri/src/mpv/yok.rs) devreye giriyor; her oynatma
çağrısı tek bir cümleyle reddediliyor, kütüphane ve çalma listeleri
çalışmaya devam ediyor. Arayüz `PlayerState.engine == false` görüp denetimleri
kapatıyor.

Bu bir taklit **değil**: hiçbir çağrı başarılı gibi davranmıyor, sahte süre
üretmiyor.

```bash
npm run tauri:kabuk    # motorsuz
npm run tauri dev      # motorlu
```

## Kurulum

```rust
Mpv::with_initializer(|init| {
    init.set_property("wid", hwnd)?;       // native yüzey
    init.set_property("idle", "yes")?;     // dosya bitince kapanma
    init.set_property("force-window", "yes")?;
    // Zemin rengi: iki ad deneniyor, hata açılışı düşürmüyor (aşağıda).
    if init.set_property("background-color", "#07090C").is_err() {
        let _ = init.set_property("background", "#07090C");
    }
    init.set_property("osc", "no")?;       // kendi denetimlerimiz var
    init.set_property("input-default-bindings", "no")?;
    init.set_property("input-vo-keyboard", "yes")?;
    init.set_property("hwdec", "auto-safe")?;
    init.set_property("sub-auto", "no")?;  // altyazıyı BİZ buluyoruz
    Ok(())
})
```

Neden `with_initializer`: `wid`, `vo` ve `input-*` başlatmadan **sonra**
yazılamıyor; bunlar seçenek, özellik değil.

`wid` **oynatıcı penceresinin** yüzeyi (`src-tauri/src/pencere.rs`). Bu
kısıtın mimari sonucu var: oynatıcı penceresi sonradan yaratılamıyor, çünkü
mpv'ye başka bir pencere gösterilemiyor. İki pencere de açılışta doğuyor ve
gizli bekliyor.

Dikkat çeken üçü:

- **`idle=yes`** olmadan mpv ilk dosyanın sonunda kapanıyor ve uygulama
  motorunu kaybediyor.
- **`input-default-bindings=no`** olmadan mpv penceresinde `q` uygulamayı
  kapatıyor.
- **`hwdec=auto-safe`**, düz `auto` değil: `auto` bozuk sürücülerde yeşil kare
  üretebiliyor ve bunun kullanıcı için bir açıklaması olmuyor.
- **`background` ARTIK RENK ALMIYOR.** mpv 0.38 seçeneği ikiye ayırdı:
  `background` `none|color|tiles`, renk `background-color`da. Eski adla
  renk yazmak `MPV_ERROR_PROPERTY_ERROR` (-11) veriyor ve `with_initializer`
  içindeki her hata **açılışı düşürüyor** — uygulama pencere bile açmadan
  `0xc0000409` ile kapanıyor. Kod iki adı da deniyor ve ikisi de tutmazsa
  devam ediyor: yanlış renkte bir zemin, açılmayan bir oynatıcıdan iyi.
- **Kurulum hatası hangi özellikten geldiğini söylemiyor.** libmpv yalnız bir
  sayı döndürüyor (`Raw(-11)`); on beş çağrıdan hangisi olduğunu bulmak için
  `kur` son denenen adı bir `Cell`de tutuyor ve hata metnine yazıyor.
  Bu olmadan hatayı bulmak, özellikleri tek tek yorum satırına almak demekti.

## Video nasıl çiziliyor

mpv, webview'in **ÜSTÜNDE** duran native bir alt pencereye çiziyor.

İlk tasarım tersiydi: şeffaf bir webview, altında mpv. Windows'ta bu güvenilir
değil — WebView2 kendi DirectComposition ağacında çiziliyor ve şeffaf
bölgesinden kardeş bir alt pencere düzenli olarak görünmüyor.

Sonuçları (kod: [`mpv/yuzey.rs`](../src-tauri/src/mpv/yuzey.rs)):

| Sonuç | Ne demek |
|---|---|
| Arayüz videonun üstüne binemiyor | Denetimler videonun ALTINDA, kendi çubuğunda |
| Video üstündeki tuşlar webview'e ulaşmıyor | Kısayollar mpv'de ayrıca bağlanıyor |
| Şeffaflık gerekmiyor | Pencere normal, işletim sistemi çerçevesi duruyor |

Üçüncüsü ailenin geri kalanıyla tutarlılık kazandırdı (Muiwatch, Muiget de
çerçeveli). Birincisi bir kısıt ama iyi bir kısıt oldu: denetimler kaybolup
aranmıyor.

Yüzeyin konumunu arayüz bildiriyor (`player_set_video_rect`), çünkü düzeni
CSS kuruyor. Ölçek çarpanı Rust tarafında uygulanıyor: doğru çarpanı pencere
biliyor, `devicePixelRatio` karışık DPI'lı iki ekran arasında yanılıyor.

## Kısayollar mpv'de

Video üstündeyken klavye ve fare webview'e hiç ulaşmadığı için, kısayollar
mpv'ye `keybind` komutuyla ayrıca bağlanıyor. İki grup:

| Grup | Nasıl | Örnek |
|---|---|---|
| mpv'nin kendi yapabildiği | doğrudan mpv komutu | `SPACE` → `cycle pause` |
| Uygulamanın işi olan | `script-message` ile geri geliyor | `f` → `script-message muiply tam-ekran` |

Birincide ayrıca haber vermeye gerek yok: `pause`, `volume`, `mute` zaten
izlenen özellikler, değişiklik olay olarak arayüze düşüyor. İkincisi
`Event::ClientMessage` olarak yakalanıp `player://request` ile arayüze
çıkıyor.

Tam liste: `mpv/gercek.rs::kisayollari_kur`.

## Olay döngüsü

Kendi iş parçacığında ve **ayrı bir mpv istemcisiyle** (`create_client`):
olay kuyruğu istemci başına, yani komutları çalıştıran kapı ile olayları
bekleyen kapı birbirini bloklamıyor.

**İstemciye ad verilmiyor ve bu bilinçli.** `libmpv2` 6.0.0'da adlı yol
bozuk: `mpv_create_client(ctx, CString::new(name)?.as_ptr())` yazıyor,
geçici `CString` çağrı tamamlanmadan düşüyor ve mpv'ye asılı bir işaretçi
gidiyor. Dönen `NULL` da crate'in içinde `NonNull::new_unchecked`e giriyor:
sonuç `Err` değil, doğrudan abort — yani aşağıdaki hata dalı hiç
çalışmıyor, uygulama `0xc0000409` ile kapanıyor. Adın bize faydası yoktu;
yalnızca `mpv_client_name()` döndürüyor ve hiçbir yerde okunmuyor. Ses
aygıtında görünen ad ayrı bir seçenek (`audio-client-name`).

```rust
let istemci = mpv.create_client(None)?;   // ad VERME, aşağıya bak
loop {
    match istemci.wait_event(1.0) {
        Some(Ok(Event::FileLoaded)) => { /* künye oku, yayınla */ }
        Some(Ok(Event::EndFile(sebep))) => { /* eof ise kuyruğu ilerlet */ }
        Some(Ok(Event::PropertyChange { name, change, .. })) => { /* ... */ }
        Some(Ok(Event::ClientMessage(p))) => { /* kısayol isteği */ }
        _ => {}
    }
}
```

`Mpv` kendisi `Send + Sync` (libmpv istemci API'si iş parçacığı güvenli), bu
yüzden `Motor` içinde **Mutex yok**. Kilit koymak, olay döngüsünün her
tıklamada komutları bekletmesi demekti.

### İzlenen özellikler

```rust
("time-pos", Format::Double, 1)
("duration", Format::Double, 2)
("pause",    Format::Flag,   3)
("volume",   Format::Int64,  4)
("mute",     Format::Flag,   5)
("speed",    Format::Double, 6)
```

`time-pos` kare hızında geliyor (saniyede ~60 olay) ve iki yerde kısıtlanıyor:

1. **Olay döngüsünde** (`gercek.rs` → `KONUM_ARALIGI`, 100 ms). Önbellekteki
   `PlayerState.position` HER olayda güncelleniyor — `player_get_state`
   sorulduğu anın doğrusunu söylemeli — ama arayüze yayın 100 ms'de bir.
   Kısıtlanmadığında saniyede altmış JSON serileştirmesi ve altmış IPC
   gidişi oluyordu; arayüz zaten çoğunu atıyordu.
2. **Arayüzde** (`useOynatici.ts` → `KONUM_ARALIGI`, 200 ms). Sürgü saniyede
   beş kez ilerliyor, süre yazısı zaten saniyelik.

Küçük resim isteği de (`kucukresim::belki_al`) birinci kısıtın arkasında:
kare bir kez alınıyor, 100 ms geç alınması fark etmiyor.

### `EndFile` ve kuyruk

Kuyruk **yalnız** `reason == eof` iken ilerliyor. `loadfile replace` de bir
`EndFile` üretiyor ama sebebi `stop`; onu bitiş saymak her dosya açılışında
bir sonrakine atlamak olurdu.

## Sonda — tarama neyi okuyor

Kütüphane taraması süre/boyut/etiket için **ikinci bir mpv örneği** kuruyor
(`mpv/sonda.rs`): `vo=null`, `ao=null`, `pause=yes`. Çalan örnekle yapmak,
kullanıcının izlediği filmi durdurup başka dosya yüklemek olurdu.

`symphonia` gibi bir meta veri crate'i eklenmedi: sesi okuyor, videoyu
okumuyor; iki taraf için iki ayrı yol gerekirdi. libmpv ikisini de aynı
sözlükle anlatıyor.

Zaman aşımı 6 saniye. Ağ sürücüsündeki ya da bozuk bir dosya taramayı
süresiz bekletebiliyor; süre dolarsa kayıt ölçümsüz yazılıyor — listede
görünmesi süresinin bilinmesinden önemli.

### Olay kuyruğu her ölçümden önce boşaltılıyor

Olay kuyruğu **istemci başına** ve sonda tek bir istemciyi yeniden
kullanıyor. `stop` ile `loadfile replace` birer `EndFile` üretiyor, yani her
ölçüm ardında bir olay bırakıyor. Boşaltılmadığında sıradaki ölçüm kendi
`FileLoaded`ını beklerken bir öncekinin `EndFile`ını görüyor ve dosyayı
"açılamadı" sayıyordu; sonraki dosyalarda daha kötüsü oluyordu — bir
öncekinin `FileLoaded`ı bu dosyanınki sanılıp künyesi **yanlış kayda**
yazılabiliyordu. Belirtisi sessiz: kütüphanede süresi sıfır ya da başka bir
dosyanın başlığını taşıyan kayıtlar.

Bu yüzden `olc` iki şey yapıyor: başlarken kuyruğu boşaltıyor
(`wait_event(0.0)` kuyruk boşken hemen dönüyor, beklemiyor) ve dönerken —
dosya açılamamış olsa bile — `stop` gönderiyor. İkincisi ayrıca dosyayı
serbest bırakıyor: açık kalan bir demuxer Windows'ta dosyayı kilitli tutuyor
ve kullanıcı onu silemiyor.

## Küçük resimler

Taramada **alınmıyor**. Kare almak bir video çıkışı istiyor; bin dosyalık bir
klasörde bu, kullanıcının hiç açmayacağı dosyalar için dakikalarca kod çözme
demek.

Bunun yerine kare, dosya **çalarken** alınıyor (8. saniyede,
`screenshot-to-file ... video`) — zaten çizen bir çıkış varken bedava. Kural
olarak da tutarlı: ızgarada resmi olan kayıtlar, açtıkların.

Ayrıntı: [`library/kucukresim.rs`](../src-tauri/src/library/kucukresim.rs).

## Platformlar

| Platform | mpv'ye verilen `wid` | Taşınan nesne | Durum |
|---|---|---|---|
| Windows | HWND | `WS_CHILD` bir HWND | ✅ |
| Linux/BSD (X11) | X11 pencere kimliği (XID) | çocuk `GdkWindow` | ✅ |
| Linux/BSD (Wayland) | — | — | mpv'nin sınırı |
| macOS | `NSView*` | `NSView` alt görünümü | ✅ |

Üç platformda da aynı üç işlem: yarat, taşı, gizle. Nesne değişiyor, iş
değişmiyor. İki nokta yine de platforma göre ayrışıyor ve kararı `yuzey.rs`
veriyor, çağıran değil:

- **Ölçek çarpanı yalnız Windows'ta uygulanıyor.** HWND koordinatları fiziksel
  piksel; GTK3 pencere koordinatları ve AppKit noktaları mantıksal. Çarpanı
  ortak yola koymak, HiDPI bir Linux ya da Mac ekranında videoyu olması
  gerekenin iki katı büyüklüğünde çizmek olurdu.
- **GTK ve AppKit çağrıları ana iş parçacığına gönderiliyor**
  (`run_on_main_thread`), oysa `player_set_video_rect` bir Tauri komutu ve
  komutlar havuzdan bir iş parçacığında çalışıyor. Windows'ta gerek yok:
  `SetWindowPos` mesajı pencerenin kendi kuyruğuna gönderiyor.

Linux'ta iki değer AYRI — taşımayı `GdkWindow` yapıyor ama mpv XID istiyor —
bu yüzden `Yuzey` iki tanıtıcı taşıyor.

**Wayland'da gömme yok ve bu mpv'nin sınırı.** `wid` bir X11 pencere kimliği
ya da bir `NSView*` alıyor; Wayland'ın karşılığı `wl_subsurface` ve mpv onu
dışarıdan kabul etmiyor. Saf bir Wayland oturumunda `GdkWindow`u
`X11Window`a dönüştürmek başarısız oluyor, `Yuzey::olustur` hata dönüyor.
XWayland altında gömme çalışıyor.

Yüzey açılamayan her durumda uygulama DURMUYOR: `lib.rs` `Yuzey::yok()`a
düşüyor ve mpv kendi ayrı penceresini açıyor. Çirkin ama çalışıyor — ses,
kütüphane, kuyruk ve denetimler yerinde.

## Desteklenen biçimler

libmpv/FFmpeg ne açıyorsa. Video: MP4, MKV, AVI, MOV, WebM, FLV, TS, M2TS,
HEVC/H.265, AV1, VP9, H.264. Ses: MP3, FLAC, AAC, OGG, OPUS, WAV, M4A, WMA,
APE, ALAC.

Kütüphane **taraması** bundan dar bir liste kullanıyor (`library/mod.rs`):
klasördeki her `.bin`i listeye almak kütüphaneyi çöplük yapardı. Ama
sürüklenip bırakılan ya da diyalogdan seçilen dosya süzülmüyor — orada
isteyen biri var.
