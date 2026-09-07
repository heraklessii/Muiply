/**
 * Oynatıcı durumu — `player://` olaylarının React karşılığı.
 *
 * Durumun TEK sahibi backend. Bu kanca onun bir yansıması: hiçbir yerde
 * "duraklattım, o hâlde duraklamıştır" varsayımı yok. Bir düğmeye
 * basıldığında komut gönderiliyor ve durum, mpv'nin olayı geldiğinde
 * değişiyor. Bu bir gecikme (birkaç ms) ama karşılığında arayüz ile motorun
 * ayrışması imkânsız hâle geliyor — mpv penceresinden gelen kısayollar,
 * dosya bitişi ve kuyruk geçişleri de aynı kanaldan akıyor.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "../ipc";
import {
  OLAY,
  type BittiYuku,
  type DuraklatYuku,
  type FileInfo,
  type HizYuku,
  type IstekEylemi,
  type KonumYuku,
  type SureYuku,
  type PlayerState,
  type SesYuku,
  type Track,
} from "../ipc/tipler";
import { tauriMi } from "../lib/platform";

/**
 * Konum güncellemelerinin en sık yenilenme aralığı (ms).
 *
 * mpv `time-pos` özelliğini kare hızında bildiriyor — saniyede 60 olay.
 * Her birini React durumuna yazmak, sürgüyü saniyede 60 kez yeniden
 * çizdirmek demek; gözle görülen kazanç yok, ölçülen maliyet var.
 * 200 ms'de sürgü saniyede beş kez ilerliyor, süre yazısı zaten saniyelik.
 */
const KONUM_ARALIGI = 200;

const BASLANGIC: PlayerState = {
  playing: false,
  paused: true,
  position: 0,
  duration: 0,
  volume: 100,
  muted: false,
  rate: 1,
  path: null,
  title: null,
  mediaType: null,
  engine: true,
};

export interface OynaticiKanca {
  durum: PlayerState;
  izler: Track[];
  /** mpv penceresinden gelen kısayol; App bunu ele alıyor. */
  istek: { eylem: IstekEylemi; sayac: number } | null;
  yenile: () => void;
}

export function useOynatici(onHata: (e: unknown) => void): OynaticiKanca {
  const [durum, setDurum] = useState<PlayerState>(BASLANGIC);
  const [izler, setIzler] = useState<Track[]>([]);
  const [istek, setIstek] = useState<{ eylem: IstekEylemi; sayac: number } | null>(null);

  // Kısıtlama için: son gelen konum ve bekleyen zamanlayıcı.
  const konumRef = useRef(0);
  const zamanlayiciRef = useRef<number | null>(null);

  const yenile = useCallback(() => {
    if (!tauriMi()) return;
    ipc
      .oynaticiDurum()
      .then(setDurum)
      .catch(onHata);
    ipc
      .oynaticiIzler()
      .then(setIzler)
      // İz listesi dosya yokken hata dönüyor; bu bir arıza değil.
      .catch(() => setIzler([]));
  }, [onHata]);

  useEffect(() => {
    if (!tauriMi()) return;
    yenile();

    // Pencere odağı geri geldiğinde durumu tazele: uygulama arkadayken
    // kaçırılmış bir olay varsa buradan kapanıyor.
    const odak = () => yenile();
    window.addEventListener("focus", odak);
    return () => window.removeEventListener("focus", odak);
  }, [yenile]);

  useEffect(() => {
    if (!tauriMi()) return;

    /**
     * Bekleyen konum yayınını iptal eder ve sayacı başa alır.
     *
     * Dosya değiştiğinde ŞART: `time-pos` kısıtlaması yüzünden yolda
     * bekleyen bir zamanlayıcı olabiliyor ve o zamanlayıcı, ateşlediğinde
     * BİR ÖNCEKİ dosyanın konumunu yazıyordu. Sonuç, yeni dosya başlarken
     * sürgünün bir anlığına eski dosyanın yerine zıplaması — üstelik
     * "sıfırla" diyen olaydan sonra.
     */
    const konumuDurdur = () => {
      konumRef.current = 0;
      if (zamanlayiciRef.current !== null) {
        window.clearTimeout(zamanlayiciRef.current);
        zamanlayiciRef.current = null;
      }
    };

    const sozler = [
      listen<FileInfo>(OLAY.dosya, (e) => {
        konumuDurdur();
        setDurum((d) => ({
          ...d,
          path: e.payload.path,
          title: e.payload.title,
          duration: e.payload.duration,
          mediaType: e.payload.mediaType,
          position: 0,
        }));
        setIzler(e.payload.tracks);
      }),

      listen<KonumYuku>(OLAY.konum, (e) => {
        konumRef.current = e.payload.position;
        if (zamanlayiciRef.current !== null) return;
        zamanlayiciRef.current = window.setTimeout(() => {
          zamanlayiciRef.current = null;
          setDurum((d) => ({ ...d, position: konumRef.current }));
        }, KONUM_ARALIGI);
      }),

      // Süre AYRI bir olayla geliyor çünkü mpv onu çoğu kapsayıcıda
      // `file-loaded`dan sonra öğreniyor. Gelmediğinde arayüzdeki süre 0
      // kalıyor ve arama sürgüsü devre dışı çiziliyor (`Surgu`:
      // `enBuyuk <= 0`) — kullanıcı için "sarma çalışmıyor".
      listen<SureYuku>(OLAY.sure, (e) => {
        setDurum((d) => ({ ...d, duration: e.payload.duration }));
      }),

      // Hız her dosya başında varsayılana dönüyor (playlist/surucu.rs);
      // haber verilmezse çubuk bir önceki dosyanın hızını gösteriyor.
      listen<HizYuku>(OLAY.hiz, (e) => {
        setDurum((d) => ({ ...d, rate: e.payload.rate }));
      }),

      listen<DuraklatYuku>(OLAY.duraklat, (e) => {
        setDurum((d) => ({ ...d, paused: e.payload.paused, playing: !e.payload.paused }));
      }),

      listen<SesYuku>(OLAY.ses, (e) => {
        setDurum((d) => ({
          ...d,
          volume: e.payload.volume ?? d.volume,
          muted: e.payload.muted ?? d.muted,
        }));
      }),

      listen<BittiYuku>(OLAY.bitti, (e) => {
        setDurum((d) => ({ ...d, playing: false }));
        // `eof` normal bitiş: kuyruk backend'de zaten ilerledi, söylenecek
        // bir şey yok. `error` ise kullanıcı bilmeli.
        if (e.payload.reason === "error") {
          onHata("Dosya oynatılamadı. Bozuk olabilir ya da kod çözücü yok.");
        }
      }),

      // Oynatıcı boşaldı: kuyruk bitti ya da durduruldu. Dosyaya ait ne
      // varsa siliniyor, oynatıcıya ait olan (ses, sessizlik, motor) kalıyor
      // — onlar dosyayla değil oynatıcıyla birlikte yaşıyor.
      //
      // Bu olay olmadan bitmiş bir video arayüzde duruyor gibi görünüyordu:
      // sürgü sonda, düğme "duraklat" ve sürgüye basınca boş mpv'ye `seek`
      // gidip hata şeridi çıkıyordu.
      listen(OLAY.temiz, () => {
        konumuDurdur();
        setDurum((d) => ({
          ...d,
          playing: false,
          paused: true,
          position: 0,
          duration: 0,
          path: null,
          title: null,
          mediaType: null,
        }));
        setIzler([]);
      }),

      listen<string>(OLAY.hata, (e) => onHata(e.payload)),

      listen<IstekEylemi>(OLAY.istek, (e) => {
        // Sayaç: aynı eylem art arda gelirse (iki kez `f`) React'in durumu
        // değişmemiş sayıp etkiyi atlamaması için.
        setIstek((o) => ({ eylem: e.payload, sayac: (o?.sayac ?? 0) + 1 }));
      }),
    ];

    return () => {
      sozler.forEach((s) => s.then((birak) => birak()));
      if (zamanlayiciRef.current !== null) {
        window.clearTimeout(zamanlayiciRef.current);
        zamanlayiciRef.current = null;
      }
    };
  }, [onHata]);

  return { durum, izler, istek, yenile };
}
