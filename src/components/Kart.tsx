/**
 * Kütüphane kartı.
 *
 * Önizleme bandı her kartta VAR ve aynı yükseklikte — birinin bandı olup
 * diğerininki olmayınca ızgarada bütün satır kayıyor (MuiLabs'taki aynı
 * kural). İçeriği ikiye ayrılıyor:
 *
 * | Durum                | Ne çizilir                                  |
 * |----------------------|---------------------------------------------|
 * | Küçük resmi var      | Görsel, `object-fit: cover`                 |
 * | Yok                  | Türün ikonu, soluk, teal yıkamalı zeminde   |
 *
 * İkinci hâl bilerek SOYUT. Temsilî bir kare koymak, dosyanın içinde
 * olmayan bir şeyi göstermek olurdu. Küçük resmi olan kayıtlar
 * *açtıkların* — gerekçe `src-tauri/src/library/kucukresim.rs` başında.
 */

import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";

import type { MediaItem, Playlist } from "../ipc/tipler";
import { sureRozeti } from "../lib/sure";
import { IconCop, IconMuzik, IconVideo } from "./Ikonlar";
import { Menu, MenuSatir } from "./Menu";

interface Ozellikler {
  oge: MediaItem;
  calan: boolean;
  listeler: Playlist[];
  onCal: () => void;
  onListeyeEkle: (listeId: number) => void;
  onKaldir: () => void;
}

export function Kart({ oge, calan, listeler, onCal, onListeyeEkle, onKaldir }: Ozellikler) {
  const [menuAcik, setMenuAcik] = useState(false);
  const sure = sureRozeti(oge.duration);
  const oran = izlenen(oge);

  return (
    <article className={calan ? "kart kart--calan" : "kart"}>
      <div className={oge.thumbnail ? "kart-bant" : "kart-bant kart-bant--soyut"}>
        {oge.thumbnail ? (
          <img
            // `convertFileSrc`: ham dosya yolu `<img src>` içinde çalışmıyor;
            // Tauri'nin `asset:` protokolüne çevrilmesi gerekiyor. İzin
            // kapsamı `tauri.conf.json > assetProtocol.scope` içinde ve
            // yalnız küçük resim klasörünü kapsıyor.
            src={convertFileSrc(oge.thumbnail)}
            alt=""
            // Izgarada yüzlerce kart olabiliyor ve küçük resimler tam
            // çözünürlükte; hepsini birden çözmek pencereyi kilitliyordu.
            loading="lazy"
            decoding="async"
          />
        ) : oge.mediaType === "audio" ? (
          <IconMuzik />
        ) : (
          <IconVideo />
        )}
        {sure ? <span className="kart-sure">{sure}</span> : null}
        {/* Yarım kalma şeridi. Sayı değil şerit: kullanıcının burada sorduğu
            soru "kaçıncı dakikadaydım" değil, "bunu bitirmiş miydim". */}
        {oran === null ? null : (
          <span className="kart-devam" role="presentation" style={{ ["--oran" as string]: oran }} />
        )}
      </div>

      <div className="menu-sarmal kart-menu-sarmal">
        <button
          className="dugme dugme--ikon kart-aksiyon"
          onClick={() => setMenuAcik((a) => !a)}
          aria-expanded={menuAcik}
          aria-label={`${oge.title} için işlemler`}
          title="İşlemler"
        >
          <UcNokta />
        </button>
        <Menu
          acik={menuAcik}
          kapat={() => setMenuAcik(false)}
          yon="asagi"
          etiket="Kart işlemleri"
        >
          <MenuSatir
            onClick={() => {
              onCal();
              setMenuAcik(false);
            }}
          >
            Çal
          </MenuSatir>

          {listeler.length > 0 ? <div className="menu-ayrac" /> : null}
          {listeler.length > 0 ? <div className="menu-baslik">Listeye ekle</div> : null}
          {listeler.map((l) => (
            <MenuSatir
              key={l.id}
              onClick={() => {
                onListeyeEkle(l.id);
                setMenuAcik(false);
              }}
            >
              {l.name}
            </MenuSatir>
          ))}

          <div className="menu-ayrac" />
          <MenuSatir
            onClick={() => {
              onKaldir();
              setMenuAcik(false);
            }}
          >
            <span style={{ display: "inline-flex", alignItems: "center", gap: "var(--space-2)" }}>
              <IconCop style={{ width: 13, height: 13 }} />
              {/* "Sil" DEĞİL: dosya diske dokunulmadan duruyor. */}
              Kütüphaneden kaldır
            </span>
          </MenuSatir>
        </Menu>
      </div>

      <div className="kart-govde">
        <button className="kart-baglanti kirp" onClick={onCal} title={oge.path}>
          {oge.title}
        </button>
        <span className="kart-alt kirp">{altYazi(oge)}</span>
      </div>
    </article>
  );
}

/**
 * İzlenen kesir (0-1) — ya da şerit çizilmeyecekse `null`.
 *
 * Süre bilinmiyorsa `null`: paydası olmayan bir kesir çizilemez. Kayıt
 * eşiklerin dışındaysa backend zaten `null` gönderiyor (`library/devam.rs`),
 * yani burada "yüzde 2 izlenmiş" gibi bir şerit hiç oluşmuyor.
 */
function izlenen(oge: MediaItem): number | null {
  if (oge.lastPosition === null || oge.duration <= 0) return null;
  return Math.min(1, Math.max(0, oge.lastPosition / oge.duration));
}

/**
 * Kartın ikinci satırı.
 *
 * Seste sanatçı, videoda çözünürlük. İkisi de yoksa uzantı — hiçbir şey
 * yazmamak, kartların yüksekliğini oynatıp ızgarayı bozardı.
 */
function altYazi(oge: MediaItem): string {
  if (oge.mediaType === "audio") {
    return oge.artist ?? oge.album ?? oge.extension.toUpperCase();
  }
  if (oge.width && oge.height) {
    return `${oge.width}×${oge.height} · ${oge.extension.toUpperCase()}`;
  }
  return oge.extension.toUpperCase();
}

function UcNokta() {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <circle cx="12" cy="5.5" r="1.7" />
      <circle cx="12" cy="12" r="1.7" />
      <circle cx="12" cy="18.5" r="1.7" />
    </svg>
  );
}
