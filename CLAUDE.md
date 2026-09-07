# Muiply — CLAUDE.md

Bu dosya, Claude Code oturumlarında context'i korumak için hazırlanmıştır.
Blanket "her oturumda şunları oku" mantığı yerine tetikleyici-durum tablosu
kullanılır. Sadece ilgili durumla eşleşen dosyayı oku — gereksiz dosya okuma
performansı düşürür.

## Proje Özeti

Muiply, bilgisayardaki videoları ve müziği çalan bir masaüstü medya
oynatıcısıdır. Kod çözme işini libmpv yapıyor: MKV, HEVC, AV1, FLAC ve
diğerleri ek codec paketi istemeden açılıyor. Klasörleri tarayan bir
kütüphanesi ve çalma listeleri var. Mui portföyünün bir parçası.

**İki pencere:** `oynatici` (video + çubuk) ve `kutuphane` (ızgara,
listeler, kuyruk, ayarlar). İkisi de açılışta yaratılıyor, gizli başlıyor;
hangisinin görüneceğine backend karar veriyor. Sonradan yaratılamıyorlar —
mpv `wid`i yalnız başlatılırken alıyor.

**Temel prensip: karar veren kod backend'de.** Arayüz saf görüntü + IPC.
Sebebi somut: dosya bitince sıradakine geçme kararını mpv'nin olay döngüsü
veriyor ve o karar arayüzün açık olup olmamasından bağımsız olmalı.

## Kapsam Dışı Kararlar (ÖNEMLİ — sapma)

Bunlar bilinçli maliyet/karmaşıklık trade-off'u, "eksik" değil. Genişletmeden
önce `docs/Roadmap.md` güncellenir.

- **Ağ akışı / YouTube YOK.** Muiply yerel dosya oynatıcısı. Uzaktan birlikte
  izleme Muiwatch'ın, indirme Muiget'in işi.
- **Meta veri internetten zenginleştirilmiyor.** Poster, afiş, oyuncu kadrosu
  için üçüncü taraf API'ye çıkılmıyor; ne varsa dosyanın içinde var.
- **Video filtreleri / renk düzeltme YOK.** mpv'nin kendi `input.conf`'u bunu
  zaten yapıyor; ikinci bir arayüz üretmek onu bölmek olurdu.
- **`library_delete_media` dosyayı SİLMİYOR**, kaydı siliyor. Muiply bir dosya
  yöneticisi değil.
- **Yüzey gömme saf Wayland'da YOK.** Windows (HWND), X11 (XID) ve macOS
  (NSView) çalışıyor; Wayland'da mpv'nin `wid` seçeneği `wl_subsurface` kabul
  etmediği için mpv kendi penceresini açıyor. XWayland altında çalışıyor.
- **Tepsiye inen kapatma yok.** Pencerenin X'i uygulamayı gerçekten
  kapatıyor; tepsi ikonu yalnızca bir kısayol.
- **Medya tuşları kapatılabilir** (`media_keys` ayarı, varsayılan açık).
  Kayıt global: açıkken tuş başka bir oynatıcıya ulaşmıyor.

## Tetikleyici Tablosu

| Durum / Görev | Okunacak Dosya |
|---|---|
| Genel mimari, katmanlama, veri akışı | `docs/Architecture.md` |
| mpv entegrasyonu, motor özelliği, video yüzeyi, olay döngüsü | `docs/Mpv_Integration.md` |
| Kütüphane, SQLite şeması, geçiş, tarama, küçük resim | `docs/Library.md` |
| Ayarlar, kaldığı yerden devam | `docs/Library.md` + `docs/IPC.md` |
| Altyazı bulma / seçme / gecikme | `docs/Subtitles.md` |
| Arayüz, tasarım dili, kancalar, kısayollar | `docs/Frontend.md` |
| Komut ve olay sözleşmesi | `docs/IPC.md` |
| Kurulum, libmpv edinme, `mpv.lib` üretme, paketleme | `docs/Setup.md` |
| Sistem tepsisi, medya tuşları | `src-tauri/src/tepsi.rs` başlığı |
| Dosya ilişkilendirmesi, çift tıklama, varsayılan oynatıcı | `docs/Setup.md` + `src-tauri/src/acilis.rs` |
| Pencereler: hangisi ne zaman görünür, kapatma politikası | `src-tauri/src/pencere.rs` + `docs/Architecture.md` |
| "Bu özellik hangi fazda", kapsam dışı kararlar | `docs/Roadmap.md` |
| Sadece bugfix / küçük stil değişikliği | Hiçbiri — direkt koda bak |

## Stack (ezberle, yeniden okuma)

- **Masaüstü:** Tauri v2
- **Backend:** Rust
- **Motor:** libmpv (`libmpv2` v6 crate) — **Cargo özelliği**, varsayılan açık
- **Veritabanı:** SQLite (`rusqlite`, bundled, WAL)
- **Arayüz:** React 19 + Vite + TypeScript, durum kütüphanesi YOK
- **Testler:** `cargo test` (Rust) · `vitest` (arayüz)

## Kod Konumu

**Rust (`src-tauri/src/`)** — üç katman: komut sarmalayıcıları, modüller, dış
dünya.

- `mpv/gercek.rs` — **en kritik dosya.** libmpv sarmalayıcısı, olay döngüsü,
  mpv kısayolları. Bozulunca oynatıcı sessizce ölüyor.
- `mpv/yok.rs` — motorsuz derlemenin karşılığı. `gercek.rs` ile yüzeyi
  **birebir aynı** olmak zorunda, yoksa `--no-default-features` kırılır.
- `mpv/yuzey.rs` — native alt pencere. Üç platform, üç `mod platform`:
  Win32 · GDK/X11 · NSView. Ölçek çarpanı yalnız Windows'ta, GTK/AppKit
  çağrıları ana iş parçacığında.
- `playlist/kuyruk.rs` — saf gezinme kararları + testler.
- `subtitle/mod.rs` — saf ad eşleştirme + testler.
- `library/tarayici.rs` — üç geçişli tarama, kendi iş parçacığında.
- `library/devam.rs` — saf eşik kararı + kaldığı yeri yaz/oku/temizle.
- `settings/mod.rs` — kalıcı tercihler, düzeltme ve mpv'ye uygulama.
- `tepsi.rs` — sistem tepsisi + medya tuşları. Oynatma mantığı YOK;
  `mpv/oynatici` ve `playlist/surucu`ya gidiyor.
- `pencere.rs` — iki pencerenin görünürlük politikası: kim ne zaman
  görünür, X'e basılınca ne olur. Etiketler burada (`OYNATICI`,
  `KUTUPHANE`) ve `capabilities/default.json` ile aynı olmak zorunda.
- `acilis.rs` — dışarıdan gelen dosya yolları: sürükle-bırak, açılışta
  komut satırı (çift tıklama / varsayılan oynatıcı), ikinci örnek. Üçü de
  `yollari_ac`a çıkıyor.
- `commands/` — ince sarmalayıcılar. **Buraya mantık yazılmaz.**

**Arayüz (`src/`)**

- `pencereler/` — iki pencere kökü: `OynaticiPenceresi` ve
  `KutuphanePenceresi`. Hangisinin çizileceğini `main.tsx` pencere
  etiketinden seçiyor.
- `ipc/` — `invoke` sarmalayıcıları ve tipler. Bileşenler `invoke` görmez.
- `hooks/` — backend durumunun yansımaları. `useKumanda` ve `useBirakma`
  iki pencerenin ORTAK davranışı; oraya yazılan her şey ikisinde de çalışır.
- `lib/` — saf ve testli: `sure.ts`, `suz.ts`.
- `styles.css` — Mui jetonları, gömülü Outfit.

Komutlar: `npm run tauri dev` · `npm run tauri:kabuk` · `npm test` ·
`cd src-tauri && cargo test --no-default-features`

## Yazarken Dikkat

1. **`commands/`ye iş mantığı EKLEME.** Aynı karara mpv'nin olay döngüsünden
   de gelinebiliyor; iki kopya birbirinden sessizce ayrışır.
2. **Arayüze iş mantığı EKLEME.** Kuyruk, tarama ve altyazı bulma backend'de.
3. **Yeni komut → önce `docs/IPC.md`.** Sözleşme orada; Rust ve TypeScript
   ondan sonra.
4. **`mpv/yok.rs`i güncellemeyi unutma.** `gercek.rs`e yeni bir metot
   eklediğinde karşılığı orada da olmalı.
5. **Renk/ölçü sabiti yazma.** Hepsi `src/styles.css` başındaki jetonlardan.
6. **`EndFile` sebebini kontrol et.** Yalnız `eof` kuyruğu ilerletir; `stop`
   `loadfile replace`in yan ürünü.
7. **mpv kısayolu eklerken arayüzdekini de ekle** (`mpv/gercek.rs` ve
   `App.tsx`). Video üstündeyken tuşlar webview'e ulaşmıyor.
8. **Türkçe ek üretme.** "Kontrol {ad}'de" çalışmıyor (Ayşe'de ama Onur'da).
   Ada ek gerektirmeyen kalıp kullan.
9. **Türkçe küçük harf `toLocaleLowerCase("tr")`.** Düz `toLowerCase()`
   "İSTANBUL"u `"istanbul"` yapmıyor; arama eşleşmiyor.
10. **`media` tablosuna sütun eklerken `db::gecisler`e de ekle.**
    `CREATE TABLE IF NOT EXISTS` var olan tabloyu değiştirmiyor; kullanıcının
    diskindeki veritabanı eski şemada kalıp ilk sorguda patlıyor.
11. **Yeni ayar → hem `Settings` hem `duzelt` hem `oku`/`yaz`.** Anahtar
    eklenip `duzelt` unutulduğunda geçersiz bir değer doğrudan mpv'ye
    gidiyor ve hata çalışma zamanına kadar görünmüyor. (`bool`da düzeltilecek
    bir şey yok; kural "tipinde geçersiz değer var mı" diye soruyor.)
    Karşılığı mpv'de olmayan bir ayar (`resume`, `media_keys`) `uygula`ya
    değil kendi yerine gidiyor — ve o yer HEM açılışta (`lib.rs`) HEM
    `settings_set`te çağrılmalı.
12. **`playlist_get_items` `PlaylistItem` döndürüyor**, `MediaItem` değil:
    satırın kimliği (`item_id`) ile medyanın kimliği (`id`) farklı şeyler ve
    aynı kayıt bir listede iki kez bulunabiliyor. Silme SATIR kimliğiyle.
13. **Uzantı listesi İKİ yerde: `library/mod.rs` ve `tauri.conf.json`
    (`bundle.fileAssociations`).** Biri taramanın, öbürü işletim sistemine
    bildirilen ilişkilendirmenin listesi; ayrıştıklarında kullanıcı çift
    tıkladığı dosyanın kütüphanede görünmediğini (ya da tersini) yaşıyor.
14. **`with_initializer` içindeki her `?` AÇILIŞI düşürüyor.** Orada
    başarısız olan tek bir özellik, pencere hiç açılmadan
    `0xc0000409` demek. Oynatmanın şartı olanlar (`wid`, `idle`,
    `input-*`) `?` ile kalsın; süsleme olanlar (renk, ad) hatasını
    yutsun. mpv seçenek adlarını sürümler arası değiştiriyor
    (`background` → `background-color`, mpv 0.38) ve o değişiklik
    kullanıcıya "uygulama açılmıyor" diye ulaşıyor. Ayrıntı:
    `docs/Mpv_Integration.md`.
15. **Yüzey yalnız OYNATICI penceresinde.** Video kütüphane penceresine
    çizilemiyor ve çizilmeye çalışılmamalı. Bir şey "video görünmüyor"
    diyorsa önce sorulacak soru: doğru pencere görünür mü
    (`pencere::gorunur_yap`), yüzey gizlenmiş mi (`useSahneOlcu`
    temizliği). İkisi de sessiz başarısız oluyor — ses gelir, görüntü
    gelmez.
16. **İzlenen bir mpv özelliğini güncellerken OLAY da yayınla.**
    `ozellik_degisti` içinde `durumu_degistir` çağırıp `app.emit`
    unutmak, arayüzün o değeri yalnız dosya açılışında öğrenmesi demek.
    `duration` ve `speed` tam olarak böyle kaçmıştı: mpv süreyi
    `FileLoaded`dan sonra öğrendiğinde arayüzdeki süre 0 kalıyor, arama
    sürgüsü sessizce devre dışı kalıyordu.
17. **Yüzey dikdörtgeni MANTIKSAL piksel.** Ölçekle çarpma kararı
    `mpv/yuzey.rs`in platform modüllerinde; ortak yola koymak HiDPI Linux ve
    Mac'te videoyu iki katı büyüklükte çizerdi.

## Diğer Mui Projeleriyle Tutarlılık

- Dokümantasyon-first: önce `docs/`, sonra implementasyon.
- CLAUDE.md blanket instruction yerine trigger-table (Muivly'den öğrenilen
  ders, Muiwatch'ta da böyle).
- Tauri v2 tercihi Muiget/Muivly/Muiwatch stack pattern'iyle uyumlu.
- **Tasarım dili kardeş projelerden birebir devralındı:** teal `#2dd4bf`
  vurgu, koyu `#0f1115` zemin, gömülü Outfit, Türkçe sınıf adları
  (`.dugme`, `.kart`, `.rozet`). Kanonik kaynak `..\Muiget\src\styles.css`;
  jeton listesi `..\MuiLabs\docs\ui-conventions.md`.
- Uygulama ikonu ailenin iskeleti: koyu yuvarlak kare + teal glyph + **tek
  dolu öğe**. Üç yerde aynı çizim: `public/icons/muiply.svg`, `index.html`
  favicon'u, `src-tauri/icons/kaynak.svg`. Biri değişirse üçü de değişir.
  Muiwatch'ın glyph'i bir ekran (işi iki kişiye aynı kareyi izletmek);
  Muiply'inki çember + üçgen, çünkü işi tek şey: dosyayı açıp çalmak.
