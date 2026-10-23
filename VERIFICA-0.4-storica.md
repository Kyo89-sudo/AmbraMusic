# Verifica della 0.4

Ambiente: Linux x86_64, Rust 1.97, slint-viewer 1.18.1 ufficiale. Data: 22 ottobre 2026.

**In questa sessione il registro crates.io non era raggiungibile**, quindi `cargo build` e `cargo test` del progetto completo non sono stati eseguiti.

Verificato:

- `crates/player` (player nuovo) compilato con `rustc` contro una copia minima di tokio: nessun errore né avviso; 4 test della logica (fine brano, volume, intestazioni, posizione) superati. Costanti e strutture dell'API C confrontate con `include/mpv/client.h` di mpv (API 2.6). Non provato con libmpv vero.

- `ui/app.slint`: compilato senza errori con `slint-viewer --check` (Slint 1.18.1, la stessa versione di `Cargo.toml`).
- GUI renderizzata nei temi scuro e chiaro, con le pagine Home, Cerca (suggerimenti e risultati), playlist, Impostazioni, vista "In riproduzione" e playlist a destra, e nelle varianti della finestra Account (non collegato, Avanzate, accesso in corso, collegato): immagini in `anteprime/`.
- `src/session.rs` e le funzioni pure di `src/login.rs` compilati con `rustc`, con una copia di `cookie_sapisid` di InnerTube: **14 test superati** (anche `src/art.rs`: URL della copertina grande e colore dominante) (lettura dei cookie, confronto, normalizzazione, salvataggio e riapertura di `session.dat`, output del processo figlio, riconoscimento dell'URL, argomenti).
- Tutto il codice Rust nuovo o modificato passa `rustfmt`, quindi la sintassi è valida.
- Le API di wry 0.56.1, tao 0.36.0 e tokio usate dal login sono state controllate sui loro sorgenti ufficiali.

Non verificati:

- La 0.2 è stata compilata e provata su Windows dall'autore; 0.2.1 e 0.3 non ancora. La 0.3 riscrive gran parte di `backend.rs`, `main.rs` e dell'interfaccia: la prima compilazione può richiedere qualche correzione.
- Area di notifica (`SystemTrayIcon` di Slint) e chiusura nella tray.

- Compilazione completa e test della 0.1 rieseguiti (247) con le nuove dipendenze.
- Il codice Windows (finestra WebView2 e DPAPI): va compilato ed eseguito su Windows.
- Accesso con un account Google reale, rinnovo della sessione e riproduzione end-to-end.

Primo controllo consigliato su Windows: `COMPILA-WINDOWS.cmd`, poi Account e opzioni → Accedi con Google, chiudere e riaprire l'app per vedere se l'accesso si ripristina.
