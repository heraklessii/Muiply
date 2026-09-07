/**
 * Sahne — videonun durduğu yer.
 *
 * Burada bir `<video>` YOK ve olmayacak. mpv görüntüyü webview'in üstünde
 * duran native bir alt pencereye çiziyor; bu bileşenin işi o pencerenin
 * nereye oturacağını söyleyen kutuyu çizmek (`useSahneOlcu`).
 *
 * Üç hâl var ve üçü de aynı kutuda:
 *
 * 1. **Video çalıyor** — kutu boş bırakılıyor, üstündeki native pencere
 *    görünüyor.
 * 2. **Ses çalıyor** — video yok; dosyanın kimliği burada duruyor.
 * 3. **Hiçbir şey açık değil** — ne yapılacağını anlatan boş durum.
 */

import { useRef } from 'react';

import { useSahneOlcu } from '../hooks/useSahneOlcu';
import type { PlayerState } from '../ipc/tipler';
import { IconBirak, IconMuzik, IconOynat } from './Ikonlar';

interface Ozellikler {
  durum: PlayerState;
  onDosyaAc: () => void;
  /** Açık menünün tepesi; video yüzeyi orada bitiyor (bkz. `Menu.tsx`). */
  menuUstSiniri?: number | null;
}

export function Sahne({ durum, onDosyaAc, menuUstSiniri }: Ozellikler) {
  const yuzeyRef = useRef<HTMLDivElement>(null);
  const video = durum.path !== null && durum.mediaType === 'video';

  // Yüzey YALNIZ video çalarken görünüyor. Ses çalarken de göstermek,
  // kapak yazısının üstüne siyah bir dikdörtgen koymak olurdu.
  useSahneOlcu(yuzeyRef, video, menuUstSiniri);

  return (
    <div className="sahne">
      <div ref={yuzeyRef} className="sahne-yuzey" />

      {durum.path === null ? (
        <BosSahne motorYok={!durum.engine} onDosyaAc={onDosyaAc} />
      ) : null}

      {durum.path !== null && !video ? (
        <div className="sahne-ses">
          <div
            className={
              durum.playing ? 'sahne-ses-kapak sahne-ses-kapak--calan' : 'sahne-ses-kapak'
            }
          >
            <IconMuzik />
          </div>
          <div className="sahne-ses-ad kirp secilebilir">{durum.title ?? 'Bilinmeyen'}</div>
          <div className="sahne-ses-alt kirp secilebilir">{durum.path}</div>
        </div>
      ) : null}
    </div>
  );
}

/**
 * Boş sahne.
 *
 * İki ayrı cümle: motorsuz derlemede sorun kullanıcının açacağı bir dosya
 * değil, derlemenin kendisi — "dosya aç" düğmesi göstermek onu çalışmayan
 * bir düğmeye yollamak olurdu.
 */
function BosSahne({ motorYok, onDosyaAc }: { motorYok: boolean; onDosyaAc: () => void }) {
  if (motorYok) {
    return (
      <div className="bos">
        <div className="bos-ikon">
          <IconOynat />
        </div>
        <div className="bos-baslik">Oynatma motoru bu derlemede yok</div>
        <p>
          Kütüphane ve çalma listeleri çalışıyor. Oynatmak için libmpv ile derlenmiş bir
          sürüm gerekiyor.
        </p>
      </div>
    );
  }

  return (
    <div className="bos">
      <div className="bos-ikon">
        <IconBirak />
      </div>
      <div className="bos-baslik">Henüz bir şey açılmadı</div>
      <p>Bir dosya aç, kütüphaneden seç ya da pencereye sürükleyip bırak.</p>
      <button className="dugme dugme--birincil" onClick={onDosyaAc}>
        Dosya aç
      </button>
    </div>
  );
}
