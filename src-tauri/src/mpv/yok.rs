//! Motorsuz derlemenin [`Motor`]'u — her çağrı aynı cümleyle reddediyor.
//!
//! `mpv` özelliği KAPALI olduğunda derleniyor. Amacı libmpv kurulu olmayan
//! bir makinede kütüphane, çalma listesi ve arayüz üstünde çalışabilmek:
//! `npm run tauri:kabuk`.
//!
//! Bu bir taklit DEĞİL. Hiçbir çağrı başarılı gibi davranmıyor, hiçbir sahte
//! süre üretmiyor. Arayüz `PlayerState.engine == false` görüp oynatma
//! denetimlerini kapatıyor; buradaki hatalar yalnızca o kapıyı zorlayan bir
//! çağrı kalırsa görünür.
//!
//! Yüzeyi [`super::gercek::Motor`] ile birebir aynı olmak zorunda: biri
//! değişip diğeri değişmezse özelliği kapatan derleme kırılır.

use tauri::AppHandle;

use crate::hata::{Hata, Sonuc};

use super::MOTOR_YOK;

pub struct Motor;

impl Motor {
    pub const VAR: bool = false;

    pub fn kur(_yuzey_id: Option<i64>) -> Sonuc<Self> {
        // Kurulum BAŞARILI: motorsuz kabuk açılabilmeli. Reddeden şey
        // kurulum değil, oynatma çağrıları.
        Ok(Motor)
    }

    pub fn komut(&self, _ad: &str, _args: &[&str]) -> Sonuc<()> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn ayar_bayrak(&self, _ad: &str, _deger: bool) -> Sonuc<()> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn ayar_sayi(&self, _ad: &str, _deger: i64) -> Sonuc<()> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn ayar_ondalik(&self, _ad: &str, _deger: f64) -> Sonuc<()> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn ayar_metin(&self, _ad: &str, _deger: &str) -> Sonuc<()> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn oku_bayrak(&self, _ad: &str) -> Sonuc<bool> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn oku_sayi(&self, _ad: &str) -> Sonuc<i64> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn oku_ondalik(&self, _ad: &str) -> Sonuc<f64> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn oku_metin(&self, _ad: &str) -> Sonuc<String> {
        Err(Hata::yeni(MOTOR_YOK))
    }

    pub fn olaylari_yayinla(&self, _app: AppHandle) {}
}
