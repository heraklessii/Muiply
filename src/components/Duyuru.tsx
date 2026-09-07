/**
 * Duyuru şeridi — tarama ilerlemesi, hata, bilgi.
 *
 * Şerit sahnenin ALTINDA doğuyor, üstünde değil: bir uyarının videoyu
 * kapatması, uyarının kendisinden rahatsız edici. Aynı sebeple tek satır ve
 * asla sarmıyor.
 *
 * Aynı anda tek şey gösteriliyor. Tarama ilerlemesi hataya yer veriyor:
 * bir şey bozulduysa yüzdenin önemi kalmıyor.
 */

import type { TaramaYuku } from "../ipc/tipler";
import { IconUyari } from "./Ikonlar";

export type DuyuruTuru = "bilgi" | "uyari" | "hata";

export interface DuyuruIcerik {
  tur: DuyuruTuru;
  metin: string;
}

interface Ozellikler {
  duyuru: DuyuruIcerik | null;
  tarama: TaramaYuku | null;
  kapat: () => void;
}

export function Duyuru({ duyuru, tarama, kapat }: Ozellikler) {
  if (duyuru) {
    const sinif =
      duyuru.tur === "hata" ? "duyuru duyuru--hata"
      : duyuru.tur === "uyari" ? "duyuru duyuru--uyari"
      : "duyuru";

    return (
      <div className={sinif} role={duyuru.tur === "hata" ? "alert" : "status"}>
        {duyuru.tur === "bilgi" ? null : <IconUyari />}
        <span className="duyuru-metin kirp secilebilir">{duyuru.metin}</span>
        <button className="dugme dugme--ikon dugme--hayalet" onClick={kapat} aria-label="Kapat">
          <IconKapatMini />
        </button>
      </div>
    );
  }

  if (!tarama) return null;

  // Toplam sıfırken (klasörde hiç medya yok) yüzde hesabı 0/0 oluyor;
  // çubuğu tam dolu göstermek yanlış bir bitmişlik hissi verirdi.
  const oran = tarama.total > 0 ? (tarama.done / tarama.total) * 100 : 0;

  return (
    <div className="duyuru" role="status">
      <span className="duyuru-metin kirp">
        Klasör taranıyor — {tarama.done} / {tarama.total}
      </span>
      <div className="ilerleme">
        <div className="ilerleme-dolu" style={{ width: `${oran}%` }} />
      </div>
    </div>
  );
}

function IconKapatMini() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}
      strokeLinecap="round" aria-hidden="true">
      <path d="m6 6 12 12M18 6 6 18" />
    </svg>
  );
}
