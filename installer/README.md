# Installer AmbraMusic

Interfaccia Windows Forms (.NET Framework 4.8) e motore di installazione scritti per AmbraMusic, sotto GPL-3.0-or-later. Non usa un wizard Inno Setup/NSIS né scarica componenti durante l’installazione.

## Compilazione

Esegui `CREA-INSTALLER.cmd`, dopo aver installato i prerequisiti dell’app. Lo script usa il compilatore .NET Framework x64 già disponibile su Windows e incorpora un archivio con l’eseguibile release, libmpv, documentazione e licenze.

In alternativa, dopo una build release: `powershell -File scripts/build-installer.ps1 -SkipBuild`.

L’output è `dist/AmbraMusic-Setup-<versione>-x64.exe`. Lo script ricava versione e metadati da Cargo.toml. Non viene applicata una firma Authenticode.

## Comportamento

- Installazione per utente in `%LOCALAPPDATA%\Programs\AmbraMusic`, senza elevazione.
- Collegamenti nel menu Start e collegamento desktop opzionale.
- Registrazione nella lista delle applicazioni di Windows, con disinstallatore dedicato.
- Verifica SHA-256 del payload, estrazione in staging, sostituzione tramite rinomina e ripristino della cartella precedente se la sostituzione fallisce.
- Rifiuto di cartelle non riconosciute, junction, collegamenti simbolici e percorsi esterni nel pacchetto.
- Rifiuto dell’installazione o aggiornamento se AmbraMusic è aperto nella sessione Windows, anche da una cartella portatile; nessuna chiusura forzata.
- Conservazione dei file aggiunti alla cartella durante l’aggiornamento. Il disinstallatore conserva file aggiunti o modificati e non accede ai dati dell’account o alle cache dell’app.

La verifica SHA-256 interna controlla l’integrità del payload incorporato; non autentica l’editore. Per verificare il download usa anche il file SHA256SUMS pubblicato nella release.

## Verifica isolata

`AmbraMusic-Setup-<versione>-x64.exe --self-test C:\percorso\report.txt`

La modalità di prova crea una cartella fixture accanto al report. Esegue installazione, aggiornamento, disinstallazione selettiva e prova dei collegamenti nella fixture. Non modifica desktop, menu Start o registrazione applicazioni di Windows. La fixture con i file di prova conservati rimane disponibile per l’ispezione.

Il disinstallatore copia il proprio piccolo eseguibile nella cartella temporanea di Windows prima di aprire la finestra: questo consente di rimuovere anche la copia installata. I file temporanei rientrano nella normale pulizia di Windows.
