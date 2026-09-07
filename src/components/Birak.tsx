/**
 * Sürükle-bırak kaplaması.
 *
 * `pointer-events: none` (styles.css): kaplama görünürken bile fare
 * olaylarını yutmuyor. Yutsaydı Tauri'nin `tauri://drag-drop` olayı
 * webview'e ulaşmaz, bırakma hiç gerçekleşmezdi.
 */

import { IconBirak } from "./Ikonlar";

export function Birak() {
  return (
    <div className="birak">
      <div className="birak-kutu">
        <IconBirak />
        <span>Çalmak için bırak</span>
        <span style={{ fontSize: "var(--font-md)", color: "var(--text-muted)", fontWeight: 400 }}>
          Klasör bırakırsan kütüphaneye eklenir
        </span>
      </div>
    </div>
  );
}
