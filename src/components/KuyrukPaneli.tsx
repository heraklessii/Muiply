/**
 * Kuyruk paneli — o an çalan sıra.
 *
 * Kuyruk düzenlenmiyor, yalnız gezinilebiliyor. Sıralama ve silme çalma
 * listesinin işi; kuyruk geçici bir şey ve onu da düzenlenebilir yapmak,
 * kullanıcıya "hangisi kalıcı" sorusunu sordurur.
 */

import type { QueueSnapshot } from "../ipc/tipler";
import { sureMetni } from "../lib/sure";
import { IconKuyruk, IconMuzik, IconVideo } from "./Ikonlar";

interface Ozellikler {
  kuyruk: QueueSnapshot;
  onCal: (indeks: number) => void;
}

export function KuyrukPaneli({ kuyruk, onCal }: Ozellikler) {
  if (kuyruk.items.length === 0) {
    return (
      <div className="icerik">
        <div className="bos">
          <IconKuyruk />
          <div className="bos-baslik">Kuyruk boş</div>
          <p>Kütüphaneden bir şey çal ya da bir çalma listesi başlat.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="icerik">
      <div className="arac-cubugu">
        <span className="alan-etiket">Kuyruk</span>
        <span className="rozet">{kuyruk.items.length}</span>
        <div className="arac-bosluk" />
        {kuyruk.shuffle ? <span className="rozet rozet--vurgu">Karışık</span> : null}
        {kuyruk.repeat === "all" ? <span className="rozet rozet--vurgu">Tekrar</span> : null}
        {kuyruk.repeat === "one" ? (
          <span className="rozet rozet--vurgu">Tek parça tekrar</span>
        ) : null}
      </div>

      <div className="liste">
        {kuyruk.items.map((oge, i) => (
          <div
            key={`${oge.id}-${i}`}
            className={
              i === kuyruk.currentIndex ? "liste-satir liste-satir--calan" : "liste-satir"
            }
          >
            <span className="liste-no">{i + 1}</span>
            {oge.mediaType === "audio" ? (
              <IconMuzik style={{ width: 15, height: 15, flex: "none", opacity: 0.7 }} />
            ) : (
              <IconVideo style={{ width: 15, height: 15, flex: "none", opacity: 0.7 }} />
            )}
            <button
              className="kart-baglanti liste-ad kirp"
              onClick={() => onCal(i)}
              title={oge.path}
            >
              {oge.title}
            </button>
            {/* Kuyruğa sürüklenip bırakılan dosyanın süresi bilinmiyor
                (kütüphanede kaydı yok); sıfır yerine çizgi. */}
            <span className="liste-sure">
              {oge.duration > 0 ? sureMetni(oge.duration) : "—"}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
