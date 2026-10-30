<p align="center"><img src="../assets/icon-256.png" width="96" alt="Logo AmbraMusic"></p>

# AmbraMusic 0.5.9 — Account visibile subito all’avvio

All’avvio, se hai un accesso salvato, AmbraMusic mostra subito il tuo account invece del pulsante Accedi, mentre verifica la sessione con YouTube Music.

## Installazione

| Artefatto | Contenuto |
| :--- | :--- |
| `AmbraMusic-Setup-0.5.9-x64.exe` | Installer per utente; aggiorna la versione installata conservando i file aggiunti. |
| `AmbraMusic-0.5.9-portable-x64.zip` | Versione portabile: estrai la cartella e avvia `AmbraMusic.exe`. |
| `SHA256SUMS.txt` | Hash SHA-256 dei pacchetti. |

Requisiti: Windows 10 aggiornato o Windows 11 x64, .NET Framework 4.8 per l’installer e WebView2 Runtime per l’accesso Google. Pacchetti senza firma Authenticode.

## Modifiche

| Area | Intervento |
| :--- | :--- |
| Account | Nome e foto dell’account compaiono subito, attenuati, finché la verifica in rete non conferma l’accesso. Se la verifica non riesce torna il pulsante Accedi. |
| Account | Dopo l’aggiornamento serve un primo avvio con accesso verificato; dal successivo l’account appare subito. |

Dettagli in [CHANGELOG.md](../CHANGELOG.md) e [VERIFICA.md](../VERIFICA.md).
