//! Çalma listeleri — SQLite'ta kalıcı, sıralı medya kümeleri.
//!
//! Listeler ile [`kuyruk`] farklı şeyler ve karıştırılmamalı:
//!
//! - **Liste** diskte duruyor, kullanıcı düzenliyor, kapatınca kaybolmuyor.
//! - **Kuyruk** o anki oynatma sırası. Bellekte, tek. Bir liste "kuyruğa
//!   alındığında" öğeleri kuyruğa kopyalanıyor — listeyi düzenlemek çalan
//!   şeyi yarıda değiştirmesin diye.

pub mod kuyruk;
pub mod surucu;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::hata::{Hata, Sonuc};
use crate::library::db::{self, LibraryDb};
use crate::library::MediaItem;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    /// İçindeki öğe sayısı. Yan sütunda ad ile birlikte gösteriliyor; ayrı
    /// bir sayım sorgusu için ikinci bir gidiş dönüş istemiyoruz.
    pub count: i64,
}

/// Bir listenin bir SATIRI: medya kaydı + satırın kendi kimliği.
///
/// İki kimlik gerekiyor çünkü aynı kayıt bir listede iki kez bulunabiliyor
/// (bkz. [`oge_ekle`]) — `media.id` o iki satırı birbirinden ayırmıyor.
/// `item_id` ayırıyor ve silme onunla anlatılıyor ([`oge_sil`]).
///
/// `flatten`: medya alanları iç içe bir nesneye değil, satırın kendi
/// alanlarının yanına yazılıyor. Arayüz tarafında karşılığı
/// `interface PlaylistItem extends MediaItem` — ızgarada ve listede aynı
/// bileşenler aynı alan adlarını okuyor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    /// `playlist_items` satırının kimliği. Medyanın kimliği DEĞİL.
    pub item_id: i64,
    #[serde(flatten)]
    pub media: MediaItem,
}

pub fn olustur(db: &LibraryDb, ad: &str) -> Sonuc<Playlist> {
    let ad = ad.trim();
    if ad.is_empty() {
        return Err(Hata::yeni("çalma listesinin adı boş olamaz"));
    }

    let simdi = crate::library::simdi();
    let conn = db.conn();
    conn.execute(
        "INSERT INTO playlists (name, created_at, updated_at) VALUES (?1, ?2, ?2)",
        params![ad, simdi],
    )?;

    Ok(Playlist {
        id: conn.last_insert_rowid(),
        name: ad.to_string(),
        created_at: simdi,
        updated_at: simdi,
        count: 0,
    })
}

pub fn sil(db: &LibraryDb, id: i64) -> Sonuc<()> {
    // `playlist_items` ON DELETE CASCADE ile gidiyor (şema `PRAGMA
    // foreign_keys = ON` ile açıldı).
    db.conn()
        .execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn ad_degistir(db: &LibraryDb, id: i64, ad: &str) -> Sonuc<()> {
    let ad = ad.trim();
    if ad.is_empty() {
        return Err(Hata::yeni("çalma listesinin adı boş olamaz"));
    }
    db.conn().execute(
        "UPDATE playlists SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, ad, crate::library::simdi()],
    )?;
    Ok(())
}

pub fn hepsi(db: &LibraryDb) -> Sonuc<Vec<Playlist>> {
    let mut sorgu = db.conn().prepare(
        "SELECT p.id, p.name, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM playlist_items i WHERE i.playlist_id = p.id)
         FROM playlists p
         ORDER BY p.name COLLATE NOCASE",
    )?;
    let satirlar = sorgu.query_map([], |s| {
        Ok(Playlist {
            id: s.get(0)?,
            name: s.get(1)?,
            created_at: s.get(2)?,
            updated_at: s.get(3)?,
            count: s.get(4)?,
        })
    })?;
    Ok(satirlar.filter_map(Result::ok).collect())
}

/// Listedeki satırlar, listedeki sırayla.
///
/// `JOIN media`: kayıt silinmişse satır hiç dönmüyor. `playlist_items` zaten
/// CASCADE ile temizleniyor, bu ikinci güvence — veritabanı elle
/// kurcalandığında arayüz boş bir satır çizmesin.
pub fn ogeler(db: &LibraryDb, liste_id: i64) -> Sonuc<Vec<PlaylistItem>> {
    // Sütun listesi ve satır okuyucusu `library/db.rs`ten: burada ikinci bir
    // kopya tutmak, bir sütun eklendiğinde sessizce ayrışan iki liste demek.
    // `i.id` medya sütunlarının SONUNA ekleniyor, başına değil: `satir_to_medya`
    // 0'dan okuyor ve öne almak bütün alanları bir kaydırırdı.
    let sutunlar = db::sutunlar("m");
    let mut sorgu = db.conn().prepare(&format!(
        "SELECT {sutunlar}, i.id
         FROM playlist_items i
         JOIN media m ON m.id = i.media_id
         WHERE i.playlist_id = ?1
         ORDER BY i.position",
    ))?;
    let satirlar = sorgu.query_map(params![liste_id], |s| {
        Ok(PlaylistItem {
            item_id: s.get(db::SUTUN_SAYISI)?,
            media: db::satir_to_medya(s)?,
        })
    })?;
    Ok(satirlar.filter_map(Result::ok).collect())
}

/// Listenin sonuna ekler.
///
/// Aynı kayıt iki kez eklenebiliyor — bilerek. Bir parçayı listede iki yerde
/// istemek geçerli bir istek; engellemek kullanıcıya sebebini açıklaması zor
/// bir kısıt olurdu.
pub fn oge_ekle(db: &LibraryDb, liste_id: i64, medya_id: &str) -> Sonuc<()> {
    let conn = db.conn();
    let sira: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_items WHERE playlist_id = ?1",
        params![liste_id],
        |s| s.get(0),
    )?;
    conn.execute(
        "INSERT INTO playlist_items (playlist_id, media_id, position) VALUES (?1, ?2, ?3)",
        params![liste_id, medya_id, sira],
    )?;
    dokun(conn, liste_id)?;
    Ok(())
}

/// Satır kimliğiyle bir öğeyi siler.
///
/// **Kimlik, sıra değil.** İlk sürüm sıra alıyordu, çünkü `playlist_get_items`
/// [`MediaItem`] döndürüyordu ve onun kimliği medyanın kimliği — satırın
/// değil; arayüzün elinde satır kimliği hiç olmuyordu. Sıra ile silmenin
/// sorunu, listenin arayüzün gördüğü ile silmenin yürüdüğü an arasında
/// değişebilmesi: bir öğe eklendiğinde ya da sıralama sürüklenmişken
/// tıklandığında, kullanıcının işaret ettiği satır ile o sıradaki satır aynı
/// olmuyordu ve **yanlış öğe siliniyordu**. [`PlaylistItem`] artık satır
/// kimliğini de taşıyor, kimlik ise değişmiyor.
///
/// `AND playlist_id`: kimlik listeler arasında benzersiz ama gelen çift
/// tutarsızsa iş başka bir listenin satırını silmek olurdu. Koşul o kaymayı
/// silme değil hata yapıyor.
pub fn oge_sil(db: &LibraryDb, liste_id: i64, oge_id: i64) -> Sonuc<()> {
    let conn = db.conn();

    let silinen = conn.execute(
        "DELETE FROM playlist_items WHERE id = ?1 AND playlist_id = ?2",
        params![oge_id, liste_id],
    )?;
    if silinen == 0 {
        return Err(Hata::yeni("bu öğe listede bulunamadı"));
    }

    sirala(conn, liste_id)?;
    dokun(conn, liste_id)?;
    Ok(())
}

/// Bir öğeyi başka bir konuma taşır (sürükleyerek sıralama).
///
/// Konumlar baştan yazılıyor, tek tek kaydırılmıyor: kaydırma sırasında
/// oluşan ara durumlar (iki satır aynı konumda) veritabanına yansırdı ve
/// tarama arada okursa sıra bozuk görünürdü.
pub fn yeniden_sirala(db: &LibraryDb, liste_id: i64, nereden: usize, nereye: usize) -> Sonuc<()> {
    let conn = db.conn();

    let mut sorgu =
        conn.prepare("SELECT id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position")?;
    let mut kimlikler: Vec<i64> = sorgu
        .query_map(params![liste_id], |s| s.get::<_, i64>(0))?
        .filter_map(Result::ok)
        .collect();
    drop(sorgu);

    if nereden >= kimlikler.len() {
        return Err(Hata::yeni("taşınacak öğe listede yok"));
    }
    let tasinan = kimlikler.remove(nereden);
    // `min`: liste sonuna bırakmak, son indeksin bir fazlasını üretiyor.
    kimlikler.insert(nereye.min(kimlikler.len()), tasinan);

    yaz_siralar(conn, &kimlikler)?;
    dokun(conn, liste_id)?;
    Ok(())
}

/// Konumları 0..n olacak şekilde baştan yazar (silme sonrası boşlukları
/// kapatmak için).
fn sirala(conn: &Connection, liste_id: i64) -> Sonuc<()> {
    let mut sorgu =
        conn.prepare("SELECT id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position")?;
    let kimlikler: Vec<i64> = sorgu
        .query_map(params![liste_id], |s| s.get::<_, i64>(0))?
        .filter_map(Result::ok)
        .collect();
    drop(sorgu);
    yaz_siralar(conn, &kimlikler)
}

/// Konumları tek bir işlemde yazar.
///
/// İşlem ŞART, yalnız hız için değil: satırlar tek tek kesinleşirse araya
/// giren bir okuma sırayı yarı yazılmış hâliyle görüyor — modülün başında
/// "ara durumlar veritabanına yansımasın" diye anlatılan şeyin ta kendisi.
/// Yan faydası, uzun bir listeyi sıralamanın satır başına bir kesinleştirme
/// yerine tek kesinleştirmeye inmesi.
///
/// `SAVEPOINT`, `BEGIN` değil: çağıran dışarıda bir işlem açmış olabilir.
fn yaz_siralar(conn: &Connection, kimlikler: &[i64]) -> Sonuc<()> {
    conn.execute_batch("SAVEPOINT siralar")?;

    let sonuc = (|| -> Sonuc<()> {
        let mut yaz = conn.prepare("UPDATE playlist_items SET position = ?2 WHERE id = ?1")?;
        for (i, id) in kimlikler.iter().enumerate() {
            yaz.execute(params![id, i as i64])?;
        }
        Ok(())
    })();

    match sonuc {
        Ok(()) => {
            conn.execute_batch("RELEASE siralar")?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK TO siralar; RELEASE siralar");
            Err(e)
        }
    }
}

/// `updated_at` damgası. Listeler ada göre sıralandığı için bu alan bugün
/// yalnız bilgi; sıralama seçeneği eklenirse tarih hazır olsun diye tutuluyor.
fn dokun(conn: &Connection, liste_id: i64) -> Sonuc<()> {
    conn.execute(
        "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
        params![liste_id, crate::library::simdi()],
    )?;
    Ok(())
}

#[cfg(test)]
mod testler {
    use super::*;

    use crate::library::db::YeniMedya;

    /// Bellekte bir kütüphane + içine `n` medya kaydı.
    fn db_ile(adlar: &[&str]) -> LibraryDb {
        let db = LibraryDb::bellekte().expect("bellek veritabanı");
        for ad in adlar {
            db.yaz(&YeniMedya {
                id: crate::library::kimlik(ad),
                path: (*ad).into(),
                title: (*ad).into(),
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
            .expect("medya yazma");
        }
        db
    }

    fn basliklar(db: &LibraryDb, liste: i64) -> Vec<String> {
        ogeler(db, liste)
            .expect("öğeler")
            .into_iter()
            .map(|o| o.media.title)
            .collect()
    }

    #[test]
    fn ogeler_satir_kimligini_de_donduruyor() {
        let db = db_ile(&["a.mkv", "b.mkv"]);
        let liste = olustur(&db, "L").expect("liste");
        oge_ekle(&db, liste.id, &crate::library::kimlik("a.mkv")).expect("ekleme");
        oge_ekle(&db, liste.id, &crate::library::kimlik("b.mkv")).expect("ekleme");

        let satirlar = ogeler(&db, liste.id).expect("öğeler");
        assert_eq!(satirlar.len(), 2);
        // Kimlikler DOLU ve birbirinden farklı; medya alanları da yerinde.
        assert_ne!(satirlar[0].item_id, satirlar[1].item_id);
        assert_eq!(satirlar[0].media.title, "a.mkv");
        assert_eq!(satirlar[1].media.title, "b.mkv");
    }

    #[test]
    fn ayni_kayit_iki_kez_eklendiginde_kimlikler_ayriliyor() {
        // Asıl gerekçe bu: `media.id` iki satırı ayırmıyor, `item_id` ayırıyor.
        let db = db_ile(&["a.mkv"]);
        let liste = olustur(&db, "L").expect("liste");
        let medya = crate::library::kimlik("a.mkv");
        oge_ekle(&db, liste.id, &medya).expect("ekleme");
        oge_ekle(&db, liste.id, &medya).expect("ekleme");

        let satirlar = ogeler(&db, liste.id).expect("öğeler");
        assert_eq!(satirlar[0].media.id, satirlar[1].media.id);
        assert_ne!(satirlar[0].item_id, satirlar[1].item_id);

        // İkinci kopyayı silmek birincisini bırakıyor.
        oge_sil(&db, liste.id, satirlar[1].item_id).expect("silme");
        let kalan = ogeler(&db, liste.id).expect("öğeler");
        assert_eq!(kalan.len(), 1);
        assert_eq!(kalan[0].item_id, satirlar[0].item_id);
    }

    #[test]
    fn silme_araya_giren_eklemeden_etkilenmiyor() {
        // Sıra ile silmenin bozulduğu yer: arayüz "b"yi işaret ediyor, ama
        // silme yürümeden önce listenin başına bir öğe giriyor. Sıra 1 artık
        // "b" değil; kimlik hâlâ "b".
        let db = db_ile(&["a.mkv", "b.mkv", "c.mkv"]);
        let liste = olustur(&db, "L").expect("liste");
        for ad in ["a.mkv", "b.mkv"] {
            oge_ekle(&db, liste.id, &crate::library::kimlik(ad)).expect("ekleme");
        }

        let b = ogeler(&db, liste.id).expect("öğeler")[1].item_id;

        oge_ekle(&db, liste.id, &crate::library::kimlik("c.mkv")).expect("ekleme");
        yeniden_sirala(&db, liste.id, 2, 0).expect("sıralama"); // c başa

        oge_sil(&db, liste.id, b).expect("silme");
        assert_eq!(basliklar(&db, liste.id), vec!["c.mkv", "a.mkv"]);
    }

    #[test]
    fn baska_listenin_satiri_silinmiyor() {
        let db = db_ile(&["a.mkv"]);
        let birinci = olustur(&db, "Bir").expect("liste");
        let ikinci = olustur(&db, "İki").expect("liste");
        oge_ekle(&db, birinci.id, &crate::library::kimlik("a.mkv")).expect("ekleme");

        let satir = ogeler(&db, birinci.id).expect("öğeler")[0].item_id;

        // Kimlik doğru ama liste yanlış: silme değil, hata.
        assert!(oge_sil(&db, ikinci.id, satir).is_err());
        assert_eq!(ogeler(&db, birinci.id).expect("öğeler").len(), 1);
    }

    #[test]
    fn silinen_kimlik_ikinci_kez_hata_veriyor() {
        let db = db_ile(&["a.mkv"]);
        let liste = olustur(&db, "L").expect("liste");
        oge_ekle(&db, liste.id, &crate::library::kimlik("a.mkv")).expect("ekleme");
        let satir = ogeler(&db, liste.id).expect("öğeler")[0].item_id;

        oge_sil(&db, liste.id, satir).expect("silme");
        // Çift tıklama: ikincisi sessizce başka bir satırı silmiyor.
        assert!(oge_sil(&db, liste.id, satir).is_err());
    }

    #[test]
    fn silme_sonrasi_konumlar_bosluksuz() {
        let db = db_ile(&["a.mkv", "b.mkv", "c.mkv"]);
        let liste = olustur(&db, "L").expect("liste");
        for ad in ["a.mkv", "b.mkv", "c.mkv"] {
            oge_ekle(&db, liste.id, &crate::library::kimlik(ad)).expect("ekleme");
        }

        let orta = ogeler(&db, liste.id).expect("öğeler")[1].item_id;
        oge_sil(&db, liste.id, orta).expect("silme");

        let konumlar: Vec<i64> = db
            .conn()
            .prepare("SELECT position FROM playlist_items WHERE playlist_id = ?1 ORDER BY position")
            .expect("sorgu")
            .query_map(params![liste.id], |s| s.get(0))
            .expect("satırlar")
            .filter_map(Result::ok)
            .collect();
        assert_eq!(konumlar, vec![0, 1]);
    }
}
