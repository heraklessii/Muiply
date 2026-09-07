//! SQLite şeması ve sorguları.
//!
//! Tek dosya, tek bağlantı. Çalma listesi tabloları da burada tanımlı
//! (`playlist/mod.rs` aynı bağlantıyı kullanıyor): şemayı ikiye bölmek,
//! yabancı anahtarların hangi dosyada kurulduğunu aramak demek olurdu.
//!
//! Not: `docs/Library.md`'deki `UNIQUE(playlist_id, position)` kısıtı BURADA
//! YOK. Sıralama değiştirirken satırlar geçici olarak aynı konuma düşüyor ve
//! kısıt her yeniden sıralamayı çok adımlı bir dansa çeviriyordu. Sıra
//! `ORDER BY position` ile korunuyor; tekrar eden konum bir görüntü hatası,
//! veri kaybı değil.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::hata::{Hata, Sonuc};

use super::{MediaFilter, MediaItem};

const SEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS media (
    id          TEXT PRIMARY KEY,
    path        TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL,
    artist      TEXT,
    album       TEXT,
    duration    REAL NOT NULL DEFAULT 0,
    width       INTEGER,
    height      INTEGER,
    size        INTEGER NOT NULL DEFAULT 0,
    media_type  TEXT NOT NULL,
    extension   TEXT NOT NULL,
    thumbnail   TEXT,
    -- Dosyanın değişme zamanı. Tarama bunu karşılaştırıp değişmemiş
    -- dosyaları hiç açmıyor; ikinci tarama birincinin onda biri sürüyor.
    mtime       INTEGER NOT NULL DEFAULT 0,
    added_at    INTEGER NOT NULL,
    last_played INTEGER,
    play_count  INTEGER NOT NULL DEFAULT 0,
    -- Yarım bırakılan yer (saniye). NULL "baştan başla" demek; dosya sonuna
    -- kadar izlendiğinde de NULL'a dönüyor (library/devam.rs).
    last_position REAL
);

CREATE INDEX IF NOT EXISTS media_tur ON media(media_type);
CREATE INDEX IF NOT EXISTS media_son ON media(last_played DESC);

CREATE TABLE IF NOT EXISTS folders (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    path     TEXT NOT NULL UNIQUE,
    added_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    media_id    TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS liste_sira ON playlist_items(playlist_id, position);

-- Ayarlar anahtar/değer olarak duruyor, sütun olarak değil: yeni bir ayar
-- eklemek şema geçişi gerektirmesin. Değer HEP metin; hangi anahtarın nasıl
-- okunacağı settings/mod.rs'de, çünkü okuyan taraf zaten ne beklediğini
-- biliyor.
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

/// `CREATE TABLE IF NOT EXISTS` var olan bir tabloya sütun EKLEMİYOR.
///
/// Yani [`SEMA`] yalnız boş bir veritabanını doğru kuruyor; kullanıcının
/// diskinde zaten duran bir `library.db` şemanın eski hâlinde kalıyor ve
/// yeni sütuna yapılan ilk sorgu "no such column" ile patlardı.
///
/// Geçişler ADLA ve KOŞULLU: `PRAGMA table_info` ile sütun aranıyor, yoksa
/// ekleniyor. Sürüm numarası (`user_version`) tutulmuyor — bugün geçişlerin
/// hepsi "sütun ekle" ve her biri kendi koşulunu kendisi biliyor; numara
/// tutmak aynı bilgiyi ikinci kez, üstelik senkron kalması gereken bir yerde
/// saklamak olurdu.
fn gecisler(conn: &Connection) -> Sonuc<()> {
    if !sutun_var(conn, "media", "last_position")? {
        conn.execute_batch("ALTER TABLE media ADD COLUMN last_position REAL")?;
    }
    Ok(())
}

fn sutun_var(conn: &Connection, tablo: &str, sutun: &str) -> Sonuc<bool> {
    // Tablo adı sorgu parametresi olamıyor (PRAGMA'nın kısıtı); adlar bu
    // dosyadaki sabitlerden geliyor, kullanıcı girdisi değil.
    let mut sorgu = conn.prepare(&format!("PRAGMA table_info({tablo})"))?;
    let mut satirlar = sorgu.query([])?;
    while let Some(s) = satirlar.next()? {
        if s.get::<_, String>(1)? == sutun {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `MediaItem` üreten her sorgunun sütun sırası. Tek yerde çünkü
/// [`satir_to_medya`] konum numarasıyla okuyor: liste ile okuyucu ayrı
/// düşerse hata "başlık alanında dosya yolu görünüyor" gibi sessiz bir şey.
const SUTUNLAR: &str = "id, path, title, artist, album, duration, width, height, size, \
                        media_type, extension, thumbnail, added_at, last_played, play_count, \
                        last_position";

/// "Bu yol şu kökün ALTINDA mı" koşulu. İki desen alıyor: `?1` ters bölü
/// (Windows), `?2` düz bölü ayraçlı hâl.
///
/// Ayraç deseni ŞART. `path LIKE 'kök%'` yazmak, kökle aynı harflerle
/// başlayan kardeş klasörleri de kapsıyordu: `C:\Videos` kaldırıldığında
/// `C:\Videos2` altındaki kayıtlar da siliniyordu.
const ALT_YOL_KOSULU: &str = r"(path LIKE ?1 ESCAPE '\' OR path LIKE ?2 ESCAPE '\')";

/// LIKE deseninde anlamı olan karakterleri kaçırır.
///
/// `%` ve `_` Windows'ta geçerli dosya adı karakterleri: `C:\Muzik_2024`
/// kaçırılmadan desene konduğunda `_` "herhangi bir karakter" oluyor ve
/// `C:\MuzikX2024` de eşleşiyordu.
fn like_kacir(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for c in metin.chars() {
        if matches!(c, '\\' | '%' | '_') {
            cikti.push('\\');
        }
        cikti.push(c);
    }
    cikti
}

/// Kökün altındaki yolları eşleyen iki LIKE deseni: (ters bölü, düz bölü).
///
/// İki desen çünkü veritabanındaki yollar işletim sisteminden geliyor ve
/// kullanıcının eklediği kök ayracı ters de olabilir düz de. Kökün kendi
/// sonundaki ayraçlar atılıyor — `C:\` gibi bir sürücü kökü de çalışsın.
fn alt_yol_desenleri(kok: &str) -> (String, String) {
    let temiz = kok.trim_end_matches(['/', '\\']);
    let govde = like_kacir(temiz);
    // Ayraç deseni kaçırılmış hâlde yazılıyor: `\` LIKE'ın kaçış karakteri.
    (format!(r"{govde}\\%"), format!("{govde}/%"))
}

pub struct LibraryDb {
    conn: Connection,
}

/// Tarayıcının yazdığı kayıt. [`MediaItem`]'dan ayrı: oynatma sayacı ve
/// eklenme zamanı taramanın işi değil, onları tarama ezmemeli.
pub struct YeniMedya {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub size: i64,
    pub media_type: String,
    pub extension: String,
    pub mtime: i64,
}

impl LibraryDb {
    pub fn ac(dizin: &Path) -> Sonuc<Self> {
        std::fs::create_dir_all(dizin)?;
        let conn = Connection::open(dizin.join("library.db"))?;
        conn.execute_batch(SEMA)?;
        gecisler(&conn)?;
        Ok(LibraryDb { conn })
    }

    /// Çalma listesi modülü aynı bağlantıyı kullanıyor.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Bellekte, şeması kurulmuş boş bir kütüphane — yalnız testler için.
    ///
    /// Burada duruyor çünkü `conn` alanı özel: modül dışındaki testler
    /// (`playlist/mod.rs`) `LibraryDb`yi kendileri kuramıyor. Diske
    /// dokunmuyor, yani test arkasında dosya bırakmıyor.
    #[cfg(test)]
    pub fn bellekte() -> Sonuc<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SEMA)?;
        gecisler(&conn)?;
        Ok(LibraryDb { conn })
    }

    // -- klasörler ----------------------------------------------------------

    pub fn klasor_ekle(&self, yol: &str) -> Sonuc<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO folders (path, added_at) VALUES (?1, ?2)",
            params![yol, super::simdi()],
        )?;
        Ok(())
    }

    /// Klasörü izlenenlerden çıkarır ve altındaki kayıtları siler.
    ///
    /// Kayıtları bırakmak, kullanıcının "kaldırdım" dediği şeyin ızgarada
    /// durmaya devam etmesi demekti.
    ///
    /// Eşleşme [`ALT_YOL_KOSULU`] ile, düz bir ön ekle DEĞİL: `C:\Videos`
    /// kaldırılırken `C:\Videos2` altındaki kayıtlar da silinirdi.
    pub fn klasor_sil(&self, yol: &str) -> Sonuc<()> {
        self.conn
            .execute("DELETE FROM folders WHERE path = ?1", params![yol])?;

        let (ters, duz) = alt_yol_desenleri(yol);
        self.conn.execute(
            &format!("DELETE FROM media WHERE {ALT_YOL_KOSULU}"),
            params![ters, duz],
        )?;
        Ok(())
    }

    pub fn klasorler(&self) -> Sonuc<Vec<String>> {
        let mut sorgu = self
            .conn
            .prepare("SELECT path FROM folders ORDER BY added_at")?;
        let satirlar = sorgu.query_map([], |s| s.get::<_, String>(0))?;
        Ok(satirlar.filter_map(Result::ok).collect())
    }

    // -- medya --------------------------------------------------------------

    /// Kaydı ekler ya da günceller.
    ///
    /// `ON CONFLICT(path)`: aynı yol ikinci kez tarandığında değişen alanlar
    /// yazılıyor ama `added_at`, `play_count`, `last_played` ve `thumbnail`
    /// KORUNUYOR — onlar kullanıcının geçmişi, taramanın bilgisi değil.
    pub fn yaz(&self, k: &YeniMedya) -> Sonuc<()> {
        self.conn.execute(
            "INSERT INTO media
                 (id, path, title, artist, album, duration, width, height, size,
                  media_type, extension, mtime, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(path) DO UPDATE SET
                 title      = excluded.title,
                 artist     = excluded.artist,
                 album      = excluded.album,
                 duration   = excluded.duration,
                 width      = excluded.width,
                 height     = excluded.height,
                 size       = excluded.size,
                 media_type = excluded.media_type,
                 extension  = excluded.extension,
                 mtime      = excluded.mtime",
            params![
                k.id,
                k.path,
                k.title,
                k.artist,
                k.album,
                k.duration,
                k.width,
                k.height,
                k.size,
                k.media_type,
                k.extension,
                k.mtime,
                super::simdi(),
            ],
        )?;
        Ok(())
    }

    /// Kayıtlı değişme zamanı. Tarama bununla "bu dosyayı atlayabilir miyim"
    /// sorusunu yanıtlıyor.
    pub fn mtime(&self, yol: &str) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT mtime FROM media WHERE path = ?1",
                params![yol],
                |s| s.get::<_, i64>(0),
            )
            .optional()
            .ok()
            .flatten()
    }

    /// Bütün kayıtların yol → değişme zamanı eşlemesi.
    ///
    /// Tarama bunu bir kez alıp bellekte soruyor. Dosya başına [`Self::mtime`]
    /// çağırmak, bin dosyalık bir klasörde bin ayrı sorgu ve bin kilit alma
    /// demekti — ikinci taramanın işi zaten "hiçbir şey yapma"yken.
    pub fn mtimeler(&self) -> Sonuc<std::collections::HashMap<String, i64>> {
        let mut sorgu = self.conn.prepare("SELECT path, mtime FROM media")?;
        let satirlar =
            sorgu.query_map([], |s| Ok((s.get::<_, String>(0)?, s.get::<_, i64>(1)?)))?;
        Ok(satirlar.filter_map(Result::ok).collect())
    }

    /// Bir küme kaydı TEK işlemde yazar.
    ///
    /// Tek tek yazmak kayıt başına bir işlem kesinleştirmesi demek; taramanın
    /// süresini belirleyen şey diskin kendisi değil o kesinleştirmeler
    /// oluyordu. Kümenin küçük tutulması bilinçli: bütün taramayı tek işleme
    /// almak, tarama boyunca yazma kilidini tutup arayüzün sorgularını
    /// bekletirdi.
    ///
    /// `SAVEPOINT`, `BEGIN` değil: çağıran bir gün dışarıda bir işlem açarsa
    /// iç içe girmesin.
    pub fn yaz_toplu(&self, kayitlar: &[YeniMedya]) -> Sonuc<()> {
        if kayitlar.is_empty() {
            return Ok(());
        }

        self.conn.execute_batch("SAVEPOINT toplu_yaz")?;
        for k in kayitlar {
            if let Err(e) = self.yaz(k) {
                let _ = self
                    .conn
                    .execute_batch("ROLLBACK TO toplu_yaz; RELEASE toplu_yaz");
                return Err(e);
            }
        }
        self.conn.execute_batch("RELEASE toplu_yaz")?;
        Ok(())
    }

    pub fn medya(&self, f: &MediaFilter) -> Sonuc<Vec<MediaItem>> {
        // Sıralama anahtarı KULLANICI GİRDİSİ; doğrudan SQL'e gömülüyor, bu
        // yüzden bilinen dizelerle eşleştiriliyor. Eşleşmeyen değer sessizce
        // varsayılana düşüyor — hata vermek, arayüzü yeni bir sıralama
        // eklendiğinde kırardı.
        let sira = match f.sort.as_deref() {
            Some("title") => "title COLLATE NOCASE ASC",
            Some("duration") => "duration DESC",
            Some("played") => "last_played DESC NULLS LAST, added_at DESC",
            _ => "added_at DESC",
        };
        let limit = f.limit.unwrap_or(-1);

        let (kosul, tur): (&str, Option<String>) = match f.media_type.as_deref() {
            Some(t @ ("video" | "audio")) => ("WHERE media_type = ?1", Some(t.to_string())),
            _ => ("", None),
        };

        let sql = format!("SELECT {SUTUNLAR} FROM media {kosul} ORDER BY {sira} LIMIT {limit}");
        let mut sorgu = self.conn.prepare(&sql)?;

        let satirlar = match tur {
            Some(t) => sorgu
                .query_map(params![t], satir_to_medya)?
                .collect::<Vec<_>>(),
            None => sorgu.query_map([], satir_to_medya)?.collect::<Vec<_>>(),
        };
        Ok(satirlar.into_iter().filter_map(Result::ok).collect())
    }

    /// En son çalınanlar. Hiç çalınmamışlar listeye girmiyor: "son çalınanlar"
    /// başlığı altında hiç çalınmamış bir dosya göstermek yanlış olurdu.
    pub fn son_calinanlar(&self, limit: i64) -> Sonuc<Vec<MediaItem>> {
        let sql = format!(
            "SELECT {SUTUNLAR} FROM media WHERE last_played IS NOT NULL \
             ORDER BY last_played DESC LIMIT ?1"
        );
        let mut sorgu = self.conn.prepare(&sql)?;
        let satirlar = sorgu.query_map(params![limit], satir_to_medya)?;
        Ok(satirlar.filter_map(Result::ok).collect())
    }

    pub fn medya_bir(&self, id: &str) -> Sonuc<Option<MediaItem>> {
        let sql = format!("SELECT {SUTUNLAR} FROM media WHERE id = ?1");
        Ok(self
            .conn
            .query_row(&sql, params![id], satir_to_medya)
            .optional()?)
    }

    pub fn yol_ile(&self, yol: &str) -> Sonuc<Option<MediaItem>> {
        let sql = format!("SELECT {SUTUNLAR} FROM media WHERE path = ?1");
        Ok(self
            .conn
            .query_row(&sql, params![yol], satir_to_medya)
            .optional()?)
    }

    pub fn sil(&self, id: &str) -> Sonuc<()> {
        self.conn
            .execute("DELETE FROM media WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Oynatma sayacını artırır.
    pub fn oynatildi(&self, id: &str) -> Sonuc<()> {
        self.conn.execute(
            "UPDATE media SET play_count = play_count + 1, last_played = ?2 WHERE id = ?1",
            params![id, super::simdi()],
        )?;
        Ok(())
    }

    /// Yarım bırakılan yeri yazar. `None` "baştan başla" demek.
    ///
    /// Anahtar YOL, kimlik değil: yazan taraf mpv'nin olay döngüsü ve onun
    /// elinde yalnız çalan dosyanın yolu var. Kütüphanede kaydı olmayan bir
    /// dosyada hiçbir satır güncellenmiyor — hata değil, yazacak yer yok.
    pub fn konum_yaz(&self, yol: &str, konum: Option<f64>) -> Sonuc<()> {
        self.conn.execute(
            "UPDATE media SET last_position = ?2 WHERE path = ?1",
            params![yol, konum],
        )?;
        Ok(())
    }

    /// Kayıtlı yarım kalma yeri. Kayıt yoksa `None`.
    ///
    /// Hata da `None`: açılan dosyanın konumunu okuyamamak oynatmayı
    /// engellememeli, en kötüsü dosyanın baştan başlaması.
    pub fn konum_oku(&self, yol: &str) -> Option<f64> {
        self.conn
            .query_row(
                "SELECT last_position FROM media WHERE path = ?1",
                params![yol],
                |s| s.get::<_, Option<f64>>(0),
            )
            .optional()
            .ok()
            .flatten()
            .flatten()
    }

    // -- ayarlar ------------------------------------------------------------

    /// Kayıtlı bütün ayarlar. Okunamayan satır listeye girmiyor; çağıran
    /// eksik anahtar için varsayılana düşüyor (`settings/mod.rs`).
    pub fn ayarlar(&self) -> Sonuc<std::collections::HashMap<String, String>> {
        let mut sorgu = self.conn.prepare("SELECT key, value FROM settings")?;
        let satirlar =
            sorgu.query_map([], |s| Ok((s.get::<_, String>(0)?, s.get::<_, String>(1)?)))?;
        Ok(satirlar.filter_map(Result::ok).collect())
    }

    /// Bir ayarı yazar. Anahtar varsa üzerine.
    pub fn ayar_yaz(&self, anahtar: &str, deger: &str) -> Sonuc<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![anahtar, deger],
        )?;
        Ok(())
    }

    pub fn kucukresim_yaz(&self, id: &str, yol: &str) -> Sonuc<()> {
        self.conn.execute(
            "UPDATE media SET thumbnail = ?2 WHERE id = ?1",
            params![id, yol],
        )?;
        Ok(())
    }

    /// Bir kök altındaki bütün kayıtların (id, yol) çiftleri.
    /// Tarama sonunda diskten silinmiş olanları ayıklamak için.
    pub fn kokteki_kayitlar(&self, kok: &str) -> Sonuc<Vec<(String, String)>> {
        let (ters, duz) = alt_yol_desenleri(kok);
        let mut sorgu = self.conn.prepare(&format!(
            "SELECT id, path FROM media WHERE {ALT_YOL_KOSULU}"
        ))?;
        let satirlar = sorgu.query_map(params![ters, duz], |s| {
            Ok((s.get::<_, String>(0)?, s.get::<_, String>(1)?))
        })?;
        Ok(satirlar.filter_map(Result::ok).collect())
    }
}

/// [`SUTUNLAR`]in bir tabloya nitelenmiş hâli — `JOIN`li sorgular için.
///
/// Gerekli çünkü `media` ile `playlist_items` iki sütun adını paylaşıyor
/// (`id`, `position`); niteliksiz liste "ambiguous column name" veriyor.
/// Listeyi elle ikinci kez yazmak, tam da [`SUTUNLAR`]in önlemek için var
/// olduğu sessiz kaymayı üretirdi: sütun eklendiğinde biri güncelleniyor,
/// diğeri unutuluyor.
pub fn sutunlar(onek: &str) -> String {
    SUTUNLAR
        .split(',')
        .map(|s| format!("{onek}.{}", s.trim()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// [`SUTUNLAR`]deki sütun sayısı — yani [`satir_to_medya`]nın okumadığı ilk
/// indeks.
///
/// Medya sütunlarının YANINA kendi sütununu ekleyen sorgular
/// ([`crate::playlist::ogeler`]) satırlarını bu indeksten okuyor. Elle `16`
/// yazmak, `media`ya sütun eklendiğinde sessizce yanlış alanı okumak
/// demekti; buradaki sayı listenin kendisinden çıkıyor.
pub const SUTUN_SAYISI: usize = {
    // `str::split` const bağlamda çağrılamıyor; virgülleri sayıp bir
    // fazlasını almak elde kalan tek yol.
    let bayt = SUTUNLAR.as_bytes();
    let mut i = 0;
    let mut adet = 1;
    while i < bayt.len() {
        if bayt[i] == b',' {
            adet += 1;
        }
        i += 1;
    }
    adet
};

pub fn satir_to_medya(s: &Row<'_>) -> rusqlite::Result<MediaItem> {
    Ok(MediaItem {
        id: s.get(0)?,
        path: s.get(1)?,
        title: s.get(2)?,
        artist: s.get(3)?,
        album: s.get(4)?,
        duration: s.get(5)?,
        width: s.get(6)?,
        height: s.get(7)?,
        size: s.get(8)?,
        media_type: s.get(9)?,
        extension: s.get(10)?,
        thumbnail: s.get(11)?,
        added_at: s.get(12)?,
        last_played: s.get(13)?,
        play_count: s.get(14)?,
        last_position: s.get(15)?,
    })
}

/// Kilidi alıp veritabanına erişmenin kısa yolu.
///
/// `Mutex` zehirlenmesi (bir iş parçacığı kilitliyken panikledi) burada
/// hataya çevriliyor: `unwrap` uygulamayı kapatırdı, oysa kütüphane
/// bozulduysa bile oynatıcı çalışmaya devam edebilir.
pub fn kilit(durum: &super::LibraryState) -> Sonuc<std::sync::MutexGuard<'_, LibraryDb>> {
    durum
        .db
        .lock()
        .map_err(|_| Hata::yeni("kütüphane veritabanı kilidi bozuldu; uygulamayı yeniden başlatın"))
}

#[cfg(test)]
mod testler {
    use super::*;

    /// Bellekte bir kütüphane. Diske dokunmuyor: test klasör bırakmasın.
    fn bos_db() -> LibraryDb {
        LibraryDb::bellekte().expect("bellek veritabanı")
    }

    fn kayit(db: &LibraryDb, yol: &str) {
        db.yaz(&YeniMedya {
            id: crate::library::kimlik(yol),
            path: yol.into(),
            title: "x".into(),
            artist: None,
            album: None,
            duration: 0.0,
            width: None,
            height: None,
            size: 0,
            media_type: "video".into(),
            extension: "mkv".into(),
            mtime: 0,
        })
        .expect("yazma");
    }

    fn yollar(db: &LibraryDb) -> Vec<String> {
        let mut s = db
            .conn
            .prepare("SELECT path FROM media ORDER BY path")
            .unwrap();
        let v: Vec<String> = s
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        v
    }

    #[test]
    fn klasor_silme_ayni_harfle_baslayan_kardesi_birakiyor() {
        // Asıl hata buydu: `LIKE 'C:\Videos%'` kardeşi de siliyordu.
        let db = bos_db();
        kayit(&db, r"C:\Videos\a.mkv");
        kayit(&db, r"C:\Videos\alt\b.mkv");
        kayit(&db, r"C:\Videos2\c.mkv");
        kayit(&db, r"C:\VideosEski\d.mkv");

        db.klasor_sil(r"C:\Videos").expect("silme");

        assert_eq!(
            yollar(&db),
            vec![r"C:\Videos2\c.mkv", r"C:\VideosEski\d.mkv"]
        );
    }

    #[test]
    fn klasor_adindaki_alt_cizgi_joker_degil() {
        // `_` LIKE'ta "herhangi bir karakter"; kaçırılmazsa MuzikX2024 de gider.
        let db = bos_db();
        kayit(&db, r"C:\Muzik_2024\a.mp3");
        kayit(&db, r"C:\MuzikX2024\b.mp3");

        db.klasor_sil(r"C:\Muzik_2024").expect("silme");

        assert_eq!(yollar(&db), vec![r"C:\MuzikX2024\b.mp3"]);
    }

    #[test]
    fn klasor_adindaki_yuzde_joker_degil() {
        let db = bos_db();
        kayit(&db, r"C:\%20\a.mp3");
        kayit(&db, r"C:\baska\b.mp3");

        db.klasor_sil(r"C:\%20").expect("silme");

        assert_eq!(yollar(&db), vec![r"C:\baska\b.mp3"]);
    }

    #[test]
    fn sondaki_ayrac_ve_surucu_koku() {
        let db = bos_db();
        kayit(&db, r"C:\Videos\a.mkv");
        kayit(&db, r"D:\Videos\b.mkv");

        // Sondaki ayraçla verilen kök de aynı şeyi eşlemeli.
        db.klasor_sil(r"C:\Videos\").expect("silme");
        assert_eq!(yollar(&db), vec![r"D:\Videos\b.mkv"]);

        // Sürücü kökü: altındaki her şey.
        db.klasor_sil(r"D:\").expect("silme");
        assert!(yollar(&db).is_empty());
    }

    #[test]
    fn duz_bolu_ayrac_da_esleniyor() {
        let db = bos_db();
        kayit(&db, "/home/ilker/Videolar/a.mkv");
        kayit(&db, "/home/ilker/Videolar2/b.mkv");

        db.klasor_sil("/home/ilker/Videolar").expect("silme");

        assert_eq!(yollar(&db), vec!["/home/ilker/Videolar2/b.mkv"]);
    }

    #[test]
    fn kokteki_kayitlar_da_ayni_siniri_kullaniyor() {
        let db = bos_db();
        kayit(&db, r"C:\Videos\a.mkv");
        kayit(&db, r"C:\Videos2\b.mkv");

        let bulunan = db.kokteki_kayitlar(r"C:\Videos").expect("sorgu");

        assert_eq!(bulunan.len(), 1);
        assert_eq!(bulunan[0].1, r"C:\Videos\a.mkv");
    }

    fn yeni_medya(yol: &str, mtime: i64) -> YeniMedya {
        YeniMedya {
            id: crate::library::kimlik(yol),
            path: yol.into(),
            title: "x".into(),
            artist: None,
            album: None,
            duration: 0.0,
            width: None,
            height: None,
            size: 0,
            media_type: "video".into(),
            extension: "mkv".into(),
            mtime,
        }
    }

    #[test]
    fn toplu_yazma_hepsini_yaziyor() {
        let db = bos_db();
        let kayitlar: Vec<YeniMedya> = (0..5)
            .map(|i| yeni_medya(&format!(r"C:\V\{i}.mkv"), i))
            .collect();

        db.yaz_toplu(&kayitlar).expect("toplu yazma");

        assert_eq!(yollar(&db).len(), 5);
        assert_eq!(db.mtime(r"C:\V\3.mkv"), Some(3));
    }

    #[test]
    fn toplu_yazma_bos_yiginda_calisiyor() {
        // Tarama son yığını koşulsuz gönderiyor; boş olabilir.
        let db = bos_db();
        db.yaz_toplu(&[]).expect("boş yığın");
        assert!(yollar(&db).is_empty());
    }

    #[test]
    fn toplu_yazma_ayni_yolu_gunceller() {
        // `ON CONFLICT(path)`: ikinci tarama aynı yolu yeniden yazıyor.
        let db = bos_db();
        db.yaz_toplu(&[yeni_medya(r"C:\V\a.mkv", 1)]).expect("ilk");
        db.yaz_toplu(&[yeni_medya(r"C:\V\a.mkv", 2)])
            .expect("ikinci");

        assert_eq!(yollar(&db).len(), 1);
        assert_eq!(db.mtime(r"C:\V\a.mkv"), Some(2));
    }

    #[test]
    fn mtimeler_kayitli_zamanlari_veriyor() {
        let db = bos_db();
        db.yaz_toplu(&[yeni_medya(r"C:\V\a.mkv", 7), yeni_medya(r"C:\V\b.mkv", 9)])
            .expect("yazma");

        let harita = db.mtimeler().expect("mtimeler");

        assert_eq!(harita.len(), 2);
        assert_eq!(harita.get(r"C:\V\a.mkv"), Some(&7));
        assert_eq!(harita.get(r"C:\V\b.mkv"), Some(&9));
        assert_eq!(harita.get(r"C:\V\yok.mkv"), None);
    }

    #[test]
    fn konum_yazilip_okunuyor() {
        let db = bos_db();
        kayit(&db, "/ev/V/a.mkv");

        assert_eq!(db.konum_oku("/ev/V/a.mkv"), None);

        db.konum_yaz("/ev/V/a.mkv", Some(123.5)).expect("yazma");
        assert_eq!(db.konum_oku("/ev/V/a.mkv"), Some(123.5));

        // Dosya sonuna kadar izlendi: kayıt siliniyor, baştan başlasın.
        db.konum_yaz("/ev/V/a.mkv", None).expect("temizleme");
        assert_eq!(db.konum_oku("/ev/V/a.mkv"), None);
    }

    #[test]
    fn konum_kutuphanede_olmayan_dosyada_sessiz() {
        // Sürükleyip bırakılan dosyanın kaydı yok; yazacak satır da yok.
        let db = bos_db();
        db.konum_yaz("/ev/baska/x.mkv", Some(10.0)).expect("yazma");
        assert_eq!(db.konum_oku("/ev/baska/x.mkv"), None);
    }

    #[test]
    fn tarama_yarim_kalma_yerini_ezmiyor() {
        // İkinci tarama aynı satırı yeniden yazıyor; kullanıcının nerede
        // kaldığı taramanın bilgisi değil.
        let db = bos_db();
        kayit(&db, "/ev/V/a.mkv");
        db.konum_yaz("/ev/V/a.mkv", Some(42.0)).expect("yazma");

        kayit(&db, "/ev/V/a.mkv");

        assert_eq!(db.konum_oku("/ev/V/a.mkv"), Some(42.0));
    }

    #[test]
    fn eski_veritabanina_sutun_ekleniyor() {
        // Şemanın last_position'dan ÖNCEKİ hâli. Geçiş çalışmazsa sonraki
        // sorgu "no such column" ile patlar.
        let conn = Connection::open_in_memory().expect("bellek veritabanı");
        conn.execute_batch(
            "CREATE TABLE media (
                 id TEXT PRIMARY KEY, path TEXT NOT NULL UNIQUE, title TEXT NOT NULL,
                 artist TEXT, album TEXT, duration REAL NOT NULL DEFAULT 0,
                 width INTEGER, height INTEGER, size INTEGER NOT NULL DEFAULT 0,
                 media_type TEXT NOT NULL, extension TEXT NOT NULL, thumbnail TEXT,
                 mtime INTEGER NOT NULL DEFAULT 0, added_at INTEGER NOT NULL,
                 last_played INTEGER, play_count INTEGER NOT NULL DEFAULT 0)",
        )
        .expect("eski şema");

        assert!(!sutun_var(&conn, "media", "last_position").unwrap());
        gecisler(&conn).expect("geçiş");
        assert!(sutun_var(&conn, "media", "last_position").unwrap());

        // Her açılışta çağrılıyor: ikinci kez çalıştırmak da güvenli olmalı.
        gecisler(&conn).expect("ikinci geçiş");
    }

    #[test]
    fn ayarlar_yazilip_okunuyor() {
        let db = bos_db();
        assert!(db.ayarlar().expect("okuma").is_empty());

        db.ayar_yaz("hwdec", "no").expect("yazma");
        db.ayar_yaz("resume", "1").expect("yazma");
        // Aynı anahtarın üzerine yazılıyor, ikinci satır açılmıyor.
        db.ayar_yaz("hwdec", "auto").expect("yazma");

        let a = db.ayarlar().expect("okuma");
        assert_eq!(a.len(), 2);
        assert_eq!(a.get("hwdec").map(String::as_str), Some("auto"));
        assert_eq!(a.get("resume").map(String::as_str), Some("1"));
    }

    #[test]
    fn kokun_kendisi_kayit_degil() {
        // Klasörün TAM yolu bir medya kaydı olamaz; desen onu eşlememeli.
        let db = bos_db();
        kayit(&db, r"C:\Videos");

        db.klasor_sil(r"C:\Videos").expect("silme");

        assert_eq!(yollar(&db), vec![r"C:\Videos"]);
    }
}
