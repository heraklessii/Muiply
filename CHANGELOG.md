# Değişiklikler

Sürümler [semantik sürümlemeyi](https://semver.org/lang/tr/) izliyor. Proje
0.x'te olduğu sürece küçük sürüm (`0.X.0`) kırıcı değişiklik taşıyabilir.

Bir sürümde **ne yapılmadığı** da yazılıyor: kapanmayan bir kabul kriteri,
tamamlanmış bir özellik listesinin arkasında kaybolmamalı.

---

## 0.1.0 — 2026-09-07

İlk yayımlanan sürüm. Yedi fazın tamamı (`docs/Roadmap.md`) yazıldı: mpv
motoru, oynatıcı arayüzü, kütüphane, altyazı, çalma listeleri, cila ve
varsayılan oynatıcı davranışı.

### Eklendi

- **Oynatma** — libmpv üstünde video ve ses; MKV, HEVC, AV1, FLAC ve
  diğerleri ek codec paketi istemeden. Donanım kod çözme `auto-safe`.
- **İki pencere** — video kendi penceresinde (`oynatici`), kütüphane ve
  listeler ikincisinde (`kutuphane`). İkisi de açılışta doğuyor ve gizli
  başlıyor; hangisinin görüneceğine backend karar veriyor. Sonradan
  yaratılamıyorlar: mpv çizeceği pencereyi (`wid`) yalnız başlatılırken alıyor.
- **Kütüphane** — klasör tarama (üç geçişli, değişmemiş dosyayı hiç açmıyor),
  SQLite, ızgara, arama, sıralama. Küçük resim dosya ÇALARKEN alınıyor;
  ızgarada resmi olan kayıtlar açtıkların.
- **Altyazı** — gömülü izler ve dosyanın yanındaki `.srt` / `.ass`
  dosyaları kendiliğinden; dil adları Türkçe, gecikme ayarı var.
- **Çalma listeleri ve kuyruk** — kalıcı listeler, sürükleyerek sıralama,
  tekrar (kapalı / liste / tek) ve karışık.
- **Kaldığı yerden devam** — eşikli (başa/sona 20 sn), varsayılan kapalı.
- **Sistem tepsisi ve medya tuşları** — medya tuşları bir ayara bağlı,
  varsayılan açık; kayıt global olduğu için kapatılabilir olması şart.
- **Varsayılan oynatıcı** — 14 video + 12 ses uzantısı işletim sistemine
  bildiriliyor; çift tıklanan dosya Muiply'de açılıyor, uygulama zaten
  açıksa aynı pencerede (tekil örnek).

### Düzeltildi

- **Geliştirme derlemesi sessizce açılmayan bir uygulama üretiyordu.**
  `libmpv-2.dll`'in elle `target/debug/` içine kopyalanması gerekiyordu.
  Unutulduğunda derleme sorunsuz bitiyor, uygulama hiç açılmıyordu —
  belirti sebebi göstermiyordu. Kopyalamayı artık `build.rs` yapıyor.
- **Tepsiden "Çıkış" kaldığı yeri kaybediyordu.** Pencerenin X'i konumu
  yazıyordu ama `app.exit(0)` pencere olayı üretmediği için tepsi yolu bunu
  atlıyordu. İki çıkış da artık ortak `devam::simdiki_konumu_kaydet`ten
  geçiyor.
- **"Son çalınanlar" bir oturum boyunca hiç güncellenmiyordu.** Bir dosya
  çalınca `play_count` ve `last_played` veritabanına yazılıyor ama olay
  yayınlanmıyordu; kütüphane penceresi yeniden tarama olana kadar eski
  listeyi gösteriyordu. `library://media-updated` artık oynatmada da çıkıyor.
- **Yanındaki altyazı harf farkında ıskalanıyordu.** `Film.mkv` + `film.tr.srt`
  eşleşmiyordu — sessizce, hata da vermeden. Windows ve macOS dosya
  sistemleri harf ayırmadığı için bu eşleşme yaygın. Karşılaştırma artık
  karakter karakter ve harf ayrımı gözetmiyor; `İ`/`i` ve `I`/`ı` bilerek
  eşleşmiyor (yanlış eşleme başka bir filmin altyazısını yüklerdi).

### Bu sürümde YAPILMADI

- **Görüntünün ekrana çizilmesi ölçülmedi.** Ses yolu gerçek dosyayla
  denendi; video yüzeyinin gömülmesi (`mpv/yuzey.rs`) yalnız kod düzeyinde
  incelendi, gözle doğrulanmadı.
- **Linux ve macOS denenmedi.** Paketleme yapılandırmaları duruyor
  (`tauri.linux.conf.json`, `tauri.macos.conf.json`) ama o platformlarda
  hiç derlenmedi.
- **Kurulum paketleri imzasız.** Windows SmartScreen uyarı gösterecek; kod
  imzalama sertifikası yok.
