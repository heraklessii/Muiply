/**
 * Yan sütun — gezinme.
 *
 * Ekranda her an okunması gereken üçüncü şey burada: **sırada ne var.**
 * Kuyruk satırı öğe sayısını taşıyor, çalma listeleri de.
 *
 * Klasörler en altta ve sessiz: kullanıcı onlara ayda bir dokunuyor, ama
 * "kütüphanem neden boş" sorusunun yanıtı orada olduğu için gizlenmiyorlar.
 *
 * Bileşen `memo` içinde. Sebebi ölçülebilir: kütüphane penceresi çalarken
 * saniyede beş kez yeniden boyanıyor (konum güncellemesi) ve yan sütunun o
 * boyamalarda değişen tek bir pikseli yok.
 */

import { memo } from 'react';

import type { MediaTuru, Playlist, QueueSnapshot } from '../ipc/tipler';
import {
  IconArti,
  IconAyar,
  IconKapat,
  IconKlasor,
  IconKuyruk,
  IconListe,
  IconMuzik,
  IconOynat,
  IconVideo,
  IconYenile,
} from './Ikonlar';

/**
 * Kütüphane penceresindeki görünümler.
 *
 * "Oynatıcı" burada YOK: video artık bu pencerede çizilmiyor, kendi
 * penceresi var (`src-tauri/src/pencere.rs`). "Şimdi çalan" satırı bir
 * görünüm değiştirmiyor, öbür pencereyi gösteriyor.
 */
export type Gorunum =
  | { tur: 'kutuphane' }
  | { tur: 'kuyruk' }
  | { tur: 'liste'; id: number }
  | { tur: 'ayarlar' };

interface Ozellikler {
  gorunum: Gorunum;
  tur: MediaTuru | 'hepsi';
  listeler: Playlist[];
  klasorler: string[];
  kuyruk: QueueSnapshot;
  /** Oynatıcı penceresinde bir şey çalıyor mu — "Şimdi çalan" satırı için. */
  calan: boolean;
  onGorunum: (g: Gorunum) => void;
  /** Oynatıcı penceresini gösterir. */
  onOynatici: () => void;
  onTur: (t: MediaTuru | 'hepsi') => void;
  onYeniListe: () => void;
  onKlasorEkle: () => void;
  onKlasorSil: (yol: string) => void;
  onTara: () => void;
}

export const Yan = memo(function Yan(p: Ozellikler) {
  const kutuphanede = p.gorunum.tur === 'kutuphane';

  return (
    <nav className="yan" aria-label="Gezinme">
      <section className="yan-bolum">
        <div className="yan-baslik">Kütüphane</div>
        <Satir
          secili={kutuphanede && p.tur === 'hepsi'}
          onClick={() => {
            p.onTur('hepsi');
            p.onGorunum({ tur: 'kutuphane' });
          }}
          ikon={<IconListe />}
        >
          Tümü
        </Satir>
        <Satir
          secili={kutuphanede && p.tur === 'video'}
          onClick={() => {
            p.onTur('video');
            p.onGorunum({ tur: 'kutuphane' });
          }}
          ikon={<IconVideo />}
        >
          Videolar
        </Satir>
        <Satir
          secili={kutuphanede && p.tur === 'audio'}
          onClick={() => {
            p.onTur('audio');
            p.onGorunum({ tur: 'kutuphane' });
          }}
          ikon={<IconMuzik />}
        >
          Müzik
        </Satir>
      </section>

      <section className="yan-bolum">
        <div className="yan-baslik">Oynatma</div>
        {/*
          "Şimdi çalan" hiçbir şey çalmıyorken de tıklanabilir: oynatıcı
          penceresi o durumda "dosya aç" diyor, yani boş bir ekrana
          götürmüyor. Satır SEÇİLİ olmuyor — burada bir görünüm
          değiştirmiyor, öbür pencereyi öne getiriyor.
        */}
        <Satir
          secili={false}
          onClick={p.onOynatici}
          ikon={<IconOynat />}
          rozet={p.calan ? <span className="rozet rozet--vurgu">Çalıyor</span> : undefined}
        >
          Şimdi çalan
        </Satir>
        <Satir
          secili={p.gorunum.tur === 'kuyruk'}
          onClick={() => p.onGorunum({ tur: 'kuyruk' })}
          ikon={<IconKuyruk />}
          sayi={p.kuyruk.items.length || undefined}
        >
          Kuyruk
        </Satir>
      </section>

      <section className="yan-bolum">
        <div className="yan-baslik">
          Çalma listeleri
          <span className="yan-eylemler">
            <button
              className="dugme dugme--ikon dugme--hayalet"
              onClick={p.onYeniListe}
              title="Yeni çalma listesi"
              aria-label="Yeni çalma listesi"
            >
              <IconArti />
            </button>
          </span>
        </div>
        {p.listeler.length === 0 ? (
          <div className="yan-durgun">Henüz liste yok</div>
        ) : (
          p.listeler.map((l) => (
            <Satir
              key={l.id}
              secili={p.gorunum.tur === 'liste' && p.gorunum.id === l.id}
              onClick={() => p.onGorunum({ tur: 'liste', id: l.id })}
              ikon={<IconListe />}
              sayi={l.count}
            >
              {l.name}
            </Satir>
          ))
        )}
      </section>

      <section className="yan-bolum">
        <div className="yan-baslik">
          Klasörler
          <span className="yan-eylemler">
            <button
              className="dugme dugme--ikon dugme--hayalet"
              onClick={p.onTara}
              title="Yeniden tara"
              aria-label="Klasörleri yeniden tara"
            >
              <IconYenile />
            </button>
            <button
              className="dugme dugme--ikon dugme--hayalet"
              onClick={p.onKlasorEkle}
              title="Klasör ekle"
              aria-label="Klasör ekle"
            >
              <IconArti />
            </button>
          </span>
        </div>
        {p.klasorler.length === 0 ? (
          <div className="yan-durgun">İzlenen klasör yok</div>
        ) : (
          p.klasorler.map((k) => (
            <div key={k} className="yan-durgun">
              <IconKlasor />
              {/*
                Yolun SONU önemli: "C:/Users/.../Filmler" içinde ayırt edici
                olan son klasörün adı, başı hepsinde aynı. Tam yol ipuçunda.
              */}
              <span className="kirp" title={k}>
                {klasorAdi(k)}
              </span>
              <button
                className="dugme dugme--ikon dugme--hayalet"
                onClick={() => p.onKlasorSil(k)}
                title="Kütüphaneden çıkar"
                aria-label={`${klasorAdi(k)} klasörünü kütüphaneden çıkar`}
              >
                <IconKapat />
              </button>
            </div>
          ))
        )}
      </section>

      {/*
        Ayarlar en altta ve başlıksız: yılda bir açılan bir yer, gezinmenin
        gövdesiyle aynı ağırlıkta durmamalı.
      */}
      <section className="yan-bolum yan-bolum--dip">
        <Satir
          secili={p.gorunum.tur === 'ayarlar'}
          onClick={() => p.onGorunum({ tur: 'ayarlar' })}
          ikon={<IconAyar />}
        >
          Ayarlar
        </Satir>
      </section>
    </nav>
  );
});

interface SatirOzellikleri {
  secili: boolean;
  onClick: () => void;
  ikon: React.ReactNode;
  /** Sağdaki sayı balonu (kuyruk uzunluğu, listedeki öğe sayısı). */
  sayi?: number;
  /** Sayının yerine geçen serbest rozet ("Çalıyor"). */
  rozet?: React.ReactNode;
  children: React.ReactNode;
}

/**
 * Gezinme satırı.
 *
 * Metin `.kirp` içinde ve rozet ONUN DIŞINDA: içeri alınsaydı uzun bir liste
 * adında rozet de kırpılır, hatta tamamen kaybolurdu.
 */
function Satir({ secili, onClick, ikon, sayi, rozet, children }: SatirOzellikleri) {
  return (
    <button
      className={secili ? 'yan-satir yan-satir--secili' : 'yan-satir'}
      onClick={onClick}
      aria-current={secili ? 'page' : undefined}
    >
      {ikon}
      <span className="kirp">{children}</span>
      {rozet ?? (sayi === undefined ? null : <span className="yan-satir-sayi">{sayi}</span>)}
    </button>
  );
}

function klasorAdi(yol: string): string {
  const parcalar = yol.split(/[\\/]/).filter(Boolean);
  return parcalar[parcalar.length - 1] ?? yol;
}
