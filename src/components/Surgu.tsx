/**
 * Sürgü — arama ve ses.
 *
 * `<input type="range">` üstüne kuruluyor: klavye (ok tuşları, Home/End),
 * ekran okuyucu ve dokunma bedavaya geliyor. Kendi çizdiğimiz bir çubukta
 * bunların üçünü de elle yazmak gerekirdi.
 *
 * # Sürüklerken ne oluyor
 *
 * Arama sürgüsü sürüklenirken mpv'ye komut GÖNDERİLMİYOR. Her piksel için
 * bir `seek` göndermek, kod çözücüyü anahtar kareler arasında koşturmak ve
 * arayüzü kilitlemek demek. Sürükleme boyunca değer yerelde tutuluyor,
 * komut bırakıldığında bir kez gidiyor.
 *
 * Aynı sebeple, sürüklerken dışarıdan gelen konum güncellemeleri
 * yoksayılıyor: yoksa parmak sürgüyü ileri çekerken mpv'nin eski konumu onu
 * geri çekiyor ve sürgü titriyor.
 *
 * # Bitişi yakalamak
 *
 * Fare için dinleyici PENCEREDE, sürgünün üstünde değil: kullanıcı topuzu
 * tutup imleci sürgünün dışına çıkarıp bırakabiliyor ve o durumda elemanın
 * kendi `pointerup`ı hiç gelmiyordu — sürgü sürükleniyor durumunda asılı
 * kalıp konum güncellemelerini yok saymaya devam ediyordu.
 *
 * Klavye ayrı: ok tuşu `change` üretiyor ama `pointerup` gelmiyor. Yalnız
 * DEĞERİ DEĞİŞTİREN tuşlar bitişi tetikliyor — Tab ile sürgüden çıkarken
 * boşuna bir `seek` göndermemek için.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

/** Değeri değiştiren tuşlar. Bunların bırakılması komutu gönderiyor. */
const DEGER_TUSLARI = new Set([
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'ArrowDown',
  'Home',
  'End',
  'PageUp',
  'PageDown',
]);

interface Ozellikler {
  /** Dışarıdan gelen değer (mpv'nin bildiği). */
  deger: number;
  enBuyuk: number;
  /** Sürüklerken her adımda — yalnız görsel geri bildirim için. */
  onDegisim?: (deger: number) => void;
  /** Bırakıldığında bir kez. Asıl komut burada gidiyor. */
  onBirak: (deger: number) => void;
  sinif?: string;
  etiket: string;
  /** Ekran okuyucunun okuyacağı biçimli değer ("3:12" gibi). */
  metin?: (deger: number) => string;
  pasif?: boolean;
  adim?: number;
}

export function Surgu({
  deger,
  enBuyuk,
  onDegisim,
  onBirak,
  sinif = '',
  etiket,
  metin,
  pasif = false,
  adim = 0.1,
}: Ozellikler) {
  const [surukleniyor, setSurukleniyor] = useState(false);
  const [yerel, setYerel] = useState(deger);
  const yerelRef = useRef(deger);
  const onBirakRef = useRef(onBirak);
  onBirakRef.current = onBirak;

  // Sürükleme bitince dışarıdaki değere geri dönülüyor. `surukleniyor`
  // bağımlılıkta: sürükleme biterken de bir kez çalışıp mpv'nin gerçek
  // konumunu alsın.
  useEffect(() => {
    if (surukleniyor) return;
    setYerel(deger);
    yerelRef.current = deger;
  }, [deger, surukleniyor]);

  const bitir = useCallback(() => {
    setSurukleniyor((s) => {
      // `setState` içinden okuyoruz: `bitir` kararlı olmalı (pencere
      // dinleyicisi onu bağımlılığında taşıyor) ama en taze `surukleniyor`u
      // görmesi gerekiyor.
      if (s) onBirakRef.current(yerelRef.current);
      return false;
    });
  }, []);

  // Fare/dokunma bırakma: PENCEREDE. Gerekçe dosya başlığında.
  useEffect(() => {
    if (!surukleniyor) return;
    window.addEventListener('pointerup', bitir);
    window.addEventListener('pointercancel', bitir);
    return () => {
      window.removeEventListener('pointerup', bitir);
      window.removeEventListener('pointercancel', bitir);
    };
  }, [surukleniyor, bitir]);

  const gosterilen = surukleniyor ? yerel : deger;
  // Sıfır uzunlukta bir sürgü (süre bilinmiyor) tamamen dolu görünmesin.
  const oran = enBuyuk > 0 ? Math.min(100, Math.max(0, (gosterilen / enBuyuk) * 100)) : 0;

  return (
    <input
      type="range"
      className={`surgu ${sinif}`.trim()}
      style={{ ['--dolu' as string]: `${oran}%` }}
      min={0}
      max={enBuyuk > 0 ? enBuyuk : 1}
      step={adim}
      value={gosterilen}
      disabled={pasif || enBuyuk <= 0}
      aria-label={etiket}
      // Ekran okuyucu ham saniyeyi ("847") değil biçimli hâlini okusun.
      aria-valuetext={metin ? metin(gosterilen) : undefined}
      onChange={(e) => {
        const v = Number(e.target.value);
        setSurukleniyor(true);
        setYerel(v);
        yerelRef.current = v;
        onDegisim?.(v);
      }}
      onKeyUp={(e) => {
        if (DEGER_TUSLARI.has(e.key)) bitir();
      }}
      // Odak kaybı son çare: sürükleme yarıda kaldıysa değeri yine de gönder.
      onBlur={bitir}
    />
  );
}
