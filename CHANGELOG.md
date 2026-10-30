# Changelog

Le versioni seguono il formato `x.y.z`. La serie 0.x è in sviluppo; le patch 0.5.1–0.5.7 raccolgono interventi progressivi su interfaccia e manutenzione. Il registro non implica che ogni patch sia stata distribuita come release GitHub separata.

I controlli eseguiti sono descritti in [VERIFICA.md](VERIFICA.md). Le [note di lavoro dettagliate](docs/CHANGELOG-DETTAGLIATO.md) conservano lo storico originale.

## [0.5.8] — 2026-10-30

### Commenti

- Nuovo pulsante Commenti nella barra del player, visibile solo con un brano in ascolto e la funzione attiva: apre la scheda Commenti anche dalla vista In riproduzione o dalle Impostazioni, e un secondo clic riporta il pannello com’era (coda, oppure chiuso).
- I commenti non si scaricano più quando la scheda non si vede: pannello chiuso, vista In riproduzione aperta o pagina Impostazioni.
- Il testo scritto resta nel campo finché YouTube non conferma la pubblicazione: un invio fallito non lo cancella più. Durante l’invio il campo è in sola lettura.
- La risposta in corso segue il commento scelto anche quando si caricano risposte sopra di esso, invece di passare alla riga che ne prende il posto.
- L’elenco si aggiorna riga per riga: foto degli autori e mi piace non ricreano più tutte le righe.
- Etichette per screen reader su commenti, mi piace, Rispondi, risposte, campo di scrittura, schede del pannello e pulsanti a icona del player.

### Discord Rich Presence

- Cambio di brano senza sfarfallio: la scheda del brano precedente resta finché parte il successivo, invece di essere tolta e rimessa (due invii su cinque consentiti ogni 20 secondi). Se il brano nuovo non parte entro 12 secondi, la scheda viene tolta.
- Espressioni regolari delle miniature compilate una sola volta.
- Documentazione del modulo aggiornata ad AmbraMusic, senza riferimenti a Limusic/Metrolist e a impostazioni inesistenti.

### Correzioni

- Riproduzione casuale: brani aggiunti in coda o dalla radio durante la casuale non spariscono più spegnendola, e i brani tolti non ricompaiono.
- Volume: cursore leggermente più corto (80 px) per fare spazio al pulsante Commenti alla larghezza minima.

### Verifiche

[Dettagli in VERIFICA.md](VERIFICA.md).

## [0.5.7] — 2026-10-25

### Distribuzione Windows

- Aggiunto installer x64 con interfaccia Windows Forms e motore di installazione dedicati ad AmbraMusic.
- Installazione per utente, collegamenti Start/desktop, registrazione tra le applicazioni Windows e disinstallatore incluso.
- Payload verificato con SHA-256, staging prima della sostituzione e ripristino se il cambio di cartella fallisce.
- Aggiornamento che conserva i file aggiunti dall’utente; disinstallazione che conserva i file modificati e i dati dell’app.
- Pipeline riproducibile in `scripts/build-installer.ps1`, raccolta delle licenze delle dipendenze e build Rust con lockfile bloccato.

### Interfaccia e correzioni

- Aiuto e info più compatto: logo accanto a nome e versione, allineamento coerente alle impostazioni, collegamenti e crediti essenziali.
- Barra laterale e indice delle impostazioni scorrevoli: i comandi restano raggiungibili anche nelle finestre basse.
- Copertina e coda della vista In riproduzione adattate allo spazio disponibile; titolo, artista e album non invadono il player nelle dimensioni verificate.
- Suggerimenti di ricerca scorrevoli e limitati all’area sopra il player; il pannello aperto lascia utilizzabili i comandi di riproduzione.
- Dimensioni dei testi e della barra di avanzamento mantenute.

### Verifiche

- Build release Windows x64 completata; avvio nativo e schermata Aiuto controllati al 125%.
- Cinque configurazioni di layout e interazioni di scorrimento/clic verificate.
- Cinque prove isolate del pacchetto reale dell’installer superate, inclusi percorsi lunghi, aggiornamento e disinstallazione selettiva.
- Pacchetti senza firma Authenticode; ambito e limiti dei controlli riportati in VERIFICA.md.

### Riferimenti

[Confronto 0.5.6 → 0.5.7](https://github.com/Kyo89-sudo/AmbraMusic/compare/v0.5.6...v0.5.7) · [Note di release](docs/RELEASE-0.5.7.md) · [Verifiche](VERIFICA.md)

## [0.5.6] — 2026-10-23

### Correzioni

- Barra inferiore ancorata all’area interna della finestra: i contenuti della pagina non possono più spostare play/pausa e avanzamento sotto il bordo visibile.
- Spazio della riga comandi adeguato al pulsante Play, senza ridurre testi o scala dell’interfaccia.
- Adattamento iniziale basato sui bordi effettivi della finestra Windows e sull’area utilizzabile del monitor, al posto di una stima delle decorazioni.
- Ricontrollo dopo l’inizializzazione della finestra e del DPI; una finestra già massimizzata viene lasciata tale.
- Altezza minima ridotta a 400 unità logiche per consentire l’adattamento su schermi con meno spazio al 125%.

### Verifiche

- Tre test della geometria superati per DPI 125%, altre scale comuni e monitor con coordinate negative.
- Riprodotto il taglio nella UI 0.5.5; nella UI 0.5.6 la barra resta entro la finestra in tre dimensioni renderizzate al 125% con Slint software.
- Controllo di compilazione finale superato. Dettagli in [VERIFICA.md](VERIFICA.md).

## [0.5.5] — 2026-10-23

### Manutenzione

- Rimossi i metodi Discord `set_album` e `set_config`, mai chiamati dall’app, con messaggi e gestori associati.
- Eliminati lo stato `cfg_dirty` e la lettura JSON del layout, chiamata sempre senza configurazione.
- Layout Discord inizializzato direttamente; mantenuti metadati del brano, copertine, timeline, play/pausa e interruttore Discord.
- Rimossi tre test dedicati soltanto ai percorsi eliminati; 25 test delle funzioni rimanenti superati.

### Documentazione

- Riscritti README e presentazione della release con logo, tabelle delle funzioni e delle tecnologie, collegamenti e licenza; aggiunte guide a uso e compilazione.
- Aggiunto screenshot 0.5.5 fornito dall’utente, con nome personale sostituito e cornice con angoli arrotondati e ombra.
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
