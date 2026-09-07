/**
 * İkonlar — küçük, elle çizilmiş SVG'ler.
 *
 * İkon kütüphanesi bağımlılığı YOK. Sebep somut: kullanılan ikon sayısı
 * otuz civarında ve bir paket kurmak hem yüzlerce kilobaytlık bir bağımlılık
 * hem de ailenin çizim diliyle uyuşmayan bir çizgi kalınlığı demek.
 *
 * Ortak kurallar — hepsi bu dosyada tutuluyor ki ikonlar yan yana
 * durduğunda birinin çizgisi diğerinden kalın görünmesin:
 *
 * - 24×24 kutu, `currentColor`, çizgi kalınlığı 1.9 (dolu olanlar hariç)
 * - yuvarlak uç ve birleşim
 * - boyut CSS'ten geliyor (`.dugme svg` vb.), buradan değil
 * - `aria-hidden`: ikonun anlamı her zaman yanındaki metinde ya da
 *   düğmenin `aria-label`ında; ekran okuyucuya iki kez söylenmemeli
 *
 * Marka ikonu ailenin iskeleti: koyu yuvarlak kare + teal glyph + tek dolu
 * öğe. Muiply'nin glyph'i çember + üçgen, çünkü işi tek şey: dosyayı açıp
 * çalmak. AYNI çizim üç yerde daha var — `public/icons/muiply.svg`,
 * `index.html` favicon'u, `src-tauri/icons/kaynak.svg`. Biri değişirse
 * dördü de değişir.
 */

import type { SVGProps } from 'react';

type Ozellik = SVGProps<SVGSVGElement>;

/** Çizgi ikonların ortak kabuğu. */
function Cizgi({ children, ...p }: Ozellik) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.9}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...p}
    >
      {children}
    </svg>
  );
}

/** Dolu ikonların ortak kabuğu — oynat/duraklat gibi ağırlık isteyenler. */
function Dolu({ children, ...p }: Ozellik) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...p}>
      {children}
    </svg>
  );
}

/* -- marka ---------------------------------------------------------------- */

export function IconMuiply(p: Ozellik) {
  return (
    <svg viewBox="0 0 32 32" aria-hidden="true" {...p}>
      <rect width="32" height="32" rx="8" fill="#0f1115" />
      <circle cx="16" cy="16" r="10.6" fill="none" stroke="#2dd4bf" strokeWidth="2.4" />
      <path d="M12.4 9.8 22.4 16 12.4 22.2z" fill="#2dd4bf" />
    </svg>
  );
}

/* -- oynatma -------------------------------------------------------------- */

export function IconOynat(p: Ozellik) {
  return (
    <Dolu {...p}>
      <path d="M7.5 4.8a1 1 0 0 1 1.52-.85l11 7.2a1 1 0 0 1 0 1.7l-11 7.2A1 1 0 0 1 7.5 19.2z" />
    </Dolu>
  );
}

export function IconDuraklat(p: Ozellik) {
  return (
    <Dolu {...p}>
      <rect x="6.5" y="4.5" width="4.2" height="15" rx="1.4" />
      <rect x="13.3" y="4.5" width="4.2" height="15" rx="1.4" />
    </Dolu>
  );
}

export function IconOnceki(p: Ozellik) {
  return (
    <Dolu {...p}>
      <rect x="5" y="5" width="2.6" height="14" rx="1.2" />
      <path d="M19.6 6.1v11.8a1 1 0 0 1-1.55.83l-8.6-5.9a1 1 0 0 1 0-1.66l8.6-5.9a1 1 0 0 1 1.55.83z" />
    </Dolu>
  );
}

export function IconSonraki(p: Ozellik) {
  return (
    <Dolu {...p}>
      <path d="M4.4 6.1v11.8a1 1 0 0 0 1.55.83l8.6-5.9a1 1 0 0 0 0-1.66l-8.6-5.9a1 1 0 0 0-1.55.83z" />
      <rect x="16.4" y="5" width="2.6" height="14" rx="1.2" />
    </Dolu>
  );
}

/** 10 saniye geri. Sayı ikonun İÇİNDE: iki ok ikonu yan yana ayırt edilmiyor. */
export function IconGeri(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M11.4 5.2 7 8.6l4.4 3.4" />
      <path d="M7.2 8.6h5.6a6 6 0 1 1-6 6" />
      <text
        x="12"
        y="19.4"
        textAnchor="middle"
        fontSize="7.4"
        fontWeight="700"
        fill="currentColor"
        stroke="none"
        fontFamily="inherit"
      >
        10
      </text>
    </Cizgi>
  );
}

/** 10 saniye ileri. */
export function IconIleri(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M12.6 5.2 17 8.6l-4.4 3.4" />
      <path d="M16.8 8.6h-5.6a6 6 0 1 0 6 6" />
      <text
        x="12"
        y="19.4"
        textAnchor="middle"
        fontSize="7.4"
        fontWeight="700"
        fill="currentColor"
        stroke="none"
        fontFamily="inherit"
      >
        10
      </text>
    </Cizgi>
  );
}

export function IconSes(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M4 9.5h3.2L12 5.4v13.2L7.2 14.5H4z" />
      <path d="M16.2 9.4a4 4 0 0 1 0 5.2" />
      <path d="M18.8 6.9a7.6 7.6 0 0 1 0 10.2" />
    </Cizgi>
  );
}

export function IconSessiz(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M4 9.5h3.2L12 5.4v13.2L7.2 14.5H4z" />
      <path d="m16.4 9.8 4.6 4.4M21 9.8l-4.6 4.4" />
    </Cizgi>
  );
}

export function IconKarisik(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M3.6 6.4h3.1c1.4 0 2.6.7 3.4 1.9l3.8 5.8c.8 1.2 2 1.9 3.4 1.9h2.9" />
      <path d="M3.6 17.6h3.1c1.4 0 2.6-.7 3.4-1.9l.7-1.1" />
      <path d="m13.6 9.3.7-1.1c.8-1.2 2-1.8 3.4-1.8h2.5" />
      <path d="m17.9 3.9 2.6 2.5-2.6 2.5" />
      <path d="m17.9 13.1 2.6 2.5-2.6 2.5" />
    </Cizgi>
  );
}

export function IconTekrar(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M6.6 5.6h10.2a3.6 3.6 0 0 1 3.6 3.6v1.6" />
      <path d="m8.8 3.2-2.6 2.4 2.6 2.4" />
      <path d="M17.4 18.4H7.2a3.6 3.6 0 0 1-3.6-3.6v-1.6" />
      <path d="m15.2 20.8 2.6-2.4-2.6-2.4" />
    </Cizgi>
  );
}

/** Tek parça tekrarı: aynı çember, ortasında "1". */
export function IconTekrarTek(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M6.6 5.6h10.2a3.6 3.6 0 0 1 3.6 3.6v1.6" />
      <path d="m8.8 3.2-2.6 2.4 2.6 2.4" />
      <path d="M17.4 18.4H7.2a3.6 3.6 0 0 1-3.6-3.6v-1.6" />
      <path d="m15.2 20.8 2.6-2.4-2.6-2.4" />
      <text
        x="12"
        y="14.6"
        textAnchor="middle"
        fontSize="8"
        fontWeight="700"
        fill="currentColor"
        stroke="none"
        fontFamily="inherit"
      >
        1
      </text>
    </Cizgi>
  );
}

export function IconAltyazi(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <rect x="2.8" y="5" width="18.4" height="14" rx="2.6" />
      <path d="M6.6 14.4h4.2M13.4 14.4h4" />
      <path d="M6.6 10.6h2.6M11.8 10.6h5.6" />
    </Cizgi>
  );
}

export function IconTamEkran(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M4 9V5.8A1.8 1.8 0 0 1 5.8 4H9" />
      <path d="M15 4h3.2A1.8 1.8 0 0 1 20 5.8V9" />
      <path d="M20 15v3.2a1.8 1.8 0 0 1-1.8 1.8H15" />
      <path d="M9 20H5.8A1.8 1.8 0 0 1 4 18.2V15" />
    </Cizgi>
  );
}

export function IconTamEkranCik(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M9.4 4v3.6a1.8 1.8 0 0 1-1.8 1.8H4" />
      <path d="M14.6 4v3.6a1.8 1.8 0 0 0 1.8 1.8H20" />
      <path d="M14.6 20v-3.6a1.8 1.8 0 0 1 1.8-1.8H20" />
      <path d="M9.4 20v-3.6a1.8 1.8 0 0 0-1.8-1.8H4" />
    </Cizgi>
  );
}

/* -- içerik türleri ------------------------------------------------------- */

export function IconVideo(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <rect x="2.6" y="5.4" width="13.2" height="13.2" rx="2.6" />
      <path d="m15.8 10.4 4.2-2.6a.8.8 0 0 1 1.2.7v6.9a.8.8 0 0 1-1.2.7l-4.2-2.5z" />
    </Cizgi>
  );
}

export function IconMuzik(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M9.4 17.6V5.6l9.2-2v11.4" />
      <circle cx="6.4" cy="17.6" r="3" />
      <circle cx="15.6" cy="15" r="3" />
    </Cizgi>
  );
}

export function IconKlasor(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M3 7.4a2 2 0 0 1 2-2h3.6l2 2.4H19a2 2 0 0 1 2 2v6.8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    </Cizgi>
  );
}

export function IconListe(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M3.6 6.6h11.2M3.6 11.4h11.2M3.6 16.2h7" />
      <path d="M18.6 8.4v8.2" />
      <circle cx="16.9" cy="17.2" r="1.9" />
    </Cizgi>
  );
}

export function IconKuyruk(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M3.6 6.6h13.4M3.6 11.4h13.4M3.6 16.2h8" />
      <path d="M16.4 14.6v5.2M13.8 17.2h5.2" />
    </Cizgi>
  );
}

export function IconAyar(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <circle cx="12" cy="12" r="3.1" />
      <path d="M19.2 14.6a1.6 1.6 0 0 0 .32 1.76l.06.06a1.9 1.9 0 1 1-2.7 2.7l-.06-.06a1.6 1.6 0 0 0-1.76-.32 1.6 1.6 0 0 0-.97 1.46v.17a1.9 1.9 0 1 1-3.8 0v-.09a1.6 1.6 0 0 0-1.04-1.46 1.6 1.6 0 0 0-1.76.32l-.06.06a1.9 1.9 0 1 1-2.7-2.7l.06-.06a1.6 1.6 0 0 0 .32-1.76 1.6 1.6 0 0 0-1.46-.97h-.17a1.9 1.9 0 1 1 0-3.8h.09a1.6 1.6 0 0 0 1.46-1.04 1.6 1.6 0 0 0-.32-1.76l-.06-.06a1.9 1.9 0 1 1 2.7-2.7l.06.06a1.6 1.6 0 0 0 1.76.32h.08a1.6 1.6 0 0 0 .97-1.46v-.17a1.9 1.9 0 1 1 3.8 0v.09a1.6 1.6 0 0 0 .97 1.46 1.6 1.6 0 0 0 1.76-.32l.06-.06a1.9 1.9 0 1 1 2.7 2.7l-.06.06a1.6 1.6 0 0 0-.32 1.76v.08a1.6 1.6 0 0 0 1.46.97h.17a1.9 1.9 0 1 1 0 3.8h-.09a1.6 1.6 0 0 0-1.46.97z" />
    </Cizgi>
  );
}

/* -- eylemler ------------------------------------------------------------- */

export function IconAra(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <circle cx="10.8" cy="10.8" r="6.4" />
      <path d="m15.6 15.6 4.2 4.2" />
    </Cizgi>
  );
}

export function IconArti(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M12 5.4v13.2M5.4 12h13.2" />
    </Cizgi>
  );
}

export function IconKapat(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="m6.4 6.4 11.2 11.2M17.6 6.4 6.4 17.6" />
    </Cizgi>
  );
}

export function IconYenile(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M20 11.4a8 8 0 1 0-.6 4.4" />
      <path d="M20.4 4.6v6.4H14" />
    </Cizgi>
  );
}

export function IconCop(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M4.6 6.8h14.8" />
      <path d="M9.4 6.8V5.2a1.4 1.4 0 0 1 1.4-1.4h2.4a1.4 1.4 0 0 1 1.4 1.4v1.6" />
      <path d="M6.6 6.8 7.4 19a1.6 1.6 0 0 0 1.6 1.4h6a1.6 1.6 0 0 0 1.6-1.4l.8-12.2" />
    </Cizgi>
  );
}

export function IconUyari(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M10.3 4.3 2.9 17a2 2 0 0 0 1.7 3h14.8a2 2 0 0 0 1.7-3L13.7 4.3a2 2 0 0 0-3.4 0z" />
      <path d="M12 9.4v4.2M12 17.2h.01" />
    </Cizgi>
  );
}

export function IconBilgi(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <circle cx="12" cy="12" r="8.4" />
      <path d="M12 11.2v5M12 7.9h.01" />
    </Cizgi>
  );
}

export function IconBirak(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M20.4 15.4v3.2a1.8 1.8 0 0 1-1.8 1.8H5.4a1.8 1.8 0 0 1-1.8-1.8v-3.2" />
      <path d="M7.6 10 12 14.4 16.4 10" />
      <path d="M12 14.4V3.6" />
    </Cizgi>
  );
}

export function IconKlavye(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <rect x="2.4" y="6.4" width="19.2" height="11.2" rx="2.2" />
      <path d="M6.4 10h.01M9.6 10h.01M12.8 10h.01M16 10h.01M17.6 13.2h.01M6.4 13.2h.01M9.2 14h5.6" />
    </Cizgi>
  );
}

/* -- görünüm -------------------------------------------------------------- */

export function IconIzgara(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <rect x="3.6" y="3.6" width="7" height="7" rx="1.8" />
      <rect x="13.4" y="3.6" width="7" height="7" rx="1.8" />
      <rect x="3.6" y="13.4" width="7" height="7" rx="1.8" />
      <rect x="13.4" y="13.4" width="7" height="7" rx="1.8" />
    </Cizgi>
  );
}

export function IconSatirlar(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M4 6.6h16M4 12h16M4 17.4h16" />
    </Cizgi>
  );
}

export function IconTut(p: Ozellik) {
  return (
    <Dolu {...p}>
      <circle cx="9.5" cy="6" r="1.5" />
      <circle cx="9.5" cy="12" r="1.5" />
      <circle cx="9.5" cy="18" r="1.5" />
      <circle cx="14.5" cy="6" r="1.5" />
      <circle cx="14.5" cy="12" r="1.5" />
      <circle cx="14.5" cy="18" r="1.5" />
    </Dolu>
  );
}

export function IconNokta(p: Ozellik) {
  return (
    <Dolu {...p}>
      <circle cx="12" cy="5.5" r="1.7" />
      <circle cx="12" cy="12" r="1.7" />
      <circle cx="12" cy="18.5" r="1.7" />
    </Dolu>
  );
}

export function IconTik(p: Ozellik) {
  return (
    <Cizgi strokeWidth={2.6} {...p}>
      <path d="m4.8 12.6 4.6 4.6L19.2 7.4" />
    </Cizgi>
  );
}

/**
 * Çalan satırın işareti — üç dalga.
 *
 * Sıra numarasının yerine geçiyor: çalan satırda numara bilgi taşımıyor,
 * "şu an bu çalıyor" taşıyor. Animasyon `prefers-reduced-motion` altında
 * kendiliğinden duruyor (styles.css'teki genel kural).
 */
export function IconDalga(p: Ozellik) {
  return (
    <Dolu {...p}>
      <rect x="4" y="9" width="3" height="6" rx="1.5">
        <animate
          attributeName="height"
          values="6;13;6"
          dur="1.1s"
          repeatCount="indefinite"
        />
        <animate attributeName="y" values="9;5.5;9" dur="1.1s" repeatCount="indefinite" />
      </rect>
      <rect x="10.5" y="6" width="3" height="12" rx="1.5">
        <animate
          attributeName="height"
          values="12;5;12"
          dur="1.1s"
          begin="0.25s"
          repeatCount="indefinite"
        />
        <animate
          attributeName="y"
          values="6;9.5;6"
          dur="1.1s"
          begin="0.25s"
          repeatCount="indefinite"
        />
      </rect>
      <rect x="17" y="8" width="3" height="8" rx="1.5">
        <animate
          attributeName="height"
          values="8;14;8"
          dur="1.1s"
          begin="0.5s"
          repeatCount="indefinite"
        />
        <animate
          attributeName="y"
          values="8;5;8"
          dur="1.1s"
          begin="0.5s"
          repeatCount="indefinite"
        />
      </rect>
    </Dolu>
  );
}

/* -- tema ----------------------------------------------------------------- */

export function IconGunes(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <circle cx="12" cy="12" r="4.2" />
      <path d="M12 2.6v2.2M12 19.2v2.2M4.4 12H2.2M21.8 12h-2.2M6.6 6.6 5 5M19 19l-1.6-1.6M6.6 17.4 5 19M19 5l-1.6 1.6" />
    </Cizgi>
  );
}

export function IconAy(p: Ozellik) {
  return (
    <Cizgi {...p}>
      <path d="M20.4 13.6A8.6 8.6 0 0 1 10.4 3.6a8.6 8.6 0 1 0 10 10z" />
    </Cizgi>
  );
}
