# Kütüphane ve Veritabanı

## Konum

`$APPDATA/com.mui.muiply/library.db` — Tauri'nin `app_data_dir()` verdiği yer.
Küçük resimler aynı klasörün altında: `kucukresim/{id}.jpg`.

Veritabanı **WAL** kipinde açılıyor: tarama arka planda yazarken arayüz
okumaya devam edebilsin.

## Şema

```sql
CREATE TABLE IF NOT EXISTS media (
    id          TEXT PRIMARY KEY,   -- yolun SHA-256'sının ilk 16 baytı
    path        TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL,
    artist      TEXT,
    album       TEXT,
    duration    REAL NOT NULL DEFAULT 0,
    width       INTEGER,
    height      INTEGER,
    size        INTEGER NOT NULL DEFAULT 0,
    media_type  TEXT NOT NULL,      -- "video" | "audio"
    extension   TEXT NOT NULL,
    thumbnail   TEXT,               -- küçük resmin tam yolu
    mtime       INTEGER NOT NULL DEFAULT 0,
    added_at    INTEGER NOT NULL,
    last_played INTEGER,
    play_count  INTEGER NOT NULL DEFAULT 0,
    last_position REAL              -- yarım bırakılan yer; NULL = baştan
);

CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    added_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    media_id    TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL
);

-- Ayarlar sütun değil anahtar/değer: yeni bir ayar şema geçişi
-- gerektirmesin. Değer hep metin; tipi `settings/mod.rs` biliyor.
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### Geçişler

`CREATE TABLE IF NOT EXISTS` var olan bir tabloya **sütun eklemiyor**. Yani
yukarıdaki şema yalnız boş bir veritabanını doğru kuruyor; kullanıcının
diskinde zaten duran `library.db` eski hâlinde kalıyordu ve yeni sütuna
yapılan ilk sorgu "no such column" ile patlıyordu.

`db::gecisler` her açılışta çalışıyor ve **ada bakıyor**: `PRAGMA table_info`
ile sütun aranıyor, yoksa `ALTER TABLE` ile ekleniyor. Sürüm numarası
(`user_version`) tutulmuyor — bugün geçişlerin hepsi "sütun ekle" ve her biri
kendi koşulunu kendisi biliyor; numara tutmak aynı bilgiyi ikinci kez, üstelik
senkron kalması gereken bir yerde saklamak olurdu.

### `last_position` — kaldığı yerden devam

Anahtar **yol**, kimlik değil: yazan taraf mpv'nin olay döngüsü ve onun elinde
yalnız çalan dosyanın yolu var. Kütüphanede kaydı olmayan bir dosyada
(sürüklenip bırakılmış) hiçbir satır güncellenmiyor — hata değil, yazacak yer
yok.

| Ne zaman | Ne oluyor |
|---|---|
| Çalarken, 5 sn'de bir | konum yazılıyor (yalnız ayar açıkken) |
| Pencere kapanırken | son konum yazılıyor |
| `EndFile` · `eof` | kayıt SİLİNİYOR — sonuna kadar izlendi |
| Dosya yüklenirken | eşikler tutuyorsa o konuma atlanıyor |

Eşikler `library/devam.rs` içinde ve saf: başa 20 sn'den yakınsa devam yok
(kullanıcı jeneriği bile geçmemiş), sona 20 sn'den yakınsa da yok (dosya
bitmiş sayılıyor). İkincisi şart çünkü kaydı silen şey `eof` ve son
saniyelerde kapatılan bir dosyada o olay hiç gelmiyor.

### İlk taslaktan sapmalar

**`media.mtime` eklendi.** Tarama bunu karşılaştırıp değişmemiş dosyaları hiç
açmıyor; ikinci tarama birincinin onda biri sürüyor.

**`UNIQUE(playlist_id, position)` kaldırıldı.** Sıralama değiştirirken satırlar
geçici olarak aynı konuma düşüyor ve kısıt her yeniden sıralamayı çok adımlı
bir dansa çeviriyordu. Sıra `ORDER BY position` ile korunuyor; tekrar eden bir
konum bir görüntü hatası, veri kaybı değil.

### Kimlik neden yoldan

Aynı dosyanın iki kopyası kütüphanede **iki kayıt** olmalı — kullanıcı ikisini
de görüyor. İçerik özeti almak her dosyayı baştan sona okumak demekti; tarama
saatler sürerdi.

## Tarama

`walkdir` ile, üç geçiş (`library/tarayici.rs`):

1. **Say.** Aday dosyaları topla, yola göre sırala, tekrarı at. Toplam
   bilinmeden ilerleme çubuğu çizilemez. Tekrar gerçek bir durum: kullanıcı
   hem `C:\A`yı hem `C:\A\B`yi eklediyse aradaki dosyalar iki kökten birden
   geliyor ve aynı dosya iki kez sondayla açılıyordu — taramanın en pahalı
   işi, boşuna.
2. **Yaz.** `mtime` aynıysa atla; değilse [sonda](Mpv_Integration.md#sonda--tarama-neyi-okuyor)
   ile ölç ve yaz.
3. **Ayıkla.** Veritabanında olup diskte olmayanları sil.

Üçüncüsü şart: kullanıcı bir dosyayı silince ızgarada tıklandığında "dosya
bulunamadı" diyen bir kart kalıyordu.

İkinci geçişin iki kısıtı var, ikisi de ölçülen maliyetten geldi:

- **Kayıtlı `mtime`ler tek sorguda** alınıyor (`db.mtimeler`), dosya başına
  değil. İkinci bir taramanın işi zaten "hiçbir şey yapma"yken bin dosya için
  bin sorgu ve bin kilit alınıyordu.
- **Yazma 64'lük yığınlar hâlinde**, tek işlemde (`db.yaz_toplu`). Tek tek
  yazmak kayıt başına bir işlem kesinleştirmesiydi. Yığın bilerek küçük:
  bütün taramayı tek işleme almak, tarama boyunca yazma kilidini tutup
  arayüzün kütüphane sorgularını bekletirdi.

Tarama kendi iş parçacığında; komut hemen dönüyor. İki taramanın çakışması
`AtomicBool::swap` ile engelleniyor — `load` + `store` arasında bir pencere
kalırdı.

Sembolik bağlar izlenmiyor (kendine dönen bir bağ taramayı sonsuz döngüye
sokardı). Okunamayan girdiler sessizce atlanıyor: izin verilmeyen bir alt
klasör yüzünden bütün taramayı durdurmak orantısız.

### Taranan uzantılar

```rust
VIDEO: mp4 mkv avi mov webm flv ts m2ts wmv 3gp ogv m4v mpg mpeg
SES:   mp3 flac aac ogg opus wav m4a wma ape alac aiff mka
```

Liste bilerek **kısa**: mpv çok daha fazlasını açıyor ama kütüphaneye
girmesini istediğimiz şey kullanıcının "film/müzik" saydığı dosyalar.
Klasördeki her `.bin`i listeye almak kütüphaneyi çöplük yapardı.

Sürüklenip bırakılan ya da diyalogdan seçilen dosya **süzülmüyor** — orada
isteyen biri var.

### Başlık seçimi

| Tür | Kaynak | Neden |
|---|---|---|
| Video | dosya adı | "Film.2019.1080p.mkv" kullanıcının tanıdığı ad; konteynerdeki `title` çoğu zaman boş ya da kodlayanın bıraktığı çöp |
| Ses | etiket | Etiket doğru, dosya adı "01.mp3" |

### Ne korunuyor

`ON CONFLICT(path) DO UPDATE` değişen alanları yazıyor ama `added_at`,
`play_count`, `last_played` ve `thumbnail` **korunuyor** — onlar kullanıcının
geçmişi, taramanın bilgisi değil.

## Küçük resimler

Taramada alınmıyor; dosya **çalarken**, 8. saniyede alınıyor. Gerekçe
[`library/kucukresim.rs`](../src-tauri/src/library/kucukresim.rs) başında ve
özeti `docs/Mpv_Integration.md`'de.

Kural olarak: **ızgarada resmi olan kayıtlar, açtıkların.** Hiç açmadıkların
soyut bantla duruyor.

Tam çözünürlükte, JPEG kalite 75 (~150 KB). mpv'nin `screenshot-to-file`
komutunda ölçekleme yok; küçültmek için bir görüntü kütüphanesi eklemek
gerekirdi ve 208 piksel genişliğinde çizilen bir bant için tarayıcının kendi
ölçeklemesi yeterli. Kartlar `loading="lazy"` kullanıyor.

Arayüzde `convertFileSrc` ile `asset:` adresine çevriliyor; izin kapsamı
`tauri.conf.json > security.assetProtocol.scope` içinde ve yalnız küçük resim
klasörünü kapsıyor.

## Sorgular

Süzme ve sıralama SQL'de, **arama arayüzde** (`src/lib/suz.ts`). Her tuş
vuruşunda IPC'ye çıkmak, bellekteki bir diziyi süzmenin yanında bedava değil
ve sonuç aynı.

Sıralama anahtarı kullanıcı girdisi ve doğrudan SQL'e gömülüyor, bu yüzden
bilinen dizelerle eşleştiriliyor (`title` · `added` · `played` · `duration`);
eşleşmeyen değer sessizce varsayılana düşüyor.

## Silme

`library_delete_media` **kaydı** siliyor, dosyayı değil. Muiply bir dosya
yöneticisi değil; diskten silmek geri alınamaz ve bu uygulamanın işi değil.
Arayüzdeki metin de "Kütüphaneden kaldır".

`library_remove_folder` klasörü izlenenlerden çıkarıyor **ve** altındaki
kayıtları siliyor: kullanıcının "kaldırdım" dediği şeyin ızgarada durmaya
devam etmesi yanlış olurdu.

### "Altındaki" ne demek

Düz bir ön ek DEĞİL. `path LIKE 'kök%'` yazmak iki ayrı şeyi bozuyordu ve
ikisi de veri kaybıydı:

- **Kardeş klasörler.** `C:\Videos` kaldırıldığında `C:\Videos2` ve
  `C:\VideosEski` altındaki kayıtlar da siliniyordu — desen kökten sonra
  ayraç aramıyordu.
- **LIKE jokerleri.** `%` ve `_` Windows'ta geçerli dosya adı karakterleri.
  `C:\Muzik_2024` kaldırıldığında `_` "herhangi bir karakter" olduğu için
  `C:\MuzikX2024` de gidiyordu.

Doğrusu `db.rs` → `alt_yol_desenleri`: kökün sonundaki ayraçlar atılıyor,
joker karakterler kaçırılıyor, desene bir ayraç ekleniyor. İki desen
üretiliyor (`\` ve `/`) çünkü kök iki türlü de yazılmış olabilir. Aynı sınır
taramanın üçüncü geçişinde de kullanılıyor (`kokteki_kayitlar`) — yoksa bir
klasörün taraması kardeş klasörün kayıtlarını ayıklamaya kalkıyordu.

Testler `db.rs` içinde: her iki tuzak da birer testle tutuluyor.
