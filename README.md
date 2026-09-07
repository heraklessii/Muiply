# Muiply

Bilgisayarındaki videoları ve müziği çalan masaüstü oynatıcı. Kod çözme işini
**libmpv** yapıyor: MKV, HEVC, AV1, FLAC ve diğerleri ek codec paketi
istemeden açılıyor. Klasörlerini tarayan bir kütüphanesi ve çalma listeleri
var.

[![CI](https://github.com/heraklessii/Muiply/actions/workflows/ci.yml/badge.svg)](https://github.com/heraklessii/Muiply/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/durum-%C3%B6n%20s%C3%BCr%C3%BCm-orange.svg)](docs/Roadmap.md)

> ⚠️ **Bu bir ön sürüm.** Yedi fazın tamamı yazıldı ve 95 otomatik testle
> doğrulanıyor, ama uygulama sahada geniş çapta denenmedi. Neyin
> denenmediği açıkça yazılı: [`CHANGELOG.md`](CHANGELOG.md).

**İndir:** [Sürümler](https://github.com/heraklessii/Muiply/releases/latest) ·
**Tanıtım:** <https://heraklessii.github.io/Muiply/>

Mui portföyünün bir parçası — Muiget, Muivly, Muifly, Muiwatch, MuiLabs ile
aynı tasarım dilini konuşuyor.

## Ne yapıyor

- **Ayrı oynatıcı penceresi** — video kendi penceresinde: yan sütun yok, menü
  yok. Kütüphane ve listeler ikinci pencerede, istendiğinde
- **Oynatma** — video ve ses, donanım hızlandırmalı; hız, ses, arama, tam ekran
- **Kütüphane** — klasörleri tarayıp SQLite'a yazıyor; ızgara, arama, sıralama
- **Altyazı** — gömülü izler ve dosyanın yanındaki `.srt` / `.ass` dosyaları
  kendiliğinden; dil adları Türkçe, gecikme ayarı var
- **Çalma listeleri** — kalıcı, sürükleyerek sıralanabilir
- **Kuyruk** — tekrar (kapalı / liste / tek) ve karışık
- **Kaldığı yerden devam** — yarım bırakılan dosya kalınan yerden (kapatılabilir)
- **Sistem tepsisi** — oynat/duraklat, önceki, sonraki; ipucunda çalan dosya
- **Medya tuşları** — klavyenin oynat/duraklat, ileri, geri tuşları
  (kapatılabilir: kayıt global, açıkken tuş başka oynatıcıya ulaşmıyor)
- **Varsayılan oynatıcı olabiliyor** — kurulum video ve ses uzantılarını
  işletim sistemine bildiriyor; çift tıklanan dosya Muiply'de açılıyor,
  uygulama zaten açıksa aynı pencerede (`docs/Setup.md`)

## Ne yapmıyor (bilerek)

Ağ akışı yok, YouTube yok, internetten meta veri çekmiyor, video filtresi yok,
kütüphaneden kaldırmak dosyayı **silmiyor**, pencereyi kapatmak tepsiye
inmiyor. Saf Wayland'da video ayrı bir pencerede açılıyor — mpv'nin sınırı,
XWayland'da gömülü. Gerekçeler `docs/Roadmap.md` sonunda.

## Kurulum

```bash
npm install
npm run tauri:kabuk     # libmpv olmadan: kütüphane + listeler + arayüz
```

Gerçek oynatıcı için libmpv gerekiyor. Windows'ta `mpv-dev` paketinden
`libmpv-2.dll` yanına `mpv.lib` üretmek gerekiyor; Linux'ta `libmpv-dev`, macOS'ta
`brew install mpv`. Adım adım `docs/Setup.md`.

```bash
npm run tauri dev       # motorlu
npm run tauri:paket        # Windows: nsis + msi
npm run tauri:paket:unix   # Linux: deb · macOS: app + dmg
```

## Neden iki derleme

libmpv **bağlama zamanında** bağlanıyor: kitaplık yoksa uygulama hiç
açılmıyor, çalışma zamanında yakalanabilen bir hata değil. Bu yüzden "libmpv
var mı" sorusu derleme zamanında soruluyor — motor bir Cargo özelliği
(`default = ["mpv"]`).

Motorsuz derleme bir taklit değil: hiçbir çağrı başarılı gibi davranmıyor,
sahte süre üretmiyor. Arayüz oynatma denetimlerini kapatıp sebebini yazıyor.

## Belgeler

| Dosya | Konu |
|---|---|
| `docs/Architecture.md` | Katmanlama, veri akışı, durumlar |
| `docs/Mpv_Integration.md` | Motor, video yüzeyi, olay döngüsü, sonda |
| `docs/Library.md` | SQLite şeması, tarama, küçük resimler |
| `docs/Subtitles.md` | Altyazı bulma, seçme, gecikme |
| `docs/Frontend.md` | Tasarım dili, kancalar, kısayollar |
| `docs/IPC.md` | Komut ve olay sözleşmesi |
| `docs/Setup.md` | Kurulum ve komutlar |
| `docs/Roadmap.md` | Fazlar ve kapsam dışı kararlar |

## Testler

```bash
npm test                                     # arayüz (vitest)
cd src-tauri && cargo test --no-default-features   # Rust
```

Rust testleri motorsuz çalıştırılıyor: test ikilisini **bağlamak** libmpv
istiyor, testlerin kendisi (kuyruk gezinme, altyazı eşleştirme, karıştırma)
mpv'ye hiç dokunmuyor.

## Lisans

Apache-2.0. libmpv LGPL-2.1+ ile ayrı bir kitaplık olarak dinamik bağlanıyor
ve depoda tutulmuyor.
