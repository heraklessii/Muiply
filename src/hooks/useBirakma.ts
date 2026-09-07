/**
 * Pencereye sürükle-bırak.
 *
 * İki pencere de dosya kabul ediyor: oynatıcıya bırakılan çalıyor,
 * kütüphaneye bırakılan klasörse taranıyor. Karar arayüzde DEĞİL:
 * "klasör mü dosya mı" sorusu dosya sistemine bakmayı gerektiriyor ve
 * arayüzün dosya sistemine erişimi yok (`capabilities/default.json`).
 * Backend'de `acilis::yollari_ac`.
 */

import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "../ipc";
import { tauriMi } from "../lib/platform";

/**
 * @param bildirHata Hata kanalı.
 * @param sonra      Bırakma bittikten sonra çalışan tazeleme (kütüphane
 *                   penceresi listelerini yeniliyor; oynatıcının yenileyecek
 *                   bir şeyi yok).
 * @returns Şu an pencerenin üstünde bir şey sürükleniyor mu.
 */
export function useBirakma(bildirHata: (e: unknown) => void, sonra?: () => void): boolean {
  const [birakiliyor, setBirakiliyor] = useState(false);

  // Ref: `sonra` her boyamada yeniden üretilebiliyor ve bağımlılığa koymak
  // dinleyicileri durmadan sökerdi. Ref ile dinleyici bir kez kuruluyor ama
  // her zaman EN TAZE işlevi çağırıyor.
  const sonraRef = useRef(sonra);
  sonraRef.current = sonra;

  useEffect(() => {
    if (!tauriMi()) return;

    const sozler = [
      listen("tauri://drag-enter", () => setBirakiliyor(true)),
      listen("tauri://drag-leave", () => setBirakiliyor(false)),
      listen<{ paths: string[] }>("tauri://drag-drop", (e) => {
        setBirakiliyor(false);
        const yollar = e.payload.paths;
        if (yollar.length === 0) return;
        ipc
          .yollariAc(yollar)
          .then(() => sonraRef.current?.())
          .catch(bildirHata);
      }),
    ];

    return () => sozler.forEach((s) => s.then((birak) => birak()));
  }, [bildirHata]);

  return birakiliyor;
}
