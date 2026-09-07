# Yol Haritası

Fazlar **dikey dilim**: her biri kendi başına çalışan bir uygulama bırakır.
"Önce bütün backend, sonra bütün frontend" yapılmıyor — yarım kalan bir faz
çalışmayan bir ürün demek olurdu.

| Faz | Ne bırakır | Durum |
|---|---|---|
| 0 | Açılan pencere, Mui tasarım dili, boş kabuk | ✅ |
| 1 | mpv motoru: dosya aç, oynat/duraklat, ara, ses | ✅ |
| 2 | Oynatıcı arayüzü: sürgü, denetimler, kısayollar, sürükle-bırak | ✅ |
| 3 | Kütüphane: klasör tara, SQLite, ızgara, küçük resim | ✅ |
| 4 | Altyazı: gömülü izler, yanındaki dosyalar, gecikme | ✅ |
| 5 | Çalma listesi: oluştur, sırala, kuyruk, tekrar/karışık | ✅ |
| 6 | Cila: ayarlar, kaldığı yerden devam, tam ekran, tepsi | ✅ |
| 7 | Varsayılan oynatıcı: dosya ilişkilendirmesi, ayrı oynatıcı penceresi | ✅ |

## Faz 0 — İskelet

Tauri v2 + React + Vite + TS kurulumu; Mui jetonları (`src/styles.css`),
gömülü Outfit, ikon üçlüsü (favicon / public / `src-tauri/icons/kaynak.svg`).

Çıkış ölçütü: `npm run tauri dev` bir pencere açıyor ve pencere ailenin geri
kalanına benziyor.

## Faz 1 — mpv motoru

`src-tauri/src/mpv/` — `libmpv2` sarmalayıcısı, olay döngüsü, `player_*`
komutları (bkz. `docs/IPC.md`).

Kritik karar: **motor bir Cargo özelliği** (`mpv`, varsayılan açık). libmpv
bağlama-zamanında bağlanıyor; DLL yoksa uygulama hiç açılmıyor. Motorsuz
derleme (`--no-default-features`) kütüphane/liste tarafını libmpv kurmadan
geliştirmeyi mümkün kılıyor. Ayrıntı: `docs/Mpv_Integration.md`.

Çıkış ölçütü: komut satırından verilen bir dosya sesli çalıyor, `player://`
olayları arayüze düşüyor.

## Faz 2 — Oynatıcı arayüzü

Zustand yerine tek bir `usePlayer` kancası + `useReducer`: durum tek yerde,
ek bağımlılık yok. Sürgü, ses, hız, kısayollar, sürükle-bırak.

Çıkış ölçütü: fare ve klavyeyle tam bir oynatma oturumu.

## Faz 3 — Kütüphane

`rusqlite` (bundled), `walkdir` ile tarama, mpv'nin `screenshot-to-file`
komutuyla küçük resim. Izgara görünümü, arama, sıralama, süzme.

Çıkış ölçütü: bir klasör eklenip taranıyor, ızgaradan çift tıkla oynuyor.

## Faz 4 — Altyazı

Gömülü izler `track-list`'ten; harici dosyalar video dosyasının yanından
otomatik bulunuyor. Gecikme ayarı ve iz seçici.

Çıkış ölçütü: `video.tr.srt` elle bir şey yapmadan seçilebilir hâlde geliyor.

## Faz 5 — Çalma listesi

SQLite'ta kalıcı listeler, sürükleyerek sıralama, kuyruk, tekrar/karışık.
Dosya bitince (`player://end-file` · `eof`) sıradaki açılıyor.

Çıkış ölçütü: bir liste baştan sona kendi kendine çalıyor.

## Faz 6 — Cila

Faz tek parça değil; her madde kendi başına biten bir dilim.

- ✅ **Ayarlar ekranı** — `settings/mod.rs` + `src/components/Ayarlar.tsx`.
  `hwdec`, ses aygıtı, varsayılan hız, kaldığı yerden devam, tema. Depolama
  `settings` tablosunda anahtar/değer: yeni bir ayar şema geçişi
  gerektirmesin. **Tema oraya girmiyor**, `localStorage`da kalıyor — ilk
  boyamadan önce bilinmesi gerekiyor ve backend'e sormak pencerenin bir kare
  yanlış renkte açılması demek olurdu.
- ✅ **Kaldığı yerden devam** — `library/devam.rs`, `media.last_position`.
  Karar saf ve eşikli (başa/sona 20 sn); yazan olay döngüsü (5 sn'de bir) ve
  pencere kapanışı, silen `EndFile` · `eof`. Varsayılan KAPALI: bir filmi
  bilerek baştan açan kullanıcı kendini ortasında bulmasın.
- ✅ **Varsayılan hız her dosyada uygulanıyor** (`playlist/surucu.rs`). mpv
  `speed`i dosyalar arasında koruyor, oysa "varsayılan hız" bir dosyanın
  BAŞLADIĞI hız demek.
- ✅ **Linux ve macOS'ta video yüzeyi gömme** (`mpv/yuzey.rs`). Üç platformda
  aynı üç işlem, farklı nesnelerle: Windows'ta `WS_CHILD` bir HWND, Linux'ta
  çocuk bir `GdkWindow` (mpv'ye XID veriliyor), macOS'ta bir `NSView` alt
  görünümü. İki şey platforma göre ayrıştı ve kararı yüzeyin kendisi veriyor:
  ölçek çarpanı yalnız Windows'ta uygulanıyor (GTK3 ve AppKit koordinatları
  zaten mantıksal) ve GTK/AppKit çağrıları ana iş parçacığına gönderiliyor.
  **Wayland hariç:** orada gömme mpv'nin `wid` seçeneğinin sınırı, bizim
  eksiğimiz değil; XWayland altında çalışıyor, saf Wayland'da mpv kendi
  penceresini açıyor.
- ✅ **Sistem tepsisi + medya tuşları** (`tepsi.rs`). Tepsi menüsü: oynat /
  duraklat, önceki, sonraki, pencereyi göster, çıkış; ipucu çalan dosyanın
  adını gösteriyor. Pencerenin X'i uygulamayı gerçekten kapatıyor, tepsiye
  inmiyor — "kapattım ama kapanmadı" bir hata gibi görünüyor. Medya tuşları
  bir AYARA bağlı (`media_keys`, varsayılan açık): kayıt global, yani açıkken
  tuş başka bir oynatıcıya ulaşmıyor ve bu kullanıcının bileceği bir şey.
- ✅ **Windows dışı paketleme.** `tauri.linux.conf.json` (deb, `libmpv2`
  bağımlılığıyla) ve `tauri.macos.conf.json` (app + dmg). Tauri bu dosyaları
  ana yapılandırmayla kendiliğinden birleştiriyor. Windows'un DLL kopyalayan
  `tauri.paketleme.conf.json`u ayrı kaldı; sebebi `docs/Setup.md`'de.
- ✅ **Dosya ilişkilendirmesi ve tekil örnek** (`acilis.rs`,
  `tauri.conf.json` > `bundle.fileAssociations`). Muiply'ın varsayılan
  oynatıcı olabilmesi için iki ayrı şey gerekiyordu ve ikisi de yoktu:
  kurulumun uzantıları işletim sistemine bildirmesi (yoksa Muiply "Varsayılan
  uygulamalar" listesinde hiç görünmüyor) ve uygulamanın komut satırındaki
  yolu okuması (yoksa çift tıklanan dosya Muiply'ı açıyor ama Muiply boş
  geliyor). Üçüncüsü tekil örnek: işletim sistemi her çift tıklamada
  uygulamayı yeniden çalıştırıyor, eklenti olmasa her dosya ikinci bir
  pencere ve aynı SQLite dosyasına ikinci bir yazar demekti.
  Uzantı listesi `library/mod.rs` ile AYNI — ikisi ayrışırsa kütüphaneye
  girmeyen bir dosya çift tıklamayla açılıyor (ya da tersi) olurdu.
  `.ts` bilerek listede: TypeScript ile paylaşılıyor ama ilişkilendirme
  yalnızca seçenek sunuyor, varsayılanı kullanıcı seçiyor.
- ✅ **`playlist_get_items` satır kimliğini de döndürüyor** (`PlaylistItem`)
  ve silme kimlikle. Sıra ile silmenin sorunu, arayüzün listeyi gördüğü an ile
  silmenin yürüdüğü an arasında listenin değişebilmesiydi: araya bir ekleme
  girdiğinde yanlış öğe siliniyordu.

## Faz 7 — Varsayılan oynatıcı

Muiply'ın işletim sisteminde bir oynatıcı gibi davranması. İki parça:

**Dosya ilişkilendirmesi** — `bundle.fileAssociations` (14 video + 12 ses
uzantısı, listeler `library/mod.rs` ile aynı) kurulumla işletim sistemine
bildiriliyor; `acilis.rs` komut satırındaki yolu okuyor; tekil örnek
eklentisi ikinci çift tıklamayı çalışan pencereye taşıyor. Üçü birden
olmadan "varsayılan oynatıcı" olmuyor: kayıt olmadan Muiply listede
görünmüyor, yol okunmadan boş açılıyor, tekil örnek olmadan her dosya yeni
bir pencere ve aynı veritabanına ikinci bir yazar demek.

**Ayrı oynatıcı penceresi** — `oynatici` ve `kutuphane` (bkz.
`docs/Architecture.md` > İki pencere). Sebep ürün tarafında: bir film
izlerken yanda duran kütüphane sütunu, oynatıcıyı oynatıcı gibi
hissettirmiyordu; çift tıklanan bir dosya için bütün uygulamanın açılması da
gereksizdi.

Kısıt mimariyi belirledi: mpv çizeceği pencereyi (`wid`) yalnız
başlatılırken alıyor. Yani oynatıcı penceresi sonradan yaratılamıyor —
açılışta (gizli) var olmak zorunda. Bu yüzden iki pencere de açılışta doğuyor
ve hangisinin görüneceğine backend karar veriyor.

Çıkış ölçütü: bir `.mkv`ye çift tıklamak yalnızca oynatıcıyı açıyor;
kütüphane ancak istendiğinde geliyor.

## Kapsam Dışı (bilerek)

Bunlar eksik değil, **karar**. Genişletmeden önce bu dosya değişmeli.

- **Ağ akışı / YouTube yok.** Muiply yerel dosya oynatıcısı. Uzaktan izleme
  Muiwatch'ın, indirme Muiget'in işi.
- **Kütüphane meta verisi internetten zenginleştirilmiyor.** Poster, afiş,
  oyuncu kadrosu için üçüncü taraf API'ye çıkılmıyor; ne varsa dosyada var.
- **Kod çözücü ayarı arayüze açılmıyor** (ayarlardaki `hwdec` hariç). mpv'nin
  varsayılanları bu işi bizden iyi biliyor. `hwdec` istisna çünkü bozuk bir
  sürücüde ortaya çıkan hata kullanıcının anlayamayacağı bir görüntü
  bozukluğu ve tek çaresi onu kapatmak.
- **Video filtreleri / renk düzeltme yok.** mpv'nin kendi `input.conf`'u bu
  işi zaten yapıyor; ikinci bir arayüz üretmek onu bölmek olurdu.
- **İKİDEN FAZLA pencere yok.** Aynı anda iki video, ayrı ayrı oynatıcılar ya
  da her çalma listesi için bir pencere yok. Her yeni pencere kendi mpv
  örneği demek ve "az RAM" iddiası taşıyan bir portföyde bu ucuz değil.
- **Pencereler tepsiye inmiyor.** Bir pencerenin X'i onu gizliyor, GÖRÜNEN
  SON pencerenin X'i uygulamayı gerçekten kapatıyor. Kapattığını sanan
  kullanıcının arkasında çalışan görünmez bir uygulama bırakmıyoruz.
- **Saf Wayland'da video gömme yok.** mpv'nin `wid` seçeneği bir X11 pencere
  kimliği ya da bir `NSView*` alıyor; Wayland'ın karşılığı `wl_subsurface` ve
  mpv onu dışarıdan kabul etmiyor. XWayland altında gömme çalışıyor, saf
  Wayland oturumunda mpv kendi penceresini açıyor. Bunu aşmanın yolu mpv'yi
  `render` API'siyle kendi OpenGL bağlamımıza çizdirmek olurdu — üç platformda
  da ayrı bir çizim yolu demek ve bu fazın bütün kazancını götürürdü.
- **Tepsiye inen kapatma yok.** Pencerenin X'i uygulamayı gerçekten
  kapatıyor. "Kapattım ama kapanmadı" bir kullanıcı için hatanın kendisi gibi
  görünüyor ve tepsi ikonunu fark etmeyen biri uygulamayı bir daha nasıl
  kapatacağını bilmiyor.
