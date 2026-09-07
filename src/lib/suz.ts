/**
 * Kütüphane araması — istemci tarafında.
 *
 * Sıralama ve tür süzgeci SQL'de (`library/db.rs`); arama burada. Sebep
 * `docs/Frontend.md`'de: her tuş vuruşunda IPC'ye çıkmak, bellekteki bir
 * diziyi süzmenin yanında bedava değil ve sonuç aynı.
 */

import type { MediaItem } from "../ipc/tipler";

/**
 * Türkçe duyarlı küçük harfe çevirme.
 *
 * `"İSTANBUL".toLowerCase()` JavaScript'te `"i̇stanbul"` üretiyor — `i`nin
 * ardına ayrı bir birleşen nokta koyuyor — ve `"istanbul"` ile eşleşmiyor.
 * `"I".toLowerCase()` de `"i"` veriyor, oysa Türkçede `"ı"` olmalı.
 * Arama kutusuna "istanbul" yazan biri "İSTANBUL.mkv"i bulamıyordu.
 */
export function kucult(metin: string): string {
  return metin.toLocaleLowerCase("tr");
}

/**
 * Kayıt başına aranacak metnin önbelleği.
 *
 * `ara` her tuş vuruşunda çağrılıyor ve `toLocaleLowerCase("tr")` yerel
 * duyarlı olduğu için ucuz değil: beş bin kayıtlık bir kütüphanede sorgunun
 * her harfi beş bin dize birleştirme + beş bin yerel çevrimi demekti. Metin
 * kaydın kendisine bağlı, sorguya değil — bir kez üretilip saklanıyor.
 *
 * `WeakMap`: anahtar IPC'den gelen kayıt nesnesi. Kütüphane yenilendiğinde
 * eski nesneler çöp toplayıcıya gidiyor, önbellek onlarla birlikte
 * boşalıyor — elle temizlenecek bir şey yok.
 */
const samanlik = new WeakMap<MediaItem, string>();

function saman(oge: MediaItem): string {
  const hazir = samanlik.get(oge);
  if (hazir !== undefined) return hazir;

  const uretilen = kucult(
    [oge.title, oge.artist ?? "", oge.album ?? "", dosyaAdi(oge.path)].join(" "),
  );
  samanlik.set(oge, uretilen);
  return uretilen;
}

/**
 * Sorguyu başlıkta, sanatçıda, albümde ve dosya adında arar.
 *
 * Sorgu boşluklarla bölünüyor ve HER parça bulunmak zorunda ("kelimelerin
 * hepsi geçsin"). Sıra önemli değil: kullanıcı "kill bill" yazdığında
 * "Bill.Kill.2003.mkv" de gelsin.
 */
export function ara(ogeler: MediaItem[], sorgu: string): MediaItem[] {
  const parcalar = kucult(sorgu).split(/\s+/).filter(Boolean);
  if (parcalar.length === 0) return ogeler;

  return ogeler.filter((oge) => {
    const metin = saman(oge);
    return parcalar.every((p) => metin.includes(p));
  });
}

/** Yoldan dosya adını ayıklar. Windows ve POSIX ayracı bir arada olabiliyor. */
export function dosyaAdi(yol: string): string {
  const parcalar = yol.split(/[\\/]/);
  return parcalar[parcalar.length - 1] ?? yol;
}
