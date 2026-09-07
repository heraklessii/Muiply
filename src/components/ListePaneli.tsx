/**
 * Çalma listesi paneli — bir listenin öğeleri.
 *
 * # Sıralama
 *
 * HTML5 sürükle-bırak kullanılıyor (`draggable` + `dragover`), bir kütüphane
 * değil. Liste tek sütun ve satırlar sabit yükseklikte; bu durumda gereken
 * tek bilgi "hangi satırın üstündeyim", onu da `dragover` veriyor.
 *
 * Bırakma hedefi satırın KENDİSİ, aralar değil: aralara nişan almak fare
 * için birkaç piksellik bir şerit demek. "Bu satırın yerine geç" hem daha
 * kolay tutturuluyor hem de sonucu tahmin edilebilir.
 */

import { useState } from "react";

import type { PlaylistItem } from "../ipc/tipler";
import { sureMetni } from "../lib/sure";
import { IconCop, IconKapat, IconListe, IconOynat } from "./Ikonlar";

interface Ozellikler {
  ad: string;
  ogeler: PlaylistItem[];
  calanYol: string | null;
  onCal: (indeks: number) => void;
  onAdDegistir: (ad: string) => void;
  onSil: () => void;
  /** SATIR kimliği (`PlaylistItem.itemId`), sıra değil: silme düğmesine
   *  basıldığında listenin bu arada değişmiş olması yanlış satırı silmesin. */
  onOgeSil: (ogeId: number) => void;
  onSirala: (nereden: number, nereye: number) => void;
}

export function ListePaneli(p: Ozellikler) {
  const [tasinan, setTasinan] = useState<number | null>(null);
  const [hedef, setHedef] = useState<number | null>(null);
  const [duzenleniyor, setDuzenleniyor] = useState(false);
  const [taslakAd, setTaslakAd] = useState(p.ad);

  const adiKaydet = () => {
    setDuzenleniyor(false);
    const yeni = taslakAd.trim();
    // Boş ad backend'de reddediliyor; buraya kadar getirmenin anlamı yok.
    if (yeni && yeni !== p.ad) p.onAdDegistir(yeni);
    else setTaslakAd(p.ad);
  };

  return (
    <div className="icerik">
      <div className="arac-cubugu">
        {duzenleniyor ? (
          <input
            type="text"
            value={taslakAd}
            autoFocus
            onChange={(e) => setTaslakAd(e.target.value)}
            onBlur={adiKaydet}
            onKeyDown={(e) => {
              if (e.key === "Enter") adiKaydet();
              if (e.key === "Escape") {
                setTaslakAd(p.ad);
                setDuzenleniyor(false);
              }
            }}
            style={{ maxWidth: 280 }}
            aria-label="Liste adı"
          />
        ) : (
          <button
            className="dugme dugme--hayalet"
            onClick={() => {
              setTaslakAd(p.ad);
              setDuzenleniyor(true);
            }}
            title="Adı değiştir"
          >
            <IconListe />
            {p.ad}
          </button>
        )}

        <span className="rozet">{p.ogeler.length}</span>

        <div className="arac-bosluk" />

        <button
          className="dugme dugme--birincil"
          onClick={() => p.onCal(0)}
          disabled={p.ogeler.length === 0}
        >
          <IconOynat />
          Listeyi çal
        </button>
        <button className="dugme dugme--tehlike" onClick={p.onSil} title="Listeyi sil">
          <IconCop />
        </button>
      </div>

      {p.ogeler.length === 0 ? (
        <div className="bos">
          <IconListe />
          <div className="bos-baslik">Bu liste boş</div>
          <p>Kütüphaneden bir kartın üç nokta menüsünü açıp bu listeye ekleyebilirsin.</p>
        </div>
      ) : (
        <div className="liste">
          {p.ogeler.map((oge, i) => (
            <div
              key={oge.itemId}
              className={satirSinifi(oge.path === p.calanYol, tasinan === i, hedef === i)}
              draggable
              onDragStart={() => setTasinan(i)}
              onDragOver={(e) => {
                // Varsayılan davranış "bırakmayı reddet"; engellemezsek
                // `drop` hiç tetiklenmiyor.
                e.preventDefault();
                setHedef(i);
              }}
              onDragEnd={() => {
                setTasinan(null);
                setHedef(null);
              }}
              onDrop={(e) => {
                e.preventDefault();
                if (tasinan !== null) p.onSirala(tasinan, i);
                setTasinan(null);
                setHedef(null);
              }}
              onDoubleClick={() => p.onCal(i)}
            >
              <span className="liste-no">{i + 1}</span>
              <button
                className="kart-baglanti liste-ad kirp"
                onClick={() => p.onCal(i)}
                title={oge.path}
              >
                {oge.title}
              </button>
              <span className="liste-sure">{sureMetni(oge.duration)}</span>
              <button
                className="dugme dugme--ikon dugme--hayalet"
                onClick={() => p.onOgeSil(oge.itemId)}
                title="Listeden çıkar"
                aria-label={`${oge.title} listeden çıkar`}
              >
                <IconKapat />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function satirSinifi(calan: boolean, tasiniyor: boolean, hedefMi: boolean): string {
  const parcalar = ["liste-satir"];
  if (calan) parcalar.push("liste-satir--calan");
  if (tasiniyor) parcalar.push("liste-satir--tasiniyor");
  // Taşınan satırın kendisi hedef olarak işaretlenmiyor: kendi üstüne
  // bırakmak bir şey değiştirmiyor, çerçeve göstermek yanıltıcı olurdu.
  if (hedefMi && !tasiniyor) parcalar.push("liste-satir--hedef");
  return parcalar.join(" ");
}
