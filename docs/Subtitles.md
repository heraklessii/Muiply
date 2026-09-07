# Altyazı

## Desteklenen biçimler

SRT, ASS/SSA, VTT, SUB (MicroDVD), IDX+SUB (VobSub) ve MKV/MP4 içindeki
gömülü izler. Çizim libmpv'nin işi — ASS biçimlendirmesi, fontlar,
konumlandırma hepsi native tarafta. Arayüzde altyazı çizen bir kod **yok**.

## Gömülü izler

mpv `track-list` içinde veriyor; ayrı bir iş yok. `mpv::izleri_oku` bütün
izleri okuyor, `subtitle_get_tracks` yalnız `kind == "sub"` olanları
süzüyor.

Arayüzde gömülü izler ile harici dosyalar **tek listede** görünüyor:
kullanıcı için ikisi de "altyazı". Ayrım `Track.external` alanında ve orada
da sadece küçük bir "dosya" eki olarak görünüyor.

## Yanındaki dosyaları bulma

mpv'nin kendi taraması **kapalı** (`sub-auto=no`). Sebep:

| mpv ayarı | Ne yapıyor | Sorun |
|---|---|---|
| `exact` | yalnız `film.srt` | `film.tr.srt` gelmiyor |
| `fuzzy` | klasördeki bütün altyazılar | on filmlik klasörde her filme on altyazı |

Aradaki doğru davranış "aynı adla başlayanlar" ve o mpv'de yok. Kural
(`subtitle/mod.rs`):

> Aynı klasörde, adı videonun köküyle başlayan ve ardından **nokta ya da dize
> sonu** gelen, uzantısı altyazı olan dosyalar.

`film.mkv` için `film.srt`, `film.tr.srt`, `film.en.ass` geliyor; `film2.srt`
gelmiyor. En sık karışan durum aynı klasördeki `Bolum1` / `Bolum10` — test
(`eslesme_eki`) tam olarak onu koruyor.

Eşleşme **büyük/küçük harf ayırmıyor**: `Film.mkv` yanındaki `film.tr.srt`
geliyor. Windows ve macOS dosya sistemleri de ayırmıyor, yani bu ikisi
kullanıcı için zaten aynı filmin dosyaları. Harfe duyarlı karşılaştırma o
altyazıyı sessizce ıskalıyordu — hata da vermiyordu, altyazı iz listesinde
hiç görünmüyordu.

Karşılaştırma karakter karakter yürüyor, iki adı küçültüp karşılaştırarak
değil (`onek_kirp`). Sebebi Unicode: küçültme uzunluğu değiştirebiliyor
(`'İ'` küçüldüğünde iki kod noktası oluyor) ve küçültülmüş dizede bulunan
eşleşme uzunluğu orijinal dizede başka bir yere düşüyor — ek yanlış yerden
kesilirdi. Kesme noktası orijinalin kendi indekslerinden alınıyor.

`İ`/`i` ve `I`/`ı` **bilerek eşleşmiyor**. Türkçede `İ`nin küçüğü `i`,
`I`nın küçüğü `ı`; Unicode'un dilden bağımsız tablosu bunu bilmiyor ve
zorlamak `I` ile `İ`yi birbirine karıştırırdı. Dosya adı eşleştirmesinde
yanlış eşleme, ıskalanan bir eşlemeden kötü: başka bir filmin altyazısını
yüklerdi. Ö/ö, Ç/ç, Ş/ş, Ğ/ğ, Ü/ü çiftlerini Unicode doğru eşliyor, onlar
çalışıyor.

Klasör **tek seferde** okunuyor. Bu dosyanın ilk taslağı her uzantı için ayrı
bir `read_dir` yapıyordu ve tam eşleşen dosyayı listeye iki kez koyuyordu;
arayüzde aynı altyazı iki satır olarak görünürdü.

### Uzantılar

```
srt  ass  ssa  vtt  sub  idx
```

`.idx` listede, çünkü VobSub çifti (`.idx` + `.sub`) birlikte çalışıyor ve
mpv'ye yalnız `.sub` vermek sessizce boş bir iz üretiyor. `.idx` verildiğinde
mpv ikisini birden alıyor.

### Dil kodu

`film.tr.srt` → `tr` → "Türkçe". Ekin ilk parçası iki-üç harfliyse ve hepsi
harf ise dil kodu sayılıyor; `forced` ya da `sdh` gibi ekler dil değil, ham
hâliyle etikette görünüyor.

Türkçe adlar `src/lib/dil.ts` içinde ve liste bilerek kısa: bütün ISO 639
tablosunu gömmek yüz kilobayt ve kullanıcının göreceği diller o on beşin
içinde. Tanınmayan kod ham gösteriliyor — "zza" yazan bir satır, yanlış
çevrilmiş bir addan iyi.

## Yükleme

Dosya açıldığında, izler okunmadan **önce**:

```rust
for altyazi in yanindakiler(video) {
    motor.komut("sub-add", &[&altyazi.path, "auto"]);
}
```

- **`auto` bayrağı:** ekle, ama zaten seçili bir altyazı varsa seçme. İlk
  eklenen seçiliyor.
- **Sıra kararlı** (`path`e göre sıralı): `read_dir` işletim sistemine göre
  değişiyor ve ilk altyazı otomatik seçildiği için sıranın değişmesi "her
  açışta başka dil geliyor" demek olurdu.
- **İzlerden önce:** sonra eklenirse ilk gönderilen iz listesi eksik kalır ve
  arayüz bir an "altyazı yok" der.
- **Hata yutuluyor:** bozuk bir `.srt` yüzünden dosyanın kendisi
  açılmamalı. Eklenemeyen altyazı iz listesinde görünmüyor, o kadar.

Kullanıcının elle seçtiği dosya `select` bayrağıyla ekleniyor — onu görmek
için seçti.

## Seçme ve kapatma

```rust
Some(id) => motor.ayar_sayi("sid", id),
None     => motor.ayar_metin("sid", "no"),
```

İki ayrı çağrı olmasının sebebi: mpv'de altyazıyı kapatmanın karşılığı
`sid=no`, yani sayı değil metin. `sid=0` "sıfır numaralı iz" demek değil —
mpv'de iz numaraları 1'den başlıyor.

Arayüzdeki `s` kısayolu izler arasında dönüyor: kapalı → 1 → 2 → ... → kapalı.

## Gecikme

`sub-delay`, saniye. Artı değer altyazıyı geciktiriyor. Çubuktaki altyazı
menüsünde 0.1 saniyelik adımlarla ve bir "Sıfırla" düğmesiyle.

Değer arayüzde `Math.round(x * 10) / 10` ile yuvarlanıyor: kayan noktalı
toplama `0.30000000000000004` üretiyor ve menüde bir ondalık gösteriliyor.

Dosya değişince mpv gecikmeyi sıfırlıyor; arayüz de `player://file-loaded`
sonrası yeniden okuyor, yoksa bir öncekinin değeri asılı kalıyordu.
