<p align="center"><img src="../assets/icon-256.png" width="96" alt="Logo AmbraMusic"></p>

# AmbraMusic 0.5.6 — Controlli visibili anche al 125%

Questa patch affronta il taglio della barra inferiore segnalato all’avvio con la scala di Windows al **125%**. Play/pausa e avanzamento hanno uno spazio riservato, indipendente dall’altezza richiesta dai contenuti della pagina.

**Beta in sviluppo · Windows x64 · Rust + Slint · GPL-3.0-or-later**

## Correzioni

| Area | Intervento |
| :--- | :--- |
| Barra del player | Ancorata al fondo dell’area interna; i contenuti superiori non possono spingerla sotto il bordo visibile. |
| Pulsante Play | Riga dei comandi con altezza coerente al pulsante, mantenendo le dimensioni dei testi. |
| Avvio e taskbar | Posizionamento basato sull’area utilizzabile del monitor e sui bordi reali della finestra Windows. |
| DPI | Secondo adattamento dopo l’inizializzazione della finestra; nessuna conversione approssimativa dei bordi. |
| Finestre basse | Altezza minima ridotta per consentire l’adattamento con scala elevata. |
| Massimizzazione | Una finestra già massimizzata non viene ridimensionata dal controllo iniziale. |

L’interfaccia mantiene il dimensionamento intermedio della serie 0.5.x. Non occorre impostare lo schermo al 100% per usare la patch.

## Download e prova

Il pacchetto **AmbraMusic-0.5.6-source.zip** contiene i sorgenti completi. Non include un installer né un eseguibile release già compilato.

1. Chiudi la versione precedente scegliendo **Esci** dal menu dell’icona nell’area di notifica: la sola X può lasciare attiva l’istanza 0.5.5.
2. Estrai lo ZIP e compila con `COMPILA-WINDOWS.cmd`, oppure usa `COMPILA-E-PROVA-DEMO.cmd` per la demo.
3. Avvia la 0.5.6 e controlla il titolo della finestra. Lascia la scala dello schermo al 125% e verifica Play e avanzamento all’apertura.

[Guida alla compilazione](BUILD.md) · [Funzioni](USO.md) · [Changelog](../CHANGELOG.md) · [Segnalazioni](https://github.com/Kyo89-sudo/AmbraMusic/issues)

## Verifiche

Controllo di compilazione superato, tre test della geometria superati e confronto del layout Slint eseguito al 125%: riprodotto il difetto nella 0.5.5 e verificata la barra nella finestra in tre dimensioni della 0.5.6.

Le verifiche e i relativi limiti sono riportati in [VERIFICA.md](../VERIFICA.md). La prova software del layout non sostituisce un avvio completo su tutte le configurazioni Windows e monitor reali.

## Licenza e crediti

Codice sotto [GPL-3.0-or-later](../LICENSE). AmbraMusic è sviluppato da **Kyo89-sudo** a partire da [Limusic](https://github.com/SimoHypers/limusic), di **SimoHypers e contributori**. Vedi [THIRD-PARTY.md](../THIRD-PARTY.md) per provenienza e dipendenze.

AmbraMusic è indipendente e non affiliato a Google, YouTube o Discord.
