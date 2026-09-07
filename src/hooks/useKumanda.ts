/**
 * Oynatma kumandası — iki pencerenin de kullandığı ortak eylemler.
 *
 * Muiply iki pencereli (bkz. `src-tauri/src/pencere.rs`) ve ikisi de aynı
 * oynatıcıyı kumanda ediyor: oynatıcı penceresi videonun üstünden,
 * kütüphane penceresi alt çubuktan. Kısayollar, altyazı döngüsü ve
 * gecikme mantığı tek yerde duruyor — iki kopya, ikisinden birinin
 * unutulacağı anlamına gelirdi.
 *
 * Durum burada TUTULMUYOR (gecikme ve tam ekran hariç, ikisi de pencereye
 * özel): oynatıcının gerçeği backend'de ve `useOynatici` onu yansıtıyor.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import * as ipc from "../ipc";
import { tauriMi } from "../lib/platform";
import type { KuyrukKanca } from "./useKuyruk";
import type { OynaticiKanca } from "./useOynatici";

/** Tam ekranda fare durduktan kaç ms sonra çubuk gizleniyor. */
const CUBUK_GIZLEME = 2500;

export interface KumandaSecenekleri {
  oynatici: OynaticiKanca;
  kuyruk: KuyrukKanca;
  bildirHata: (e: unknown) => void;
  /** Bilgi mesajı (hata değil): "bu dosyada altyazı yok" gibi. */
  bildir: (metin: string) => void;
  /**
   * Bu pencerede video var mı.
   *
   * Yalnız oynatıcı penceresinde `true`. Kütüphane penceresinde tam ekran
   * anlamsız — orada `f` tuşu ve tam ekran düğmesi videoyu nerede
   * arayacağını bilen tek yere, oynatıcı penceresine götürüyor.
   */
  videolu: boolean;
}

export interface Kumanda {
  sar: (islem: () => Promise<unknown>) => void;
  tamEkran: boolean;
  cubukGorunur: boolean;
  tamEkranDegistir: (istenen?: boolean) => void;
  altyaziDongu: () => void;
  gecikme: number;
  gecikmeAyarla: (saniye: number) => void;
  /** Kısayol yardımı açık mı (`?` ve tepe barındaki klavye düğmesi). */
  kisayollar: boolean;
  kisayollariDegistir: () => void;
  kisayollariKapat: () => void;
}

export function useKumanda(s: KumandaSecenekleri): Kumanda {
  const { oynatici, kuyruk, bildirHata, bildir, videolu } = s;
  const { durum } = oynatici;

  const [tamEkran, setTamEkran] = useState(false);
  const [cubukGorunur, setCubukGorunur] = useState(true);
  const [gecikme, setGecikme] = useState(0);
  // Yardım kaplaması burada, iki pencerenin ORTAK kancasında: ikisinde de
  // aynı tuş açıyor ve aynı liste görünüyor.
  const [kisayollar, setKisayollar] = useState(false);
  const gizlemeRef = useRef<number | null>(null);

  const sar = useCallback(
    (islem: () => Promise<unknown>) => {
      islem().catch(bildirHata);
    },
    [bildirHata],
  );

  const tamEkranDegistir = useCallback(
    (istenen?: boolean) => {
      if (!tauriMi()) return;

      // Videosuz pencerede tam ekran yapılacak bir şey yok; istek videonun
      // olduğu pencereye devrediliyor.
      if (!videolu) {
        sar(ipc.pencereOynatici);
        return;
      }

      const yeni = istenen ?? !tamEkran;
      getCurrentWindow()
        .setFullscreen(yeni)
        .then(() => {
          setTamEkran(yeni);
          // Tam ekrana girer girmez çubuk görünsün: kullanıcı ilk anda
          // "denetimler nerede" diye sormasın, sonra kendiliğinden gizlensin.
          setCubukGorunur(true);
        })
        .catch(bildirHata);
    },
    [bildirHata, sar, tamEkran, videolu],
  );

  const altyaziDongu = useCallback(() => {
    const altyazilar = oynatici.izler.filter((i) => i.kind === "sub");
    if (altyazilar.length === 0) {
      bildir("Bu dosyada altyazı yok.");
      return;
    }
    const simdiki = altyazilar.findIndex((i) => i.selected);
    // Döngü: kapalı → 1 → 2 → ... → kapalı.
    const sonraki = simdiki + 1 >= altyazilar.length ? null : altyazilar[simdiki + 1].id;
    sar(async () => {
      await ipc.altyaziSec(sonraki);
      oynatici.yenile();
    });
  }, [bildir, oynatici, sar]);

  // Kararlı işlevler: `Kisayollar` `kapat`ı bağımlılığında taşıyor ve her
  // boyamada yenisini üretmek Escape dinleyicisini durmadan söküp yeniden
  // kurardı (aynı kural `Menu.tsx`te de var).
  const kisayollariDegistir = useCallback(() => setKisayollar((a) => !a), []);
  const kisayollariKapat = useCallback(() => setKisayollar(false), []);

  const gecikmeAyarla = useCallback(
    (saniye: number) => {
      // Kayan noktalı toplama 0.30000000000000004 üretiyor; menüde bir
      // ondalık gösterildiği için değeri de oraya yuvarlıyoruz.
      const yuvarlanmis = Math.round(saniye * 10) / 10;
      setGecikme(yuvarlanmis);
      sar(() => ipc.altyaziGecikme(yuvarlanmis));
    },
    [sar],
  );

  // Dosya değişince altyazı gecikmesi mpv'de sıfırlanıyor; arayüzdeki sayı
  // da sıfırlanmalı, yoksa bir öncekinin değeri asılı kalıyor.
  useEffect(() => {
    if (!tauriMi()) return;
    ipc.altyaziGecikmeOku().then(setGecikme).catch(() => setGecikme(0));
  }, [durum.path]);

  // mpv penceresinden gelen kısayollar. Video üstündeyken tuşlar webview'e
  // hiç ulaşmıyor (yüzey webview'in üstünde), bu yüzden mpv onları
  // `script-message` ile geri gönderiyor — bkz. mpv/gercek.rs.
  //
  // Yalnız oynatıcı penceresi dinliyor: olay iki pencereye birden düşüyor
  // ve ikisi de karşılık verse "sonraki parça" bir tıklamada iki kez
  // ilerlerdi.
  useEffect(() => {
    if (!videolu || !oynatici.istek) return;
    switch (oynatici.istek.eylem) {
      case "tam-ekran":
        tamEkranDegistir();
        break;
      case "tam-ekran-kapat":
        if (tamEkran) tamEkranDegistir(false);
        break;
      case "sonraki":
        kuyruk.sonraki();
        break;
      case "onceki":
        kuyruk.onceki();
        break;
      case "altyazi":
        altyaziDongu();
        break;
    }
    // `sayac` bağımlılıkta: aynı eylem art arda gelirse de çalışsın.
  }, [oynatici.istek?.sayac]);

  // Klavye kısayolları (webview tarafı). mpv penceresi odaktayken buraya
  // hiçbir şey gelmiyor; aynı kısayollar orada mpv'nin kendi bağlarıyla var.
  //
  // İki pencerede de çalışıyor ve çakışma yok: tuş yalnız ODAKTAKİ pencereye
  // gidiyor, ikisine birden değil.
  useEffect(() => {
    const tus = (e: KeyboardEvent) => {
      const hedef = e.target as HTMLElement | null;
      // Arama kutusunda "space" yazmak duraklatmasın.
      if (hedef && /^(INPUT|TEXTAREA|SELECT)$/.test(hedef.tagName)) return;
      if (e.ctrlKey || e.altKey || e.metaKey) return;

      const eylemler: Record<string, () => void> = {
        " ": () => sar(ipc.oynaticiDegistir),
        ArrowRight: () => sar(() => ipc.oynaticiGoreliAra(e.shiftKey ? 60 : 5)),
        ArrowLeft: () => sar(() => ipc.oynaticiGoreliAra(e.shiftKey ? -60 : -5)),
        ArrowUp: () => sar(() => ipc.oynaticiSes(Math.min(100, durum.volume + 5))),
        ArrowDown: () => sar(() => ipc.oynaticiSes(Math.max(0, durum.volume - 5))),
        m: () => sar(() => ipc.oynaticiSessiz(!durum.muted)),
        f: () => tamEkranDegistir(),
        s: altyaziDongu,
        n: () => kuyruk.sonraki(),
        p: () => kuyruk.onceki(),
        // Yardım kaplaması açıkken Escape'i kaplamanın kendisi yakalayıp
        // durduruyor (`Kisayollar.tsx`), buraya hiç gelmiyor.
        Escape: () => {
          if (tamEkran) tamEkranDegistir(false);
        },
        '?': kisayollariDegistir,
      };

      // Türkçe klavyede de aynı tuşlar: `key` düzenden bağımsız olmadığı
      // için küçük harfe indiriliyor.
      const eylem = eylemler[e.key] ?? eylemler[e.key.toLowerCase()];
      if (!eylem) return;
      e.preventDefault();
      eylem();
    };

    window.addEventListener("keydown", tus);
    return () => window.removeEventListener("keydown", tus);
  }, [
    altyaziDongu,
    durum.muted,
    durum.volume,
    kisayollariDegistir,
    kuyruk,
    sar,
    tamEkran,
    tamEkranDegistir,
  ]);

  // Tam ekranda çubuğu fare durunca gizle.
  useEffect(() => {
    if (!tamEkran) {
      setCubukGorunur(true);
      return;
    }

    const kipirda = () => {
      setCubukGorunur(true);
      if (gizlemeRef.current) window.clearTimeout(gizlemeRef.current);
      gizlemeRef.current = window.setTimeout(() => setCubukGorunur(false), CUBUK_GIZLEME);
    };

    kipirda();
    window.addEventListener("mousemove", kipirda);
    return () => {
      window.removeEventListener("mousemove", kipirda);
      if (gizlemeRef.current) window.clearTimeout(gizlemeRef.current);
    };
  }, [tamEkran]);

  return {
    sar,
    tamEkran,
    cubukGorunur,
    tamEkranDegistir,
    altyaziDongu,
    gecikme,
    gecikmeAyarla,
    kisayollar,
    kisayollariDegistir,
    kisayollariKapat,
  };
}
