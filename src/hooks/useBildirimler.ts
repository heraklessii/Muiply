/**
 * Bildirim yığını.
 *
 * Eskiden tek bir şerit vardı ve aynı anda tek şey gösterebiliyordu: tarama
 * sürerken çıkan bir hata, ilerlemenin yerini alıyordu. Yığın ikisini de
 * gösteriyor.
 *
 * Üç karar burada:
 *
 * 1. **Kendiliğinden kapanma.** Bilgi mesajı dört saniye, hata dokuz. Hata
 *    daha uzun çünkü okunması gereken bir cümle; bilgi ("Klasör eklendi")
 *    zaten olan bitenin tekrarı.
 * 2. **Aynı mesaj tekrar gelirse yenisi açılmıyor**, var olanın süresi
 *    yenileniyor. Kuyruk ilerlerken art arda gelen aynı hata, ekranı üst
 *    üste on şeritle doldurmasın.
 * 3. **Yığın üç ile sınırlı.** Daha fazlası ana alanı kaplıyor ve dördüncü
 *    mesaj zaten en eskisini okunmadan itiyor; sınırı koymak en azından
 *    hangisinin gideceğini belirli kılıyor.
 *
 * `bildir` / `bildirHata` KARARLI işlevler: kancalar bunları bağımlılık
 * olarak alıyor ve her boyamada yenisini üretmek olay dinleyicilerini
 * durmadan söküp yeniden kurardı.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import { hataMetni } from '../ipc';

export type BildirimTuru = 'bilgi' | 'uyari' | 'hata';

export interface Bildirim {
  id: number;
  tur: BildirimTuru;
  metin: string;
}

/** Ekranda aynı anda duran en fazla bildirim. */
const EN_FAZLA = 3;

/** Tür başına kendiliğinden kapanma süresi (ms). */
const SURELER: Record<BildirimTuru, number> = {
  bilgi: 4000,
  uyari: 7000,
  hata: 9000,
};

export interface BildirimKanca {
  bildirimler: Bildirim[];
  /** Bilgi mesajı — bir eylemin olduğunu doğrulayan cümle. */
  bildir: (metin: string) => void;
  bildirUyari: (metin: string) => void;
  /** Herhangi bir hata değerini okunur bir cümleye çevirip gösterir. */
  bildirHata: (e: unknown) => void;
  kapat: (id: number) => void;
}

export function useBildirimler(): BildirimKanca {
  const [bildirimler, setBildirimler] = useState<Bildirim[]>([]);

  // Artan kimlik. `Date.now()` değil: aynı milisaniyede iki bildirim
  // eklenebiliyor ve React aynı anahtarlı iki satırı doğru çizmiyor.
  const sonrakiId = useRef(1);
  // Kapanma zamanlayıcıları; bileşen sökülürken hepsi iptal ediliyor.
  const zamanlayicilar = useRef(new Map<number, number>());

  /**
   * Yığının asıl kopyası. `bildirimler` durumu onun yansıması.
   *
   * Gerekli çünkü listeye eklemenin YAN ETKİLERİ var: kimlik üretmek,
   * zamanlayıcı kurmak, taşan satırınkini iptal etmek. Bunlar bir
   * `setState` güncelleyicisinin içinde duruyordu, oysa güncelleyici SAF
   * olmak zorunda — React onu (StrictMode'da her zaman, eşzamanlı kipte
   * gerektiğinde) birden çok kez çağırıyor ve her çağrı bir kimlik daha
   * harcayıp sahipsiz bir zamanlayıcı daha bırakıyordu. Karar burada, yayın
   * `setBildirimler`de.
   */
  const yigin = useRef<Bildirim[]>([]);

  const yayinla = useCallback((yeni: Bildirim[]) => {
    yigin.current = yeni;
    setBildirimler(yeni);
  }, []);

  const kapat = useCallback(
    (id: number) => {
      const z = zamanlayicilar.current.get(id);
      if (z !== undefined) {
        window.clearTimeout(z);
        zamanlayicilar.current.delete(id);
      }
      yayinla(yigin.current.filter((b) => b.id !== id));
    },
    [yayinla],
  );

  /** Kapanma zamanlayıcısını (yeniden) kurar. */
  const zamanla = useCallback(
    (id: number, tur: BildirimTuru) => {
      const eski = zamanlayicilar.current.get(id);
      if (eski !== undefined) window.clearTimeout(eski);
      zamanlayicilar.current.set(id, window.setTimeout(() => kapat(id), SURELER[tur]));
    },
    [kapat],
  );

  const ekle = useCallback(
    (tur: BildirimTuru, metin: string) => {
      const temiz = metin.trim();
      if (!temiz) return;

      // Aynı mesaj zaten duruyorsa yenisi açılmıyor, süresi yenileniyor.
      const ayni = yigin.current.find((b) => b.tur === tur && b.metin === temiz);
      if (ayni) {
        zamanla(ayni.id, tur);
        return;
      }

      const id = sonrakiId.current++;
      zamanla(id, tur);

      const yeni = [...yigin.current, { id, tur, metin: temiz }];
      // Sınırı aşanlar en ESKİden atılıyor; yenisi her zaman görünüyor.
      while (yeni.length > EN_FAZLA) {
        const dusen = yeni.shift();
        if (dusen) {
          const z = zamanlayicilar.current.get(dusen.id);
          if (z !== undefined) window.clearTimeout(z);
          zamanlayicilar.current.delete(dusen.id);
        }
      }
      yayinla(yeni);
    },
    [yayinla, zamanla],
  );

  const bildir = useCallback((metin: string) => ekle('bilgi', metin), [ekle]);
  const bildirUyari = useCallback((metin: string) => ekle('uyari', metin), [ekle]);
  const bildirHata = useCallback((e: unknown) => ekle('hata', hataMetni(e)), [ekle]);

  // Sökülürken bekleyen zamanlayıcılar iptal: kapanmış bir bileşende
  // `setState` çağırmak React'te uyarı, en kötüsü sızıntı.
  useEffect(() => {
    const kayit = zamanlayicilar.current;
    return () => {
      kayit.forEach((z) => window.clearTimeout(z));
      kayit.clear();
    };
  }, []);

  return { bildirimler, bildir, bildirUyari, bildirHata, kapat };
}
