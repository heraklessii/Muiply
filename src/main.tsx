/**
 * Giriş noktası — hangi pencerede olduğumuzu burada seçiyoruz.
 *
 * Muiply iki pencereli (oynatıcı + kütüphane) ama TEK arayüz paketi: ikisi
 * de aynı `index.html`i yüklüyor. Ayrımı pencere ETİKETİ yapıyor
 * (`getCurrentWindow().label`, `src-tauri/src/pencere.rs` ile aynı dizeler).
 *
 * URL'ye parametre koymak da mümkündü ama etiket zaten var ve Tauri onu
 * geliştirme sunucusunda da doğru veriyor; iki ayrı HTML girişi ise Vite
 * yapılandırmasını ve varlık paylaşımını ikiye bölerdi.
 */

import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { HataSiniri } from "./components/HataSiniri";
import { tauriMi, temayiYukle } from "./lib/platform";
import { KutuphanePenceresi } from "./pencereler/KutuphanePenceresi";
import { OynaticiPenceresi } from "./pencereler/OynaticiPenceresi";
import "./styles.css";

temayiYukle();

// Masaüstü penceresinde metin seçimi kapalı (bkz. styles.css). Sınıf burada
// veriliyor çünkü CSS'in "Tauri'de miyim" diye sorma yolu yok.
if (tauriMi()) {
  document.body.classList.add("tauri");
}

// Tarayıcıda (saf `npm run dev`) pencere diye bir şey yok; orada kütüphane
// kabuğu gösteriliyor — oynatıcı zaten motorsuz boş bir çerçeve olurdu.
const oynaticiMi = tauriMi() && getCurrentWindow().label === "oynatici";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <HataSiniri>{oynaticiMi ? <OynaticiPenceresi /> : <KutuphanePenceresi />}</HataSiniri>
  </React.StrictMode>,
);
