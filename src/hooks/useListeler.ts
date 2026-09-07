/**
 * Çalma listeleri.
 *
 * Liste ile kuyruk farklı şeyler (bkz. `src-tauri/src/playlist/mod.rs`):
 * liste diskte duruyor ve düzenleniyor, kuyruk o anki oynatma sırası.
 * Bu kanca yalnız birincisini yönetiyor.
 */

import { useCallback, useEffect, useState } from "react";

import * as ipc from "../ipc";
import type { Playlist, PlaylistItem } from "../ipc/tipler";
import { tauriMi } from "../lib/platform";

export interface ListelerKanca {
  listeler: Playlist[];
  /** Ana alanda açık olan listenin satırları, listedeki sırayla. */
  ogeler: PlaylistItem[];
  acikListe: number | null;
  ac: (id: number | null) => void;
  yenile: () => void;
  olustur: (ad: string) => Promise<Playlist | null>;
  sil: (id: number) => Promise<void>;
  adDegistir: (id: number, ad: string) => Promise<void>;
  ogeEkle: (listeId: number, medyaId: string) => Promise<void>;
  /** İkinci parametre SATIR kimliği (`PlaylistItem.itemId`), sıra değil. */
  ogeSil: (listeId: number, ogeId: number) => Promise<void>;
  sirala: (listeId: number, nereden: number, nereye: number) => Promise<void>;
}

export function useListeler(onHata: (e: unknown) => void): ListelerKanca {
  const [listeler, setListeler] = useState<Playlist[]>([]);
  const [ogeler, setOgeler] = useState<PlaylistItem[]>([]);
  const [acikListe, setAcikListe] = useState<number | null>(null);

  const yenile = useCallback(() => {
    if (!tauriMi()) return;
    ipc.listeHepsi().then(setListeler).catch(onHata);
  }, [onHata]);

  useEffect(yenile, [yenile]);

  const ogeleriTazele = useCallback(
    (id: number) => {
      ipc.listeOgeler(id).then(setOgeler).catch(onHata);
    },
    [onHata],
  );

  // Açık listenin öğeleri ayrı çekiliyor: yan sütundaki adlar her
  // değişiklikte tazeleniyor ama öğeler yalnız o liste açıkken gerekiyor.
  useEffect(() => {
    if (!tauriMi() || acikListe === null) {
      setOgeler([]);
      return;
    }
    ogeleriTazele(acikListe);
  }, [acikListe, ogeleriTazele]);

  const olustur = useCallback(
    async (ad: string) => {
      try {
        const liste = await ipc.listeOlustur(ad);
        yenile();
        return liste;
      } catch (e) {
        onHata(e);
        return null;
      }
    },
    [onHata, yenile],
  );

  const sil = useCallback(
    async (id: number) => {
      try {
        await ipc.listeSil(id);
        // Silinen liste açıksa ana alan artık var olmayan bir listeyi
        // göstermeye devam ederdi; kapatmak zorunda.
        setAcikListe((a) => (a === id ? null : a));
        yenile();
      } catch (e) {
        onHata(e);
      }
    },
    [onHata, yenile],
  );

  const adDegistir = useCallback(
    async (id: number, ad: string) => {
      try {
        await ipc.listeAdDegistir(id, ad);
        yenile();
      } catch (e) {
        onHata(e);
      }
    },
    [onHata, yenile],
  );

  const ogeEkle = useCallback(
    async (listeId: number, medyaId: string) => {
      try {
        await ipc.listeOgeEkle(listeId, medyaId);
        yenile();
        if (acikListe === listeId) ogeleriTazele(listeId);
      } catch (e) {
        onHata(e);
      }
    },
    [acikListe, ogeleriTazele, onHata, yenile],
  );

  const ogeSil = useCallback(
    async (listeId: number, ogeId: number) => {
      try {
        await ipc.listeOgeSil(listeId, ogeId);
        yenile();
        if (acikListe === listeId) ogeleriTazele(listeId);
      } catch (e) {
        onHata(e);
      }
    },
    [acikListe, ogeleriTazele, onHata, yenile],
  );

  const sirala = useCallback(
    async (listeId: number, nereden: number, nereye: number) => {
      if (nereden === nereye) return;

      // İyimser güncelleme: sürükleyip bırakan kullanıcı satırın anında
      // yerine oturmasını bekliyor, IPC gidiş dönüşünü değil. Hata olursa
      // tazeleme gerçeği geri getiriyor.
      setOgeler((liste) => {
        const kopya = [...liste];
        const [tasinan] = kopya.splice(nereden, 1);
        if (tasinan) kopya.splice(nereye, 0, tasinan);
        return kopya;
      });

      try {
        await ipc.listeSirala(listeId, nereden, nereye);
      } catch (e) {
        onHata(e);
        ogeleriTazele(listeId);
      }
    },
    [ogeleriTazele, onHata],
  );

  return {
    listeler,
    ogeler,
    acikListe,
    ac: setAcikListe,
    yenile,
    olustur,
    sil,
    adDegistir,
    ogeEkle,
    ogeSil,
    sirala,
  };
}
