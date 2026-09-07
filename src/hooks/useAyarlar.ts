/**
 * Ayarlar kancası.
 *
 * Diğer kancalardan bir farkı var: burada **iyimser güncelleme yok.** Yazılan
 * değer backend'de düzeltilebiliyor (aralık dışı hız, tanınmayan `hwdec`) ve
 * ekranda gönderdiğimizi göstermek, kutudaki sayı ile mpv'nin gerçeğinin
 * ayrışması demek olurdu. Dönen değer neyse o gösteriliyor.
 *
 * Ses aygıtları bir KEZ okunuyor. Liste mpv'nin açılışta gördüğü aygıtlar;
 * arada takılan bir kulaklığı görmek için `aygitlariYenile` var, ama kendi
 * kendine yoklamıyoruz — saniyede bir aygıt listesi istemek mpv'yi boşuna
 * meşgul eder.
 */

import { useCallback, useEffect, useState } from "react";

import * as ipc from "../ipc";
import type { AudioDevice, Settings } from "../ipc/tipler";
import { tauriMi } from "../lib/platform";

/** Backend'deki `Settings::default` ile AYNI. İkisi ayrışırsa tarayıcıda
 *  (Tauri dışı) gösterilen değerler uygulamadakinden farklı olur. */
export const VARSAYILAN: Settings = {
  hwdec: "auto-safe",
  audioDevice: "auto",
  defaultRate: 1,
  resume: false,
  mediaKeys: true,
};

export interface AyarlarKanca {
  ayarlar: Settings;
  aygitlar: AudioDevice[];
  /** Yazma sürerken denetimler kilitleniyor: art arda iki kayıt, hangisinin
   *  kazandığı belirsiz bir yarış demek. */
  yaziliyor: boolean;
  yaz: (degisiklik: Partial<Settings>) => Promise<void>;
  aygitlariYenile: () => void;
}

export function useAyarlar(onHata: (e: unknown) => void): AyarlarKanca {
  const [ayarlar, setAyarlar] = useState<Settings>(VARSAYILAN);
  const [aygitlar, setAygitlar] = useState<AudioDevice[]>([]);
  const [yaziliyor, setYaziliyor] = useState(false);

  useEffect(() => {
    if (!tauriMi()) return;
    ipc.ayarlarOku().then(setAyarlar).catch(onHata);
  }, [onHata]);

  const aygitlariYenile = useCallback(() => {
    if (!tauriMi()) return;
    // Hata YUTULMUYOR ama liste boş kalıyor: arayüz o durumda yalnız sistem
    // varsayılanını gösteriyor ve o seçenek her makinede çalışıyor.
    ipc.sesAygitlari().then(setAygitlar).catch(onHata);
  }, [onHata]);

  useEffect(aygitlariYenile, [aygitlariYenile]);

  const yaz = useCallback(
    async (degisiklik: Partial<Settings>) => {
      if (!tauriMi()) {
        // Tarayıcıda backend yok; ayar yalnız bu oturumda değişiyor ki
        // arayüzü düzenlerken denetimler ölü görünmesin.
        setAyarlar((a) => ({ ...a, ...degisiklik }));
        return;
      }
      setYaziliyor(true);
      try {
        setAyarlar(await ipc.ayarlarYaz({ ...ayarlar, ...degisiklik }));
      } catch (e) {
        onHata(e);
      } finally {
        setYaziliyor(false);
      }
    },
    [ayarlar, onHata],
  );

  return { ayarlar, aygitlar, yaziliyor, yaz, aygitlariYenile };
}
