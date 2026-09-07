/**
 * Ayarlar paneli.
 *
 * Altı satır, hepsi tek ekranda, sekme yok. Ayar sayısı bilerek az (bkz.
 * `docs/Roadmap.md` → Kapsam Dışı): mpv'nin bütün seçeneklerini açmak,
 * kullanıcıya bizim yanıtlamamız gereken soruları sordurmak olurdu.
 *
 * Her satırın bir AÇIKLAMASI var. Bir medya oynatıcının ayarları yılda bir
 * kez açılıyor; "hwdec" başlığını görüp ne olduğunu hatırlaması beklenemez.
 *
 * Tema satırı diğerlerinden farklı: değeri backend'de değil `localStorage`da
 * (`src/lib/platform.ts`). Sebep orada — ilk boyamadan önce bilinmesi
 * gerekiyor. Kullanıcı için ikisi de "ayar", ayrım burada görünmüyor.
 */

import { useTema } from "../hooks/useTema";
import type { AudioDevice, Settings } from "../ipc/tipler";
import type { Tema } from "../lib/platform";
import { IconAyar, IconUyari, IconYenile } from "./Ikonlar";

/** Çubuktaki hız menüsüyle AYNI basamaklar: iki yerde iki farklı liste,
 *  kullanıcıya "neden burada 1.6 yok" dedirtir. */
const HIZLAR = [0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];

/** `hwdec` seçenekleri ve karşılıklarının Türkçesi. Sıra bilerek "önerilen
 *  → riskli → kapalı": listede yukarıdan aşağı gidildikçe kullanıcı daha
 *  fazla sorumluluk alıyor. */
const HWDEC: { deger: string; ad: string }[] = [
  { deger: "auto-safe", ad: "Güvenli (önerilen)" },
  { deger: "auto", ad: "Elinden geleni yap" },
  { deger: "no", ad: "Kapalı" },
];

interface Ozellikler {
  ayarlar: Settings;
  aygitlar: AudioDevice[];
  yaziliyor: boolean;
  motorYok: boolean;
  onYaz: (degisiklik: Partial<Settings>) => void;
  onAygitlariYenile: () => void;
}

export function Ayarlar(p: Ozellikler) {
  // Kanca, yerel durum DEĞİL: aynı temayı tepe barındaki düğme ve öbür
  // pencere de değiştiriyor. Kendi kopyasını tutan bir açılır kutu, düğmeye
  // basıldıktan sonra eski değeri göstermeye devam ederdi.
  const [tema, temaSec] = useTema();

  return (
    <div className="icerik">
      <div className="arac-cubugu">
        <IconAyar />
        <span className="alan-etiket">Ayarlar</span>
        <div className="arac-bosluk" />
        {p.yaziliyor ? <span className="rozet">Kaydediliyor…</span> : null}
      </div>

      <div className="ayarlar">
        {/* Motorsuz derlemede mpv ayarlarının hiçbir karşılığı yok. Denetimleri
            kapatmak yerine sebebi yazıyoruz: kapalı bir kutu "bozuk" gibi
            görünür, açıklamalı bir uyarı görünmez. */}
        {p.motorYok ? (
          <div className="ayar-uyari">
            <IconUyari />
            <span>
              Bu derlemede oynatma motoru yok. Aşağıdaki ayarlar kaydediliyor ama
              libmpv kurulu bir sürüm açılana kadar bir etkileri olmayacak.
            </span>
          </div>
        ) : null}

        <Satir
          ad="Kaldığı yerden devam"
          aciklama="Yarım bırakılan bir dosya yeniden açıldığında kalınan yere atlanır. Başa ve sona yakın duraklamalar sayılmıyor; sonuna kadar izlenen dosya baştan başlıyor."
        >
          <Anahtar
            acik={p.ayarlar.resume}
            etiket="Kaldığı yerden devam"
            devre={p.yaziliyor}
            onDegistir={(a) => p.onYaz({ resume: a })}
          />
        </Satir>

        <Satir
          ad="Medya tuşları"
          aciklama="Klavyenin oynat/duraklat, ileri ve geri tuşları Muiply'yi yönetir. Bu tuşları işletim sistemi tek bir uygulamaya veriyor: açıkken başka bir oynatıcıya ulaşmıyorlar."
        >
          <Anahtar
            acik={p.ayarlar.mediaKeys}
            etiket="Medya tuşları"
            devre={p.yaziliyor}
            onDegistir={(a) => p.onYaz({ mediaKeys: a })}
          />
        </Satir>

        <Satir
          ad="Varsayılan hız"
          aciklama="Her dosyanın başladığı hız. Çubuktan yapılan değişiklik yalnız o dosya için geçerli."
        >
          <select
            value={String(p.ayarlar.defaultRate)}
            disabled={p.yaziliyor}
            onChange={(e) => p.onYaz({ defaultRate: Number(e.target.value) })}
            aria-label="Varsayılan oynatma hızı"
          >
            {/* Kayıtlı değer listede yoksa (elle yazılmış ya da eski bir
                sürümden kalmış) kendi satırı ekleniyor; yoksa açılır kutu
                başka bir hız gösterip kullanıcıyı yanıltırdı. */}
            {(HIZLAR.includes(p.ayarlar.defaultRate)
              ? HIZLAR
              : [...HIZLAR, p.ayarlar.defaultRate].sort((a, b) => a - b)
            ).map((h) => (
              <option key={h} value={String(h)}>
                {h}×
              </option>
            ))}
          </select>
        </Satir>

        <Satir
          ad="Donanım kod çözme"
          aciklama="Videoyu ekran kartına çözdürür: işlemciyi ve pili rahatlatır. Görüntüde bozulma görürseniz kapatın."
        >
          <select
            value={p.ayarlar.hwdec}
            disabled={p.yaziliyor}
            onChange={(e) => p.onYaz({ hwdec: e.target.value })}
            aria-label="Donanım kod çözme"
          >
            {HWDEC.map((s) => (
              <option key={s.deger} value={s.deger}>
                {s.ad}
              </option>
            ))}
          </select>
        </Satir>

        <Satir
          ad="Ses aygıtı"
          aciklama="Sesin gideceği çıkış. Sistem varsayılanı, işletim sisteminde seçili olanı izler."
        >
          <span className="ayar-denetim">
            <select
              value={p.ayarlar.audioDevice}
              disabled={p.yaziliyor}
              onChange={(e) => p.onYaz({ audioDevice: e.target.value })}
              aria-label="Ses aygıtı"
            >
              <option value="auto">Sistem varsayılanı</option>
              {p.aygitlar
                // mpv listede kendi "auto" girdisini de veriyor; iki kez
                // görünmesin.
                .filter((a) => a.name !== "auto")
                .map((a) => (
                  <option key={a.name} value={a.name}>
                    {a.description}
                  </option>
                ))}
              {/* Kayıtlı aygıt artık takılı değilse seçim boşa düşerdi ve
                  kutu ilk satırı gösterip "sistem varsayılanı seçili"
                  yalanını söylerdi. */}
              {p.ayarlar.audioDevice !== "auto" &&
              !p.aygitlar.some((a) => a.name === p.ayarlar.audioDevice) ? (
                <option value={p.ayarlar.audioDevice}>
                  {p.ayarlar.audioDevice} (bağlı değil)
                </option>
              ) : null}
            </select>
            <button
              className="dugme dugme--ikon dugme--hayalet"
              onClick={p.onAygitlariYenile}
              title="Aygıtları yeniden tara"
              aria-label="Ses aygıtlarını yeniden tara"
            >
              <IconYenile />
            </button>
          </span>
        </Satir>

        <Satir ad="Tema" aciklama="Koyu tema bir oynatıcının varsayılanı: film siyah zeminde izlenir.">
          <select
            value={tema}
            onChange={(e) => temaSec(e.target.value as Tema)}
            aria-label="Tema"
          >
            <option value="dark">Koyu</option>
            <option value="light">Açık</option>
          </select>
        </Satir>
      </div>
    </div>
  );
}

interface SatirOzellikleri {
  ad: string;
  aciklama: string;
  children: React.ReactNode;
}

function Satir({ ad, aciklama, children }: SatirOzellikleri) {
  return (
    <div className="ayar-satir">
      <div className="ayar-metin">
        <div className="ayar-ad">{ad}</div>
        <p className="ayar-aciklama">{aciklama}</p>
      </div>
      {children}
    </div>
  );
}

interface AnahtarOzellikleri {
  acik: boolean;
  etiket: string;
  devre: boolean;
  onDegistir: (acik: boolean) => void;
}

/**
 * Açık/kapalı anahtarı.
 *
 * `<input type="checkbox">` değil `role="switch"` taşıyan bir düğme:
 * onay kutusu "bir form gönderilecek" demek, oysa buradaki değişiklik
 * tıklandığı anda kaydediliyor.
 */
function Anahtar({ acik, etiket, devre, onDegistir }: AnahtarOzellikleri) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={acik}
      aria-label={etiket}
      className={acik ? "anahtar anahtar--acik" : "anahtar"}
      disabled={devre}
      onClick={() => onDegistir(!acik)}
    >
      <span className="anahtar-topuz" />
    </button>
  );
}
