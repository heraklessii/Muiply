# Arayüz — React / Vite / TypeScript

## Tasarım dili

Mui ailesinden **birebir** devralındı. Kanonik kaynak `..\Muiget\src\styles.css`,
jeton listesi `..\MuiLabs\docs\ui-conventions.md`.

- Teal vurgu `#2dd4bf` (açık temada `#0d9488`), koyu zemin `#0f1115`
- Gömülü **Outfit** (CDN yok — Tauri sürümü çevrimdışı açılmalı ve Google
  Fonts'a giden bir istek kullanıcıyı üçüncü tarafa bildirmek olurdu)
- Türkçe sınıf adları: `.dugme`, `.kart`, `.rozet`, `.alan`, `.marka`
- İkon kuralı: koyu yuvarlak kare + teal glyph + **tek dolu öğe**

**Kural: bileşen kodunda renk/ölçü sabiti yok.** Hepsi `src/styles.css`
başındaki değişkenlerden.

## İki pencere, tek paket

```
   OYNATICI penceresi              KÜTÜPHANE penceresi
┌───────────────────────────┐   ┌──────────────────────────────┐
│ Tepe: ad · aç · kütüphane │   │ Tepe: marka · ad · aç/klasör │
├───────────────────────────┤   ├───────────┬──────────────────┤
│                           │   │ Yan       │  Tarama şeridi   │
│      Sahne (video)        │   │ Kütüphane ├──────────────────┤
│                           │   │ Oynatma   │  Izgara/Liste    │
│                  ┌────────┤   │ Listeler  │  Kuyruk/Ayarlar  │
│                  │Bildirim│   │ Klasörler ├─────────┬────────┤
├──────────────────┴────────┤   │           │  Çubuk  │Bildirim│
│ Çubuk: sürgü + denetimler │   │           │         │        │
└───────────────────────────┘   └───────────┴─────────┴────────┘
```

Tarama şeridi YER KAPLIYOR (süren bir iş, dakikalarca sürebiliyor);
bildirimler ana alanın sağ altında YÜZÜYOR ve kendiliğinden kapanıyor.
İkisi tek bir şeritti ve biri diğerinin yerini alıyordu: tarama sürerken
çıkan bir hata ilerlemeyi siliyordu.

İkisi de aynı `index.html`i yüklüyor; ayrımı pencere ETİKETİ yapıyor
(`getCurrentWindow().label`, `src/main.tsx`). İki ayrı HTML girişi Vite
yapılandırmasını ve varlık paylaşımını ikiye bölerdi; URL parametresi ise
etiket zaten varken fazladan bir sözleşme olurdu.

Kökler `src/pencereler/` altında. Ortak olan her şey kancalarda:
`useKumanda` (kısayollar, altyazı döngüsü, tam ekran) ve `useBirakma`
(sürükle-bırak) iki pencerede de aynı — iki kopya, birinin unutulacağı
anlamına gelirdi.

Yan sütun YALNIZ kütüphane penceresinde. Oynatıcıda tam ekran dışında da
sadece ince bir tepe barı var: dosya adı ve iki düğme.

**Tam ekran yalnız oynatıcı penceresinde.** Kütüphanede `f` tuşu ve tam
ekran düğmesi videoyu nerede arayacağını bilen tek yere, oynatıcı
penceresine götürüyor.

Ekranda her an üç şey okunabilir olmalı:

1. **ne çalıyor** — üst bardaki başlık
2. **neresindeyiz** — sürgü + süre
3. **sırada ne var** — yan sütundaki kuyruk sayısı

Geri kalan her şey sahnenin çevresindeki çerçeve.

Kütüphane penceresinde 860 pikselin altında yan sütun kayboluyor: ızgara ile
içerik arasında seçim yapmak, ikisini birden sıkıştırmaktan iyi. Oynatıcı
penceresinin alt sınırı 480 piksel — orada zaten sütun yok.

## Video neden `<video>` değil

mpv görüntüyü webview'in **üstünde** duran native bir alt pencereye çiziyor.
`Sahne` bileşeninin işi o pencerenin nereye oturacağını söyleyen kutuyu
çizmek; ölçüyü `useSahneOlcu` `ResizeObserver` ile alıp
`player_set_video_rect` ile gönderiyor.

Gerekçe ve sonuçları `docs/Mpv_Integration.md` → "Video nasıl çiziliyor".
Arayüz açısından iki sonucu var:

- **Denetimler videonun üstüne binemiyor.** Çubuk videonun altında, kendi
  yerinde. Tam ekranda fare durunca gizleniyor, kıpırdayınca dönüyor.
- **Video üstündeki tuşlar buraya ulaşmıyor.** Aynı kısayollar mpv'de ayrıca
  bağlı; mpv onları `player://request` olayıyla geri gönderiyor ve `useKumanda`
  aynı işlevleri çağırıyor (yalnız OYNATICI penceresinde: olay ikisine de
  düşüyor ve ikisi de karşılık verse "sonraki" bir tıklamada iki kez
  ilerlerdi).

## Durum — Zustand yok

Bu dosyanın ilk taslağı Zustand öngörüyordu. Kullanılmadı: durumun tek sahibi
zaten **backend**. Arayüzdeki her şey onun yansıması ve React'in kendi
kancaları bunun için yeterli. Ailenin geri kalanı da (Muiget, Muiwatch)
durum kütüphanesi kullanmıyor.

| Kanca | Ne tutuyor |
|---|---|
| `useOynatici` | `PlayerState`, izler, mpv'den gelen kısayol istekleri |
| `useKutuphane` | kayıtlar, klasörler, tarama ilerlemesi, süzgeç |
| `useKuyruk` | `QueueSnapshot` |
| `useListeler` | çalma listeleri ve açık listenin öğeleri |
| `useSahneOlcu` | video yüzeyinin dikdörtgeni |
| `useAyarlar` | kalıcı tercihler ve ses aygıtı listesi |
| `useKumanda` | iki pencerenin ORTAK eylemleri: kısayollar, tam ekran, altyazı döngüsü, gecikme |
| `useBirakma` | pencere üstünde bir şey sürükleniyor mu |
| `useBildirimler` | bildirim yığını (en fazla üç, kendiliğinden kapanan) |
| `useTema` | koyu/açık — değeri `localStorage`da, backend'de değil |

`useAyarlar`da **iyimser güncelleme yok**, diğerlerinden farkı bu: yazılan
değer backend'de düzeltilebiliyor (aralık dışı hız, tanınmayan `hwdec`) ve
gönderdiğimizi göstermek, kutudaki sayı ile mpv'nin gerçeğinin ayrışması
demek olurdu. Dönen değer neyse o gösteriliyor.

Hiçbir yerde "duraklattım, o hâlde duraklamıştır" varsayımı yok. Bir düğmeye
basıldığında komut gönderiliyor ve durum, mpv'nin olayı geldiğinde değişiyor.
Birkaç milisaniyelik gecikme; karşılığında arayüz ile motorun ayrışması
imkânsız — mpv kısayolları, dosya bitişi ve kuyruk geçişleri de aynı kanaldan
akıyor.

### `time-pos` kısıtlaması

mpv konumu kare hızında bildiriyor (saniyede ~60 olay). Hepsini React
durumuna yazmak sürgüyü saniyede 60 kez yeniden çizdirmek demek; gözle
görülen kazanç yok. `useOynatici` 200 ms'de bir işliyor — sürgü saniyede beş
kez ilerliyor, süre yazısı zaten saniyelik.

## IPC

Bileşenler `invoke` görmez. `src/ipc/index.ts` her komutu bir fonksiyona
sarıyor: komut adı bir dize ve yanlış yazılmış bir dize çalışma zamanına
kadar sessiz kalıyor.

Tipler `src/ipc/tipler.ts` içinde, elle yazılıyor ve `docs/IPC.md` ile
eşleşiyor. Üretilmiyorlar — TypeScript backend'i görmüyor, yani bir alan
Rust'ta değişip burada değişmezse derleyici uyarmaz. Kural bu yüzden:
`docs/IPC.md` değişmeden ne Rust ne burası değişir.

## Saf mantık ve testler

`vitest`, dosyalar kaynağın yanında (`*.test.ts`). Test edilen her şey saf:

- `src/lib/sure.ts` — süre/boyut biçimleme. Bozulunca hata sessiz: "1:5"
  yazan bir oynatıcı bozuk görünür ama hiçbir yerde hata basmaz.
- `src/lib/suz.ts` — arama. En kritik testi Türkçe küçük harf:
  `"İSTANBUL".toLowerCase()` JavaScript'te `i`nin ardına birleşen bir nokta
  koyuyor ve `"istanbul"` ile eşleşmiyordu.

  Aranacak metin kayıt başına bir `WeakMap`te saklanıyor. `ara` her tuş
  vuruşunda çağrılıyor ve `toLocaleLowerCase("tr")` yerel duyarlı olduğu için
  ucuz değil: beş bin kayıtlık bir kütüphanede "istanbul" yazmak (sekiz
  arama) ölçülen ~41 ms'ten ~3 ms'e indi. Anahtar IPC'den gelen kayıt
  nesnesi, yani kütüphane yenilendiğinde önbellek kendiliğinden boşalıyor —
  elle temizlenecek bir şey yok.

DOM'a ya da IPC'ye dokunan kod test edilmiyor: onu jsdom taklidiyle sınamak
yanlış bir güven verirdi. Karşılığı Rust tarafındaki testler (kuyruk gezinme,
altyazı eşleştirme).

## Kısayollar

| Tuş | Eylem |
|---|---|
| Boşluk | Duraklat / devam |
| ← / → | 5 sn geri / ileri |
| Shift + ← / → | 60 sn geri / ileri |
| ↑ / ↓ | Ses +5 / −5 |
| M | Sessiz |
| F | Tam ekran |
| Escape | Tam ekrandan çık |
| S | Altyazı izleri arasında dön |
| N / P | Sonraki / önceki |
| ? | Kısayol yardımını aç / kapat |

Girdi alanlarında (arama kutusu, liste adı) hiçbiri çalışmıyor — "space"
yazmak videoyu duraklatmamalı.

Liste ÜÇ yerde ve üçü birlikte değiştirilmeli: burası, `Kisayollar.tsx`teki
yardım kaplaması ve mpv'nin kendi bağları
(`mpv/gercek.rs::kisayollari_kur`). Sonuncusu şart çünkü video üstündeyken
tuşlar webview'e hiç ulaşmıyor. Yardım kaplamasında olmayan bir kısayol
kullanıcı için var değildir.

`?` yalnız webview tarafında: mpv'nin gönderdiği istekler
(`player://request`) yardım kaplamasını değil oynatmayı ilgilendiriyor.

## Sürükle-bırak

`tauri://drag-enter` / `drag-leave` kaplamayı açıp kapatıyor,
`tauri://drag-drop` yolları `app_open_paths`e veriyor. Klasör mü dosya mı
kararı **backend'de**: arayüzün dosya sistemine erişimi yok ve uzantıya bakıp
tahmin etmek "My.Videos" adlı bir klasörde yanlış cevap verir.

Kaplama `pointer-events: none` — yutsaydı Tauri'nin bırakma olayı webview'e
ulaşmaz, bırakma hiç gerçekleşmezdi.

## Bileşenler

| Dosya | İş |
|---|---|
| `pencereler/OynaticiPenceresi.tsx` | Oynatıcı kökü: sahne + çubuk |
| `pencereler/KutuphanePenceresi.tsx` | Kütüphane kökü: yan sütun + ana alan + çubuk |
| `Tepe.tsx` | Üst bar |
| `Yan.tsx` | Gezinme |
| `Sahne.tsx` | Video kutusu / ses kapağı / boş durum |
| `Cubuk.tsx` | Sürgü + denetimler + altyazı ve hız menüleri |
| `Surgu.tsx` | `<input type="range">` sarmalayıcısı |
| `Kutuphane.tsx` | Araç çubuğu + ızgara + üç ayrı boş durum |
| `Kart.tsx` | Izgara kartı ("gerilmiş bağlantı" deseni) |
| `ListePaneli.tsx` | Çalma listesi, sürükleyerek sıralama |
| `KuyrukPaneli.tsx` | Kuyruk |
| `Ayarlar.tsx` | Ayarlar paneli — altı satır, sekme yok |
| `Menu.tsx` | Açılır menü (dışarı tıklama + Escape) |
| `Bildirimler.tsx` | Yüzen bildirim yığını + tarama şeridi (`TaramaSeridi`) |
| `Kisayollar.tsx` | Klavye kısayolları kaplaması (`?`) |
| `TemaDugmesi.tsx` | Tepe barındaki koyu/açık düğmesi |
| `Birak.tsx` | Sürükle-bırak kaplaması |
| `HataSiniri.tsx` | Tepe hata sınırı — depodaki tek sınıf bileşeni |
| `Ikonlar.tsx` | Yirmi küçük SVG; ikon kütüphanesi bağımlılığı yok |

### Kart neden `<button>` değil

Kartın tamamı çalmaya götürüyor ama içinde ikinci bir düğme var (üç nokta
menüsü) ve iç içe düğme hem geçersiz HTML hem de ekran okuyucuda tek bir dev
düğme demek. Bunun yerine başlık düğme, `::after` kartı kaplıyor; menü düğmesi
`z-index: 1` ile kaplamanın üstünde. Aynı çözüm MuiLabs'ın uygulama
kartlarında da kullanılıyor.

### Önizleme bandı

Her kartta **var ve aynı yükseklikte** — birinin bandı olup diğerininki
olmayınca ızgarada bütün satır kayıyor. Küçük resmi olmayan kayıt soyut bir
ikonla duruyor; temsilî bir kare koymak, dosyanın içinde olmayan bir şeyi
göstermek olurdu.

Bandın dibinde ince bir **yarım kalma şeridi** var (`MediaItem.lastPosition`).
Sayı değil şerit, çünkü kullanıcının ızgaraya bakarken sorduğu şey "kaçıncı
dakikadaydım" değil, "bunu bitirmiş miydim". Oran bileşenden bir CSS özel
özelliğiyle (`--oran`) geliyor; ölçü `styles.css`te, bileşende piksel yok.

### Ayarlar paneli

Her satırın bir **açıklaması** var: bir oynatıcının ayarları yılda bir kez
açılıyor ve kullanıcının "hwdec" başlığını görüp ne olduğunu hatırlaması
beklenemez.

Tema satırı diğerlerinden farklı — değeri backend'de değil `localStorage`da
(`src/lib/platform.ts`), çünkü ilk boyamadan önce bilinmesi gerekiyor.
Kullanıcı için ikisi de "ayar"; ayrım arayüzde görünmüyor.

**Medya tuşları satırının açıklaması bir uyarı taşıyor**, diğerleri gibi
yalnız "ne yapar" demiyor: kayıt global, yani açıkken tuşlar başka bir
oynatıcıya ulaşmıyor. Bu, kullanıcının başka bir yerde yaşayacağı bir sorunun
sebebi; ayarı bulup kapatabilmesi için nedenini burada okuması gerekiyor.

Açık/kapalı anahtarı `<input type="checkbox">` değil, `role="switch"` taşıyan
bir düğme: onay kutusu "bir form gönderilecek" demek, oysa buradaki değişiklik
tıklandığı anda kaydediliyor.

## Boş durumlar

Boşluğun sebebi kadar boş durum var. Kütüphanede üç tane: hiç klasör
eklenmemiş, klasör var ama bu türde kayıt yok, arama eşleşmedi. Üçüne aynı
metni yazmak kullanıcıyı yanlış düğmeye götürür.
