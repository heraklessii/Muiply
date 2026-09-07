/**
 * Kütüphane durumu — kayıtlar, klasörler, tarama ilerlemesi.
 *
 * Sıralama ve tür süzgeci backend'de (SQL), arama arayüzde (`lib/suz.ts`).
 * Bu kanca yalnız birincisini biliyor: süzgeç değişince yeniden sorguluyor,
 * arama değişince hiçbir şey yapmıyor.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "../ipc";
import {
  OLAY,
  type MediaItem,
  type MediaTuru,
  type SiralamaAnahtari,
  type TaramaBittiYuku,
  type TaramaYuku,
} from "../ipc/tipler";
import { tauriMi } from "../lib/platform";

export interface KutuphaneKanca {
  ogeler: MediaItem[];
  klasorler: string[];
  son: MediaItem[];
  tarama: TaramaYuku | null;
  tur: MediaTuru | "hepsi";
  siralama: SiralamaAnahtari;
  setTur: (t: MediaTuru | "hepsi") => void;
  setSiralama: (s: SiralamaAnahtari) => void;
  yenile: () => void;
  klasorEkle: (yol: string) => Promise<void>;
  klasorSil: (yol: string) => Promise<void>;
  tara: () => Promise<void>;
  kayitSil: (id: string) => Promise<void>;
}

/** "Son çalınanlar" listesinin uzunluğu. Yan sütuna sığan kadar. */
const SON_SAYI = 12;

export function useKutuphane(onHata: (e: unknown) => void): KutuphaneKanca {
  const [ogeler, setOgeler] = useState<MediaItem[]>([]);
  const [klasorler, setKlasorler] = useState<string[]>([]);
  const [son, setSon] = useState<MediaItem[]>([]);
  const [tarama, setTarama] = useState<TaramaYuku | null>(null);
  const [tur, setTur] = useState<MediaTuru | "hepsi">("hepsi");
  const [siralama, setSiralama] = useState<SiralamaAnahtari>("added");

  const yenile = useCallback(() => {
    if (!tauriMi()) return;
    ipc
      .kutuphaneMedya({
        mediaType: tur === "hepsi" ? undefined : tur,
        sort: siralama,
      })
      .then(setOgeler)
      .catch(onHata);
    ipc.kutuphaneKlasorler().then(setKlasorler).catch(onHata);
    ipc.kutuphaneSon(SON_SAYI).then(setSon).catch(onHata);
  }, [tur, siralama, onHata]);

  useEffect(yenile, [yenile]);

  useEffect(() => {
    if (!tauriMi()) return;

    const sozler = [
      listen<TaramaYuku>(OLAY.tarama, (e) => setTarama(e.payload)),

      listen<TaramaBittiYuku>(OLAY.taramaBitti, () => {
        setTarama(null);
        yenile();
      }),

      // Tek bir kayıt değişti: küçük resmi yazıldı ya da çalındığı için
      // sayacı arttı. Bütün listeyi yeniden çekmek yerine o satır
      // güncelleniyor — ızgarayı baştan kurmak kaydırma konumunu sıfırlardı.
      listen<{ id: string }>(OLAY.medya, (e) => {
        ipc
          .kutuphaneKayit(e.payload.id)
          .then((yeni) => {
            if (!yeni) return;
            setOgeler((liste) => liste.map((o) => (o.id === yeni.id ? yeni : o)));
          })
          .catch(() => {
            // Kayıt bu arada silinmiş olabilir; listeyi olduğu gibi bırak.
          });

        // "Son çalınanlar" AYRI çekiliyor: satırı yerinde güncellemek
        // yetmiyor, çünkü bu listenin sırasını `last_played` belirliyor ve
        // az önce çalan kayıt başa geçmeli. Sıralamayı arayüzde taklit
        // etmek, SQL'deki sırayı ikinci bir yerde tekrarlamak olurdu.
        ipc.kutuphaneSon(SON_SAYI).then(setSon).catch(() => {});
      }),
    ];

    return () => sozler.forEach((s) => s.then((birak) => birak()));
  }, [yenile]);

  const klasorEkle = useCallback(
    async (yol: string) => {
      try {
        await ipc.kutuphaneKlasorEkle(yol);
        // Tarama arka planda başladı; klasör listesi hemen görünmeli.
        setKlasorler(await ipc.kutuphaneKlasorler());
      } catch (e) {
        onHata(e);
      }
    },
    [onHata],
  );

  const klasorSil = useCallback(
    async (yol: string) => {
      try {
        await ipc.kutuphaneKlasorSil(yol);
        yenile();
      } catch (e) {
        onHata(e);
      }
    },
    [onHata, yenile],
  );

  const tara = useCallback(async () => {
    try {
      await ipc.kutuphaneTara();
    } catch (e) {
      onHata(e);
    }
  }, [onHata]);

  const kayitSil = useCallback(
    async (id: string) => {
      try {
        await ipc.kutuphaneSil(id);
        setOgeler((liste) => liste.filter((o) => o.id !== id));
        setSon((liste) => liste.filter((o) => o.id !== id));
      } catch (e) {
        onHata(e);
      }
    },
    [onHata],
  );

  return {
    ogeler,
    klasorler,
    son,
    tarama,
    tur,
    siralama,
    setTur,
    setSiralama,
    yenile,
    klasorEkle,
    klasorSil,
    tara,
    kayitSil,
  };
}
