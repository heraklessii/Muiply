/**
 * `invoke()` çağrılarının tek sarmalayıcı katmanı.
 *
 * Bileşenler `invoke` görmez. Sebep: komut adı bir dize ve yanlış yazılmış
 * bir dize çalışma zamanına kadar sessiz kalıyor. Burada her komut bir
 * fonksiyon, yani yazım hatası derlemede çıkıyor.
 *
 * İkinci sebep: motorsuz derlemede oynatma komutları hata dönüyor
 * (`MOTOR_YOK`). O hatayı yakalayıp ekrana taşımak tek bir yerde olmalı —
 * her düğmede ayrı `try/catch` yazmak, birini unutmak demektir.
 */

import { invoke } from "@tauri-apps/api/core";

import type {
  AudioDevice,
  MediaFilter,
  MediaItem,
  NearbySubtitle,
  Playlist,
  PlaylistItem,
  PlayerState,
  QueueSnapshot,
  Repeat,
  Settings,
  Track,
} from "./tipler";

/* -- oynatıcı ------------------------------------------------------------ */

export const oynaticiAc = (path: string) => invoke<void>("player_open", { path });
export const oynaticiOynat = () => invoke<void>("player_play");
export const oynaticiDuraklat = () => invoke<void>("player_pause");
export const oynaticiDegistir = () => invoke<void>("player_toggle_pause");
export const oynaticiAra = (position: number) => invoke<void>("player_seek", { position });
export const oynaticiGoreliAra = (delta: number) =>
  invoke<void>("player_seek_relative", { delta });
export const oynaticiSes = (volume: number) => invoke<void>("player_set_volume", { volume });
export const oynaticiSessiz = (muted: boolean) => invoke<void>("player_set_mute", { muted });
export const oynaticiDur = () => invoke<void>("player_stop");
export const oynaticiHiz = (rate: number) => invoke<void>("player_set_playback_rate", { rate });
export const oynaticiDurum = () => invoke<PlayerState>("player_get_state");
export const oynaticiIzler = () => invoke<Track[]>("player_get_tracks");

/** Video yüzeyinin duracağı dikdörtgen — CSS pikseli. */
export const oynaticiVideoAlani = (x: number, y: number, width: number, height: number) =>
  invoke<void>("player_set_video_rect", { x, y, width, height });
export const oynaticiVideoGorunur = (visible: boolean) =>
  invoke<void>("player_set_video_visible", { visible });

/** Sürüklenip bırakılan yollar. Klasör mü dosya mı kararı BACKEND'de:
 *  arayüzün dosya sistemine erişimi yok. */
export const yollariAc = (paths: string[]) => invoke<void>("app_open_paths", { paths });

/* -- kütüphane ----------------------------------------------------------- */

export const kutuphaneKlasorEkle = (path: string) =>
  invoke<void>("library_add_folder", { path });
export const kutuphaneKlasorSil = (path: string) =>
  invoke<void>("library_remove_folder", { path });
export const kutuphaneKlasorler = () => invoke<string[]>("library_get_folders");
export const kutuphaneTara = () => invoke<void>("library_scan");
export const kutuphaneMedya = (filter?: MediaFilter) =>
  invoke<MediaItem[]>("library_get_media", { filter });
export const kutuphaneSon = (limit: number) =>
  invoke<MediaItem[]>("library_get_recent", { limit });
export const kutuphaneSil = (id: string) => invoke<void>("library_delete_media", { id });
export const kutuphaneKayit = (id: string) =>
  invoke<MediaItem | null>("library_get_item", { id });

/* -- çalma listeleri ----------------------------------------------------- */

export const listeOlustur = (name: string) => invoke<Playlist>("playlist_create", { name });
export const listeSil = (id: number) => invoke<void>("playlist_delete", { id });
export const listeAdDegistir = (id: number, name: string) =>
  invoke<void>("playlist_rename", { id, name });
export const listeHepsi = () => invoke<Playlist[]>("playlist_get_all");
export const listeOgeler = (id: number) =>
  invoke<PlaylistItem[]>("playlist_get_items", { id });
export const listeOgeEkle = (playlistId: number, mediaId: string) =>
  invoke<void>("playlist_add_item", { playlistId, mediaId });
/** Öğe SATIR kimliğiyle siliniyor, sırayla değil: sıra, arayüzün gördüğü an
 *  ile silmenin yürüdüğü an arasında kayabiliyor — gerekçe
 *  `src-tauri/src/playlist/mod.rs::oge_sil` başında. */
export const listeOgeSil = (playlistId: number, itemId: number) =>
  invoke<void>("playlist_remove_item", { playlistId, itemId });
export const listeSirala = (playlistId: number, from: number, to: number) =>
  invoke<void>("playlist_reorder", { playlistId, from, to });
export const listeCal = (id: number, startIndex?: number) =>
  invoke<void>("playlist_play", { id, startIndex });

/* -- kuyruk -------------------------------------------------------------- */

export const kuyrukKur = (mediaIds: string[], startIndex?: number) =>
  invoke<void>("queue_set", { mediaIds, startIndex });
export const kuyrukOku = () => invoke<QueueSnapshot>("queue_get");
export const kuyrukCal = (index: number) => invoke<void>("queue_play_at", { index });
export const kuyrukSonraki = () => invoke<boolean>("queue_next");
export const kuyrukOnceki = () => invoke<boolean>("queue_prev");
export const kuyrukTekrar = (repeat: Repeat) => invoke<void>("queue_set_repeat", { repeat });
export const kuyrukKarisik = (shuffle: boolean) =>
  invoke<void>("queue_set_shuffle", { shuffle });

/* -- altyazı ------------------------------------------------------------- */

export const altyaziIzler = () => invoke<Track[]>("subtitle_get_tracks");
export const altyaziSec = (trackId: number | null) =>
  invoke<void>("subtitle_select", { trackId });
export const altyaziDosyaEkle = (path: string) => invoke<void>("subtitle_add_file", { path });
export const altyaziGecikme = (seconds: number) =>
  invoke<void>("subtitle_set_delay", { seconds });
export const altyaziGecikmeOku = () => invoke<number>("subtitle_get_delay");
export const altyaziYanindakiler = (path: string) =>
  invoke<NearbySubtitle[]>("subtitle_find_nearby", { path });

/* -- ayarlar ------------------------------------------------------------- */

export const ayarlarOku = () => invoke<Settings>("settings_get");
/** DÜZELTİLMİŞ ayarı döndürüyor: aralık dışı bir değer hata değil, sınıra
 *  çekiliyor. Arayüz gönderdiğini değil DÖNEN değeri gösteriyor. */
export const ayarlarYaz = (settings: Settings) =>
  invoke<Settings>("settings_set", { settings });
/** Motorsuz derlemede boş liste — "aygıt yok" doğru cevap, hata değil. */
export const sesAygitlari = () => invoke<AudioDevice[]>("settings_audio_devices");

/* -- pencereler ---------------------------------------------------------- */

/**
 * Pencereler açılışta YARATILIYOR, bu komutlar yalnız gösteriyor
 * (`docs/IPC.md` > Pencere komutları). Sebep mpv: çizeceği pencereyi
 * başlatılırken alıyor, sonradan değiştirilemiyor.
 *
 * Video başlayınca oynatıcı penceresini backend zaten kendisi gösteriyor;
 * buradaki iki komut kullanıcının açıkça bastığı düğmeler için.
 */
export const pencereOynatici = () => invoke<void>("window_show_player");
export const pencereKutuphane = () => invoke<void>("window_show_library");

/**
 * Backend hatasını okunabilir bir cümleye çevirir.
 *
 * Rust tarafı `Hata`yı düz dize olarak gönderiyor (`hata.rs`), ama Tauri'nin
 * kendi katmanı (izin reddi, seri hâle getirme) başka biçimler de
 * üretebiliyor. Arayüz her ihtimalde bir cümle görmek zorunda: boş bir hata
 * şeridi, hata olmamasından daha kötü.
 */
export function hataMetni(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  if (e && typeof e === "object" && "message" in e) {
    return String((e as { message: unknown }).message);
  }
  return "Beklenmeyen bir hata oluştu.";
}
