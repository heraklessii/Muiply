//! Oynatma işleri — mpv özelliklerinin üstündeki ince anlam katmanı.
//!
//! Buradaki her fonksiyon bir kullanıcı eyleminin karşılığı. Komut dosyaları
//! (`commands/player.rs`) bunları çağırıyor, mpv'ye doğrudan dokunmuyor:
//! "duraklat" iki yerde iki farklı şey yapmasın diye.

use std::path::Path;

use tauri::{AppHandle, Emitter, Manager};

use crate::hata::{Hata, Sonuc};

use super::{izleri_oku, saniye_metni, MpvState, Track};

/// Ses seviyesinin üst sınırı.
///
/// mpv 100'ün üstüne çıkabiliyor (yazılımsal yükseltme) ama bozulma
/// başlıyor. Sürgü 0-100; sınır burada çünkü hem komut hem kısayol aynı
/// kapıdan geçmeli.
pub const SES_UST: i64 = 100;

/// Dosyayı açar ve çalmaya başlar.
///
/// Yolun var olduğu ÖNCE kontrol ediliyor: mpv olmayan bir dosya için
/// `loadfile`i kabul edip sonra sessizce `EndFile(error)` üretiyor, yani
/// kullanıcı hiçbir açıklama görmeden boş bir sahneyle kalıyor.
pub fn ac(durum: &MpvState, yol: &str) -> Sonuc<()> {
    if !Path::new(yol).is_file() {
        return Err(Hata::yeni(format!("dosya bulunamadı: {yol}")));
    }

    durum.motor.komut("loadfile", &[yol, "replace"])?;
    // Bir önceki dosya duraklatılmış bitmiş olabilir; yeni dosya çalmalı.
    durum.motor.ayar_bayrak("pause", false)?;

    durum.guncelle(|d| {
        d.path = Some(yol.to_string());
        d.position = 0.0;
        d.paused = false;
    });
    Ok(())
}

pub fn oynat(durum: &MpvState) -> Sonuc<()> {
    durum.motor.ayar_bayrak("pause", false)
}

pub fn duraklat(durum: &MpvState) -> Sonuc<()> {
    durum.motor.ayar_bayrak("pause", true)
}

pub fn duraklat_degistir(durum: &MpvState) -> Sonuc<()> {
    durum.motor.komut("cycle", &["pause"])
}

/// Mutlak konuma atlar.
pub fn ara(durum: &MpvState, konum: f64) -> Sonuc<()> {
    let konum = konum.max(0.0);
    durum
        .motor
        .komut("seek", &[&saniye_metni(konum), "absolute"])
}

/// Bulunulan yere göre atlar (kısayollar ve ileri/geri düğmeleri).
pub fn goreli_ara(durum: &MpvState, delta: f64) -> Sonuc<()> {
    durum
        .motor
        .komut("seek", &[&saniye_metni(delta), "relative"])
}

pub fn ses_ayarla(durum: &MpvState, ses: i64) -> Sonuc<()> {
    durum.motor.ayar_sayi("volume", ses.clamp(0, SES_UST))
}

pub fn sessiz_ayarla(durum: &MpvState, sessiz: bool) -> Sonuc<()> {
    durum.motor.ayar_bayrak("mute", sessiz)
}

/// Oynatmayı bitirir ve sahneyi boşaltır.
pub fn dur(app: &AppHandle, durum: &MpvState) -> Sonuc<()> {
    durum.motor.komut("stop", &[])?;
    bosalt(app);
    Ok(())
}

/// Oynatıcıyı boş duruma çeker ve arayüze haber verir.
///
/// İki yerden çağrılıyor: `player_stop` ve kuyruğun sonu
/// (`playlist/surucu.rs`). İkisinde de mpv'de artık yüklü dosya YOK —
/// `keep-open` kapalı, mpv dosya bitince onu boşaltıyor.
///
/// Durumu temizlemek şart, çünkü arayüz onun yansıması: temizlenmediğinde
/// bitmiş bir video duruyor gibi görünüyor, oynat düğmesi hiçbir şey
/// yapmıyor ve sürgüye basmak boş mpv'ye `seek` gönderip kullanıcıya hata
/// şeridi çıkarıyor.
///
/// Kuyruk SİLİNMİYOR — kullanıcı listeyi yeniden başlatabilsin.
pub fn bosalt(app: &AppHandle) {
    if let Some(durum) = app.try_state::<MpvState>() {
        durum.guncelle(|d| {
            d.playing = false;
            d.paused = true;
            d.position = 0.0;
            d.duration = 0.0;
            d.path = None;
            d.title = None;
            d.media_type = None;
        });
    }
    let _ = app.emit(super::OLAY_TEMIZ, ());
}

/// Oynatma hızı. 0.25 altı ve 4 üstü mpv'de sesi kullanılamaz hâle getiriyor.
pub fn hiz_ayarla(durum: &MpvState, hiz: f64) -> Sonuc<()> {
    if !(0.25..=4.0).contains(&hiz) {
        return Err(Hata::yeni("oynatma hızı 0.25 ile 4 arasında olmalı"));
    }
    durum.motor.ayar_ondalik("speed", hiz)
}

pub fn izler(durum: &MpvState) -> Sonuc<Vec<Track>> {
    izleri_oku(&durum.motor)
}
