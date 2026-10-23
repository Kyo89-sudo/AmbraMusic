# AmbraMusic 0.5.5 — Più ordine, stessa musica

AmbraMusic porta YouTube Music in una finestra desktop dedicata per Windows: ricerca, raccolta, coda e copertine, con temi chiaro e scuro e controlli sempre a portata di mano.

La 0.5.5 prosegue il consolidamento della serie 0.5.x. Questa patch elimina percorsi Discord mai utilizzati dall’app e semplifica la gestione interna, conservando l’esperienza di ascolto delle versioni precedenti.

## In questa patch

- Rimossi metodi, messaggi e gestori Discord senza chiamanti nell’app.
- Eliminati lo stato e il caricamento della configurazione usati soltanto da quei percorsi.
- Mantenuti album iniziale del brano, copertine, timeline, play/pausa e attivazione/disattivazione di Discord Rich Presence.
- Controllo di compilazione completato senza avvisi sul codice e 25 test della logica Discord superati.

## Cosa trovi nella serie 0.5.x

- UI con dimensioni intermedie, testi più leggibili e barra progresso più ampia.
- Finestra adattata all’area disponibile del monitor, considerando taskbar e DPI.
- Scheda Aiuto e info rivista e temi neutri con accenti ambra e sfumature dalle copertine.
- Istanza unica: un nuovo avvio richiama la finestra esistente.
- Ricerca dei preferiti e delle copertine in memoria ottimizzata.
- Commenti del brano nel pannello laterale.

Il dettaglio delle singole patch è nel [changelog](../CHANGELOG.md).

## Pacchetto e verifiche

Questa distribuzione contiene **i sorgenti completi**, non un installer o un eseguibile Windows già pronto. Per compilare usa `COMPILA-WINDOWS.cmd`; per provare la demo usa `COMPILA-E-PROVA-DEMO.cmd`.

I 25 test Discord usano il modulo originale con dipendenze reali in un progetto di test isolato. `cargo check` non sostituisce la build release né una prova dell’app con un account reale. Build release, resa visiva e integrazioni complete non sono state verificate end-to-end in questa sessione. Vedi [VERIFICA.md](../VERIFICA.md).

**Stato: beta in sviluppo.** Le segnalazioni riproducibili su interfaccia, riproduzione e integrazioni aiutano a indirizzare i prossimi interventi.

Progetto indipendente, derivato da Limusic e distribuito sotto GPL-3.0-or-later. Nessuna affiliazione con Google, YouTube o Discord.
