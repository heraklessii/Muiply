/**
 * Oynatıcı penceresi — video ve kumandası, başka hiçbir şey.
 *
 * Ayrı pencere olmasının sebebi: bir film izlerken kenarda duran kütüphane
 * ızgarası ve yan sütun, oynatıcıyı oynatıcı gibi hissettirmiyordu. Burada
 * yan sütun YOK; tepe barı yalnız dosya adını ve düğmeleri taşıyor.
 *
 * Kütüphane, çalma listeleri, kuyruk ve ayarlar öbür pencerede
 * (`KutuphanePenceresi`). Aralarındaki tek bağ backend: ikisi de aynı
 * oynatıcının durumunu yansıtıyor, biri kapalıyken öbürü eksik çalışmıyor.
 */

import { useCallback } from "react";
import { open as dosyaSec } from "@tauri-apps/plugin-dialog";

import { Bildirimler } from "../components/Bildirimler";
import { Birak } from "../components/Birak";
import { Cubuk } from "../components/Cubuk";
import { Kisayollar } from "../components/Kisayollar";
import { useMenuUstSiniri } from "../components/Menu";
import { Sahne } from "../components/Sahne";
import { Tepe } from "../components/Tepe";
import { useBildirimler } from "../hooks/useBildirimler";
import { useBirakma } from "../hooks/useBirakma";
import { useKumanda } from "../hooks/useKumanda";
import { useKuyruk } from "../hooks/useKuyruk";
import { useOynatici } from "../hooks/useOynatici";
import * as ipc from "../ipc";
import { MEDYA_UZANTILARI } from "../lib/uzantilar";

export function OynaticiPenceresi() {
  // `bildir` / `bildirHata` KARARLI: kancalar bunları bağımlılık olarak
  // alıyor ve her boyamada yeni bir işlev üretmek olay dinleyicilerini
  // durmadan yeniden kurardı.
  const { bildirimler, bildir, bildirHata, kapat } = useBildirimler();

  const oynatici = useOynatici(bildirHata);
  const kuyrukK = useKuyruk(bildirHata);
  const kumanda = useKumanda({
    oynatici,
    kuyruk: kuyrukK,
    bildirHata,
    bildir,
    videolu: true,
  });

  const birakiliyor = useBirakma(bildirHata);

  // Çubuktaki menüler yukarı, video alanına açılıyor ve mpv oraya webview'in
  // ÜSTÜNDEN çiziyor: sınır verilmezse hız/altyazı menüleri video oynarken
  // hiç görünmüyor. Yüzey menünün tepesinde bitiyor.
  const menuUstSiniri = useMenuUstSiniri();
  const { durum } = oynatici;
  const { sar, tamEkran, cubukGorunur } = kumanda;

  const dosyaAc = useCallback(async () => {
    try {
      const secim = await dosyaSec({
        multiple: false,
        directory: false,
        filters: [{ name: "Medya", extensions: MEDYA_UZANTILARI }],
      });
      if (typeof secim === "string") await ipc.oynaticiAc(secim);
    } catch (e) {
      bildirHata(e);
    }
  }, [bildirHata]);

  // Tam ekranda çubuk fare durunca kayboluyor; pencere kipinde hep duruyor.
  const cubukGoster = !tamEkran || cubukGorunur;

  return (
    <div className={tamEkran ? "kabuk kabuk--tam-ekran" : "kabuk kabuk--oynatici"}>
      {tamEkran ? null : (
        <Tepe
          oynaticiMi
          durum={durum}
          onDosyaAc={dosyaAc}
          onKutuphane={() => sar(ipc.pencereKutuphane)}
          onKisayollar={kumanda.kisayollariDegistir}
        />
      )}

      <main className="ana">
        <Sahne durum={durum} onDosyaAc={dosyaAc} menuUstSiniri={menuUstSiniri} />

        {cubukGoster ? (
          <Cubuk
            durum={durum}
            izler={oynatici.izler}
            kuyruk={kuyrukK.kuyruk}
            gecikme={kumanda.gecikme}
            tamEkran={tamEkran}
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
        ) : null}

        {/* Tam ekranda da duruyor: bildirimler kendiliğinden kapanıyor ve
            "dosya oynatılamadı" tam da tam ekranda görülmesi gereken şey. */}
        <Bildirimler bildirimler={bildirimler} kapat={kapat} />
      </main>

      <Kisayollar acik={kumanda.kisayollar} kapat={kumanda.kisayollariKapat} />
      {birakiliyor ? <Birak /> : null}
    </div>
  );
}
