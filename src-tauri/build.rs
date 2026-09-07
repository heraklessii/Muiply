//! Derleme betiği.
//!
//! İki iş yapıyor: Tauri'nin kendi üretimi, bir de Windows'ta libmpv DLL'ini
//! üretilen ikili dosyanın yanına kopyalamak.

fn main() {
    tauri_build::build();
    dll_kopyala();
}

/// `libs/*.dll`i `target/<profil>/` altına kopyalar.
///
/// Bağlayıcı `mpv.lib`i `-Lnative=libs` ile buluyor (`.cargo/config.toml`)
/// ama ÇALIŞMA zamanı ayrı bir soru: Windows DLL'i önce çalıştırılabilir
/// dosyanın kendi klasöründe arıyor ve orada yoksa uygulama hiç açılmıyor —
/// üstelik yakalanabilen bir hatayla değil, pencere doğmadan. Tauri
/// kaynakları yalnız PAKETE kopyalıyor (`tauri.paketleme.conf.json`), yani
/// `npm run tauri dev` ve düz `cargo run` bu adımdan geçmiyordu.
///
/// Elle kopyalamanın (eski `docs/Setup.md` adımı) sorunu, unutulduğunda
/// verdiği belirtiydi: derleme sorunsuz bitiyor, uygulama sessizce
/// açılmıyor. Kopya derlemenin parçası olunca soru ortadan kalkıyor.
///
/// Hata derlemeyi DURDURMUYOR: dosya o an kilitli olabilir (uygulamanın
/// çalışan bir kopyası DLL'i tutuyor) ve bu durumda hedefte zaten doğru
/// dosya var demektir. Uyarı basılıyor.
fn dll_kopyala() {
    // Yalnız Windows'ta ve yalnız motor derlemeye dahilken. Linux ve macOS'ta
    // libmpv sistem paketinden geliyor, kopyalanacak bir şey yok.
    if !cfg!(target_os = "windows") || std::env::var_os("CARGO_FEATURE_MPV").is_none() {
        return;
    }

    let kaynak = std::path::Path::new("libs");
    println!("cargo:rerun-if-changed=libs");

    // OUT_DIR: `target/<profil>/build/<paket>-<özet>/out`. İkili dosya üç üst
    // klasörde. Cargo bu yolu doğrudan vermiyor; yerleşik çözüm bu.
    let Some(hedef) = std::env::var_os("OUT_DIR")
        .map(std::path::PathBuf::from)
        .and_then(|o| o.ancestors().nth(3).map(|a| a.to_path_buf()))
    else {
        println!("cargo:warning=muiply: OUT_DIR okunamadı, libmpv DLL'i kopyalanmadı");
        return;
    };

    let Ok(girdiler) = std::fs::read_dir(kaynak) else {
        println!("cargo:warning=muiply: src-tauri/libs bulunamadı, libmpv DLL'i kopyalanmadı");
        return;
    };

    for girdi in girdiler.flatten() {
        let yol = girdi.path();
        if yol.extension().and_then(|u| u.to_str()) != Some("dll") {
            continue;
        }
        let Some(ad) = yol.file_name() else { continue };
        let varis = hedef.join(ad);

        // Aynı dosya zaten oradaysa dokunma: her derlemede yazmak, uygulama
        // çalışırken derleme yapıldığında gereksiz bir "erişim reddedildi"
        // uyarısı üretiyordu.
        if ayni_dosya(&yol, &varis) {
            continue;
        }
        if let Err(e) = std::fs::copy(&yol, &varis) {
            println!(
                "cargo:warning=muiply: {} kopyalanamadı ({e})",
                ad.to_string_lossy()
            );
        }
    }
}

/// İkisi de var ve boyut/değişme zamanı aynı mı.
///
/// İçerik karşılaştırmıyor: DLL 100 MB'ın üstünde olabiliyor ve bu kontrol
/// her derlemede çalışıyor. Boyut+zaman, elle değiştirilmemiş bir dosya için
/// yeterli ayrım.
fn ayni_dosya(a: &std::path::Path, b: &std::path::Path) -> bool {
    let (Ok(x), Ok(y)) = (a.metadata(), b.metadata()) else {
        return false;
    };
    x.len() == y.len() && x.modified().ok() == y.modified().ok()
}
