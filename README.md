<p align="center">
  <img src="assets/icon-256.png" width="96" alt="Icona AmbraMusic">
</p>

# AmbraMusic

**La tua musica, in una finestra dedicata.**

AmbraMusic è un client desktop indipendente per YouTube Music, pensato per Windows e scritto in **Rust e Slint**. Riunisce ricerca, raccolta, coda e controlli di riproduzione in un’interfaccia con temi chiaro e scuro, accenti ambra e colori ispirati alle copertine.

Il progetto nasce dal lavoro di [Limusic](https://github.com/SimoHypers/limusic), di cui conserva parte del codice e l’impostazione di alcune funzioni, e prosegue con un’interfaccia Slint e un player basato direttamente su libmpv.

**Versione attuale: 0.5.5 · Beta in sviluppo · Windows x64 · GPL-3.0-or-later**

[Novità](CHANGELOG.md) · [Compilazione](docs/BUILD.md) · [Note della 0.5.5](docs/RELEASE-0.5.5.md) · [Verifiche](VERIFICA.md) · [Segnala un problema](https://github.com/Kyo89-sudo/AmbraMusic/issues)

## Ascolto al centro

- **Trova cosa ascoltare.** Ricerca con suggerimenti e filtri, Home a scaffali e pagine dedicate a playlist, album e artisti.
- **Organizza la riproduzione.** Coda, riproduzione casuale, ripeti e continuazione automatica con brani simili.
- **Scegli la tua vista.** Player sempre disponibile e vista In riproduzione con copertina grande, titolo, artista e album.
- **Ritrova la tua raccolta.** Accesso Google integrato, playlist e brani piaciuti dell’account; preferiti locali anche senza account.
- **Leggi la conversazione.** Pannello commenti con ordinamento, risposte e interazioni disponibili per l’account collegato.
- **Tieni la musica con te.** Area di notifica con comandi, ripristino della finestra e protezione dagli avvii duplicati.
- **Mostra cosa ascolti.** Discord Rich Presence, attivabile dalle Impostazioni.

## Un’interfaccia che si adatta

La serie 0.5.x rivede dimensioni e leggibilità: comandi più equilibrati, barra di avanzamento più ampia e scheda Aiuto e info con testi a capo e contenuti scorrevoli. All’avvio, su Windows, la finestra viene adattata all’area utilizzabile del monitor, considerando taskbar e scala DPI.

I temi chiaro e scuro usano superfici neutre, mentre gli accenti ambra identificano i comandi principali. La copertina del brano aggiunge una sfumatura di colore al player e alla vista In riproduzione.

Un secondo avvio richiama l’istanza già aperta, anche quando è nascosta nell’area di notifica. La finestra separata dell’accesso Google resta consentita.

## Provala dai sorgenti

Questo repository distribuisce i sorgenti. **Non è un installer Windows e non contiene un eseguibile release verificato in questa sessione.**

Occorrono Rust stable con toolchain MSVC x64, Visual Studio o Build Tools con strumenti C++ e Windows SDK. Per l’accesso Google serve WebView2 Runtime.

1. Scarica o clona il repository ed estrailo in una cartella con un percorso breve.
2. Esegui `COMPILA-E-PROVA-DEMO.cmd` per compilare e aprire la demo grafica con dati di esempio.
3. Per l’uso normale esegui `COMPILA-WINDOWS.cmd`, poi avvia `dist\AmbraMusic-0.5.5\AmbraMusic.exe`.

Mantieni `AmbraMusic.exe` e `libmpv-2.dll` nella stessa cartella. La demo usa contenuti di esempio e non richiede un account Google.

La [guida alla compilazione](docs/BUILD.md) spiega prerequisiti, cache, log e controlli disponibili.

## Accesso e dati locali

Apri **Impostazioni → Account → Accedi con Google**. L’accesso avviene sulla pagina di Google in una finestra WebView2 separata. AmbraMusic acquisisce i cookie della sessione per collegarsi a YouTube Music.

Su Windows la sessione salvata viene cifrata con DPAPI e associata all’utente locale. Il profilo WebView2 è separato dal browser abituale. Puoi cambiare account, annullare l’accesso o disconnetterti dalle Impostazioni.

È disponibile anche l’inserimento manuale del Cookie header nelle opzioni avanzate. **Cookie e file di sessione sono credenziali: non inserirli nelle issue, nei log condivisi o nei commit.**

## Stato del progetto

AmbraMusic è una beta in evoluzione. Il lavoro attuale si concentra su usabilità, ordine del codice e consolidamento delle funzioni esistenti.

Per la 0.5.5 sono stati completati il controllo di compilazione con `cargo check` e **25 test della logica Discord** eseguiti sul modulo originale in un progetto di test isolato con dipendenze reali. Non sono emersi avvisi di codice inutilizzato.

Questi controlli non sostituiscono una prova completa dell’app: build release, accesso con un account reale, ascolto prolungato, tray e comportamento visivo su diversi monitor non sono stati verificati end-to-end in questa sessione. Il [rapporto delle verifiche](VERIFICA.md) distingue i controlli attuali da quelli storici.

### Limiti noti

- La riproduzione dipende dai servizi e dalle risposte di YouTube Music: alcuni brani o sessioni possono richiedere ulteriori correzioni del client.
- Non è disponibile la scelta del canale per account con più canali YouTube.
- Testi sincronizzati, video, modifica delle playlist, tasti multimediali di sistema e aggiornamenti automatici non sono inclusi.
- Linux è un ambiente di sviluppo secondario: l’accesso Google integrato è specifico di Windows.
- Finché il repository resta privato, il logo Discord ospitato su GitHub non è recuperabile pubblicamente tramite il suo URL.
- Le immagini in `anteprime/` documentano versioni precedenti e non rappresentano la UI attuale della 0.5.5.

## Segnalazioni utili

Apri una [issue](https://github.com/Kyo89-sudo/AmbraMusic/issues) indicando versione di AmbraMusic, versione di Windows, scala DPI, passaggi per riprodurre il problema e risultato atteso. Uno screenshot può aiutare per i difetti dell’interfaccia.

Rimuovi da log e immagini cookie, dati di sessione e informazioni personali prima di condividerli.

## Crediti e licenza

AmbraMusic è sviluppato da **Kyo89-sudo** a partire da [Limusic](https://github.com/SimoHypers/limusic), di **SimoHypers e contributori**. Il riferimento upstream e le parti derivate sono documentati in [THIRD-PARTY.md](THIRD-PARTY.md).

Il codice è distribuito sotto [GPL-3.0-or-later](LICENSE). Le licenze delle dipendenze e delle icone restano quelle dei rispettivi autori.

AmbraMusic è un progetto indipendente e non è affiliato a Google, YouTube o Discord.
