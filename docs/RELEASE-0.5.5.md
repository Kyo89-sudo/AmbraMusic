<p align="center"><img src="../assets/icon-256.png" width="96" alt="Logo AmbraMusic"></p>

# AmbraMusic 0.5.5 — Più ordine, stessa musica

YouTube Music in una finestra desktop dedicata per Windows: ricerca, raccolta, coda e copertine, con temi chiaro e scuro e controlli sempre a portata di mano.

**Beta in sviluppo · Windows x64 · Rust + Slint · GPL-3.0-or-later**

<p align="center"><img src="images/ambramusic-0.5.5-light.png" width="1100" alt="AmbraMusic 0.5.5 nel tema chiaro, con copertina e coda laterale"></p>

Screenshot fornito dall’utente; il nome personale è sostituito da “AmbraMusic”. Sono stati aggiunti soltanto angoli arrotondati e ombra alla cornice, preservando il resto dell’interfaccia.

## Funzioni principali

| Area | Cosa puoi fare | Approfondisci |
| :--- | :--- | :--- |
| **Ricerca e scoperta** | Cerca con suggerimenti e filtri; esplora Home, playlist, album e artisti. | [Ricerca e navigazione](USO.md#ricerca-e-navigazione) |
| **Player e coda** | Riproduci, metti in pausa, salta, regola volume e posizione; usa shuffle, ripeti e continuazione automatica. | [Riproduzione e coda](USO.md#riproduzione-e-coda) |
| **Raccolta e preferiti** | Accedi a playlist e brani piaciuti dell’account; gestisci i preferiti locali anche senza accesso. | [Raccolta e preferiti](USO.md#raccolta-e-preferiti) |
| **Copertine e temi** | Scegli tema chiaro o scuro e apri la vista In riproduzione con copertina grande e sfondo armonizzato. | [Aspetto e copertine](USO.md#aspetto-e-copertine) |
| **Accesso Google** | Collega l’account in una finestra dedicata, salva la sessione su Windows o usa il Cookie header avanzato. | [Account e sessione](USO.md#account-e-sessione) |
| **Commenti** | Leggi e ordina i commenti, espandi le risposte e usa le interazioni disponibili per l’account collegato. | [Commenti](USO.md#commenti) |
| **Area di notifica** | Continua l’ascolto dalla tray; richiama la finestra e usa i comandi del menu. Un secondo avvio richiama l’istanza esistente. | [Finestra e area di notifica](USO.md#finestra-e-area-di-notifica) |
| **Discord Rich Presence** | Mostra il brano in ascolto sul profilo Discord e attiva o disattiva l’integrazione dalle Impostazioni. | [Discord](USO.md#discord) |
| **Demo** | Esplora l’interfaccia con dati di esempio senza collegare un account. | [Compilazione e demo](BUILD.md#compilazione-guidata) |


## Novità della patch

La 0.5.5 elimina percorsi Discord mai utilizzati dall’app e semplifica lo stato interno. Conserva album iniziale del brano, copertine, timeline, play/pausa e attivazione/disattivazione della presenza Discord.

Nella serie 0.5.x sono già inclusi UI di dimensione intermedia, barra progresso ampliata, adattamento iniziale alla taskbar, scheda Aiuto e info rivista e gestione dell’istanza unica. Il dettaglio progressivo è nel [changelog](../CHANGELOG.md).

## Download e avvio

Il pacchetto **AmbraMusic-0.5.5-source-documentato.zip** include i sorgenti, la documentazione revisionata e lo screenshot anonimizzato. Il precedente `source-GitHub.zip` rimane il pacchetto iniziale associato al tag: il codice applicativo è lo stesso.

Questi pacchetti contengono **sorgenti**, non un installer né un eseguibile Windows già pronto. Dopo aver preparato i prerequisiti, usa `COMPILA-WINDOWS.cmd` per compilare oppure `COMPILA-E-PROVA-DEMO.cmd` per aprire la demo.

[Prerequisiti e compilazione](BUILD.md) · [Guida alle funzioni](USO.md) · [README](../README.md)

## Verifiche

| Controllo | Esito |
| :--- | :--- |
| Compilazione del codice e della UI con `cargo check` | Superata; nessun avviso sul codice. |
| Logica Discord | 25 test superati sul modulo originale con dipendenze reali. |
| Screenshot | Fornito dall’utente, con ritocco deterministico del nome e della cornice. |
| Build release e prova end-to-end | Non eseguite in questa sessione. |

I test Discord sono stati eseguiti in un progetto isolato; non verificano un account Discord reale. `cargo check` non produce l’eseguibile finale. Per dettagli e limiti consulta [VERIFICA.md](../VERIFICA.md).

## Licenza e crediti

Codice sotto [GPL-3.0-or-later](../LICENSE). AmbraMusic è sviluppato da **Kyo89-sudo** e deriva da [Limusic](https://github.com/SimoHypers/limusic), di **SimoHypers e contributori**. Dipendenze, componenti e provenienza sono documentati in [THIRD-PARTY.md](../THIRD-PARTY.md).

Le copertine musicali restano dei rispettivi titolari. AmbraMusic è indipendente e non affiliato a Google, YouTube o Discord.
