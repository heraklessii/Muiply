/**
 * Süre ve boyut biçimleme.
 *
 * Saf ve testli: sürgünün yanındaki sayı her saniye yeniden çiziliyor ve
 * yanlış biçim burada sessiz bir hata — "1:5" yazan bir oynatıcı bozuk
 * görünür ama hiçbir yerde hata basmaz.
 */

/**
 * Saniyeyi `s:ss` ya da `sa:dd:ss` biçimine çevirir.
 *
 * Bir saatin altındaki süreler saat hanesi TAŞIMIYOR: üç haneli bir sayı
 * (`0:03:21`) çoğu dosyada boşuna yer kaplıyor ve sürgünün iki yanındaki
 * yazıların genişliği oynadıkça sürgü de kayıyor.
 */
export function sureMetni(saniye: number): string {
  if (!Number.isFinite(saniye) || saniye < 0) return "0:00";

  const toplam = Math.floor(saniye);
  const sa = Math.floor(toplam / 3600);
  const dd = Math.floor((toplam % 3600) / 60);
  const ss = toplam % 60;

  const iki = (n: number) => String(n).padStart(2, "0");
  return sa > 0 ? `${sa}:${iki(dd)}:${iki(ss)}` : `${dd}:${iki(ss)}`;
}

/**
 * Süresi bilinmeyen kayıtlar için.
 *
 * Kütüphanede süre `0` iki farklı şey olabiliyor: gerçekten sıfır uzunlukta
 * bir dosya (yok denecek kadar seyrek) ya da sondanın okuyamamış olması
 * (motorsuz derleme, bozuk dosya). İkisini de "0:00" yazmak yanlış bilgi;
 * kısa çizgi "bilmiyoruz" diyor.
 */
export function sureRozeti(saniye: number): string | null {
  if (!Number.isFinite(saniye) || saniye <= 0) return null;
  return sureMetni(saniye);
}

/** Bayt sayısını okunur hâle getirir. */
export function boyutMetni(bayt: number): string {
  if (!Number.isFinite(bayt) || bayt <= 0) return "—";

  const birimler = ["B", "KB", "MB", "GB", "TB"];
  let deger = bayt;
  let i = 0;
  while (deger >= 1024 && i < birimler.length - 1) {
    deger /= 1024;
    i += 1;
  }
  // Bayt ve KB'de ondalık gereksiz; MB'den sonra bir hane anlamlı.
  const basamak = i <= 1 ? 0 : 1;
  return `${deger.toFixed(basamak)} ${birimler[i]}`;
}

/**
 * Sürgünün dolu kısmının yüzdesi.
 *
 * Süre bilinmiyorken (`0`) sürgü boş kalıyor — yarısına kadar dolu bir
 * sürgü, konumun bilindiği izlenimi verirdi.
 */
export function yuzde(konum: number, sure: number): number {
  if (!Number.isFinite(konum) || !Number.isFinite(sure) || sure <= 0) return 0;
  return Math.min(100, Math.max(0, (konum / sure) * 100));
}
