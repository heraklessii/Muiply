/**
 * Tema kancası.
 *
 * İki yerden kullanılıyor: tepe barındaki düğme ve ayarlar panelindeki
 * açılır kutu. Ortak olmasının sebebi öbür pencere: Muiply iki pencereli ve
 * biri temayı değiştirdiğinde diğerinin de dönmesi gerekiyor. O aboneliği
 * iki bileşende ayrı ayrı yazmak, birini unutmak demekti.
 */

import { useEffect, useState } from 'react';

import { temayiIzle, temayiOku, temayiSec, type Tema } from '../lib/platform';

export function useTema(): [Tema, (t: Tema) => void] {
  const [tema, setTema] = useState<Tema>(temayiOku);

  // Öbür pencerede değişirse buradaki düğme de dönsün.
  useEffect(() => temayiIzle(setTema), []);

  const sec = (yeni: Tema) => {
    if (yeni === tema) return;
    setTema(temayiSec(yeni));
  };

  return [tema, sec];
}
