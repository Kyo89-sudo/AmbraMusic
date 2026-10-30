<p align="center"><img src="../assets/icon-256.png" width="96" alt="Logo AmbraMusic"></p>

# AmbraMusic 0.5.8 — Commenti a portata di mano e Discord più stabile

Commenti raggiungibili da un pulsante del player senza ingombrare l’interfaccia, Discord Rich Presence senza sfarfallio al cambio di brano e una correzione alla riproduzione casuale.

## Installazione

| Artefatto | Contenuto |
| :--- | :--- |
| `AmbraMusic-Setup-0.5.8-x64.exe` | Installer per utente: applicazione, libmpv, runtime Visual C++ necessari, documentazione e licenze. Aggiorna la 0.5.7 conservando i file aggiunti. |
| `AmbraMusic-0.5.8-portable-x64.zip` | Versione portabile: estrai la cartella e avvia `AmbraMusic.exe`, senza installazione. I dati dell’app restano nel profilo utente come nella versione installata. |
| `SHA256SUMS.txt` | Hash SHA-256 dei pacchetti. |

Requisiti: Windows 10 aggiornato o Windows 11 x64, .NET Framework 4.8 per l’installer e WebView2 Runtime per l’accesso Google. Pacchetti senza firma Authenticode.

## Modifiche

| Area | Intervento |
| :--- | :--- |
| Commenti | Pulsante nella barra del player, visibile solo con un brano in ascolto: apre la scheda da qualsiasi vista e la richiude riportando il pannello com’era. |
| Commenti | I commenti si scaricano solo mentre la scheda Commenti è sullo schermo, non più in sottofondo a ogni cambio di brano. |
| Commenti | Se YouTube non accetta un commento, il testo resta nel campo e puoi riprovare senza riscriverlo. |
| Commenti | Una risposta resta sotto il commento a cui stai rispondendo, anche se nel frattempo espandi le risposte di un altro commento. |
| Commenti | Foto profilo e mi piace aggiornano solo la loro riga: l’elenco non salta più mentre lo scorri. |
| Accessibilità | Etichette per screen reader su commenti, azioni, campo di scrittura, schede e pulsanti a icona. |
| Discord | La scheda resta durante il cambio di brano invece di sparire e ricomparire; viene tolta se il brano nuovo non parte entro 12 secondi. |
| Coda | Con la riproduzione casuale, brani aggiunti o dalla radio non spariscono spegnendola e quelli tolti non ricompaiono. |

Dettagli in [CHANGELOG.md](../CHANGELOG.md) e [VERIFICA.md](../VERIFICA.md).
