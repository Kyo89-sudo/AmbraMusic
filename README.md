<p align="center">
  <img src="assets/icon-256.png" width="112" alt="Logo AmbraMusic">
</p>
<h1 align="center">AmbraMusic</h1>
<p align="center"><strong>La tua musica, in una finestra dedicata.</strong></p>
<p align="center">
  <a href="https://github.com/Kyo89-sudo/AmbraMusic/releases/tag/v0.5.5"><img src="https://img.shields.io/badge/versione-0.5.5-f59e0b?style=flat-square" alt="Versione 0.5.5"></a>
  <img src="https://img.shields.io/badge/piattaforma-Windows_x64-2563eb?style=flat-square" alt="Windows x64">
  <img src="https://img.shields.io/badge/stato-beta_in_sviluppo-64748b?style=flat-square" alt="Beta in sviluppo">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licenza-GPL--3.0--or--later-16a34a?style=flat-square" alt="Licenza GPL-3.0-or-later"></a>
</p>
<p align="center">
  <a href="https://github.com/Kyo89-sudo/AmbraMusic/releases/tag/v0.5.5">Scarica i sorgenti</a> ·
  <a href="docs/BUILD.md">Compila e prova</a> ·
  <a href="#funzioni">Funzioni</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="https://github.com/Kyo89-sudo/AmbraMusic/issues">Segnala un problema</a>
</p>

AmbraMusic è un client desktop indipendente per **YouTube Music**, pensato per Windows e scritto in **Rust e Slint**. Ricerca, raccolta e coda convivono in un’interfaccia dedicata all’ascolto, con temi chiaro e scuro, accenti ambra e sfumature ispirate alle copertine.

Il progetto deriva da [Limusic](https://github.com/SimoHypers/limusic), di cui conserva parte del codice e l’impostazione di alcune funzioni, e prosegue con un’interfaccia Slint e un player basato direttamente su libmpv.

## Anteprima

<p align="center">
  <img src="docs/images/ambramusic-0.5.5-light.png" width="1100" alt="AmbraMusic 0.5.5 nel tema chiaro: copertina del brano, coda laterale e controlli di riproduzione">
</p>
<p align="center"><sub>Vista In riproduzione · Tema chiaro · AmbraMusic 0.5.5</sub></p>

Screenshot fornito dall’utente, con nome personale sostituito da “AmbraMusic” e sola cornice editoriale. Copertina, contenuti e controlli dell’interfaccia sono quelli dello screenshot originale.

## Funzioni

| Area | Cosa puoi fare | Approfondisci |
| :--- | :--- | :--- |
| **Ricerca e scoperta** | Cerca con suggerimenti e filtri; esplora Home, playlist, album e artisti. | [Ricerca e navigazione](docs/USO.md#ricerca-e-navigazione) |
| **Player e coda** | Riproduci, metti in pausa, salta, regola volume e posizione; usa shuffle, ripeti e continuazione automatica. | [Riproduzione e coda](docs/USO.md#riproduzione-e-coda) |
| **Raccolta e preferiti** | Accedi a playlist e brani piaciuti dell’account; gestisci i preferiti locali anche senza accesso. | [Raccolta e preferiti](docs/USO.md#raccolta-e-preferiti) |
| **Copertine e temi** | Scegli tema chiaro o scuro e apri la vista In riproduzione con copertina grande e sfondo armonizzato. | [Aspetto e copertine](docs/USO.md#aspetto-e-copertine) |
| **Accesso Google** | Collega l’account in una finestra dedicata, salva la sessione su Windows o usa il Cookie header avanzato. | [Account e sessione](docs/USO.md#account-e-sessione) |
| **Commenti** | Leggi e ordina i commenti, espandi le risposte e usa le interazioni disponibili per l’account collegato. | [Commenti](docs/USO.md#commenti) |
| **Area di notifica** | Continua l’ascolto dalla tray; richiama la finestra e usa i comandi del menu. Un secondo avvio richiama l’istanza esistente. | [Finestra e area di notifica](docs/USO.md#finestra-e-area-di-notifica) |
| **Discord Rich Presence** | Mostra il brano in ascolto sul profilo Discord e attiva o disattiva l’integrazione dalle Impostazioni. | [Discord](docs/USO.md#discord) |
| **Demo** | Esplora l’interfaccia con dati di esempio senza collegare un account. | [Compilazione e demo](docs/BUILD.md#compilazione-guidata) |

La tabella descrive le funzioni implementate; i controlli effettivamente eseguiti sono documentati in [VERIFICA.md](VERIFICA.md).

## Inizia da qui

Questa distribuzione contiene **i sorgenti completi**. Non è un installer Windows e non include un eseguibile release verificato in questa sessione.

1. Scarica i sorgenti dalla [release v0.5.5](https://github.com/Kyo89-sudo/AmbraMusic/releases/tag/v0.5.5) oppure clona il repository.
2. Prepara Rust stable MSVC x64, gli strumenti C++ di Visual Studio/Build Tools e Windows SDK. Per l’accesso Google serve WebView2 Runtime.
3. Avvia `COMPILA-E-PROVA-DEMO.cmd` per la demo oppure `COMPILA-WINDOWS.cmd` per compilare l’app.
4. Dopo la build, apri `dist\AmbraMusic-0.5.5\AmbraMusic.exe`, mantenendo `libmpv-2.dll` nella stessa cartella.

[Guida completa alla compilazione](docs/BUILD.md) · [Guida alle funzioni](docs/USO.md) · [Note della release](docs/RELEASE-0.5.5.md)

## Cosa cambia nella 0.5.5

Questa patch consolida la serie 0.5.x: rimuove percorsi Discord mai utilizzati dall’app e semplifica lo stato interno. Conserva la UI di dimensione intermedia, la barra progresso ampliata, l’adattamento iniziale alla taskbar e la gestione dell’istanza unica introdotti nelle patch precedenti.

Per l’elenco progressivo delle modifiche consulta il [changelog](CHANGELOG.md). Le [note dettagliate archiviate](docs/CHANGELOG-DETTAGLIATO.md) mantengono lo storico del lavoro.

## Verifiche e stato del progetto

| Controllo | Esito sulla 0.5.5 | Cosa dimostra |
| :--- | :--- | :--- |
| Controllo di compilazione Rust e Slint | Superato | Codice e UI compilabili con `cargo check`; nessun avviso di codice inutilizzato. |
| Logica Discord | 25 test superati | Test sul modulo originale con dipendenze reali in un progetto di test isolato. |
| Interfaccia mostrata sopra | Screenshot fornito dall’utente | Aspetto della vista In riproduzione; non sostituisce una verifica di ogni schermata. |
| Build release e prova end-to-end | Non eseguite in questa sessione | Da verificare con librerie native, account reale e ascolto prolungato. |

AmbraMusic è una **beta in sviluppo**. La priorità è consolidare l’ascolto e correggere problemi riproducibili, mantenendo il lavoro per versioni successive. Il [rapporto di verifica](VERIFICA.md) distingue i controlli attuali da quelli storici.

### Limiti noti

- La riproduzione dipende dalle risposte di YouTube Music; alcuni brani o sessioni possono richiedere ulteriori correzioni del client.
- Non è disponibile la scelta del canale per account con più canali YouTube.
- Testi sincronizzati, video, modifica delle playlist, tasti multimediali di sistema e aggiornamenti automatici non sono inclusi.
- Linux è un ambiente di sviluppo secondario; l’accesso Google integrato è specifico di Windows.
- Finché il repository resta privato, il logo Discord ospitato su GitHub non è recuperabile pubblicamente tramite il suo URL.
- Le immagini in `anteprime/` sono storiche; lo screenshot attuale è in `docs/images/`.

## Account e riservatezza

L’accesso si apre dalla pagina **Impostazioni → Account**. Google viene mostrato in una finestra WebView2 separata; AmbraMusic acquisisce i cookie di sessione necessari per collegarsi a YouTube Music. Su Windows la sessione salvata viene cifrata con DPAPI e associata all’utente locale.

Cookie, file di sessione e dati di accesso sono credenziali: non inserirli nelle issue, nei log condivisi o nei commit. I dettagli sono nella [guida account e sessione](docs/USO.md#account-e-sessione).

## Segnalazioni e contributi

Per una segnalazione utile, [apri una issue](https://github.com/Kyo89-sudo/AmbraMusic/issues) con versione di AmbraMusic, versione di Windows, scala DPI, passaggi per riprodurre il problema e risultato atteso. Per difetti grafici aggiungi, se possibile, uno screenshot senza dati personali.

Per proporre un intervento, descrivi prima il problema e mantieni le modifiche circoscritte. I contributi devono rispettare la licenza del progetto e i crediti del codice derivato.

## Tecnologie, crediti e licenza

| Componente | Ruolo | Riferimento |
| :--- | :--- | :--- |
| **Rust** | Logica applicativa e coordinamento dei servizi | [rust-lang.org](https://www.rust-lang.org/) |
| **Slint** | Interfaccia desktop | [slint.dev](https://slint.dev/) |
| **libmpv** | Riproduzione audio tramite API C | [mpv.io](https://mpv.io/) |
| **WebView2** | Finestra di accesso Google su Windows | [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) |
| **Lucide** | Icone dell’interfaccia | [Lucide](https://lucide.dev/) · [licenza inclusa](assets/fonts/LUCIDE-LICENSE.txt) |
| **Limusic** | Progetto di origine e parti di codice derivate | [SimoHypers/limusic](https://github.com/SimoHypers/limusic) |

AmbraMusic è sviluppato da **Kyo89-sudo**, a partire dal lavoro di **SimoHypers e contributori** su Limusic. Provenienza, commit di riferimento e dipendenze sono descritti in [THIRD-PARTY.md](THIRD-PARTY.md).

Il codice è distribuito sotto **GPL-3.0-or-later**: consulta il [testo della licenza](LICENSE). Dipendenze, icone e altri componenti mantengono le rispettive licenze; le copertine musicali mostrate nello screenshot appartengono ai rispettivi titolari.

AmbraMusic è un progetto indipendente e non è affiliato a Google, YouTube o Discord.
