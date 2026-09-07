//! Tek hata tipi.
//!
//! Tauri komutlarının hata türü `Serialize` olmak zorunda; arayüz de hatayı
//! ekranda gösterebilmek için düz bir metin istiyor. Araya bir hata ağacı
//! koymanın karşılığı yok: burada üretilen her hata sonunda kullanıcının
//! gördüğü bir cümleye dönüşüyor.
//!
//! Metinler TÜRKÇE ve kullanıcıya gösterilebilir olmalı — `unwrap` yerine
//! `?` kullanıldığında ortaya çıkan cümle doğrudan duyuru şeridine düşüyor.

use std::fmt;

#[derive(Debug, serde::Serialize)]
#[serde(transparent)]
pub struct Hata(String);

pub type Sonuc<T> = std::result::Result<T, Hata>;

impl Hata {
    pub fn yeni(mesaj: impl Into<String>) -> Self {
        Hata(mesaj.into())
    }
}

impl fmt::Display for Hata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Hata {}

impl From<String> for Hata {
    fn from(m: String) -> Self {
        Hata(m)
    }
}

impl From<&str> for Hata {
    fn from(m: &str) -> Self {
        Hata(m.to_string())
    }
}

impl From<tauri::Error> for Hata {
    fn from(e: tauri::Error) -> Self {
        Hata(format!("pencere katmanı: {e}"))
    }
}

impl From<rusqlite::Error> for Hata {
    fn from(e: rusqlite::Error) -> Self {
        Hata(format!("kütüphane veritabanı: {e}"))
    }
}

impl From<std::io::Error> for Hata {
    fn from(e: std::io::Error) -> Self {
        Hata(format!("dosya sistemi: {e}"))
    }
}
