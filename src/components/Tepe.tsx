/**
 * Üst bar.
 *
 * İki pencerede de var ama aynı şeyi anlatmıyor:
 *
 * - **Kütüphanede** kimlik barı: marka ve kütüphaneye bir şey ekleyen
 *   düğmeler. Çalan dosyanın adı burada DEĞİL, alt çubukta — orada zaten
 *   kapağıyla ve süresiyle duruyor, iki yerde yazmak yer israfı.
 * - **Oynatıcıda** çalan dosyanın adı. Orada alt çubuk tam ekranda
 *   kayboluyor ve ad başka bir yerde görünmüyor.
 *
 * Bar sabit yükseklikte ve asla sarmıyor; daralınca kısalan şey dosya adı,
 * düğmeler değil.
 */

import type { PlayerState } from '../ipc/tipler';
import { sureMetni } from '../lib/sure';
import { IconKlasor, IconKlavye, IconListe, IconMuiply, IconVideo } from './Ikonlar';
import { TemaDugmesi } from './TemaDugmesi';

interface Ozellikler {
  /** Oynatıcı penceresinde çalan dosya gösteriliyor; kütüphanede marka. */
  oynaticiMi?: boolean;
  durum: PlayerState;
  onDosyaAc: () => void;
  /** Yalnız kütüphane penceresinde: klasör ekleme oranın işi. */
  onKlasorEkle?: () => void;
  /** Yalnız oynatıcı penceresinde: öbür pencereye geçiş. */
  onKutuphane?: () => void;
  onKisayollar: () => void;
}

export function Tepe({
  oynaticiMi = false,
  durum,
  onDosyaAc,
  onKlasorEkle,
  onKutuphane,
  onKisayollar,
}: Ozellikler) {
  return (
    <header className={oynaticiMi ? 'tepe tepe--oynatici' : 'tepe'}>
      <div className="marka">
        <IconMuiply className="marka-ikon" />
        {oynaticiMi ? null : <span className="marka-ad">Muiply</span>}
      </div>

      {oynaticiMi ? (
        <div className="tepe-baslik">
          {durum.path ? (
            <>
              <span className="tepe-ad kirp secilebilir" title={durum.path}>
                {durum.title ?? durum.path}
              </span>
              {/* Süre bilinmiyorsa hiç yazılmıyor: "0:00" yanlış bilgi. */}
              {durum.duration > 0 ? (
                <span className="tepe-alt">{sureMetni(durum.duration)}</span>
              ) : null}
            </>
          ) : (
            <span className="tepe-alt">Bir şey çalmıyor</span>
          )}
        </div>
      ) : null}

      <div className="tepe-eylem">
        <button className="dugme dugme--hayalet" onClick={onDosyaAc}>
          <IconVideo />
          Dosya aç
        </button>

        {onKlasorEkle ? (
          <button className="dugme dugme--hayalet" onClick={onKlasorEkle}>
            <IconKlasor />
            Klasör ekle
          </button>
        ) : null}

        {onKutuphane ? (
          <button className="dugme dugme--hayalet" onClick={onKutuphane}>
            <IconListe />
            Kütüphane
          </button>
        ) : null}

        <span className="tepe-ayrac" aria-hidden="true" />

        <button
          className="dugme dugme--ikon dugme--hayalet"
          onClick={onKisayollar}
          title="Klavye kısayolları (?)"
          aria-label="Klavye kısayolları"
        >
          <IconKlavye />
        </button>
        <TemaDugmesi />
      </div>
    </header>
  );
}
