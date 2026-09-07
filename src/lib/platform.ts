/**
 * Çalışma ortamı ve tema.
 *
 * Uygulama hem Tauri penceresinde hem düz tarayıcıda (vite dev) açılabiliyor.
 * Tarayıcıda `invoke` yok: oynatma da kütüphane de çalışmıyor ama arayüzü
 * düzenlerken tarayıcının araçları elimizde kalıyor. Fark tek bir yerde
 * biliniyor — burada.
 */

export function tauriMi(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export type Tema = 'dark' | 'light';

const ANAHTAR = 'muiply-tema';

/**
 * Kayıtlı tema. Kayıt yoksa KOYU.
 *
 * İşletim sisteminin tercihine bakılmıyor: bir medya oynatıcı koyu zeminde
 * açılır, açık tema isteyen açar. Pencere de `tauri.conf.json`'da `Dark`
 * olarak açılıyor; ikisinin ayrışması ilk boyamada beyaz bir çakma üretirdi.
 *
 * Okuma UYGULAMIYOR: düğmeler başlangıç değerini buradan alıyor ve
 * uygulamayı `temayiYukle` bir kez, açılışta yapıyor.
 */
export function temayiOku(): Tema {
  try {
    return localStorage.getItem(ANAHTAR) === 'light' ? 'light' : 'dark';
  } catch {
    // Gizli sekme / kapatılmış depolama. Varsayılanla devam.
    return 'dark';
  }
}

/** Kayıtlı temayı belgeye uygular. Açılışta bir kez (`main.tsx`). */
export function temayiYukle(): Tema {
  const tema = temayiOku();
  uygula(tema);
  return tema;
}

/**
 * AYNI belgedeki tema aboneleri.
 *
 * `storage` olayı yalnız YAZAN belgenin DIŞINDAKİ belgelere düşüyor. Bir
 * pencerede iki ayrı `useTema` var (tepe barındaki düğme ve ayarlardaki
 * açılır kutu) ve ikisi birbirinin değişikliğini hiç görmüyordu: ayarlardan
 * açık temaya geçen kullanıcı, tepe bardaki ikonu hâlâ "açık temaya geç"
 * derken buluyordu. Yazma kapısı tek olduğu için haber verilecek yer de tek.
 */
const aboneler = new Set<(tema: Tema) => void>();

/**
 * Temayı uygular ve kaydeder.
 *
 * Tek yazma kapısı: hem tepe barındaki düğme hem ayarlardaki açılır kutu
 * buradan geçiyor. "Geçiş yap" biçiminde ikinci bir işlev yoktu çünkü
 * açılır kutunun elinde hedef tema zaten var; geçişi hesaplamak çağıranın
 * işi ve tek satır.
 */
export function temayiSec(tema: Tema): Tema {
  uygula(tema);
  try {
    localStorage.setItem(ANAHTAR, tema);
  } catch {
    // Yazılamadıysa tema yine de bu oturumda değişti; ısrar etmiyoruz.
  }
  // Aynı belgedeki öbür aboneler: `storage` onlara ulaşmıyor.
  aboneler.forEach((f) => f(tema));
  return tema;
}

/**
 * Tema değişimini izler — hem ÖBÜR pencerede hem AYNI pencerede.
 *
 * İki kanal var ve ikisi ayrı şeyi çözüyor:
 *
 * - `storage` öbür pencereyi yakalıyor. Muiply iki pencereli ve ikisi aynı
 *   kaynağı paylaşıyor; bu olmadan kütüphanede temayı değiştiren kullanıcı,
 *   oynatıcı penceresine geçtiğinde onu eski temada buluyordu.
 * - [`aboneler`] aynı pencereyi yakalıyor. `storage` yazan belgeye
 *   düşmüyor, yani bir pencerenin içindeki iki `useTema` birbirini
 *   duymuyordu.
 *
 * `storage` bazı gömülü webview'lerde hiç gelmiyor; o durumda pencereler
 * arası eşleme yeniden yüklemeye kalıyor, pencere İÇİNDEKİ eşleme yine
 * çalışıyor.
 *
 * @returns Aboneliği bırakan işlev.
 */
export function temayiIzle(dinleyici: (tema: Tema) => void): () => void {
  const isle = (e: StorageEvent) => {
    if (e.key !== null && e.key !== ANAHTAR) return;
    const tema = temayiOku();
    uygula(tema);
    dinleyici(tema);
  };
  window.addEventListener('storage', isle);
  aboneler.add(dinleyici);
  return () => {
    window.removeEventListener('storage', isle);
    aboneler.delete(dinleyici);
  };
}

function uygula(tema: Tema) {
  document.documentElement.dataset.theme = tema;
}
