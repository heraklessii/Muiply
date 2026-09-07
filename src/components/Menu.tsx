/**
 * Açılır menü.
 *
 * Portal yok, `<dialog>` yok: menü tetikleyen düğmenin yanında,
 * `position: relative` bir sarmalayıcının içinde duruyor. Tek katman, tek
 * kapatma yolu, taşınacak z-index yığını yok.
 *
 * Kapatma iki yoldan: dışarı tıklama ve Escape. İkisi de olmak zorunda —
 * yalnız dışarı tıklama klavyeyle gezineni menüde kilitler, yalnız Escape
 * fareyle gezineni.
 *
 * # Menüler videonun altında kalıyordu
 *
 * Çubuktaki menüler YUKARI açılıyor, yani video alanına giriyor. mpv o alana
 * webview'in ÜSTÜNDEN çiziyor (bkz. `src-tauri/src/mpv/yuzey.rs`), yani
 * hız ve altyazı menüleri video oynarken görünmüyordu bile — kullanıcı
 * açısından düğmeler çalışmıyordu.
 *
 * Çözüm: açık bir menü varken video yüzeyi menünün ÜSTÜNDE bitiyor. Menü
 * kendi tepesini aşağıdaki kayda bildiriyor, oynatıcı penceresi bunu
 * `useSahneOlcu`ya veriyor ve yüzey o kadar kısalıyor. Video küçülüyor ama
 * görünmeye devam ediyor — menü açıkken görüntüyü tamamen karartmak, hız
 * değiştirirken sonucu görememek demekti.
 *
 * Kayıt modül düzeyinde çünkü menüyü açan bileşenle (çubuk) yüzeyi ölçen
 * bileşen (sahne) arasında ortak bir ata yok; her menüye ayrı bir geri
 * çağırma taşımak, birini unutmak demekti.
 */

import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react';

/** Açık menülerin viewport'taki en üst noktası. Menü yoksa boş. */
const acikMenuler = new Map<object, number>();
const aboneler = new Set<(y: number | null) => void>();

function enUst(): number | null {
  if (acikMenuler.size === 0) return null;
  let en = Number.POSITIVE_INFINITY;
  for (const y of acikMenuler.values()) {
    if (y < en) en = y;
  }
  return Number.isFinite(en) ? en : null;
}

function bildir() {
  const y = enUst();
  aboneler.forEach((f) => f(y));
}

/**
 * Açık menülerin en üst noktası (CSS pikseli, viewport'tan). Menü yoksa
 * `null`.
 *
 * Oynatıcı penceresi bunu video yüzeyinin alt sınırı olarak kullanıyor.
 */
export function useMenuUstSiniri(): number | null {
  const [y, setY] = useState<number | null>(enUst);
  useEffect(() => {
    aboneler.add(setY);
    setY(enUst());
    return () => {
      aboneler.delete(setY);
    };
  }, []);
  return y;
}

interface Ozellikler {
  acik: boolean;
  kapat: () => void;
  /** Menü aşağı mı açılsın (üst bardaki) yoksa yukarı mı (çubuktaki). */
  yon?: 'yukari' | 'asagi';
  etiket: string;
  children: ReactNode;
}

export function Menu({ acik, kapat, yon = 'yukari', etiket, children }: Ozellikler) {
  const kutuRef = useRef<HTMLDivElement>(null);
  const kimlik = useRef({}).current;

  /*
   * `useLayoutEffect`: ölçü boyamadan ÖNCE gitmeli, yoksa menü bir kare
   * boyunca videonun altında kalıyor ve gözle görülür bir titreme oluyor.
   *
   * Bağımlılık YALNIZ `acik`. `children`ı bağımlılığa koymak, her boyamada
   * yeni bir eleman nesnesi üretildiği için etkiyi her boyamada söküp
   * yeniden kurmak demekti — menü açıkken saniyede onlarca kez. Menünün
   * yüksekliği içeriğiyle değişebiliyor ama boyu değiştiren şey her zaman
   * bir tıklama ve o tıklama zaten menüyü kapatıyor.
   */
  useLayoutEffect(() => {
    const el = kutuRef.current;
    if (!acik || !el) return;

    acikMenuler.set(kimlik, el.getBoundingClientRect().top);
    bildir();
    return () => {
      acikMenuler.delete(kimlik);
      bildir();
    };
  }, [acik, kimlik]);

  useEffect(() => {
    if (!acik) return;

    const disariTik = (e: MouseEvent) => {
      // Sarmalayıcının İÇİ: tetikleyen düğme de burada. Düğmeye basınca
      // önce bu dinleyici kapatıp sonra düğmenin kendisi yeniden açardı.
      if (!kutuRef.current?.parentElement?.contains(e.target as Node)) kapat();
    };
    const kacis = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      // Menü açıkken Escape menüyü kapatır, tam ekrandan çıkmaz.
      e.stopPropagation();
      kapat();
    };

    // `capture`: Escape'i pencere düzeyindeki kısayoldan ÖNCE yakalamak için.
    document.addEventListener('mousedown', disariTik);
    document.addEventListener('keydown', kacis, true);
    return () => {
      document.removeEventListener('mousedown', disariTik);
      document.removeEventListener('keydown', kacis, true);
    };
  }, [acik, kapat]);

  if (!acik) return null;

  return (
    <div
      ref={kutuRef}
      className={yon === 'asagi' ? 'menu menu--asagi' : 'menu'}
      role="menu"
      aria-label={etiket}
    >
      {children}
    </div>
  );
}

/**
 * Menüyü açıp kapatan ortak durum.
 *
 * `kapat` KARARLI bir işlev olmak zorunda: `Menu` onu bağımlılığında
 * taşıyor ve her boyamada yenisini üretmek dinleyicileri durmadan söküp
 * yeniden kurardı. Her menü sahibinin bunu ayrı ayrı hatırlaması yerine
 * kanca bir kez doğru yazılıyor.
 */
export function useMenu(): {
  acik: boolean;
  ac: () => void;
  kapat: () => void;
  degistir: () => void;
} {
  const [acik, setAcik] = useState(false);
  const ac = useCallback(() => setAcik(true), []);
  const kapat = useCallback(() => setAcik(false), []);
  const degistir = useCallback(() => setAcik((a) => !a), []);
  return { acik, ac, kapat, degistir };
}

interface SatirOzellikleri {
  secili?: boolean;
  tehlike?: boolean;
  onClick: () => void;
  children: ReactNode;
}

/**
 * Menü satırı. Seçili olan tikle işaretleniyor; tik kutusu her satırda var
 * ki etiketler hizadan çıkmasın.
 */
export function MenuSatir({
  secili = false,
  tehlike = false,
  onClick,
  children,
}: SatirOzellikleri) {
  const siniflar = ['menu-satir'];
  if (secili) siniflar.push('menu-satir--secili');
  if (tehlike) siniflar.push('menu-satir--tehlike');

  return (
    <button className={siniflar.join(' ')} role="menuitem" onClick={onClick}>
      <span className="menu-tik">{secili ? <TikMini /> : null}</span>
      <span className="kirp">{children}</span>
    </button>
  );
}

/** Menünün kendi tiki — `Ikonlar`daki büyük tikten daha ince çizgili. */
function TikMini() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2.6}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="m4.8 12.6 4.6 4.6L19.2 7.4" />
    </svg>
  );
}
