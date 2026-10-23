# Changelog

Le versioni seguono il formato `x.y.z`. La serie 0.x è in sviluppo; le patch 0.5.1–0.5.5 raccolgono interventi progressivi su interfaccia e manutenzione. Il registro non implica che ogni patch sia stata distribuita come release GitHub separata.

I controlli eseguiti sono descritti in [VERIFICA.md](VERIFICA.md). Le [note di lavoro dettagliate](docs/CHANGELOG-DETTAGLIATO.md) conservano lo storico originale.

## [0.5.5] — 2026-10-23

### Manutenzione

- Rimossi i metodi Discord `set_album` e `set_config`, mai chiamati dall’app, con messaggi e gestori associati.
- Eliminati lo stato `cfg_dirty` e la lettura JSON del layout, chiamata sempre senza configurazione.
- Layout Discord inizializzato direttamente; mantenuti metadati del brano, copertine, timeline, play/pausa e interruttore Discord.
- Rimossi tre test dedicati soltanto ai percorsi eliminati; 25 test delle funzioni rimanenti superati.

### Documentazione

- Riscritti README e presentazione della release; separata la guida alla compilazione.
- Changelog organizzato per versione e tipo di modifica, con note dettagliate conservate in archivio.
- Corretti riferimenti obsoleti alle prime alpha e all’assenza dell’area di notifica.

## [0.5.4] — 2026-10-23

### Ottimizzazione

- Preferiti indicizzati una sola volta per aggiornamento, evitando una scansione completa per ogni riga.
- Ricerca delle copertine in memoria interrotta al primo risultato; scansione saltata quando manca la miniatura.
- Eliminate copie temporanee di titolo e ID nel calcolo dei colori delle copertine generate.
- Nessun guadagno percentuale dichiarato: benchmark end-to-end non eseguiti.

## [0.5.3] — 2026-10-23

### Manutenzione

- Estratte da `main.rs` le conversioni dei modelli UI in `ui_models.rs` e le integrazioni desktop in `desktop.rs`.
- Denominate le costanti Windows dell’istanza unica.
- Riorganizzazione senza modifiche intenzionali al comportamento.

## [0.5.2] — 2026-10-23

### Interfaccia e correzioni

- Dimensionamento intermedio di testi, comandi, righe e copertine dopo la prima revisione compatta.
- Barra di avanzamento estesa nello spazio centrale, con spessore di 5 px e cursore da 14 px.
- Finestra iniziale 1180 × 700, con adattamento all’area disponibile del monitor.
- Istanza unica su Windows: il secondo avvio richiama la finestra esistente; login Google in processo separato conservato.
- Riavvio dalla tray posticipato al rilascio dell’istanza precedente.

## [0.5.1] — 2026-10-23

### Interfaccia e correzioni

- Prima revisione compatta di barre, righe e comandi; versione completa nel titolo finestra.
- Adattamento iniziale alla taskbar e alla scala DPI.
- Scheda Aiuto e info con icona ridotta, testi a capo e pulsanti adattabili.
- Palette chiaro/scuro neutra e sfumatura del player dal colore della copertina.
- Lockfile riallineato alle dipendenze dichiarate.

## [0.5.0] — 2026-10-22

### Funzioni

- Commenti del brano nel pannello laterale: ordinamento, risposte e interazioni dell’account.
- Caricamento dei commenti limitato alla scheda aperta; opzione per disattivarli.
- Personalizzazione AmbraMusic della presenza Discord e nuove icone per i commenti.

## [0.4.0] — 2026-10-22

### Player e correzioni

- Player basato direttamente sull’API C di libmpv, al posto del precedente wrapper.
- Aggiornamenti della posizione limitati e scala del volume uniforme in dB.
- Copertine nella coda, migliore disposizione delle Impostazioni e opzioni mpv non essenziali gestite senza bloccare l’audio.

## [0.3.1] — 2026-10-22

### Interfaccia e ottimizzazione

- Copertine generate quando manca l’immagine e miniature richieste a risoluzione più adatta.
- Una sola immagine decodificata per elemento e cache di compilazione condivisa fra versioni.
- Foto profilo Google, icone e testi rivisti; pagina Informazioni con collegamenti e crediti.

## [0.3.0] — 2026-10-22

### Funzioni

- Home a scaffali, ricerca con suggerimenti e filtri, pagine di playlist/album/artista.
- Coda con radio, riproduzione automatica, shuffle e ripeti.
- Raccolta nella barra laterale, navigazione Indietro e Impostazioni a schede.
- Area di notifica con comandi, ripristino, riavvio e uscita.

## [0.2.1] — 2026-10-22

### Interfaccia

- Vista In riproduzione, copertine grandi e sfondo dal colore dominante.
- Renderer FemtoVG e revisione delle icone.

## [0.2.0] — 2026-10-22

### Funzioni

- Nome e icona AmbraMusic, temi chiaro/scuro e controlli ambra.
- Accesso Google integrato tramite WebView2 in un processo separato.
- Sessione salvata con DPAPI su Windows, ripristino e rinnovo; cambio account e accesso avanzato tramite Cookie header.

## [0.1.0] — 2026-10-22

### Prima versione

- Prima GUI Slint ispirata a Limusic, con motore Rust e funzioni iniziali di ricerca e riproduzione.
