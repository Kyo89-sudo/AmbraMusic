# Compilare AmbraMusic

## Windows x64

### Prerequisiti

- [Rust stable](https://rustup.rs/) con toolchain `x86_64-pc-windows-msvc`.
- Visual Studio o Build Tools con il carico **Sviluppo di applicazioni desktop con C++**, strumenti MSVC x64 e Windows SDK.
- WebView2 Runtime per la finestra di accesso Google.
- Connessione Internet per scaricare le dipendenze mancanti e libmpv.

Usa un percorso breve, per esempio `C:\Progetti\AmbraMusic`, per ridurre i problemi dei tool nativi con nomi di file molto lunghi.

### Compilazione guidata

- `COMPILA-WINDOWS.cmd`: prepara le dipendenze e compila la versione release.
- `COMPILA-E-PROVA-DEMO.cmd`: compila e apre la demo con dati di esempio.
- `CREA-INSTALLER.cmd`: compila l’app e genera l’installer Windows personalizzato.

Lo script `scripts/build-windows.ps1` individua Visual Studio tramite `vswhere`, inizializza gli strumenti x64, prepara il target Rust e scarica una versione fissata di libmpv con verifica SHA256. Poi prepara la libreria MSVC e avvia Cargo.

L’output si trova in `dist\AmbraMusic-<versione>\`. L’eseguibile deve restare accanto a `libmpv-2.dll`.

### Cache e diagnosi

La cache comune è in `%LOCALAPPDATA%\AmbraMusic\build-cache`. Conserva libmpv e i prodotti di compilazione riutilizzabili. La prima build può richiedere diversi minuti.

In caso di errore consulta `COMPILAZIONE.log`. Prima di condividere log, rimuovi eventuali informazioni personali. Un errore di linker relativo a librerie C++ può indicare strumenti MSVC o Windows SDK mancanti, oppure un ambiente di compilazione non inizializzato.

### Controlli per sviluppo

Da una shell con ambiente Visual Studio x64 attivo:

```powershell
cargo check --locked
cargo test --workspace
```

`cargo check` controlla codice Rust e UI Slint, ma non collega né produce l’eseguibile finale. I test che collegano il player richiedono anche le librerie native di libmpv. Alcune prove di rete richiedono un ambiente adeguato: non confondere un comando suggerito con un test già eseguito.

Per conoscere i controlli effettivamente completati consulta [VERIFICA.md](../VERIFICA.md).

## Linux: sviluppo

Il progetto include percorsi Linux, ma il target principale resta Windows. Il login WebView2 non è disponibile su Linux: l’alternativa è il Cookie header; il salvataggio della sessione non usa DPAPI.

Servono toolchain Rust, strumenti C/C++, librerie di sviluppo di mpv e dipendenze grafiche del backend Slint. Esegui `cargo build` e `cargo run -- --demo` in un ambiente configurato; la build Linux completa non è stata verificata in questa sessione.

## Generare l’installer

`CREA-INSTALLER.cmd` usa la build release e compila interfaccia e motore dell’installer dai sorgenti C# in `installer/`, tramite il compilatore .NET Framework x64 di Windows. Non richiede Inno Setup o NSIS.

Dopo una build già completata puoi eseguire `powershell -File scripts/build-installer.ps1 -SkipBuild`. Per mantenere la cache in una cartella specifica, passa `-CacheDir C:\Percorso\Cache` agli script di build.

Dettagli su formato del pacchetto, aggiornamento e test isolati: [installer/README.md](../installer/README.md).
