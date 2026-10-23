# Provenienza del codice

## Limusic

- Progetto: https://github.com/SimoHypers/limusic
- Autore originale: SimoHypers e contributori.
- Commit di riferimento: `9fb803b651f49f9a0ee6df25915dd02514fd624e`.
- Licenza dichiarata dal workspace: GPL-3.0-or-later; testo originale in LICENSE.
- Copie ancora presenti: `vendor/innertube`, `src/discord.rs`, `src/audioproxy.rs`.
- Sostituito: il player di Limusic (`vendor/player`, basato sul crate `libmpv2`) è stato rimosso; al suo posto `crates/player`, scritto da zero per AmbraMusic direttamente sull'API C di libmpv (`include/mpv/client.h`, API 2.x).
- Modifiche: proxy con runtime Tokio al posto di Tauri; Discord con rilevamento locale e ridimensionamento copertine indipendenti dall'app Tauri. I file vendorizzati conservano i commenti originali.

## Nuovo codice (0.1 e 0.2)

GUI `ui/app.slint`, coordinamento `src/backend.rs`, bootstrap `src/main.rs`, accesso Google `src/login.rs` (strategia ripresa da `src-tauri/src/session.rs` di Limusic), sessione salvata `src/session.rs`, client HTTP minimale, script di compilazione e documentazione: GPL-3.0-or-later.

## Dipendenze

- Slint: https://slint.dev — usato con opzione GPLv3. Le altre opzioni di licenza del toolkit non cambiano la licenza del fork.
- libmpv/mpv: https://mpv.io — licenze del relativo pacchetto; lo script scarica il pacchetto fissato dall'upstream, non incluso nello ZIP sorgente.
- RustyPipe, Tokio, reqwest, discord-rich-presence, image e altre dipendenze Rust: versioni esatte e relative fonti in Cargo.lock. I metadati di licenza sono consultabili con `cargo metadata`.
- wry e tao (Tauri Programme, MIT/Apache-2.0): finestra di accesso WebView2, solo Windows.
- Icone Lucide 1.61 (ISC, https://lucide.dev): sottoinsieme in `assets/fonts/lucide-subset.ttf`, licenza in `assets/fonts/LUCIDE-LICENSE.txt`.
- Il comportamento di coda, radio, ricerca e tray segue quello di Limusic (GPL-3.0), reimplementato in `src/backend.rs`, `src/pages.rs` e `ui/app.slint`; i testi dell'interfaccia sono propri.
- L'icona AmbraMusic (A con nota) è disegnata apposta per questo progetto: sorgente in `assets/icon-source.svg`. I parametri della presenza Discord sono definiti in `src/discord.rs`; il logo AmbraMusic punta all’asset del repository.

Questo pacchetto è un prototipo indipendente: nessuna affiliazione con Google, YouTube o Discord. Le eventuali modifiche future devono conservare gli obblighi di licenza del codice di origine.
