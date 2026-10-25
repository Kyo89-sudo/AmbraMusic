<p align="center"><img src="../assets/icon-256.png" width="96" alt="Logo AmbraMusic"></p>

# AmbraMusic 0.5.7 — Installer Windows e correzioni UI

Questa versione introduce un installer Windows x64 dedicato ad AmbraMusic e corregge alcuni problemi di layout nelle finestre basse. Interfaccia, motore di aggiornamento e disinstallatore sono scritti per il progetto; non sono un tema applicato a un wizard di terze parti.

## Installazione

Scarica **AmbraMusic-Setup-0.5.7-x64.exe**, chiudi le istanze già aperte scegliendo **Esci** dall’area di notifica e avvia l’installer. L’installazione è per utente, con collegamenti nel menu Start e collegamento desktop opzionale.

| Artefatto | Contenuto |
| :--- | :--- |
| `AmbraMusic-Setup-0.5.7-x64.exe` | Applicazione, libmpv, runtime Visual C++ necessari, documentazione e licenze. |
| `AmbraMusic-0.5.7-source.zip` | Sorgenti completi dell’app e dell’installer, lockfile e script di compilazione. |
| `SHA256SUMS.txt` | Hash SHA-256 dei due pacchetti della release. |

Requisiti: Windows 10 aggiornato o Windows 11 x64, .NET Framework 4.8 e WebView2 Runtime per l’accesso Google. L’installer non scarica componenti durante l’installazione e non richiede elevazione. Il pacchetto non ha firma Authenticode.

<p align="center"><img src="images/ambramusic-setup-0.5.7.png" width="500" alt="Installer personalizzato AmbraMusic 0.5.7"></p>

## Modifiche

| Area | Intervento |
| :--- | :--- |
| Distribuzione | Installer e disinstallatore dedicati, registrazione tra le applicazioni Windows e avanzamento dell’installazione. |
| Aggiornamento | Verifica SHA-256 del payload, preparazione in staging e ripristino della cartella precedente se la sostituzione fallisce. |
| File dell’utente | Conservazione dei file aggiunti durante l’aggiornamento; la disinstallazione conserva quelli aggiunti o modificati e non accede ai dati dell’account. |
| Aiuto e info | Intestazione più compatta, collegamenti raggruppati e crediti coerenti con il resto delle impostazioni. |
| Navigazione | Barra laterale e indice delle impostazioni scorrevoli nelle finestre basse. |
| In riproduzione | Copertina e coda adattate allo spazio; area delle informazioni del brano più larga anche con copertina piccola. |
| Ricerca | Suggerimenti scorrevoli e ancorati sopra il player, senza intercettare i clic sui comandi inferiori. |
| Build | Lockfile bloccato, cache configurabile e raccolta delle licenze dei pacchetti risolti da Cargo. |

## Verifiche

- Controllo Rust/Slint e build release Windows x64 completati; avvio nativo, Home, Impostazioni e Aiuto verificati al 125%.
- Cinque configurazioni di layout alle scale 100%, 125%, 150% e 200%, entrambi i temi e interazioni di scorrimento/clic provate. Confronto prima/dopo della vista In riproduzione in quattro dimensioni.
- Cinque prove sul payload reale dell’installer: installazione, aggiornamento, disinstallazione selettiva, protezione delle cartelle estranee e collegamenti. Verificati anche i percorsi lunghi.

Le prove del motore operano in una fixture e non modificano la registrazione applicazioni reale. Non è stata eseguita una matrice su PC Windows puliti; login Google da zero e ascolto prolungato non sono stati ripetuti in questa patch. Dettagli in [VERIFICA.md](../VERIFICA.md).

## Riferimenti

[Confronto con v0.5.6](https://github.com/Kyo89-sudo/AmbraMusic/compare/v0.5.6...v0.5.7) · [Changelog](../CHANGELOG.md) · [Installazione](INSTALLAZIONE.md) · [Compilazione](BUILD.md) · [Sorgenti dell’installer](../installer/README.md)

## Licenza e crediti

AmbraMusic e il suo installer sono distribuiti sotto [GPL-3.0-or-later](../LICENSE). Il progetto è sviluppato da **Kyo89-sudo** a partire da [Limusic](https://github.com/SimoHypers/limusic), di SimoHypers e contributori. Le dipendenze mantengono le proprie licenze: [THIRD-PARTY.md](../THIRD-PARTY.md).

Progetto indipendente, non affiliato a Google, YouTube o Discord.
