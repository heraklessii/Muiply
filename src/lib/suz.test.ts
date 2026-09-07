import { describe, expect, it } from "vitest";

import type { MediaItem } from "../ipc/tipler";
import { ara, dosyaAdi, kucult } from "./suz";

function oge(kismi: Partial<MediaItem>): MediaItem {
  return {
    id: "x",
    path: "C:/m/x.mkv",
    title: "x",
    artist: null,
    album: null,
    duration: 0,
    width: null,
    height: null,
    size: 0,
    mediaType: "video",
    extension: "mkv",
    thumbnail: null,
    addedAt: 0,
    lastPlayed: null,
    playCount: 0,
    lastPosition: null,
    ...kismi,
  };
}

describe("kucult", () => {
  it("Türkçe büyük İ'yi noktasız i'ye çevirmez", () => {
    // Varsayılan toLowerCase burada birleşen nokta bırakıyor ve arama
    // eşleşmiyordu; bu testin tek işi o davranışın geri gelmesini engellemek.
    expect(kucult("İSTANBUL")).toBe("istanbul");
  });

  it("noktasız I'yı ı yapar", () => {
    expect(kucult("IRMAK")).toBe("ırmak");
  });
});

describe("ara", () => {
  const liste = [
    oge({ id: "1", title: "İstanbul Hatırası", path: "C:/m/istanbul.mkv" }),
    oge({ id: "2", title: "Kill Bill", path: "C:/m/Bill.Kill.2003.mkv" }),
    oge({ id: "3", title: "Track 01", artist: "Barış Manço", album: "Nick the Chopper" }),
  ];

  it("boş sorguda hepsini döner", () => {
    expect(ara(liste, "")).toHaveLength(3);
    expect(ara(liste, "   ")).toHaveLength(3);
  });

  it("büyük/küçük harf ve Türkçe karakterden bağımsız", () => {
    expect(ara(liste, "istanbul").map((o) => o.id)).toEqual(["1"]);
    expect(ara(liste, "HATIRASI").map((o) => o.id)).toEqual(["1"]);
  });

  it("kelime sırası önemsiz", () => {
    expect(ara(liste, "kill bill").map((o) => o.id)).toEqual(["2"]);
    expect(ara(liste, "bill kill").map((o) => o.id)).toEqual(["2"]);
  });

  it("sanatçı ve albümde de arar", () => {
    expect(ara(liste, "manço").map((o) => o.id)).toEqual(["3"]);
    expect(ara(liste, "chopper").map((o) => o.id)).toEqual(["3"]);
  });

  it("dosya adında arar", () => {
    // Başlık "Kill Bill" ama kullanıcı dosya adındaki yılı hatırlıyor.
    expect(ara(liste, "2003").map((o) => o.id)).toEqual(["2"]);
  });

  it("eşleşme yoksa boş döner", () => {
    expect(ara(liste, "olmayan")).toEqual([]);
  });

  it("art arda aramalar aynı sonucu verir", () => {
    // Aranacak metin kayıt başına önbelleğe alınıyor; bu test önbelleğin
    // ikinci aramada yanlış cevap vermediğinin bekçisi.
    expect(ara(liste, "istanbul").map((o) => o.id)).toEqual(["1"]);
    expect(ara(liste, "kill").map((o) => o.id)).toEqual(["2"]);
    expect(ara(liste, "istanbul").map((o) => o.id)).toEqual(["1"]);
    expect(ara(liste, "").map((o) => o.id)).toEqual(["1", "2", "3"]);
  });

  it("içeriği aynı iki ayrı kayıt da eşleşir", () => {
    // Önbellek nesne kimliğine bakıyor; aynı içerikli ikinci bir nesne
    // önbellekte yok diye elenmemeli.
    const ikiz = [
      oge({ id: "a", title: "Kill Bill" }),
      oge({ id: "b", title: "Kill Bill" }),
    ];
    expect(ara(ikiz, "kill bill").map((o) => o.id)).toEqual(["a", "b"]);
  });
});

describe("dosyaAdi", () => {
  it("iki ayracı da tanır", () => {
    expect(dosyaAdi(String.raw`C:\Filmler\a.mkv`)).toBe("a.mkv");
    expect(dosyaAdi("/home/x/a.mkv")).toBe("a.mkv");
    expect(dosyaAdi("a.mkv")).toBe("a.mkv");
  });
});
