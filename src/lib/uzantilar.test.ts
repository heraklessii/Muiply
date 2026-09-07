/**
 * Uzantı listelerinin ayrışmasına karşı bekçi.
 *
 * Aynı liste ÜÇ yerde duruyor ve üçü de farklı bir işi yapıyor:
 *
 * - `src-tauri/src/library/mod.rs` — kütüphane taramasının aldığı dosyalar,
 * - `src-tauri/tauri.conf.json` — kurulumun işletim sistemine bildirdiği
 *   ilişkilendirmeler ("Varsayılan uygulamalar" listesi),
 * - `src/lib/uzantilar.ts` — dosya seçme diyaloğundaki süzgeç.
 *
 * Ayrıştıklarında belirti kullanıcı tarafında görünüyor ve sebebi hiç
 * göstermiyor: diyalogda seçilebilen ama kütüphaneye girmeyen bir dosya,
 * ya da çift tıklandığında açılan ama ızgarada hiç çıkmayan bir tür.
 * Kural CLAUDE.md'de yazılı (13) — ama yazılı bir kural unutulabiliyor.
 *
 * İlk iki listenin birbirine eşitliğini `cargo test` sınıyor
 * (`library::testler`); bu dosya üçüncüyü onlara bağlıyor.
 */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { MEDYA_UZANTILARI } from "./uzantilar";

const kok = fileURLToPath(new URL("../../", import.meta.url));

/** `library/mod.rs`teki bir `&[&str]` sabitinin içindeki uzantılar. */
function rustListesi(ad: string): string[] {
  const kaynak = readFileSync(`${kok}src-tauri/src/library/mod.rs`, "utf8");

  // Düzenli ifade yerine düz dizi araması: kalıptaki `[` ve `]`i hem şablon
  // dizesinden hem RegExp'ten kaçırmak, okunması güç ve sessizce bozulan
  // bir kalıp üretiyordu (kaçış şablon dizesinde tükenip karakter sınıfı
  // bozuluyor).
  const bas = kaynak.indexOf(`pub const ${ad}`);
  if (bas < 0) throw new Error(`${ad} sabiti library/mod.rs içinde bulunamadı`);

  // `= &[` aranıyor, düz `&[` değil: sabitin TİPİ de `&[&str]` ve ilk
  // eşleşme oraya düşüyordu — gövde `&[&str` olarak kesilip liste boş
  // çıkıyordu. Boş listeyi boş listeyle karşılaştıran bir bekçi sessizce
  // geçerdi; aşağıdaki "okuyucu gerçekten uzantı buluyor" testi bunun için.
  const govdeBas = kaynak.indexOf("= &[", bas);
  const govdeSon = kaynak.indexOf("]", govdeBas);
  if (govdeBas < 0 || govdeSon < 0) throw new Error(`${ad} gövdesi ayrıştırılamadı`);

  const govde = kaynak.slice(govdeBas, govdeSon);
  return [...govde.matchAll(/"([^"]+)"/g)].map((m) => m[1]);
}

describe("uzantı listeleri", () => {
  it("Rust'taki tarama listesiyle birebir aynı", () => {
    const rust = [...rustListesi("VIDEO_UZANTILARI"), ...rustListesi("SES_UZANTILARI")];
    // Sıra da aynı: dosyalar okunurken kaynak sırayı koruyor ve iki listeyi
    // yan yana okuyabilmek karşılaştırmayı kolaylaştırıyor.
    expect(MEDYA_UZANTILARI).toEqual(rust);
  });

  it("okuyucu gerçekten uzantı buluyor", () => {
    // Bekçinin kendisi bozulabilir ve bozulduğunda SESSİZ: ayrıştırma
    // yanlış yerden keserse yukarıdaki test iki boş listeyi karşılaştırıp
    // geçerdi. Bu test yazılırken tam olarak o oldu — arama sabitin
    // gövdesini değil TİPİNİ (`&[&str]`) buluyordu.
    expect(rustListesi("VIDEO_UZANTILARI")).toContain("mkv");
    expect(rustListesi("SES_UZANTILARI")).toContain("flac");
  });

  it("aynı uzantı iki kez geçmiyor", () => {
    expect(new Set(MEDYA_UZANTILARI).size).toBe(MEDYA_UZANTILARI.length);
  });
});
