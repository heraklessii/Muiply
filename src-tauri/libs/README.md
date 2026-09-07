# libs/

mpv'nin çalışma zamanı kütüphanesi buraya konur. Depoda **yok**: LGPL lisanslı
ve dosya ~115 MB. Nasıl edinileceği `docs/Setup.md` içinde.

Windows'ta burada üç dosya olur:

- `libmpv-2.dll` — çalışma zamanı. Paketlemede uygulamanın **yanına**
  kopyalanıyor (`tauri.paketleme.conf.json`). Adı sürümle değişti: eski
  shinchiro arşivlerinde `mpv-2.dll` idi.
- `mpv.def` — DLL'in dışa aktarma listesi. Arşivde gelmiyor, `dumpbin` ile
  üretiliyor.
- `mpv.lib` — bağlayıcı için içe aktarma kitaplığı, `mpv.def` dosyasından
  `lib` ile üretiliyor. Paketlenmiyor, yalnızca derlemede.

Arşivden çıkan `libmpv.dll.a` MinGW içe aktarma kitaplığı; MSVC onu
kullanamıyor, `mpv.lib` bu yüzden ayrıca üretiliyor. `include/` gerekmiyor,
başlıkları `libmpv2-sys` taşıyor.

Bu klasör `.gitignore`'da; yalnızca bu README izleniyor.
