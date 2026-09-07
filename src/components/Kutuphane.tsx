/**
 * Kütüphane ızgarası ve üstündeki süzme satırı.
 *
 * İş bölümü: sıralama ve tür süzgeci backend'de (SQL), arama burada
 * (`lib/suz.ts`). Kutuya yazılan her harf için IPC'ye çıkmak, bellekteki
 * bir diziyi süzmenin yanında bedava değil ve sonuç aynı.
 *
 * Bir karta tıklamak GÖRÜNEN bütün kayıtları kuyruğa alıyor ve tıklanandan
 * başlıyor. Böylece "tıkladığım çalsın, sonra bunlar devam etsin" davranışı
 * ayrı bir kural yazmadan çıkıyor — ve "bunlar" tam olarak kullanıcının o an
 * gördüğü liste, süzülmüş hâliyle.
 */

import { useMemo, useState } from "react";

import type { MediaItem, MediaTuru, Playlist, SiralamaAnahtari } from "../ipc/tipler";
import { ara } from "../lib/suz";
import { IconAra, IconKlasor } from "./Ikonlar";
import { Kart } from "./Kart";

interface Ozellikler {
  ogeler: MediaItem[];
  listeler: Playlist[];
  tur: MediaTuru | "hepsi";
  siralama: SiralamaAnahtari;
  calanYol: string | null;
  klasorVar: boolean;
  onSiralama: (s: SiralamaAnahtari) => void;
  onCal: (gorunen: MediaItem[], indeks: number) => void;
  onListeyeEkle: (listeId: number, medyaId: string) => void;
  onKaldir: (id: string) => void;
  onKlasorEkle: () => void;
}

const SIRALAMA_ADLARI: Record<SiralamaAnahtari, string> = {
  added: "Eklenme",
  title: "Ad",
  played: "Son çalınan",
  duration: "Süre",
};

export function Kutuphane(p: Ozellikler) {
  const [sorgu, setSorgu] = useState("");

  // Arama her tuşta çalışıyor ve liste binlerce kayıt olabiliyor; sonuç
  // sorgu ya da kaynak değişmedikçe yeniden hesaplanmıyor.
  const gorunen = useMemo(() => ara(p.ogeler, sorgu), [p.ogeler, sorgu]);

  return (
    <div className="icerik">
      <div className="arac-cubugu">
        <IconAra style={{ width: 15, height: 15, color: "var(--text-muted)", flex: "none" }} />
        <input
          type="search"
          placeholder="Kütüphanede ara"
          value={sorgu}
          onChange={(e) => setSorgu(e.target.value)}
          aria-label="Kütüphanede ara"
        />

        <div className="arac-bosluk" />

        <label className="alan-etiket" htmlFor="siralama">
          Sırala
        </label>
        <select
          id="siralama"
          value={p.siralama}
          onChange={(e) => p.onSiralama(e.target.value as SiralamaAnahtari)}
        >
          {(Object.keys(SIRALAMA_ADLARI) as SiralamaAnahtari[]).map((k) => (
            <option key={k} value={k}>
              {SIRALAMA_ADLARI[k]}
            </option>
          ))}
        </select>

        <span className="rozet">{gorunen.length}</span>
      </div>

      {gorunen.length === 0 ? (
        <BosDurum
          klasorVar={p.klasorVar}
          arandi={sorgu.trim().length > 0}
          tur={p.tur}
          onKlasorEkle={p.onKlasorEkle}
        />
      ) : (
        <div className="izgara">
          {gorunen.map((oge, i) => (
            <Kart
              key={oge.id}
              oge={oge}
              calan={oge.path === p.calanYol}
              listeler={p.listeler}
              onCal={() => p.onCal(gorunen, i)}
              onListeyeEkle={(listeId) => p.onListeyeEkle(listeId, oge.id)}
              onKaldir={() => p.onKaldir(oge.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Boşluğun üç ayrı sebebi var ve üçü ayrı cümle hak ediyor: hiç klasör
 * eklenmemiş, klasör var ama bu türde kayıt yok, ya da arama eşleşmedi.
 * Üçüne aynı metni yazmak, kullanıcıyı yanlış düğmeye götürür.
 */
function BosDurum({
  klasorVar,
  arandi,
  tur,
  onKlasorEkle,
}: {
  klasorVar: boolean;
  arandi: boolean;
  tur: MediaTuru | "hepsi";
  onKlasorEkle: () => void;
}) {
  if (arandi) {
    return (
      <div className="bos">
        <IconAra />
        <div className="bos-baslik">Eşleşen kayıt yok</div>
        <p>Aramayı kısaltmayı deneyin.</p>
      </div>
    );
  }

  if (!klasorVar) {
    return (
      <div className="bos">
        <IconKlasor />
        <div className="bos-baslik">Kütüphane boş</div>
        <p>Videolarının ve müziğinin durduğu klasörü ekle; Muiply gerisini tarar.</p>
        <button className="dugme dugme--birincil" onClick={onKlasorEkle}>
          Klasör ekle
        </button>
      </div>
    );
  }

  const neYok =
    tur === "video" ? "video"
    : tur === "audio" ? "ses dosyası"
    : "medya dosyası";

  return (
    <div className="bos">
      <IconKlasor />
      <div className="bos-baslik">Bu klasörlerde {neYok} bulunamadı</div>
      <p>Başka bir klasör ekleyebilir ya da yeniden tarayabilirsin.</p>
    </div>
  );
}
