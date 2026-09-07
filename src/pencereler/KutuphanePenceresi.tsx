/**
 * Kütüphane penceresi — koleksiyon, listeler, kuyruk, ayarlar.
 *
 * Video BURADA çizilmiyor: mpv'nin yüzeyi oynatıcı penceresine bağlı
 * (`src-tauri/src/pencere.rs`). Buradan bir şey çalmaya başlandığında
 * pencereyi backend gösteriyor — video ise kendiliğinden, ses ise hiç
 * (müzik dinlerken kütüphanede gezinmek doğru davranış).
 *
 * Alt çubuk yine de burada: çalanı duraklatmak için pencere değiştirmek
 * gerekmesin.
 */

import { useCallback, useEffect, useState } from "react";
import { open as dosyaSec } from "@tauri-apps/plugin-dialog";

import { Ayarlar } from "../components/Ayarlar";
import { Bildirimler, TaramaSeridi } from "../components/Bildirimler";
import { Birak } from "../components/Birak";
import { Cubuk } from "../components/Cubuk";
import { Kisayollar } from "../components/Kisayollar";
import { Kutuphane } from "../components/Kutuphane";
import { KuyrukPaneli } from "../components/KuyrukPaneli";
import { ListePaneli } from "../components/ListePaneli";
import { Tepe } from "../components/Tepe";
import { Yan, type Gorunum } from "../components/Yan";
import { useAyarlar } from "../hooks/useAyarlar";
import { useBildirimler } from "../hooks/useBildirimler";
import { useBirakma } from "../hooks/useBirakma";
import { useKumanda } from "../hooks/useKumanda";
import { useKutuphane } from "../hooks/useKutuphane";
import { useKuyruk } from "../hooks/useKuyruk";
import { useListeler } from "../hooks/useListeler";
import { useOynatici } from "../hooks/useOynatici";
import * as ipc from "../ipc";
import type { MediaItem } from "../ipc/tipler";

export function KutuphanePenceresi() {
  const { bildirimler, bildir, bildirHata, kapat } = useBildirimler();

  const oynatici = useOynatici(bildirHata);
  const kutuphane = useKutuphane(bildirHata);
  const kuyrukK = useKuyruk(bildirHata);
  const listeler = useListeler(bildirHata);
  const ayarlar = useAyarlar(bildirHata);
  const kumanda = useKumanda({
    oynatici,
    kuyruk: kuyrukK,
    bildirHata,
    bildir,
    videolu: false,
  });

  const [gorunum, setGorunum] = useState<Gorunum>({ tur: "kutuphane" });
  const { durum } = oynatici;
  const { sar } = kumanda;

  const birakiliyor = useBirakma(bildirHata, () => {
    kutuphane.yenile();
    listeler.yenile();
  });

  /* -- eylemler --------------------------------------------------------- */

  const klasorEkle = useCallback(async () => {
    try {
      const secim = await dosyaSec({ directory: true, multiple: false });
      if (typeof secim === "string") {
        await kutuphane.klasorEkle(secim);
        bildir("Klasör eklendi, tarama başladı.");
      }
    } catch (e) {
      bildirHata(e);
    }
  }, [bildir, bildirHata, kutuphane]);

  const dosyaAc = useCallback(async () => {
    try {
      const secim = await dosyaSec({ multiple: false, directory: false });
      if (typeof secim === "string") await ipc.oynaticiAc(secim);
    } catch (e) {
      bildirHata(e);
    }
  }, [bildirHata]);

  /** Izgaradan çalma: görünen bütün kayıtlar kuyruğa, tıklanandan başla. */
  const izgaradanCal = useCallback(
    (gorunen: MediaItem[], indeks: number) => {
      sar(() => ipc.kuyrukKur(gorunen.map((o) => o.id), indeks));
    },
    [sar],
  );

  /* -- yan etkiler ------------------------------------------------------ */

  const acikListe =
    gorunum.tur === "liste" ? listeler.listeler.find((l) => l.id === gorunum.id) : undefined;

  // Açık liste silinmişse (başka bir yoldan) ana alanı boş bırakmak yerine
  // kütüphaneye dön.
  useEffect(() => {
    if (gorunum.tur === "liste" && listeler.listeler.length > 0 && !acikListe) {
      setGorunum({ tur: "kutuphane" });
    }
  }, [acikListe, gorunum, listeler.listeler.length]);

  useEffect(() => {
    listeler.ac(gorunum.tur === "liste" ? gorunum.id : null);
  }, [gorunum]);

  /* -- çizim ------------------------------------------------------------ */

  return (
    <div className="kabuk">
      <Tepe
        durum={durum}
        onDosyaAc={dosyaAc}
        onKlasorEkle={klasorEkle}
        onKisayollar={kumanda.kisayollariDegistir}
      />

      <Yan
        gorunum={gorunum}
        tur={kutuphane.tur}
        listeler={listeler.listeler}
        klasorler={kutuphane.klasorler}
        kuyruk={kuyrukK.kuyruk}
        calan={durum.path !== null}
        onGorunum={setGorunum}
        onOynatici={() => sar(ipc.pencereOynatici)}
        onTur={kutuphane.setTur}
        onYeniListe={() => {
          const ad = window.prompt("Çalma listesinin adı");
          if (!ad) return;
          listeler.olustur(ad).then((l) => {
            if (l) setGorunum({ tur: "liste", id: l.id });
          });
        }}
        onKlasorEkle={klasorEkle}
        onKlasorSil={kutuphane.klasorSil}
        onTara={kutuphane.tara}
      />

      <main className="ana">
        {/* Tarama bir bildirim DEĞİL, süren bir iş: ana alanın üstünde kendi
            satırında duruyor ve yer kaplıyor (bkz. `Bildirimler.tsx`). */}
        <TaramaSeridi tarama={kutuphane.tarama} />

        {gorunum.tur === "ayarlar" ? (
          <Ayarlar
            ayarlar={ayarlar.ayarlar}
            aygitlar={ayarlar.aygitlar}
            yaziliyor={ayarlar.yaziliyor}
            motorYok={!durum.engine}
            onYaz={ayarlar.yaz}
            onAygitlariYenile={ayarlar.aygitlariYenile}
          />
        ) : gorunum.tur === "kuyruk" ? (
          <KuyrukPaneli kuyruk={kuyrukK.kuyruk} onCal={kuyrukK.cal} />
        ) : gorunum.tur === "liste" && acikListe ? (
          <ListePaneli
            ad={acikListe.name}
            ogeler={listeler.ogeler}
            calanYol={durum.path}
            onCal={(i) => sar(() => ipc.listeCal(acikListe.id, i))}
            onAdDegistir={(ad) => listeler.adDegistir(acikListe.id, ad)}
            onSil={() => listeler.sil(acikListe.id)}
            onOgeSil={(ogeId) => listeler.ogeSil(acikListe.id, ogeId)}
            onSirala={(a, b) => listeler.sirala(acikListe.id, a, b)}
          />
        ) : (
          <Kutuphane
            ogeler={kutuphane.ogeler}
            listeler={listeler.listeler}
            tur={kutuphane.tur}
            siralama={kutuphane.siralama}
            calanYol={durum.path}
            klasorVar={kutuphane.klasorler.length > 0}
            onSiralama={kutuphane.setSiralama}
            onCal={izgaradanCal}
            onListeyeEkle={listeler.ogeEkle}
            onKaldir={kutuphane.kayitSil}
            onKlasorEkle={klasorEkle}
          />
        )}

        <Cubuk
          durum={durum}
          izler={oynatici.izler}
          kuyruk={kuyrukK.kuyruk}
          gecikme={kumanda.gecikme}
          tamEkran={false}
          onDegistir={() => sar(ipc.oynaticiDegistir)}
          onAra={(k) => sar(() => ipc.oynaticiAra(k))}
          onGoreliAra={(d) => sar(() => ipc.oynaticiGoreliAra(d))}
          onSes={(s) => sar(() => ipc.oynaticiSes(s))}
          onSessiz={(s) => sar(() => ipc.oynaticiSessiz(s))}
          onHiz={(h) => sar(() => ipc.oynaticiHiz(h))}
          onSonraki={kuyrukK.sonraki}
          onOnceki={kuyrukK.onceki}
          onAltyazi={(id) =>
            sar(async () => {
              await ipc.altyaziSec(id);
              oynatici.yenile();
            })
          }
          onGecikme={kumanda.gecikmeAyarla}
          onKarisik={kuyrukK.karisikDegistir}
          onTekrar={kuyrukK.tekrarDegistir}
          onTamEkran={() => kumanda.tamEkranDegistir()}
        />

        <Bildirimler bildirimler={bildirimler} kapat={kapat} />
      </main>

      <Kisayollar acik={kumanda.kisayollar} kapat={kumanda.kisayollariKapat} />
      {birakiliyor ? <Birak /> : null}
    </div>
  );
}
