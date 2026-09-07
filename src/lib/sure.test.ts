import { describe, expect, it } from "vitest";

import { boyutMetni, sureMetni, sureRozeti, yuzde } from "./sure";

describe("sureMetni", () => {
  it("bir saatin altında saat hanesi yazmaz", () => {
    expect(sureMetni(0)).toBe("0:00");
    expect(sureMetni(5)).toBe("0:05");
    expect(sureMetni(65)).toBe("1:05");
    expect(sureMetni(3599)).toBe("59:59");
  });

  it("bir saatten uzunda saat hanesi ekler", () => {
    expect(sureMetni(3600)).toBe("1:00:00");
    expect(sureMetni(3661)).toBe("1:01:01");
    expect(sureMetni(36000)).toBe("10:00:00");
  });

  it("saniyeyi aşağı yuvarlar", () => {
    // mpv `time-pos`u ondalıklı veriyor; 9.99 saniye henüz 10 değil.
    expect(sureMetni(9.99)).toBe("0:09");
  });

  it("anlamsız girdide sıfır gösterir", () => {
    // Dosya yokken mpv `duration`ı okuyamıyor ve NaN geliyor; sürgünün
    // yanında "NaN:NaN" görmek istemiyoruz.
    expect(sureMetni(Number.NaN)).toBe("0:00");
    expect(sureMetni(-5)).toBe("0:00");
    expect(sureMetni(Number.POSITIVE_INFINITY)).toBe("0:00");
  });
});

describe("sureRozeti", () => {
  it("bilinmeyen süre için null döner", () => {
    expect(sureRozeti(0)).toBeNull();
    expect(sureRozeti(Number.NaN)).toBeNull();
  });

  it("bilinen süreyi biçimler", () => {
    expect(sureRozeti(90)).toBe("1:30");
  });
});

describe("boyutMetni", () => {
  it("birimi büyütür", () => {
    expect(boyutMetni(512)).toBe("512 B");
    expect(boyutMetni(2048)).toBe("2 KB");
    expect(boyutMetni(5 * 1024 * 1024)).toBe("5.0 MB");
    expect(boyutMetni(3 * 1024 ** 3)).toBe("3.0 GB");
  });

  it("bilinmeyen boyutta çizgi gösterir", () => {
    expect(boyutMetni(0)).toBe("—");
    expect(boyutMetni(Number.NaN)).toBe("—");
  });
});

describe("yuzde", () => {
  it("konumu oranlar", () => {
    expect(yuzde(50, 100)).toBe(50);
    expect(yuzde(0, 100)).toBe(0);
  });

  it("süre bilinmiyorken sıfır", () => {
    expect(yuzde(30, 0)).toBe(0);
  });

  it("aralık dışına taşmaz", () => {
    // mpv `time-pos`u dosya sonunda süreyi bir tık aşabiliyor.
    expect(yuzde(101, 100)).toBe(100);
    expect(yuzde(-1, 100)).toBe(0);
  });
});
