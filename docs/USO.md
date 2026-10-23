# Guida alle funzioni

[← Torna al README](../README.md) · [Compilazione](BUILD.md) · [Changelog](../CHANGELOG.md)

## Ricerca e navigazione

Usa il campo di ricerca nella parte superiore della finestra per cercare brani, video, album, artisti e playlist. I suggerimenti aiutano a completare la ricerca; nei risultati puoi scegliere un filtro per tipo di contenuto.

La Home propone contenuti a scaffali. Le pagine di album e playlist includono una testata e i comandi di riproduzione; il comando Indietro consente di tornare alla pagina precedente. I contenuti disponibili dipendono dalle risposte di YouTube Music e dall’account collegato.

## Riproduzione e coda

La barra inferiore raccoglie play/pausa, precedente e successivo, riproduzione casuale, ripeti, volume e avanzamento. Clicca o trascina sulla barra progresso per cambiare posizione nel brano.

Apri il pannello della coda per vedere i brani in attesa, scegliere quale riprodurre o rimuovere un elemento. I comandi di aggiunta presenti negli elenchi consentono di accodare brani. Dalle Impostazioni puoi scegliere se continuare con brani simili al termine della coda.

L’avvio di un brano singolo e quello di una playlist possono costruire la coda in modo diverso: il primo può usare la radio del brano, mentre album e playlist mantengono la propria sequenza.

## Raccolta e preferiti

Con un account collegato, la barra laterale mostra le playlist disponibili e consente di accedere alla raccolta e ai brani piaciuti. Il cuore rappresenta la valutazione del brano quando l’operazione è disponibile per l’account.

Senza account, il cuore gestisce i preferiti locali. Questi non equivalgono a una modifica della raccolta Google. La modifica e la creazione di playlist non sono incluse.

## Aspetto e copertine

Passa dal tema chiaro a quello scuro con il comando sole/luna nella barra laterale oppure da **Impostazioni → Aspetto**. L’impostazione viene conservata tra gli avvii.

Clicca sulla copertina o sulle informazioni del brano nel player per aprire **In riproduzione**. La vista mostra una copertina grande e uno sfondo armonizzato al suo colore. Quando manca un’immagine, AmbraMusic usa una copertina generata con un colore stabile per l’elemento.

La finestra iniziale si adatta all’area utilizzabile del monitor su Windows, considerando taskbar e scala DPI. Questo adattamento non sostituisce una prova su ogni configurazione di schermo.

## Account e sessione

Apri **Impostazioni → Account → Accedi con Google**. Completa l’accesso nella pagina Google della finestra dedicata. Sono disponibili anche cambio account e annullamento dell’accesso in corso.

Su Windows la sessione può essere salvata in `session.dat`, cifrata con DPAPI per l’utente locale. Il profilo della finestra WebView2 è separato dal browser abituale. La disconnessione elimina la sessione salvata dall’app e ferma la riproduzione; il profilo WebView2 può ancora ricordare l’account Google.

L’accesso avanzato tramite **Cookie header** è un’alternativa al login integrato. Il valore deve provenire dalla propria sessione YouTube Music e contenere le informazioni richieste dal client. Non condividere cookie, file di sessione, profili WebView2 o schermate con dati personali.

Gli account con più canali usano il canale predefinito: non è presente un selettore del canale.

## Commenti

Nel pannello laterale seleziona la scheda **Commenti** per leggere la discussione del brano corrente. Sono disponibili ordinamento, risposte espandibili e interazioni quando consentite dall’account e dal servizio.

Puoi disattivare la funzione da **Impostazioni → Generale → Commenti dei brani**. I commenti vengono richiesti quando la scheda è aperta; i token restano nel motore e sono associati all’account che li ha ottenuti.

## Finestra e area di notifica

Se è attiva l’opzione di permanenza nell’area di notifica, la X nasconde la finestra mentre la musica continua. Il menu dell’icona consente di richiamare l’app, controllare l’ascolto, riavviare o uscire.

Su Windows è consentita una sola istanza desktop. Un secondo avvio segnala quella già aperta e ne richiede la visualizzazione; il processo separato del login Google resta consentito. Il comando Riavvia attende la chiusura prima di aprire una nuova istanza.

## Discord

Attiva o disattiva **Stato su Discord** da **Impostazioni → Generale**. L’integrazione usa il client Discord locale per mostrare le informazioni del brano; non richiede un accesso Discord separato dentro AmbraMusic.

La presenza dipende dalla disponibilità del client e del collegamento IPC. Finché il repository resta privato, il logo AmbraMusic ospitato su GitHub non è pubblicamente recuperabile da Discord. La configurazione grafica della presenza non è modificabile dall’interfaccia della 0.5.5.

## Demo e limiti delle prove

Per aprire l’interfaccia con dati di esempio usa `COMPILA-E-PROVA-DEMO.cmd` oppure avvia l’eseguibile con `--demo`. La demo non verifica l’accesso Google né la riproduzione reale.

Per distinguere le funzioni implementate dai controlli già eseguiti consulta [VERIFICA.md](../VERIFICA.md).
