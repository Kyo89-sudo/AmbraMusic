# Installare AmbraMusic su Windows

## Requisiti

Windows 10 aggiornato o Windows 11 x64, .NET Framework 4.8, connessione Internet per YouTube Music e WebView2 Runtime per l’accesso Google. Il motore audio è incluso nel pacchetto. L’installer usa .NET Framework di Windows e non scarica runtime o componenti aggiuntivi.

## Installazione

1. Scarica l’installer dalla [release GitHub](https://github.com/Kyo89-sudo/AmbraMusic/releases).
2. Chiudi AmbraMusic, comprese le copie portatili: dal menu dell’icona nell’area di notifica scegli **Esci**.
3. Avvia `AmbraMusic-Setup-0.5.7-x64.exe`, scegli se creare il collegamento desktop e premi **Installa**.
4. Al termine scegli **Avvia**, oppure apri AmbraMusic dal menu Start.

L’installazione è per utente e non richiede un account amministratore. Il percorso è `%LOCALAPPDATA%\Programs\AmbraMusic`. Il programma e il motore audio restano nella stessa cartella.

L’installer non è firmato con un certificato Authenticode. Gli hash pubblicati in `SHA256SUMS.txt` permettono di verificare che il file scaricato corrisponda all’artefatto della release.

## Aggiornamento

Esegui il nuovo installer dopo aver chiuso l’app. La cartella viene aggiornata dopo aver preparato e verificato i nuovi file. I file aggiunti manualmente vengono conservati; i file del programma vengono sostituiti. Account, preferenze e cache restano nelle cartelle dati dell’app.

Una cartella AmbraMusic preesistente ma non creata da questo installer non viene sostituita automaticamente: sposta o rinomina quella copia e riprova. Non serve cancellare i dati dell’account.

## Disinstallazione

Usa **Impostazioni Windows → App → AmbraMusic → Disinstalla**, oppure **Disinstalla AmbraMusic** dal menu Start. Il disinstallatore rimuove i file originali riconosciuti e i collegamenti creati per questa installazione. Mantiene file aggiunti o modificati, account e preferenze.

## Diagnosi

Se l’app è aperta, l’installer chiede di chiuderla senza interromperla forzatamente. Gli errori dell’installer vengono registrati in `%LOCALAPPDATA%\AmbraMusic\Installer\setup.log`. Per segnalazioni usa [GitHub Issues](https://github.com/Kyo89-sudo/AmbraMusic/issues), indicando versione di Windows, scala dello schermo e messaggio d’errore. Rimuovi eventuali informazioni personali prima di allegare log.

[Compilare l’app](BUILD.md) · [Sorgenti dell’installer](../installer/README.md) · [Licenze](../THIRD-PARTY.md)
