# Kurulum

## Gereksinimler

- Rust (stable, ≥ 1.77.2) + cargo
- Node.js 18+
- Windows: MSVC derleme araçları (Visual Studio Build Tools) ve **libmpv**
- Linux: `libmpv-dev`, `pkg-config`
- macOS: `brew install mpv`

Tauri CLI ayrıca kurulmuyor; `@tauri-apps/cli` npm bağımlılığı olarak geliyor
(`npx tauri ...` / `npm run tauri ...`).

## Hızlı başlangıç

```bash
npm install
npm run tauri:kabuk     # libmpv OLMADAN: kütüphane + liste + arayüz
```

`tauri:kabuk` motoru derlemeye almıyor (`--no-default-features`). Oynatma
çalışmıyor, arayüz bunu açıkça söylüyor. Gerçek oynatıcı için libmpv gerekiyor:

```bash
npm run tauri dev       # motorlu (varsayılan özellik)
```

## Windows'ta libmpv

Bu adım **zorunlu** ve tek seferlik. libmpv bağlama zamanında bağlanıyor: DLL
yoksa uygulama hiç açılmıyor, çalışma zamanında yakalanabilen bir hata değil.

### 1. mpv'yi indir

<https://mpv.io/installation/> → Windows builds (shinchiro). İki paket var,
**`mpv-dev`** olanı gerekiyor (`mpv-dev-x86_64-*.7z`); normal paket `mpv.exe`
veriyor ama bağlayıcının istediği dosyaları vermiyor.

### 2. Dosyaları yerleştir

Arşivin kökünden `src-tauri/libs/` altına:

| Dosya | Ne işe yarıyor |
|---|---|
| `libmpv-2.dll` | Çalışma zamanı. Pakete kopyalanıyor. |
| `libmpv.dll.a` | MinGW içe aktarma kitaplığı. **MSVC kullanamıyor**, yalnız arşivde ne olduğunu bilmek için burada. |

Arşivdeki `include/` gerekmiyor: `libmpv2-sys` başlıkları kendi içinde
taşıyor.

**Dosya adı sürümle değişti.** Eski shinchiro arşivlerinde `mpv-2.dll` ve
yanında bir `mpv.def` vardı; 2026 derlemelerinde ad `libmpv-2.dll` ve
`.def` hiç gelmiyor. Aşağıdaki adım bu yüzden var.

### 3. `mpv.lib` üret

`libmpv2-sys` bağlayıcıya `mpv` kitaplığını arattırıyor
(`rustc-link-lib=mpv`) ve MSVC bunun için bir içe aktarma kitaplığı istiyor.
Arşivde `.lib` yok, `.def` de yok — ikisini de DLL'in kendi dışa aktarma
tablosundan üretiyoruz. **x64 Native Tools Command Prompt** içinde,
`src-tauri/libs/` klasöründe:

```
dumpbin /exports libmpv-2.dll > exports.txt
```

Çıkan dosyada `ordinal hint RVA name` başlığından sonraki satırların son
sütunu sembol adları. Onları alt alta yazıp başına `EXPORTS` koyan bir
`mpv.def` üret, sonra:

```
lib /def:mpv.def /name:libmpv-2.dll /out:mpv.lib /machine:x64
```

`/name:` şart ve **DLL'in gerçek adı** olmalı: yanlış yazılırsa üretilen
`.lib` olmayan bir dosyayı arar ve uygulama açılışta "DLL bulunamadı" ile
kapanır. Doğru adı `dumpbin` çıktısının ilk satırlarındaki
`Section contains the following exports for ...` söylüyor.

Not: DLL, libbluray'den gelen `Java_*` sembollerini de dışa aktarıyor.
`.def` dosyasına girmeleri zararsız — içe aktarma kitaplığı yalnızca ad
listesi.

### 4. Bağlayıcıya klasörü göster

`src-tauri/.cargo/config.toml` (depoda yok, makineye özel):

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-Lnative=libs"]
```

Alternatif: `mpv.lib`i `%USERPROFILE%\.rustup\...\lib` gibi arama yolundaki bir
klasöre koymak — ama depo içinde tutmak, makine değiştiğinde ne yapıldığını
hatırlatıyor.

### 5. Çalıştır

```bash
npm run tauri dev
```

`libmpv-2.dll` çalışma zamanında da gerekiyor ve Windows onu önce
uygulamanın kendi klasöründe arıyor. Kopyalamayı **`build.rs` yapıyor**:
`libs/*.dll` her derlemede `target/<profil>/` altına düşüyor, yani
`npm run tauri dev` ve düz `cargo run` ek bir adım istemiyor.

Elle kopyalama adımı bilerek kaldırıldı: unutulduğunda derleme sorunsuz
bitiyor ve uygulama sessizce açılmıyordu — belirti sebebi göstermiyordu.

## Linux

```bash
# Debian/Ubuntu
sudo apt install libmpv-dev pkg-config \
     libwebkit2gtk-4.1-dev build-essential libssl-dev \
     libayatana-appindicator3-dev librsvg2-dev

# Arch
sudo pacman -S mpv webkit2gtk-4.1 base-devel libayatana-appindicator librsvg
```

`libmpv2-sys` sistem kitaplığını `pkg-config` ile buluyor, ek adım yok.

`libayatana-appindicator` **sistem tepsisi** için: onsuz uygulama açılıyor ama
tepsi ikonu görünmüyor (`tepsi.rs` hatayı yazıp geçiyor). Geri kalanı Tauri'nin
standart Linux gereksinimleri.

> **Wayland:** video gömme yalnız X11'de çalışıyor. mpv'nin `wid` seçeneği bir
> X11 pencere kimliği istiyor ve Wayland'ın karşılığı olan `wl_subsurface`'i
> dışarıdan kabul etmiyor. Saf bir Wayland oturumunda mpv kendi ayrı
> penceresini açıyor — ses, kütüphane, kuyruk ve denetimler çalışmaya devam
> ediyor. XWayland altında gömme çalışıyor. Ayrıntı `docs/Mpv_Integration.md`
> → "Platformlar".

## macOS

```bash
brew install mpv
```

Video gömme çalışıyor (`NSView` alt görünümü). `brew` kurulumu **çalışma
zamanında da gerekiyor**: `libmpv.dylib` .app paketinin içine kopyalanmıyor,
sistemden yükleniyor.

## Paketleme

Hedefler platforma göre ayrı yapılandırma dosyalarında ve Tauri onları ana
`tauri.conf.json` ile **kendiliğinden birleştiriyor** — ad kalıbı
`tauri.<platform>.conf.json`:

| Dosya | Hedefler | Notlar |
|---|---|---|
| `tauri.windows.conf.json` | `nsis`, `msi` | |
| `tauri.linux.conf.json` | `deb` | `libmpv2 \| libmpv1` ve appindicator bağımlılıkları burada |
| `tauri.macos.conf.json` | `app`, `dmg` | en düşük sürüm 10.15 |

```bash
npm run tauri:paket        # Windows
npm run tauri:paket:unix   # Linux ve macOS
```

**İki komut, çünkü yalnız Windows'un ek bir işi var.** `tauri:paket` bir
yapılandırma daha bindiriyor (`src-tauri/tauri.paketleme.conf.json`) ve
`libs/*.dll`i pakete kopyalıyor. Kopyalama **eşleşme biçiminde**
(`{"libs/*.dll": "./"}`), dizi biçiminde değil: dizi verildiğinde Tauri
göreli yolu koruyor ve DLL `<kurulum>/libs/` altına düşüyor. Windows'un DLL
yükleyicisi orayı aramıyor (yalnız exe'nin klasörü, sistem klasörleri ve
PATH), yani uygulama açılışta sessizce kapanırdı. O dosyanın ayrı tutulmasının sebebi, `libs/`
boşken `cargo check`in "kaynak bulunamadı" diye durmasıydı — geliştirme,
libmpv kurulu olmayan bir makinede de mümkün olmalı. Linux ve macOS'ta
kopyalanacak bir şey yok: libmpv sistemden geliyor, `tauri:paket:unix` düz
`tauri build`.

**deb ve dmg libmpv'yi İÇERMİYOR.** deb onu bağımlılık olarak istiyor
(paket yöneticisi kuruyor); dmg'de böyle bir mekanizma yok, kullanıcının
`brew install mpv` yapmış olması gerekiyor. Bu bilinçli: libmpv'yi paketin
içine gömmek, her platform için ayrı bir kod çözücü dağıtım sorumluluğu
almak demek.

Çapraz derleme yok — her paket kendi işletim sisteminde üretiliyor.

## Varsayılan oynatıcı yapmak

Muiply kendini varsayılan yapamıyor ve denememeli: Windows 10/11'de bu ayar
kullanıcının onayına kapalı bir yerde ve bir uygulamanın onu sessizce
değiştirmesi işletim sisteminin bilerek engellediği bir şey. Muiply'ın işi
listede **görünmek**; seçimi kullanıcı yapıyor.

Görünmesi iki parçaya bağlı, ikisi de kurulumla geliyor:

1. **Uzantı kaydı** — `tauri.conf.json` > `bundle.fileAssociations`.
   NSIS/MSI kurulumu bu listedeki uzantıları Windows'a bildiriyor, Linux'ta
   `.desktop` girdisinin `MimeType` satırına dönüşüyor. Liste
   `library/mod.rs` içindeki tarama uzantılarıyla AYNI; birini
   değiştirirken öbürü de değişmeli.
2. **Yolun okunması** — `src/acilis.rs`. İşletim sistemi dosyayı bir komut
   satırı argümanı olarak veriyor; Muiply açılışta onu okuyup kuyruğa
   koyuyor. Uygulama zaten açıksa tekil örnek eklentisi argümanı çalışan
   pencereye geçiriyor, ikinci bir pencere açılmıyor.

Kurulumdan sonra (geliştirme derlemesi değil — **kurulmuş** sürüm gerekiyor,
kayıt kurulumla yapılıyor):

- **Tek uzantı için:** dosyaya sağ tık → *Birlikte aç* → *Başka bir uygulama
  seç* → Muiply → *Her zaman bu uygulamayı kullan*.
- **Toplu:** Ayarlar → Uygulamalar → Varsayılan uygulamalar → Muiply → uzantı
  uzantı *Varsayılan yap*. Windows 11 tek tuşla hepsini atamıyor.

Denemesi: bir `.mkv`ye çift tıkla. Muiply açılıp dosyayı çalmalı; açıkken
ikinci bir dosyaya çift tıklamak aynı pencerede yeni dosyayı açmalı.


## Komutlar

| Komut | Ne yapıyor |
|---|---|
| `npm run dev` | Yalnız Vite (tarayıcıda arayüz; IPC yok) |
| `npm run tauri dev` | Motorlu masaüstü uygulaması |
| `npm run tauri:kabuk` | Motorsuz masaüstü uygulaması |
| `npm test` | Arayüz testleri (vitest) |
| `npm run build` | `tsc` + Vite üretim derlemesi |
| `npm run tauri:paket` | Windows paketleri (nsis + msi, DLL'lerle) |
| `npm run tauri:paket:unix` | Linux (deb) / macOS (app + dmg) paketleri |
| `cd src-tauri && cargo test --no-default-features` | Rust testleri (libmpv gerekmiyor) |

Rust testleri motorsuz çalıştırılıyor çünkü test ikilisini **bağlamak**
libmpv istiyor; testlerin kendisi (altyazı eşleştirme, kuyruk gezinme,
karıştırma) mpv'ye hiç dokunmuyor.
