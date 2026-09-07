//! `playlist_*` ve `queue_*` komutları.
//!
//! İki grup bilerek aynı dosyada: aralarındaki tek köprü `playlist_play` ve
//! onu ayrı dosyalara bölmek, "liste çalmak" işinin nerede olduğunu
//! aramaya çevirirdi. Ayrım `playlist/mod.rs` başında anlatılıyor.

use tauri::{Manager, State};

use crate::hata::{Hata, Sonuc};
use crate::library::db::kilit;
use crate::library::LibraryState;
use crate::playlist::kuyruk::{self, QueueItem, QueueSnapshot, QueueState, Repeat};
use crate::playlist::{self, surucu, Playlist, PlaylistItem};

// -- listeler ---------------------------------------------------------------

#[tauri::command]
pub fn playlist_create(durum: State<'_, LibraryState>, name: String) -> Sonuc<Playlist> {
    let db = kilit(&durum)?;
    playlist::olustur(&db, &name)
}

#[tauri::command]
pub fn playlist_delete(durum: State<'_, LibraryState>, id: i64) -> Sonuc<()> {
    let db = kilit(&durum)?;
    playlist::sil(&db, id)
}

#[tauri::command]
pub fn playlist_rename(durum: State<'_, LibraryState>, id: i64, name: String) -> Sonuc<()> {
    let db = kilit(&durum)?;
    playlist::ad_degistir(&db, id, &name)
}

#[tauri::command]
pub fn playlist_get_all(durum: State<'_, LibraryState>) -> Sonuc<Vec<Playlist>> {
    let db = kilit(&durum)?;
    playlist::hepsi(&db)
}

#[tauri::command]
pub fn playlist_get_items(durum: State<'_, LibraryState>, id: i64) -> Sonuc<Vec<PlaylistItem>> {
    let db = kilit(&durum)?;
    playlist::ogeler(&db, id)
}

#[tauri::command]
pub fn playlist_add_item(
    durum: State<'_, LibraryState>,
    playlist_id: i64,
    media_id: String,
) -> Sonuc<()> {
    let db = kilit(&durum)?;
    playlist::oge_ekle(&db, playlist_id, &media_id)
}

/// Öğeyi SATIR kimliğiyle siler; sırayla değil. Gerekçe
/// `playlist/mod.rs::oge_sil` başında: sıra, arayüzün gördüğü an ile silmenin
/// yürüdüğü an arasında kayabiliyordu.
#[tauri::command]
pub fn playlist_remove_item(
    durum: State<'_, LibraryState>,
    playlist_id: i64,
    item_id: i64,
) -> Sonuc<()> {
    let db = kilit(&durum)?;
    playlist::oge_sil(&db, playlist_id, item_id)
}

#[tauri::command]
pub fn playlist_reorder(
    durum: State<'_, LibraryState>,
    playlist_id: i64,
    from: usize,
    to: usize,
) -> Sonuc<()> {
    let db = kilit(&durum)?;
    playlist::yeniden_sirala(&db, playlist_id, from, to)
}

/// Listeyi kuyruğa alır ve çalmaya başlar.
#[tauri::command]
pub fn playlist_play(
    app: tauri::AppHandle,
    durum: State<'_, LibraryState>,
    id: i64,
    start_index: Option<usize>,
) -> Sonuc<()> {
    // Kilit blok içinde bırakılıyor: sonrasında oynatma başlıyor ve o sırada
    // veritabanını tutmak, arka plandaki taramayı bekletmek olurdu.
    let kayitlar = {
        let db = kilit(&durum)?;
        playlist::ogeler(&db, id)?
    };
    let ogeler: Vec<QueueItem> = kayitlar.iter().map(|k| QueueItem::from(&k.media)).collect();

    if ogeler.is_empty() {
        return Err(Hata::yeni("bu çalma listesi boş"));
    }
    kuyruga_koy(&app, ogeler)?;
    surucu::indeksten_oynat(&app, start_index.unwrap_or(0))
}

// -- kuyruk -----------------------------------------------------------------

/// Kuyruğu verilen kütüphane kayıtlarıyla doldurur ve birinden başlar.
///
/// Izgaradan bir karta çift tıklandığında arayüz bunu çağırıyor: görünen
/// bütün kayıtları kuyruğa koyup tıklananın indeksini veriyor. Böylece
/// "tıkladığım şey çalsın, sonra listedekiler devam etsin" davranışı,
/// arayüzde ayrı bir kural yazmadan çıkıyor.
///
/// `start_index` arayüzün GÖNDERDİĞİ listeye ait; kuyruğa giren listeye
/// değil. Kaybolmuş kayıtlar atlandığı için ikisi ayrışabiliyor ve indeksi
/// olduğu gibi kullanmak, kullanıcının tıkladığından BAŞKA bir dosyayı
/// çalmak demekti. Kaydırma [`kuyruk::baslangici_esle`] ile düzeltiliyor.
#[tauri::command]
pub fn queue_set(
    app: tauri::AppHandle,
    durum: State<'_, LibraryState>,
    media_ids: Vec<String>,
    start_index: Option<usize>,
) -> Sonuc<()> {
    let db = kilit(&durum)?;
    let mut ogeler = Vec::with_capacity(media_ids.len());
    let mut bulunan = Vec::with_capacity(media_ids.len());
    for (i, id) in media_ids.iter().enumerate() {
        // Kaybolmuş kayıt kuyruğu bozmuyor, atlanıyor: tarama ile arayüzün
        // listesi arasında saniyeler geçmiş olabilir.
        if let Some(m) = db.medya_bir(id)? {
            ogeler.push(QueueItem::from(&m));
            bulunan.push(i);
        }
    }
    drop(db);

    if ogeler.is_empty() {
        return Err(Hata::yeni("çalınacak kayıt bulunamadı"));
    }
    let baslangic = kuyruk::baslangici_esle(&bulunan, start_index.unwrap_or(0));
    kuyruga_koy(&app, ogeler)?;
    surucu::indeksten_oynat(&app, baslangic)
}

#[tauri::command]
pub fn queue_get(durum: State<'_, QueueState>) -> Sonuc<QueueSnapshot> {
    Ok(durum
        .0
        .lock()
        .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?
        .anlik())
}

#[tauri::command]
pub fn queue_play_at(app: tauri::AppHandle, index: usize) -> Sonuc<()> {
    surucu::indeksten_oynat(&app, index)
}

/// Kullanıcı "sonraki" dedi. Liste bittiyse `false` dönüyor — arayüz bunu
/// hata olarak değil, "yapacak bir şey yoktu" olarak gösteriyor.
#[tauri::command]
pub fn queue_next(app: tauri::AppHandle) -> Sonuc<bool> {
    Ok(surucu::ilerle(&app, false)?.is_some())
}

#[tauri::command]
pub fn queue_prev(app: tauri::AppHandle) -> Sonuc<bool> {
    Ok(surucu::gerile(&app)?.is_some())
}

#[tauri::command]
pub fn queue_set_repeat(app: tauri::AppHandle, repeat: Repeat) -> Sonuc<()> {
    surucu::repeat_ayarla(&app, repeat)
}

#[tauri::command]
pub fn queue_set_shuffle(app: tauri::AppHandle, shuffle: bool) -> Sonuc<()> {
    surucu::shuffle_ayarla(&app, shuffle)
}

pub(crate) fn kuyruga_koy(app: &tauri::AppHandle, ogeler: Vec<QueueItem>) -> Sonuc<()> {
    let durum = app
        .try_state::<QueueState>()
        .ok_or_else(|| Hata::yeni("kuyruk hazır değil"))?;
    durum
        .0
        .lock()
        .map_err(|_| Hata::yeni("kuyruk kilidi bozuldu"))?
        .yerlestir(ogeler);
    Ok(())
}
