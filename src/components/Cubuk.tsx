/**
 * Oynatma çubuğu.
 *
 * Ekranda her an okunması gereken üç şeyin ikisi burada: **ne çalıyor**
 * (soldaki kapak + ad) ve **neresindeyiz** (sürgü + süre). Üçüncüsü —
 * sırada ne var — yan sütunda.
 *
 * # Neden üç bölge
 *
 * Satır bir IZGARA (`1fr auto 1fr`), esnek kutu değil: oynat düğmesi
 * pencerenin GERÇEK ortasında durmalı. Esnek kutuda soldaki dosya adı
 * uzadıkça düğme sağa kayıyordu ve fare kas hafızasıyla gittiği yerde
 * düğmeyi bulamıyordu.
 *
 * # Neden videonun üstüne binmiyor
 *
 * mpv webview'in ÜSTÜNE çiziyor (bkz. `src-tauri/src/mpv/yuzey.rs`), yani
 * pencere kipinde videonun üstüne arayüz koymak teknik olarak mümkün değil.
 * Bu bir kısıt ama iyi bir kısıt oldu — denetimler kaybolup aranmıyor.
 * Tam ekranda video bütün pencereyi kapladığı için çubuk zaten yüzüyor ve
 * fare durunca gizleniyor.
 */

import { memo } from 'react';

import type { PlayerState, QueueSnapshot, Track } from '../ipc/tipler';
import { dilAdi } from '../lib/dil';
import { sureMetni } from '../lib/sure';
import {
  IconAltyazi,
  IconDuraklat,
  IconGeri,
  IconIleri,
  IconKarisik,
  IconMuzik,
  IconOnceki,
  IconOynat,
  IconSes,
  IconSessiz,
  IconSonraki,
  IconTamEkran,
  IconTamEkranCik,
  IconTekrar,
  IconTekrarTek,
  IconVideo,
} from './Ikonlar';
import { Menu, MenuSatir, useMenu } from './Menu';
import { Surgu } from './Surgu';

/** Hız seçenekleri. mpv 0.25–4 arasını kabul ediyor; buradakiler kullanılanlar. */
const HIZLAR = [0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];

/** Altyazı gecikmesinin adımı (saniye). mpv'nin kendi adımı da bu. */
const GECIKME_ADIMI = 0.1;

/** İleri/geri düğmelerinin adımı (saniye). Kısayoldaki 5 sn'den büyük:
 *  düğmeye basan kullanıcı daha büyük bir sıçrama bekliyor. */
const ATLAMA = 10;

interface Ozellikler {
  durum: PlayerState;
  izler: Track[];
  kuyruk: QueueSnapshot;
  gecikme: number;
  tamEkran: boolean;
  onDegistir: () => void;
  onAra: (konum: number) => void;
  onGoreliAra: (delta: number) => void;
  onSes: (ses: number) => void;
  onSessiz: (sessiz: boolean) => void;
  onHiz: (hiz: number) => void;
  onSonraki: () => void;
  onOnceki: () => void;
  onAltyazi: (izId: number | null) => void;
  onGecikme: (saniye: number) => void;
  onKarisik: () => void;
  onTekrar: () => void;
  onTamEkran: () => void;
}

export const Cubuk = memo(function Cubuk(p: Ozellikler) {
  const hiz = useMenu();
  const altyazi = useMenu();

  const { durum, kuyruk } = p;
  const acik = durum.path !== null;
  const pasif = !acik || !durum.engine;
  const altyazilar = p.izler.filter((i) => i.kind === 'sub');
  const seciliAltyazi = altyazilar.find((i) => i.selected) ?? null;
  // Önceki/sonraki tek öğelik kuyrukta anlamsız; tekrar açıksa değil.
  const gezinilebilir = kuyruk.items.length > 1 || kuyruk.repeat !== 'off';

  return (
    <div className={p.tamEkran ? 'cubuk cubuk--yuzen' : 'cubuk'}>
      <div className="cubuk-sira cubuk-ara">
        <span className="sure sure--simdi">{sureMetni(durum.position)}</span>
        <Surgu
          sinif="surgu--ara"
          etiket="Konum"
          metin={sureMetni}
          deger={durum.position}
          enBuyuk={durum.duration}
          onBirak={p.onAra}
          pasif={pasif}
        />
        <span className="sure">{sureMetni(durum.duration)}</span>
      </div>

      <div className="cubuk-denetim">
        {/* -- ne çalıyor -------------------------------------------------- */}
        <div className="cubuk-simdi">
          <div className="cubuk-kapak">
            {durum.mediaType === 'audio' ? <IconMuzik /> : <IconVideo />}
          </div>
          <div className="cubuk-metin">
            <span className="cubuk-ad kirp" title={durum.path ?? undefined}>
              {durum.title ?? 'Bir şey çalmıyor'}
            </span>
            <span className="cubuk-alt kirp">{altSatir(durum, kuyruk)}</span>
          </div>
        </div>

        {/* -- taşıma ------------------------------------------------------ */}
        <div className="cubuk-tasima">
          <TekTus etiket="Önceki" pasif={pasif || !gezinilebilir} onClick={p.onOnceki}>
            <IconOnceki />
          </TekTus>
          <TekTus
            etiket={`${ATLAMA} saniye geri`}
            pasif={pasif}
            onClick={() => p.onGoreliAra(-ATLAMA)}
          >
            <IconGeri />
          </TekTus>

          <button
            className="dugme dugme--oynat"
            onClick={p.onDegistir}
            disabled={pasif}
            aria-label={durum.paused ? 'Oynat' : 'Duraklat'}
            title={durum.paused ? 'Oynat (Boşluk)' : 'Duraklat (Boşluk)'}
          >
            {durum.paused ? <IconOynat /> : <IconDuraklat />}
          </button>

          <TekTus
            etiket={`${ATLAMA} saniye ileri`}
            pasif={pasif}
            onClick={() => p.onGoreliAra(ATLAMA)}
          >
            <IconIleri />
          </TekTus>
          <TekTus etiket="Sonraki" pasif={pasif || !gezinilebilir} onClick={p.onSonraki}>
            <IconSonraki />
          </TekTus>
        </div>

        {/* -- araçlar ----------------------------------------------------- */}
        <div className="cubuk-arac">
          <TekTus
            etiket={kuyruk.shuffle ? 'Karışık: açık' : 'Karışık: kapalı'}
            etkin={kuyruk.shuffle}
            pasif={!durum.engine}
            onClick={p.onKarisik}
          >
            <IconKarisik />
          </TekTus>
          <TekTus
            etiket={
              kuyruk.repeat === 'one'
                ? 'Tekrar: tek parça'
                : kuyruk.repeat === 'all'
                  ? 'Tekrar: liste'
                  : 'Tekrar: kapalı'
            }
            etkin={kuyruk.repeat !== 'off'}
            pasif={!durum.engine}
            onClick={p.onTekrar}
          >
            {kuyruk.repeat === 'one' ? <IconTekrarTek /> : <IconTekrar />}
          </TekTus>

          {/* Altyazı */}
          <div className="menu-sarmal">
            <TekTus
              etiket="Altyazı"
              etkin={seciliAltyazi !== null}
              pasif={pasif}
              onClick={altyazi.degistir}
            >
              <IconAltyazi />
            </TekTus>
            <Menu acik={altyazi.acik} kapat={altyazi.kapat} etiket="Altyazı">
              <div className="menu-baslik">Altyazı</div>
              <MenuSatir
                secili={seciliAltyazi === null}
                onClick={() => {
                  p.onAltyazi(null);
                  altyazi.kapat();
                }}
              >
                Kapalı
              </MenuSatir>
              {altyazilar.map((iz) => (
                <MenuSatir
                  key={iz.id}
                  secili={iz.selected}
                  onClick={() => {
                    p.onAltyazi(iz.id);
                    altyazi.kapat();
                  }}
                >
                  {izAdi(iz)}
                </MenuSatir>
              ))}
              {altyazilar.length === 0 ? (
                <div className="menu-not">
                  Bu dosyada altyazı yok. Yanındaki `.srt` dosyaları açılırken
                  kendiliğinden yükleniyor.
                </div>
              ) : null}

              <div className="menu-ayrac" />
              <div className="menu-baslik">Gecikme</div>
              <div className="menu-adim">
                <button
                  className="dugme dugme--ikon dugme--hayalet"
                  onClick={() => p.onGecikme(p.gecikme - GECIKME_ADIMI)}
                  aria-label="Altyazıyı öne al"
                >
                  −
                </button>
                <span className="menu-adim-deger">{p.gecikme.toFixed(1)} s</span>
                <button
                  className="dugme dugme--ikon dugme--hayalet"
                  onClick={() => p.onGecikme(p.gecikme + GECIKME_ADIMI)}
                  aria-label="Altyazıyı geciktir"
                >
                  +
                </button>
                <button
                  className="dugme dugme--hayalet"
                  onClick={() => p.onGecikme(0)}
                  disabled={p.gecikme === 0}
                >
                  Sıfırla
                </button>
              </div>
            </Menu>
          </div>

          {/* Hız */}
          <div className="menu-sarmal">
            <button
              className={
                durum.rate === 1 ? 'dugme dugme--hayalet' : 'dugme dugme--etkin'
              }
              onClick={hiz.degistir}
              disabled={pasif}
              title="Oynatma hızı"
              aria-label={`Oynatma hızı: ${durum.rate}×`}
            >
              {durum.rate}×
            </button>
            <Menu acik={hiz.acik} kapat={hiz.kapat} etiket="Oynatma hızı">
              <div className="menu-baslik">Oynatma hızı</div>
              {HIZLAR.map((h) => (
                <MenuSatir
                  key={h}
                  // Kayan noktalı karşılaştırma: 1.25 iki farklı yoldan
                  // gelince son bitleri ayrışabiliyor.
                  secili={Math.abs(durum.rate - h) < 0.01}
                  onClick={() => {
                    p.onHiz(h);
                    hiz.kapat();
                  }}
                >
                  {h}×
                </MenuSatir>
              ))}
            </Menu>
          </div>

          {/*
            Ses düğmesi ile sürgüsü tek küme: sürgü yalnız üstüne gelince
            açılıyor (styles.css). Durgun hâlde çubuk sakin kalıyor ama
            sesi değiştirmek için menü açmak gerekmiyor.
          */}
          <div className="ses-kume">
            <TekTus
              etiket={durum.muted ? 'Sesi aç' : 'Sesi kapat'}
              pasif={!durum.engine}
              onClick={() => p.onSessiz(!durum.muted)}
            >
              {durum.muted || durum.volume === 0 ? <IconSessiz /> : <IconSes />}
            </TekTus>
            <Surgu
              sinif="surgu--ses"
              etiket="Ses düzeyi"
              metin={(v) => `yüzde ${Math.round(v)}`}
              deger={durum.muted ? 0 : durum.volume}
              enBuyuk={100}
              adim={1}
              pasif={!durum.engine}
              // Ses sürgüsü sürüklerken de gönderiyor: arama komutunun aksine
              // ses değiştirmek ucuz ve kullanıcı sonucunu ANINDA duymalı.
              onDegisim={p.onSes}
              onBirak={p.onSes}
            />
          </div>

          <TekTus
            etiket={p.tamEkran ? 'Tam ekrandan çık' : 'Tam ekran'}
            pasif={!acik}
            onClick={p.onTamEkran}
          >
            {p.tamEkran ? <IconTamEkranCik /> : <IconTamEkran />}
          </TekTus>
        </div>
      </div>
    </div>
  );
});

interface TusOzellikleri {
  etiket: string;
  pasif?: boolean;
  etkin?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}

function TekTus({ etiket, pasif = false, etkin = false, onClick, children }: TusOzellikleri) {
  return (
    <button
      className={`dugme dugme--ikon ${etkin ? 'dugme--etkin' : 'dugme--hayalet'}`}
      onClick={onClick}
      disabled={pasif}
      title={etiket}
      aria-label={etiket}
    >
      {children}
    </button>
  );
}

/**
 * Kapağın altındaki ikinci satır.
 *
 * Kuyrukta neredeyiz ("3 / 12") en yararlı bilgi; kuyruk yokken dosyanın
 * türü. Hiçbir şey çalmıyorken de bir cümle var — boş bırakmak satırın
 * yüksekliğini oynatıp çubuğu zıplatırdı.
 */
function altSatir(durum: PlayerState, kuyruk: QueueSnapshot): string {
  if (durum.path === null) {
    return durum.engine ? 'Bir dosya aç ya da kütüphaneden seç' : 'Oynatma motoru yok';
  }
  if (kuyruk.currentIndex !== null && kuyruk.items.length > 1) {
    return `Kuyrukta ${kuyruk.currentIndex + 1} / ${kuyruk.items.length}`;
  }
  return durum.mediaType === 'audio' ? 'Ses' : 'Video';
}

/**
 * İz satırının adı.
 *
 * Üç kaynak var ve hepsi boş olabiliyor: başlık, dil, ve son çare olarak
 * numara. "Altyazı 2" bilgisiz ama en azından ayırt edilebilir; boş bir
 * satır seçilemez.
 */
function izAdi(iz: Track): string {
  const dil = dilAdi(iz.lang);
  const parcalar = [dil, iz.title].filter(Boolean);
  const ad = parcalar.length > 0 ? parcalar.join(' · ') : `Altyazı ${iz.id}`;
  return iz.external ? `${ad} · dosya` : ad;
}
