/**
 * Video yüzeyini sahnenin dikdörtgenine oturtan kanca.
 *
 * mpv ekrana bir <video> etiketiyle değil, webview'in ÜSTÜNDE duran native
 * bir alt pencereye çiziyor (gerekçe `src-tauri/src/mpv/yuzey.rs` başında).
 * O pencerenin nerede duracağını yalnız arayüz biliyor — düzeni CSS
 * kuruyor. Bu kanca ölçüyü alıp backend'e söylüyor.
 *
 * Ölçü CSS pikseli olarak gidiyor; ekran ölçeğine çevirme Rust tarafında,
 * çünkü doğru çarpanı pencerenin kendisi biliyor (karışık DPI'lı iki ekran
 * arasında `devicePixelRatio` yanılabiliyor).
 */

import { useEffect, useRef, type RefObject } from "react";

import * as ipc from "../ipc";
import { tauriMi } from "../lib/platform";

/**
 * @param ref      Sahnenin yüzey kutusu.
 * @param gorunur  Video şu an görünmeli mi (ses çalarken hayır).
 * @param altSinir Yüzeyin geçmemesi gereken viewport y'si — açık bir menünün
 *                 tepesi (`useMenuUstSiniri`). Çubuktaki menüler yukarı,
 *                 yani video alanına açılıyor; sınır verilmezse mpv onların
 *                 üstünü örtüyor ve menüler video oynarken görünmüyor.
 */
export function useSahneOlcu(
  ref: RefObject<HTMLElement | null>,
  gorunur: boolean,
  altSinir?: number | null,
) {
  // Sınır bir REF'te tutuluyor, bağımlılıkta değil: bağımlılığa koymak menü
  // her açılıp kapandığında etkiyi söküp yeniden kurmak, yani yüzeyi bir kez
  // gizleyip yeniden göstermek demekti — video gözle görülür biçimde
  // kırpıyordu.
  const sinirRef = useRef(altSinir);
  sinirRef.current = altSinir;

  // Ölçüyü dışarıdan (sınır değişince) de tetikleyebilmek için.
  const bildirRef = useRef<() => void>(() => {});

  useEffect(() => {
    if (!tauriMi()) return;

    // Görünmezken ölçü göndermenin anlamı yok; gizlemek yeterli ve gerekli,
    // yoksa siyah dikdörtgen kütüphane ızgarasının üstünde asılı kalıyor.
    if (!gorunur) {
      ipc.oynaticiVideoGorunur(false).catch(() => {});
      return;
    }

    const kutu = ref.current;
    if (!kutu) return;

    let sonKare = 0;
    // Yüzey şu an gizli mi. Her ölçüde "göster" göndermemek için: ölçü
    // saniyede onlarca kez gidebiliyor (yeniden boyutlama), görünürlük ise
    // ayda bir değişiyor.
    let gizli = false;

    const bildir = () => {
      // Aynı karede birden çok kez tetiklenebiliyor (yeniden boyutlama +
      // kaydırma). rAF ile kare başına tek çağrıya indiriliyor: her biri
      // bir IPC gidişi ve bir SetWindowPos.
      if (sonKare) cancelAnimationFrame(sonKare);
      sonKare = requestAnimationFrame(() => {
        const r = kutu.getBoundingClientRect();
        // Sıfır genişlik: bileşen bir an gizlenmiş (görünüm değişimi).
        // Yüzeyi 0 boyutlu yapmak mpv'de bozuk çıktı üretebiliyor.
        if (r.width < 1) return;

        // Açık menü varsa yüzey onun tepesinde bitiyor: video küçülüyor ama
        // görünmeye devam ediyor. Menü açıkken videoyu tamamen karartmak,
        // hız değiştirirken sonucu görememek demekti.
        const sinir = sinirRef.current;
        const alt = sinir != null ? Math.min(r.bottom, sinir) : r.bottom;
        const boy = alt - r.top;

        // Menü sahnenin tamamını kaplıyor (küçük pencere): gösterilecek
        // video kalmadı.
        if (boy < 1) {
          if (!gizli) {
            gizli = true;
            ipc.oynaticiVideoGorunur(false).catch(() => {});
          }
          return;
        }

        ipc.oynaticiVideoAlani(r.left, r.top, r.width, boy).catch(() => {});
        if (gizli) {
          gizli = false;
          ipc.oynaticiVideoGorunur(true).catch(() => {});
        }
      });
    };

    bildirRef.current = bildir;
    bildir();
    ipc.oynaticiVideoGorunur(true).catch(() => {});

    const gozlemci = new ResizeObserver(bildir);
    gozlemci.observe(kutu);
    // Pencere taşınınca boyut değişmiyor ama ekran ölçeği değişebiliyor;
    // kaydırma da kutunun viewport'taki yerini değiştiriyor.
    window.addEventListener("resize", bildir);
    window.addEventListener("scroll", bildir, true);

    return () => {
      if (sonKare) cancelAnimationFrame(sonKare);
      bildirRef.current = () => {};
      gozlemci.disconnect();
      window.removeEventListener("resize", bildir);
      window.removeEventListener("scroll", bildir, true);

      // YÜZEYİ GİZLEMEK BURADA ŞART. Sahne bileşeni söküldüğünde (pencere
      // kapandı, dosya bitti) kanca bir daha `gorunur=false` ile
      // çalışmıyor — temizlik tek haber verme fırsatı. Gizlenmezse mpv'nin
      // native penceresi webview'in üstünde asılı kalıyor: altındaki arayüz
      // görünmüyor ve o alandaki tıklamalar webview'e hiç ulaşmıyor.
      ipc.oynaticiVideoGorunur(false).catch(() => {});
    };
  }, [ref, gorunur]);

  // Menü açılıp kapandığında yalnız ÖLÇÜ yenileniyor; yüzey sökülmüyor.
  useEffect(() => {
    bildirRef.current();
  }, [altSinir]);
}
