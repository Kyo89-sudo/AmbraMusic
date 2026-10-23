# Note dettagliate precedenti alla preparazione GitHub

Archivio delle note di lavoro della serie 0.1–0.5.5. Per il riepilogo mantenuto consulta [CHANGELOG.md](../CHANGELOG.md). Le verifiche di ogni versione si riferiscono al momento indicato.

# 0.5.5 — 23 ottobre 2026

Blocco aggiuntivo: rimozione di codice morto, senza nuove funzionalità.

- Rimossi i metodi Discord `set_album` e `set_config`, privi di chiamanti nell’app, e i messaggi/gestori associati.
- Eliminato lo stato `cfg_dirty`: poteva essere attivato esclusivamente dai messaggi rimossi.
- Rimossa la lettura JSON del layout Discord, chiamata sempre con `None`; il layout predefinito viene inizializzato direttamente. Tipo di configurazione reso interno al modulo.
- Rimossi tre test dedicati esclusivamente ai percorsi eliminati; conservati i test delle funzioni Discord attive.
- Conservati metadati iniziali dell’album, copertine, timeline, riproduzione/pausa e attivazione/disattivazione Discord.
- Verifiche: compilazione controllata con successo, nessun avviso di codice morto; 25 test Discord superati usando il modulo originale con dipendenze reali in un progetto di test isolato.
- Versione 0.5.5 allineata in manifest, lockfile, titolo finestra, README e ZIP. Dettagli delle verifiche in VERIFICA.md.

# 0.5.4 — 23 ottobre 2026

Blocco 2: ottimizzazione mirata, senza cambiamenti intenzionali a UI o comportamento.

- Aggiornamento dei preferiti: indice HashSet costruito una sola volta per evento, invece di cercare ogni brano nell’intera lista dei preferiti. Costo atteso da O(righe × preferiti) a O(righe + preferiti), con memoria temporanea O(preferiti).
- Ricerca della copertina in memoria: ritorno immediato al primo brano trovato e nessuna scansione se manca l’URL della miniatura. Ordine di priorità dei modelli e fallback conservati.
- Colore delle copertine generate calcolato prendendo in prestito titolo o ID, evitando una copia temporanea della stringa per ogni riga.
- Versione 0.5.4 allineata in progetto, titolo finestra, documentazione e pacchetto.
- Verifiche: controllo di compilazione offline e rustfmt superati; rapporto in VERIFICA.md.
- Nessun aumento prestazionale percentuale dichiarato: non sono stati eseguiti benchmark end-to-end. Bugfixing rimandato al blocco successivo.

# 0.5.3 — 23 ottobre 2026

Blocco 1: cleanup strutturale, senza modifiche intenzionali al comportamento.

- Estratte in `src/ui_models.rs` le conversioni dei dati backend in modelli Slint, il formato dei tempi e gli aggiornamenti dei preferiti.
- Estratte in `src/desktop.rs` le funzioni desktop: directory dati, apertura URL, gestione memoria nella tray e adattamento della finestra al monitor.
- `main.rs` concentrato su avvio, collegamenti UI/backend e ciclo di vita dell’app; import aggiornati.
- Costanti Windows dell’istanza unica denominate esplicitamente al posto dei valori numerici.
- Versione 0.5.3 allineata in manifest, lockfile, UI e documentazione.
- Ottimizzazione e bugfixing rinviati ai blocchi successivi; UI media e barra progresso della 0.5.2 conservate.

# 0.5.2 — 23 ottobre 2026

Patch della 0.5.1: correzioni di usabilità e gestione degli avvii, senza cambiamenti incompatibili.

- UI di dimensione intermedia: testi aumentati di 1 px, pulsanti principali da 38 px, righe da 52–54 px, album da circa 160 px, barra laterale da 230 px e player da 88 px.
- Barra di avanzamento estesa nello spazio centrale del player, spessore 5 px e cursore da 14 px; volume con cursore sottile separato.
- Finestra iniziale 1180 × 700; resta attivo l’adattamento all’area disponibile del monitor, esclusa la taskbar.
- Istanza unica su Windows: il secondo avvio segnala l’istanza esistente e ne richiama la finestra, evitando un secondo player. Il processo separato di accesso Google resta consentito.
- Riavvio dalla tray eseguito dopo la chiusura del player e il rilascio del blocco di istanza.
- Versione 0.5.2 allineata in manifest, lockfile, titolo finestra e pacchetto sorgenti.
- Verifiche: controllo di compilazione della revisione UI e istanza unica superato; test del blocco con due processi Windows superato. Dettagli in VERIFICA.md.

# 0.5.1 — 23 ottobre 2026

- Prima revisione compatta dell’interfaccia: finestra 1120 × 640, barra laterale 210 px, player 80 px e cursori più sottili.
- Versione completa visibile nel titolo finestra.
- Dimensioni e posizione iniziali adattate all’area utilizzabile del monitor su Windows, considerando taskbar e DPI.
- Scheda Aiuto e info con icona ridotta, testi a capo e pulsanti adattabili; coda temporaneamente nascosta nelle Impostazioni.
- Palette bianco/nero neutra, testi secondari più leggibili e sfumatura del player dal colore della copertina.
- Lockfile riallineato al manifest.

# 0.5.0 — 22 ottobre 2026

- **Commenti del brano in ascolto**: scheda "Commenti" nel pannello di destra, accanto a "In coda". Commenti più popolari o più recenti, risposte rientrate ("Mostra N risposte"), mi piace, commento fissato, cuore dell'autore, foto degli autori. Con l'account si scrivono commenti e risposte dal campo in basso (Invio per inviare).
- Logica dei commenti scritta da noi (`src/comments.rs`, con test): usa solo le richieste del client InnerTube. I token di YouTube restano nel motore e valgono solo per l'account con cui sono stati letti; se l'account cambia, i commenti si ricaricano.
- **Impostazioni → Generale → Commenti dei brani**: spento, la scheda sparisce e non si scarica nessun commento.
- I commenti si caricano solo con la scheda aperta e si aggiornano quando cambia brano.
- **Discord dice AmbraMusic**: Application ID nostro, pulsanti in italiano ("Ascolta su YouTube Music", "AmbraMusic su GitHub") e logo di AmbraMusic sulla copertina.
- Icone nuove nel sottoinsieme Lucide: commento, invia, pollice, puntina, risposta, ordina.

# 0.4.0 — 22 ottobre 2026

- **Player nostro** (`crates/player`): scritto da zero sull'API C di libmpv, senza il player di Limusic né il crate `libmpv2`. Solo audio: carica, play/pausa, stop, salto, volume, eventi di fine brano, errore e dispositivo audio perso.
- La posizione arriva all'interfaccia al massimo 4 volte al secondo (prima a ogni aggiornamento di mpv): meno ridisegni, meno CPU.
- Volume su una scala di 50 dB uniforme (ogni passo del cursore ≈ 0,5 dB), 0 = silenzio.
- **Copertine nella coda**: si usano subito quelle già in cache e le mancanti si scaricano in background.
- **Impostazioni con finestra stretta**: i testi a capo ricevono una larghezza esplicita, quindi Informazioni e le altre schede non si tagliano più in fondo; colonna delle schede più stretta e pulsanti uno sotto l'altro quando manca spazio.
- Corretti due errori di compilazione (`h.title`/`h.label` usati dopo lo spostamento) e un avviso.
- Le opzioni di mpv non indispensabili sono "al meglio": una build di libmpv che non ne conosce una non blocca più l'audio.

# 0.3.1 — 22 ottobre 2026

- **Copertine generate** al posto del logo dove manca l'immagine: sfumatura del colore del titolo (sempre lo stesso per lo stesso titolo) con il simbolo del tipo (nota, disco, persona, elenco), come Limusic.
- **Copertine nitide**: YouTube Music manda nei risultati miniature da 60 px; ora si chiede al server la versione da 320 px (`=wN-hN`/`=sN`, `hqdefault` per i video), come `thumb.ts` di Limusic. Cache rinominata (`v2-`) per non riusare le vecchie miniature sfocate.
- Copertine scaricate per tutte le righe di una pagina (fino a 500, prima 50-120).
- **Meno RAM**: ogni elemento decodifica una sola immagine (96 px per righe, coda, playlist e suggerimenti; 240 px solo per schede e testate) invece di due; nell'area di notifica la memoria inutilizzata torna a Windows.
- Nel mixer audio di Windows l'app compare come "AmbraMusic".
- **Informazioni** riscritte: icona grande al centro, collegamenti a github.com/Kyo89-sudo/AmbraMusic (repo, versioni, segnalazioni) e crediti a Limusic. I collegamenti aprono solo i siti del progetto.
- Icona dell'app pre-scalata (96 e 256 px) dove compare piccola o media: niente bordi seghettati su sfondo chiaro.
- **Testi riscritti**: le frasi riprese dalla traduzione italiana di Limusic sono state riscritte con parole nostre ("Cosa vuoi ascoltare?", "Continua con brani simili", "In evidenza", "I tuoi mi piace", "Ora in ascolto", "Archivio e cache"…). Restano solo i termini standard (Playlist, Impostazioni, Precedente…).
- **Foto del profilo Google** nel pulsante dell'account in alto e nella scheda Account (dall'`account_menu` di YouTube, salvata in `avatar.png`); senza foto, l'iniziale del nome.
- Tolti i simboli che Segoe UI non contiene (✕, ▸, ▾, ●) e che comparivano come quadratini: al loro posto testo o icone Lucide.
- **Compilazione più rapida**: libmpv e le librerie Rust compilate stanno in una cache comune (`%LOCALAPPDATA%\AmbraMusic\build-cache`). libmpv si scarica una volta sola (SHA256 verificato), e con uno ZIP nuovo si ricompila solo AmbraMusic invece di ~300 librerie. La cartella `dist` prende la versione da `Cargo.toml`.

# 0.3.0 — 22 ottobre 2026

Riallineamento a Limusic: comportamento e struttura presi dai suoi sorgenti (`state.rs`, `SearchSuggest.svelte`, pagina Cerca, `tray.rs`), testi dalla sua traduzione italiana.

- **Barra laterale come Limusic**: Home, Cerca, Libreria, Brani che ti piacciono, Preferiti locali, Impostazioni e le playlist dell'account con copertina e numero di brani. Possono stare anche a destra, accanto alla coda (Impostazioni → Aspetto).
- **Ricerca come Limusic**: suggerimenti sotto il campo mentre scrivi (completamenti e risultati rapidi), pagina dei risultati con filtri Tutti / Brani / Video / Album / Artisti / Playlist, risultato principale accanto ai brani, scaffali di album, artisti e playlist.
- **Home a scaffali** dal feed di YouTube Music, con saluto.
- **Pagine di playlist, album e artista** con testata (copertina, titolo, numero di brani e durata) e pulsanti Riproduci / Riproduzione casuale; scaffali correlati sotto.
- **Coda come Limusic**: un brano singolo parte con la sua radio (`RDAMVM<id>`, senza doppioni, "<titolo> Radio"); da playlist e album la coda è l'elenco. Riproduzione automatica di brani simili a fine coda (disattivabile).
- **Barra del player** con riproduzione casuale, precedente, Play, successivo, ripeti (tutto / brano), volume con silenzia, coda e vista "In riproduzione".
- **Indietro** tra le pagine.
- **Impostazioni** a schede: Account, Generale, Aspetto, Riproduzione, Dati e memoria, Informazioni.
- **Area di notifica**: la ✕ riduce nella tray (predefinito, come in Limusic), menu Mostra / Riproduci / Successivo / Precedente / Riavvia / Esci.
- **Grafica**: icone Lucide come font (niente più tracciati sottili seghettati), copertine salvate anche a 96 px per righe e barra laterale, icona dell'app a 64 px, colori del tema chiaro rivisti; i contenuti non finiscono più sotto la barra di scorrimento.

# 0.2.1 — 22 ottobre 2026

- **Vista "In riproduzione"** come in Limusic: copertina grande e arrotondata al centro, sfondo tinto con il colore dominante della copertina, titolo/artista/album con cuore, coda a destra. Si apre cliccando copertina o titolo nella barra del player (o con la freccia ▲, o da "In riproduzione" nel menu). Clic sulla copertina = play/pausa, rotella = volume.
- Copertina in alta risoluzione (720 px quadrata): `maxresdefault`/`hqdefault` per le miniature video e `=wN-hN`/`=sN` per le copertine, come `thumb.ts` di Limusic; cache limitata a 120 file in `art/`.
- Renderer **FemtoVG (GPU)** al posto di quello software, con ritorno automatico al software se OpenGL manca: risolve i pulsanti senza trasparenza e la finestra rovinata quando si ripristina dalla barra delle applicazioni.
- Icone vettoriali al posto dei simboli Unicode (⏮ ⏭ ☀ ☾ ♥ …) che Windows può disegnare come emoji opache. Niente più `opacity` né ombre sui pulsanti.

# 0.2.0 — 22 ottobre 2026

- Nuovo nome: **AmbraMusic** (eseguibile `AmbraMusic.exe`, cartella dati `AmbraMusic`).
- Nuova icona: una A che è anche una nota, nello stile ambra lucido dell'originale. ICO multi-dimensione da 16 a 256 px.
- Tema scuro e chiaro con interruttore, salvato nelle impostazioni.
- Controlli coerenti con l'icona: pulsanti e Play con sfumatura ambra, cursori e interruttori personalizzati, campo di ricerca arrotondato.
- **Accesso Google integrato**: finestra WebView2 in un processo separato, con la pagina ufficiale di Google.
- Accesso salvato e cifrato con DPAPI (`session.dat`) e ripristino automatico all'avvio.
- I cookie ruotati da Google vengono riscritti su disco; la sessione rifiutata si rinnova in background e le richieste ripartono da sole.
- "Cambia account Google", "Annulla" durante l'accesso, Cookie header spostato in "Avanzate".
- Disconnetti rimuove anche l'accesso salvato.

# 0.1.0 — 22 ottobre 2026

- Prima GUI Slint ispirata al layout di Limusic, con tema scuro e icona arancione.
- Riutilizzo del client InnerTube, di libmpv, del proxy audio e di Discord Rich Presence.
- Ricerca, raccolta tramite cookie, coda e preferiti locali.
- Demo offline; script Windows per Visual Studio 2026 con rilevamento automatico degli strumenti.
- Migrazione web cipher/PoToken, login integrato, tray e tasti multimediali ancora da completare.
