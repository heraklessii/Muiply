/**
 * Altyazı dil kodlarının Türkçe adları.
 *
 * Liste KISA ve bilerek: bütün ISO 639 tablosunu gömmek yüz kilobayt ve
 * kullanıcının göreceği dillerin çoğu bu on beşin içinde. Tanınmayan kod
 * ham hâliyle gösteriliyor — "zza" yazan bir satır, yanlış çevrilmiş bir
 * addan iyi.
 */

const ADLAR: Record<string, string> = {
  tr: "Türkçe",
  tur: "Türkçe",
  en: "İngilizce",
  eng: "İngilizce",
  de: "Almanca",
  ger: "Almanca",
  deu: "Almanca",
  fr: "Fransızca",
  fra: "Fransızca",
  es: "İspanyolca",
  spa: "İspanyolca",
  it: "İtalyanca",
  ita: "İtalyanca",
  ru: "Rusça",
  rus: "Rusça",
  ar: "Arapça",
  ara: "Arapça",
  ja: "Japonca",
  jpn: "Japonca",
  ko: "Korece",
  kor: "Korece",
  zh: "Çince",
  chi: "Çince",
  zho: "Çince",
  pt: "Portekizce",
  por: "Portekizce",
  nl: "Felemenkçe",
  nld: "Felemenkçe",
  el: "Yunanca",
  fa: "Farsça",
  ku: "Kürtçe",
};

export function dilAdi(kod: string | null): string | null {
  if (!kod) return null;
  const k = kod.toLowerCase();
  return ADLAR[k] ?? kod;
}
