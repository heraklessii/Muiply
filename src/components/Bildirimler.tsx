/**
 * Bildirim yığını ve tarama şeridi.
 *
 * İkisi aynı dosyada ama AYNI ŞEY DEĞİL ve bilerek ayrı çiziliyorlar:
 *
 * - **Bildirim** olup bitmiş bir şeyin haberi. Ana alanın sağ altında
 *   yüzüyor, kendiliğinden kapanıyor, yer kaplamıyor.
 * - **Tarama** süren bir iş. Ana alanın üstünde kendi satırında duruyor ve
 *   yer KAPLIYOR: yüzen bir çubuk, altındaki ızgarayı örterdi ve tarama
 *   dakikalarca sürebiliyor.
 *
 * Eski sürümde ikisi tek bir şeritti ve biri diğerinin yerini alıyordu —
 * tarama sürerken çıkan bir hata ilerlemeyi siliyordu.
 */

import type { Bildirim } from '../hooks/useBildirimler';
import type { TaramaYuku } from '../ipc/tipler';
import { IconBilgi, IconKapat, IconUyari } from './Ikonlar';

interface YiginOzellikleri {
  bildirimler: Bildirim[];
  kapat: (id: number) => void;
}

export function Bildirimler({ bildirimler, kapat }: YiginOzellikleri) {
  if (bildirimler.length === 0) return null;

  return (
    /*
     * `aria-live="polite"`: ekran okuyucu okuduğunu bölmeden, sırası
     * geldiğinde söylüyor. Hata satırının kendisi ayrıca `role="alert"`
     * taşıyor — o bölmeli.
     */
    <div className="bildirimler" aria-live="polite">
      {bildirimler.map((b) => (
        <div
          key={b.id}
          className={b.tur === 'bilgi' ? 'bildirim' : `bildirim bildirim--${b.tur}`}
          role={b.tur === 'hata' ? 'alert' : 'status'}
        >
          {b.tur === 'bilgi' ? <IconBilgi /> : <IconUyari />}
          <span className="bildirim-metin secilebilir">{b.metin}</span>
          <button
            className="dugme dugme--ikon dugme--hayalet"
            onClick={() => kapat(b.id)}
            aria-label="Bildirimi kapat"
          >
            <IconKapat />
          </button>
        </div>
      ))}
    </div>
  );
}

interface TaramaOzellikleri {
  tarama: TaramaYuku | null;
}

export function TaramaSeridi({ tarama }: TaramaOzellikleri) {
  if (!tarama) return null;

  // Toplam sıfırken (klasörde hiç medya yok) yüzde hesabı 0/0 oluyor;
  // çubuğu tam dolu göstermek yanlış bir bitmişlik hissi verirdi.
  const oran = tarama.total > 0 ? (tarama.done / tarama.total) * 100 : 0;

  return (
    <div className="tarama" role="status">
      <span className="tarama-metin">
        Klasör taranıyor — {tarama.done} / {tarama.total}
      </span>
      <div
        className="ilerleme"
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={tarama.total}
        aria-valuenow={tarama.done}
        aria-label="Tarama ilerlemesi"
      >
        <div className="ilerleme-dolu" style={{ width: `${oran}%` }} />
      </div>
    </div>
  );
}
