# Verifica AmbraMusic 0.5.6

## Correzione barra inferiore al 125% — 0.5.6

- `cargo check --locked --offline -j 2`: superato sul progetto finale. Nessun avviso sul codice; soltanto il messaggio dell’ambiente relativo agli hard link della cache incrementale.
- Tre test della geometria fisica Windows superati: caso di avvio limitato in altezza al 125%, scale 100/125/150/175/200% con coordinate di monitor negative, mantenimento di una finestra già più piccola dello spazio disponibile.
- Layout Slint 1.18.1 compilato ed eseguito con renderer software e piattaforma headless, usando la UI originale prima e dopo la correzione. Nessuna simulazione manuale dei layout.
- Riprodotto il difetto nella 0.5.5 con area interna 1280 × 600 pixel e scala 1,25: altezza logica 480, barra a y=491 con fondo a 579, quindi fuori dalla finestra.
- Dopo la correzione, nelle aree interne 1280 × 600, 1600 × 850 e 1475 × 875 pixel, il fondo della barra coincide rispettivamente con 480, 680 e 700 unità logiche: sempre entro la finestra. Verificato anche l’ancoraggio dei contenuti all’origine y=0.
- Render delle tre dimensioni ispezionati per la posizione dei comandi; il renderer software di prova può differire dal renderer GPU nell’aspetto di gradienti e bordi. Il progetto di prova resta esterno al pacchetto distribuito.
- File Rust modificati verificati con rustfmt; collegamenti della documentazione controllati.

La prova headless verifica il layout al 125%, non sostituisce l’avvio completo sul monitor Windows dell’utente. Build release con libmpv, accesso Google e ascolto non sono stati rieseguiti per questa patch. La logica Discord è invariata: i 25 test citati sotto si riferiscono alla 0.5.5.

## Documentazione e screenshot

È stato aggiunto uno screenshot della 0.5.5 fornito dall’utente. La modifica finale è deterministica, senza rigenerare l’interfaccia: il nome personale della playlist è sostituito da “AmbraMusic”, mentre il resto dei pixel interni è preservato. Angoli arrotondati e ombra riguardano la sola cornice editoriale. Il risultato generativo preliminare non è incluso nel repository.

La schermata documenta l’aspetto mostrato dall’utente; non è una nuova prova end-to-end eseguita dall’agente. Logo, tabelle e guide aggiornano la presentazione senza modificare il codice applicativo né la versione 0.5.5.

## Cleanup codice morto — 0.5.5

- `cargo check --locked --offline -j 2`: superato sul codice finale. Nessun avviso relativo al codice, inclusi quelli per elementi inutilizzati. Resta soltanto l’avviso dell’ambiente sugli hard link della cache incrementale.
- `rustfmt --edition 2021 --check` sui file Rust modificati: superato.
- 25 test Discord superati sul file src/discord.rs originale incluso direttamente in un progetto di test isolato con dipendenze reali (discord-rich-presence, innertube, tracing e regex), senza simulazioni delle librerie. Il progetto di test è esterno allo ZIP.
- Coperti: timeline e cambio brano, play/pausa, limiti e attese di invio, connessione/backoff, configurazione predefinita, collegamenti, dati mancanti, miniature e brani locali.
- Eliminati tre test che esercitavano esclusivamente i messaggi Album e Config rimossi perché mai inviati dall’app. Il test non si collega a un account Discord reale.
- Verificata assenza dei simboli rimossi nei sorgenti e conservazione della UI 0.5.4, salvo la versione nel titolo.
- Nessuna nuova dipendenza nel progetto distribuito. Nessuna build release o prova visiva dell’app completa eseguita in questo blocco.

## Blocco 2 — Ottimizzazione 0.5.4

- `cargo check --locked --offline -j 2`: superato sulla versione 0.5.4. Due avvisi preesistenti per API Discord inutilizzate, nessun errore e nessun avviso di layout circolare.
- `rustfmt --edition 2021 --check` su main.rs e ui_models.rs: superato.
- Preferiti: un indice HashSet temporaneo per evento sostituisce le scansioni ripetute. Memoria aggiuntiva proporzionale al numero dei preferiti; vengono memorizzati riferimenti, senza clonare gli ID.
- Copertine: mantenuti ordine di ricerca e criterio del primo ID corrispondente; saltata la scansione se manca la miniatura e interrotta la visita degli scaffali dopo un risultato.
- Colori generati: algoritmo invariato, eliminata la copia temporanea del titolo/ID.
- Nessun benchmark end-to-end, nuova prova visiva, build release o test funzionale dell’app completa eseguito in questo blocco. I test dei blocchi precedenti non sono presentati come nuovi test della 0.5.4.
- UI media, barra progresso e gestione istanza unica conservate. Bugfixing previsto nel blocco successivo.

## Blocco 1 — Cleanup 0.5.3

- `cargo check --locked --offline -j 2`: superato sulla versione 0.5.3; restano due avvisi preesistenti per parti Discord inutilizzate.
- Funzioni trasferite da main.rs a ui_models.rs e desktop.rs confrontate con i sorgenti 0.5.2: corpi invariati, eccetto visibilità e formattazione.
- Test del blocco istanza unica rieseguito con un processo figlio reale: superato, incluso rilascio alla chiusura.
- Sei test esistenti sulle copertine rieseguiti: tutti superati; sorgente art.rs invariato.
- File Rust modificati formattati con rustfmt.
- UI e logica applicativa della 0.5.2 conservate; ottimizzazione e bugfixing esclusi da questo blocco.
- Nessuna prova visiva o build release aggiunta: restano i limiti descritti sotto.

## Verifiche precedenti

Nota versione: la 0.5.2 pubblica la revisione UI media e istanza unica già verificata come revisione della 0.5.1. Controllo finale della 0.5.2, inclusa la barra di progresso ampliata: `cargo check --locked --offline -j 2` superato.

Data: 23 ottobre 2026. Ambiente: Windows x64, Rust/Cargo 1.99.1, Visual Studio 2026, Slint 1.18.1.

## Revisione UI media e istanza unica

- UI intermedia: testi principali +1 px, pulsanti 38 px, righe 52–54 px, album circa 160 px, barra laterale 230 px, player 88 px. Finestra iniziale 1180 × 700 con adattamento alla taskbar conservato.
- `cargo check --locked --offline -j 2`: superato dopo le modifiche a UI, istanza unica e riavvio.
- Test Windows dell’istanza unica superato con un vero processo figlio: il secondo processo viene rifiutato, il primo riceve la richiesta di apertura, l’evento viene consumato una sola volta e alla chiusura il blocco si libera.
- Richiamo visivo dalla tray, foreground e riavvio dell’app completa non provati interattivamente.

## Controlli eseguiti

- `cargo check --locked --offline -j 2`: completato con successo sul progetto finale, inclusi sorgenti Rust, compilazione UI Slint e risorse Windows.
- Nessun avviso Slint di calcoli circolari nel layout nella verifica finale.
- `rustfmt --edition 2021 --check src/main.rs`: superato.
- Sei test esistenti di `src/art.rs`, compilati ed eseguiti con Rust reale: tutti superati (URL copertine, miniature, ripieghi, colore dominante e colori generati).
- Archivio ZIP verificato per integrità e presenza dei file originali, esclusi i file di compilazione generati.

Gli avvisi finali riguardano due gruppi di elementi Rust già inutilizzati e la copia dei file della cache al posto degli hard link. Nessun errore di compilazione.

Il lockfile originale non era allineato al manifest: è stato rigenerato offline usando le versioni disponibili, senza modificare le versioni dichiarate nel manifest, salvo la versione del progetto (0.5.2).

## Limiti della verifica

`cargo check` controlla la compilazione ma non produce né collega l’eseguibile finale. Build release, rendering visivo e prove interattive su taskbar/DPI, accesso Google, riproduzione con libmpv, Discord e tray non eseguiti. Le immagini in anteprime sono storiche, non screenshot della 0.5.2.

Per la build completa usare `COMPILA-WINDOWS.cmd`; lo script originale prepara libmpv e genera la cartella dist. Per una prova senza account usare `COMPILA-E-PROVA-DEMO.cmd`.

Controlli manuali consigliati: finestra normale e massimizzata con taskbar su diversi lati, DPI 100/125/150%, pagina Aiuto e info con finestra ridotta, temi chiaro/scuro, coda e commenti, riproduzione e ripristino dalla tray.

Il precedente rapporto è conservato in VERIFICA-0.4-storica.md.
