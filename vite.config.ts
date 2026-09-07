// `vitest/config` uzerinden: `vite`in kendi `defineConfig`i `test` alanini
// tanimiyor ve tsc burada hata veriyor. Ikisi ayni yapilandirmayi uretiyor.
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],

  // Tauri CLI'in urettigi hatalari gizlememek icin
  clearScreen: false,

  server: {
    // Tauri sabit portu bekliyor; port doluysa fail etmeli ki Tauri penceresi
    // yanlis bir adrese baglanmasin.
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      // Rust tarafi degisince Vite'in bosuna yeniden derlemesini engelle
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    sourcemap: true,
  },

  test: {
    include: ["src/**/*.test.ts"],
    // Test edilen modullerin hepsi saf: sure bicimleme, dosya adi ayiklama,
    // kuyruk sirasi, altyazi eslestirme. mpv'ye ya da SQLite'a dokunan kod
    // Rust tarafinda ve kendi `cargo test`i var.
    environment: "node",
  },
});
