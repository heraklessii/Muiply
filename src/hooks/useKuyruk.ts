/**
 * Kuyruk durumu.
 *
 * Kuyruğun kendisi backend'de (`playlist/kuyruk.rs`) — sebebi orada yazılı:
 * dosya bitince sıradakine geçme kararını mpv'nin olay döngüsü veriyor.
 * Burası yalnızca o kararların yansıması.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "../ipc";
import { OLAY, type QueueSnapshot, type Repeat } from "../ipc/tipler";
import { tauriMi } from "../lib/platform";

const BOS: QueueSnapshot = {
  items: [],
  currentIndex: null,
  repeat: "off",
  shuffle: false,
};

export interface KuyrukKanca {
  kuyruk: QueueSnapshot;
  cal: (indeks: number) => Promise<void>;
  sonraki: () => Promise<void>;
  onceki: () => Promise<void>;
  tekrarDegistir: () => Promise<void>;
  karisikDegistir: () => Promise<void>;
}

/** Tekrar düğmesinin döngüsü: kapalı → hepsi → tek → kapalı. */
const TEKRAR_DONGUSU: Record<Repeat, Repeat> = {
  off: "all",
  all: "one",
  one: "off",
};

export function useKuyruk(onHata: (e: unknown) => void): KuyrukKanca {
  const [kuyruk, setKuyruk] = useState<QueueSnapshot>(BOS);

  useEffect(() => {
    if (!tauriMi()) return;
    ipc.kuyrukOku().then(setKuyruk).catch(onHata);

    const soz = listen<QueueSnapshot>(OLAY.kuyruk, (e) => setKuyruk(e.payload));
    return () => {
      soz.then((birak) => birak());
    };
  }, [onHata]);

  const cal = useCallback(
    async (indeks: number) => {
      try {
        await ipc.kuyrukCal(indeks);
      } catch (e) {
        onHata(e);
      }
    },
    [onHata],
  );

  const sonraki = useCallback(async () => {
    try {
      await ipc.kuyrukSonraki();
    } catch (e) {
      onHata(e);
    }
  }, [onHata]);

  const onceki = useCallback(async () => {
    try {
      await ipc.kuyrukOnceki();
    } catch (e) {
      onHata(e);
    }
  }, [onHata]);

  const tekrarDegistir = useCallback(async () => {
    try {
      await ipc.kuyrukTekrar(TEKRAR_DONGUSU[kuyruk.repeat]);
    } catch (e) {
      onHata(e);
    }
  }, [kuyruk.repeat, onHata]);

  const karisikDegistir = useCallback(async () => {
    try {
      await ipc.kuyrukKarisik(!kuyruk.shuffle);
    } catch (e) {
      onHata(e);
    }
  }, [kuyruk.shuffle, onHata]);

  return { kuyruk, cal, sonraki, onceki, tekrarDegistir, karisikDegistir };
}
