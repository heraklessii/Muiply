/**
 * Klavye kısayolları yardımı.
 *
 * `?` ile ya da tepe barındaki klavye düğmesiyle açılıyor. Bir oynatıcının
 * kısayolları öğrenilene kadar hatırlanmıyor ve her birini bir ipucu
 * balonuna sığdırmak mümkün değil — hepsi tek listede duruyor.
 *
 * Liste `docs/Frontend.md` → Kısayollar ile AYNI ve aynı kısayollar mpv'nin
 * kendi `keybind`lerinde de var (`mpv/gercek.rs::kisayollari_kur`). Üçü
 * birlikte değişiyor; burada olmayan bir kısayol kullanıcı için var
 * değildir.
 *
 * Kapatma iki yoldan: kaplamaya tıklama ve Escape. İkisi de olmak zorunda —
 * yalnız tıklama klavyeyle gezineni kaplamada kilitler.
 *
 * Escape `capture` aşamasında yakalanıp DURDURULUYOR: pencere düzeyindeki
 * kısayol (`useKumanda`) aynı tuşla tam ekrandan çıkıyor ve yardım açıkken
 * Escape'in işi yalnız yardımı kapatmak. Aynı desen `Menu.tsx`te de var.
 */

import { useEffect } from 'react';

import { IconKapat, IconKlavye } from './Ikonlar';

/** Bir bölüm: başlık + satırlar. Satır = [tuşlar, ne yaptığı]. */
const BOLUMLER: { ad: string; satirlar: [string[], string][] }[] = [
  {
    ad: 'Oynatma',
    satirlar: [
      [['Boşluk'], 'Duraklat / devam'],
      [['←', '→'], '5 saniye geri / ileri'],
      [['Shift', '←', '→'], '60 saniye geri / ileri'],
      [['N', 'P'], 'Sonraki / önceki'],
    ],
  },
  {
    ad: 'Ses',
    satirlar: [
      [['↑', '↓'], 'Ses +5 / −5'],
      [['M'], 'Sessiz'],
    ],
  },
  {
    ad: 'Görüntü',
    satirlar: [
      [['F'], 'Tam ekran'],
      [['Esc'], 'Tam ekrandan çık'],
      [['S'], 'Altyazı izleri arasında dön'],
    ],
  },
  {
    ad: 'Yardım',
    satirlar: [[['?'], 'Bu listeyi aç / kapat']],
  },
];

interface Ozellikler {
  acik: boolean;
  kapat: () => void;
}

export function Kisayollar({ acik, kapat }: Ozellikler) {
  useEffect(() => {
    if (!acik) return;

    const kacis = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      e.stopPropagation();
      e.preventDefault();
      kapat();
    };

    document.addEventListener('keydown', kacis, true);
    return () => document.removeEventListener('keydown', kacis, true);
  }, [acik, kapat]);

  if (!acik) return null;

  return (
    <div
      className="kaplama"
      role="dialog"
      aria-modal="true"
      aria-label="Klavye kısayolları"
      // Kutunun İÇİNE yapılan tıklama kapatmıyor: metin seçmek için
      // tıklayan kullanıcı yardımı kaybetmemeli.
      onClick={(e) => {
        if (e.target === e.currentTarget) kapat();
      }}
    >
      <div className="kaplama-kutu">
        <div className="kaplama-tepe">
          <IconKlavye />
          <span className="kaplama-baslik">Klavye kısayolları</span>
          <button
            className="dugme dugme--ikon dugme--hayalet"
            onClick={kapat}
            aria-label="Kapat"
            title="Kapat (Esc)"
          >
            <IconKapat />
          </button>
        </div>

        <div className="kaplama-govde">
          {BOLUMLER.map((bolum) => (
            <section key={bolum.ad} className="kisayol-bolum">
              <div className="yan-baslik">{bolum.ad}</div>
              {bolum.satirlar.map(([tuslar, ne]) => (
                <div key={ne} className="kisayol-satir">
                  <span className="kisayol-ad">{ne}</span>
                  {tuslar.map((t) => (
                    <kbd key={t} className="tus">
                      {t}
                    </kbd>
                  ))}
                </div>
              ))}
            </section>
          ))}

          {/*
            Video üstündeyken tuşlar webview'e ulaşmıyor (mpv'nin yüzeyi
            onun üstünde) ve kullanıcı bunu "kısayollar bazen çalışmıyor"
            diye yaşıyor. Aynı bağlar mpv'de de kurulu, yani çalışıyorlar;
            cümle o kafa karışıklığını baştan kesiyor.
          */}
          <section className="kisayol-bolum">
            <div className="kisayol-satir">
              <span className="kisayol-ad">
                Kısayollar videonun üstündeyken de çalışıyor; fareyle çift
                tıklamak tam ekrana geçiriyor.
              </span>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
