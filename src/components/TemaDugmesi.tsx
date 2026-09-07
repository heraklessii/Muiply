/**
 * Tema düğmesi.
 *
 * İkon, GEÇİLECEK temayı gösteriyor (koyu temadayken güneş): düğmeler
 * bulundukları durumu değil yapacakları işi anlatır.
 *
 * Değer `localStorage`da, backend'de değil (`src/lib/platform.ts`): ilk
 * boyamadan önce bilinmesi gerekiyor ve backend'e sormak pencerenin bir kare
 * yanlış renkte açılması demek olurdu.
 */

import { useTema } from '../hooks/useTema';
import { IconAy, IconGunes } from './Ikonlar';

export function TemaDugmesi() {
  const [tema, sec] = useTema();

  const koyu = tema === 'dark';
  const etiket = koyu ? 'Açık temaya geç' : 'Koyu temaya geç';

  return (
    <button
      className="dugme dugme--ikon dugme--hayalet"
      onClick={() => sec(koyu ? 'light' : 'dark')}
      title={etiket}
      aria-label={etiket}
    >
      {koyu ? <IconGunes /> : <IconAy />}
    </button>
  );
}
