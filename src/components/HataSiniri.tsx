/**
 * Tepe hata sınırı.
 *
 * Depodaki tek sınıf bileşeni: React'in hata sınırının kanca karşılığı yok.
 *
 * Olmadığında bir bileşendeki hata bütün ağacı söküyor ve kullanıcı bomboş
 * bir pencereyle kalıyor — masaüstü uygulamasında bu "program açılmıyor"
 * demek. Buradaki ekran en azından ne olduğunu söylüyor ve yeniden yükleme
 * yolu bırakıyor.
 */

import { Component, type ErrorInfo, type ReactNode } from "react";

interface Ozellikler {
  children: ReactNode;
}

interface Durum {
  hata: Error | null;
}

export class HataSiniri extends Component<Ozellikler, Durum> {
  state: Durum = { hata: null };

  static getDerivedStateFromError(hata: Error): Durum {
    return { hata };
  }

  componentDidCatch(hata: Error, bilgi: ErrorInfo) {
    // Geliştirme konsoluna bırak: kullanıcıya gösterilen metin kısa,
    // ayıklamak için gereken yığın izi burada.
    console.error("Muiply arayüzünde hata:", hata, bilgi.componentStack);
  }

  render() {
    if (!this.state.hata) return this.props.children;

    return (
      <div className="bos">
        <div className="bos-baslik">Arayüz beklenmedik bir hatayla durdu</div>
        <p className="secilebilir">{this.state.hata.message}</p>
        <button className="dugme dugme--birincil" onClick={() => window.location.reload()}>
          Yeniden yükle
        </button>
      </div>
    );
  }
}
