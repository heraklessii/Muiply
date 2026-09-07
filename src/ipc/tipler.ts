/**
 * Backend ile paylaşılan tipler.
 *
 * Bunlar Rust tarafındaki `#[derive(Serialize)]` yapılarının birebir
 * karşılığı (`src-tauri/src/mpv/mod.rs`, `library/mod.rs`,
 * `playlist/kuyruk.rs`). Alan adları İngilizce çünkü sözleşme
 * `docs/IPC.md`'de öyle yazılı; arayüzün geri kalanı Türkçe.
 *
 * Elle yazılıyorlar, üretilmiyorlar. Bir alan Rust'ta değişip burada
 * değişmezse TypeScript uyarmaz — derleyici backend'i görmüyor. Bu yüzden
 * kural: `docs/IPC.md` değişmeden ne Rust ne burası değişir.
 */

export interface PlayerState {
  playing: boolean;
  paused: boolean;
  position: number;
  duration: number;
  volume: number;
  muted: boolean;
  rate: number;
  path: string | null;
  title: string | null;
  mediaType: MediaTuru | null;
  /** Motor derlemeye dahil mi. `false` ise oynatma denetimleri kapalı. */
  engine: boolean;
}

export type MediaTuru = "video" | "audio";

export interface Track {
  id: number;
  kind: "video" | "audio" | "sub";
  title: string | null;
  lang: string | null;
  selected: boolean;
  external: boolean;
  codec: string | null;
}

export interface FileInfo {
  path: string;
  title: string;
  duration: number;
  mediaType: MediaTuru;
  tracks: Track[];
}

export interface MediaItem {
  id: string;
  path: string;
  title: string;
  artist: string | null;
  album: string | null;
  duration: number;
  width: number | null;
  height: number | null;
  size: number;
  mediaType: MediaTuru;
  extension: string;
  /** Küçük resmin diskteki TAM yolu. `<img>` içinde doğrudan kullanılamaz;
   *  `convertFileSrc` ile `asset:` adresine çevrilmesi gerekiyor. */
  thumbnail: string | null;
  addedAt: number;
  lastPlayed: number | null;
  playCount: number;
  /** Yarım bırakılan yer (saniye). Bitmiş ya da hiç açılmamış dosyada
   *  `null`. Eşikleri backend belirliyor (`library/devam.rs`). */
  lastPosition: number | null;
}

export interface MediaFilter {
  mediaType?: MediaTuru;
  sort?: SiralamaAnahtari;
  limit?: number;
}

export type SiralamaAnahtari = "title" | "added" | "played" | "duration";

export interface Playlist {
  id: number;
  name: string;
  createdAt: number;
  updatedAt: number;
  count: number;
}

/**
 * Bir listenin bir SATIRI.
 *
 * `MediaItem`'ı genişletiyor çünkü Rust tarafı medya alanlarını iç içe bir
 * nesneye değil satırın yanına yazıyor (`#[serde(flatten)]`); aynı bileşenler
 * ızgarada ve listede aynı alan adlarını okuyabilsin diye.
 *
 * `itemId`, `id` DEĞİL: `id` medyanın kimliği ve aynı kayıt bir listede iki
 * kez bulunabiliyor. Silme `itemId` ile anlatılıyor.
 */
export interface PlaylistItem extends MediaItem {
  /** `playlist_items` satırının kimliği. */
  itemId: number;
}

export interface QueueItem {
  id: string;
  path: string;
  title: string;
  duration: number;
  mediaType: MediaTuru;
}

export type Repeat = "off" | "all" | "one";

export interface QueueSnapshot {
  items: QueueItem[];
  currentIndex: number | null;
  repeat: Repeat;
  shuffle: boolean;
}

/**
 * Kalıcı tercihler.
 *
 * Tema BURADA YOK: o `localStorage`da (`src/lib/platform.ts`) çünkü ilk
 * boyamadan önce bilinmesi gerekiyor — backend'e sormak pencerenin bir kare
 * yanlış renkte açılması demek olurdu.
 */
export interface Settings {
  /** `"auto-safe"` · `"auto"` · `"no"` */
  hwdec: string;
  /** mpv aygıt adı, ya da sistem varsayılanı için `"auto"`. */
  audioDevice: string;
  /** Her dosyanın başladığı hız. 0.25 - 4. */
  defaultRate: number;
  /** Kaldığı yerden devam edilsin mi. */
  resume: boolean;
  /** Klavyenin medya tuşları Muiply'yi mi yönetsin.
   *
   *  Kayıt GLOBAL: açıkken tuş, Muiply odakta olmasa da ona gidiyor ve
   *  başka bir oynatıcıya ulaşmıyor. Ayar olmasının sebebi bu. */
  mediaKeys: boolean;
}

export interface AudioDevice {
  /** mpv'ye yazılan ad. */
  name: string;
  /** Kullanıcıya gösterilen ad. */
  description: string;
}

export interface NearbySubtitle {
  path: string;
  label: string;
  lang: string | null;
}

/* --------------------------------------------------------------------------
 * Olay yükleri
 *
 * Adlar `docs/IPC.md`'deki tabloyla aynı. Sabit olarak tutuluyorlar çünkü
 * bir olay adını yanlış yazmak sessiz bir hata: dinleyici hiç tetiklenmiyor,
 * konsolda da bir şey görünmüyor.
 * ----------------------------------------------------------------------- */

export const OLAY = {
  dosya: "player://file-loaded",
  konum: "player://time-pos",
  duraklat: "player://pause-change",
  ses: "player://volume-change",
  bitti: "player://end-file",
  /** Sürenin yeni değeri; mpv onu dosya yüklendikten sonra öğrenebiliyor. */
  sure: "player://duration-change",
  /** Oynatma hızının yeni değeri; her dosya varsayılan hızla başlıyor. */
  hiz: "player://rate-change",
  hata: "player://error",
  /** Oynatıcıda dosya kalmadı (kuyruk bitti ya da durduruldu). Yük yok. */
  temiz: "player://cleared",
  /** mpv penceresinden gelen kısayol; yük: eylemin adı. */
  istek: "player://request",
  kuyruk: "playlist://queue",
  tarama: "library://scan-progress",
  taramaBitti: "library://scan-complete",
  medya: "library://media-updated",
} as const;

export interface KonumYuku {
  position: number;
}
export interface SureYuku {
  duration: number;
}
export interface HizYuku {
  rate: number;
}
export interface DuraklatYuku {
  paused: boolean;
}
/** Ses olayı iki ayrı şeyi taşıyor; hangisi geldiyse o alan dolu. */
export interface SesYuku {
  volume?: number;
  muted?: boolean;
}
export interface BittiYuku {
  reason: "eof" | "stop" | "quit" | "error";
}
export interface TaramaYuku {
  done: number;
  total: number;
}
export interface TaramaBittiYuku {
  added: number;
  updated: number;
  removed: number;
}
export interface MedyaYuku {
  id: string;
}

/** mpv penceresinden gelen kısayolların adları (`mpv/gercek.rs`). */
export type IstekEylemi =
  | "tam-ekran"
  | "tam-ekran-kapat"
  | "sonraki"
  | "onceki"
  | "altyazi";
