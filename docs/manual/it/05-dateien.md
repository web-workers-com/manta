# Gestire i file {#dateien}

Copiare, spostare, rinominare, eliminare e creare cartelle: sul tuo Mac, sui server, nei container e tra tutti questi. Leggi con attenzione soprattutto la sezione sull’[eliminazione](#loeschen): sul tuo Mac gli elementi eliminati finiscono nel Cestino; sui server l’eliminazione è definitiva.

## Copiare file in un altro pannello {#kopieren}

1. Nel **pannello di destinazione**, apri la cartella in cui devono andare i file.
2. Nel **pannello attivo**, seleziona i file e le cartelle.
3. Premi **F5** (**Copia nell’altro pannello…**).
4. Nella finestra controlla cosa va dove e scegli cosa succede con i nomi uguali (vedi [Conflitti di nome](#konflikte)).
5. Fai clic su **Copia**.

La copia avviene in background. Puoi continuare subito a lavorare.

![Finestra di copia con destinazione e scelta in caso di conflitto](images/copy-dialog.png)

**Sposta** funziona allo stesso modo con **F6** (**Sposta nell’altro pannello…**). Gli elementi spostati scompaiono dalla posizione precedente.

Entrambi i comandi si trovano anche nella barra degli strumenti (“Copia nel pannello destro”, “Sposta nel pannello sinistro” …) e nel menu contestuale (clic destro).

### Qual è il pannello di destinazione? {#ziel}

Con due pannelli è l’altro. Con tre o più è l’altro pannello attivo più di recente (con il contrassegno **→**, vedi [Pannello di destinazione](02-bereiche-tabs.md#zielbereich)). In **In**, nella finestra, puoi scegliere qualsiasi altro pannello.

Manta previene gli errori tipici:

- “Entrambi i pannelli mostrano la stessa cartella — apri una cartella diversa nell’altro pannello”
- “Una cartella non può essere copiata in se stessa”

### Tra posizioni diverse {#orte}

Copia e spostamento funzionano tra tutte le posizioni: dal tuo Mac a un server SFTP, da un bucket S3 a un container Docker, da un archivio alla Scrivania. Se ne occupa Manta stesso; non ti servono strumenti aggiuntivi.

Dalle posizioni di **sola lettura** (archivi, container arrestati) uno spostamento si limita a copiare. Manta te lo segnala, ad esempio con “Gli archivi sono di sola lettura: gli elementi vengono estratti (copiati), non spostati”.

## Conflitti di nome {#konflikte}

Se nella destinazione esiste già un elemento con lo stesso nome, decide l’impostazione **Se un elemento esiste già** nella finestra:

| Opzione | Cosa succede |
|---|---|
| **Chiedi** | Manta si ferma a ogni elemento già esistente e ti chiede cosa fare (impostazione predefinita). |
| **Sostituisci** | Sovrascrive sempre l’elemento esistente. |
| **Salta** | Lascia invariati gli elementi esistenti. |
| **Sostituisci se più recente** | Sovrascrive solo se l’origine è più recente o ha una dimensione diversa. |
| **Mantieni entrambi** | La copia riceve un nuovo nome (“nome 2”). |

Le altre opzioni valgono per l’intera operazione senza chiedere. Imposta il valore predefinito in **Impostazioni** → **Generali** → **Copia e spostamento**.

### Chiedi {#nachfragen}

Con **Chiedi**, l’operazione si ferma al primo elemento che esiste già nella destinazione. Manta mostra “‘nome’ esiste già in ‘cartella’” con entrambi gli elementi (dimensione, data di modifica e l’etichetta **Più recente** su quello più recente) e tu scegli:

| Pulsante | Cosa succede |
|---|---|
| **Mantieni entrambi** | La copia riceve un nuovo nome (“nome 2”). È la scelta predefinita: **↩** la seleziona. |
| **Sostituisci** | L’elemento esistente viene sovrascritto. |
| **Sostituisci se più recente** | Sovrascrive solo se l’origine è più recente; altrimenti resta quello esistente. |
| **Salta** | L’elemento esistente resta e questo non viene trasferito (uno spostamento lo lascia dov’era). |
| **Annulla** (**esc**) | L’operazione si interrompe. Ciò che è già stato trasferito resta nella destinazione. |

Se esiste già una **cartella** con quel nome, Manta propone **Unisci** (come il Finder): il contenuto va nella cartella esistente e per i file già presenti ti viene chiesto uno per uno. In alternativa ci sono **Mantieni entrambi** (“foto 2”) e **Salta**. Le sottocartelle vengono sempre unite.

Con **Applica a tutti gli altri conflitti di questa operazione**, la risposta vale per il resto dell’operazione e Manta non chiede più. Quando più operazioni chiedono contemporaneamente, fanno a turno. Gli altri trasferimenti continuano mentre uno aspetta; l’elenco dei trasferimenti lo indica con “In attesa della tua decisione”. Se esci da Manta mentre una domanda è aperta, l’operazione chiede di nuovo quando viene ripresa.

## Seguire i trasferimenti {#uebertragungen}

**⌥⌘L** (**Mostra trasferimenti**) o il pulsante **Trasferimenti** nella barra degli strumenti mostra tutte le operazioni in corso: copia, spostamento, eliminazione, estrazione e compressione.

- Ogni operazione mostra avanzamento, velocità e tempo rimanente.
- **Pausa** sospende un’operazione, **Riprendi** la fa proseguire (vedi sotto).
- **Interrompi** annulla un’operazione. I file copiati a metà vengono rimossi.
- **Rimuovi completati** fa pulizia nell’elenco.
- Se qualcosa è andato storto, l’operazione mostra “N problemi”. Anche l’avviso “Completato con problemi” rimanda qui.

Quando nascondi il pannello, la barra di stato mostra in basso a destra una piccola pillola con un anello di avanzamento, così vedi sempre a che punto è un’operazione, anche per i file rilasciati da altre app o copiati nelle cartelle cloud:

- un’operazione: “Copia di 3 elementi… 42%”, “Spostamento di “Vacanze.mov”… 80%”,
- più operazioni: “2 trasferimenti · 37%”,
- in pausa o in attesa: “… · in pausa”, “… · in attesa”, “… · in attesa di connessione”,
- alla fine **Completato** (o **Interrotto**) per qualche secondo,
- se qualcosa è andato storto, la pillola rossa (“Trasferimento non riuscito”, “Terminato con errori”) resta finché non ci fai clic.

Fai clic sulla pillola per aprire i trasferimenti; il suo suggerimento elenca ogni operazione in corso con avanzamento e tempo rimanente.

Quando copi in una cartella cloud (Google Drive, iCloud Drive, OneDrive, Dropbox), il trasferimento è completato non appena i file sono sul tuo Mac: è poi l’app del fornitore a caricarli. Anche questo compare nella barra di stato, ad esempio “Google Drive sta caricando 2 elementi” oppure, in rosso, “Google Drive: 1 errore di caricamento” (vedi [Archiviazione cloud](09-cloud.md#abzeichen)).

![Trasferimenti con un’operazione in corso](images/transfers-panel.png)

## Sospendere e riprendere i trasferimenti {#pausieren}

Ogni operazione di copia, spostamento ed eliminazione può essere sospesa e poi ripresa dallo stesso punto, anche a metà di un file grande.

- **Una sola operazione**: fai clic sull’icona di pausa nella sua riga (**Pausa**). La riga mostra allora ad esempio “In pausa · 1,2 GB di 3,0 GB”. L’icona di riproduzione (**Riprendi**) la fa proseguire.
- **Tutte insieme**: in cima a **Trasferimenti** trovi **Metti tutto in pausa** e **Riprendi tutto** ogni volta che c’è qualcosa da sospendere o riprendere.

Un’operazione ripresa salta i file già completati e continua il file in corso dal punto in cui si era fermata (“Ripresa da 1,2 GB”).

La **compressione** non può essere sospesa.

### Quando cade la connessione {#verbindung-weg}

Se la rete cade o il server sparisce per un momento, Manta non abbandona l’operazione. **Aspetta** e riprova da solo:

- La riga mostra “In attesa di connessione… (tentativo 3, prossimo tra 8 s)”. Le pause si allungano (1, 2, 4 … secondi, al massimo 1 minuto); Manta continua a provare.
- Non appena il tuo Mac è di nuovo online, l’operazione riprende subito, dal punto in cui si era fermata.
- Se non vuoi aspettare, fai clic su **Riprova ora** (icona a freccia).
- Quando il tuo Mac non ha alcuna rete, **Trasferimenti** mostra il banner **Nessuna connessione a Internet**: “I trasferimenti riprendono dal punto in cui si sono interrotti non appena torna la connessione.”

Un’operazione in attesa può essere sospesa o interrotta come qualsiasi altra.

![Trasferimenti: un’operazione in pausa e una in attesa, banner “Nessuna connessione a Internet”](images/transfers-paused-waiting.png)

### File parziali (`.manta-part`) {#teildateien}

Durante la copia, Manta scrive prima il file in un **file parziale nascosto** accanto alla destinazione: `film.mov` diventa `.film.mov.manta-part`. Solo quando il file è completo riceve il suo vero nome. Così nella destinazione non c’è mai un file a metà con il nome vero.

- Vedi i file parziali solo quando i file nascosti sono visibili (**⇧⌘.**).
- Non eliminare un file parziale mentre l’operazione è in pausa o in attesa, altrimenti quel file riparte da zero.
- **Interrompi** e **Scarta** (vedi sotto) li rimuovono per te.
- I caricamenti su **S3** e **Google Drive** non hanno bisogno di file parziali: il server tiene invisibile il file incompleto finché non è terminato.

### Dopo l’uscita o un arresto anomalo {#neustart}

Manta ricorda i trasferimenti non completati anche se esci dall’app, se si chiude in modo anomalo o se il Mac si riavvia. All’avvio successivo chiede:

**Riprendere i trasferimenti interrotti?**, ad esempio “2 trasferimenti non sono stati completati l’ultima volta.” Sotto elenca le operazioni con il loro avanzamento e, se necessario, quale server manca (“richiede files.example.com (SFTP)”).

- **Riprendi** (o ↩): le operazioni proseguono dal punto in cui si erano fermate.
- **Scarta**: le operazioni vengono abbandonate e i loro file parziali rimossi, per quanto Manta riesca a raggiungerli. Su un server non connesso un file parziale rimane; puoi eliminarlo tu più tardi.
- **esc** (chiudere la finestra): decidi più tardi. Manta chiede di nuovo all’avvio successivo.

![La richiesta “Riprendere i trasferimenti interrotti?”](images/resume-prompt.png)

#### “Richiede una connessione” {#benoetigt-verbindung}

Quando un’operazione ripresa ha bisogno di un server a cui non sei ancora connesso, la sua riga mostra “Richiede una connessione a files.example.com (SFTP)” e il pulsante **Riconnetti**.

- I **server nei Preferiti** con una password memorizzata (o con ssh-agent) si riconnettono da soli in background; non devi fare nulla.
- Altrimenti fai clic su **Riconnetti**. Manta apre la finestra di connessione con i dati del server; inserisci la password. Non appena la connessione è attiva, l’operazione prosegue.

Per questo Manta **non** memorizza **mai** password o passphrase: l’elenco dei trasferimenti interrotti (`transfers.json`) contiene solo server, nomi utente e percorsi.

#### Note “copiato di nuovo dall’inizio” {#von-vorn}

A volte Manta non può proseguire un file in modo sicuro e lo copia di nuovo. L’operazione mostra allora “1 nota”; fai clic per espandere le note:

| Nota | Cosa è successo |
|---|---|
| “l’origine è cambiata dopo l’interruzione; copiato di nuovo dall’inizio” | Il file originale ora ha una dimensione o una data di modifica diversa. Proseguire mescolerebbe vecchio e nuovo. |
| “questo server non può riprendere i file parziali; copiato di nuovo dall’inizio” | La destinazione non permette di aggiungere dati a un file incompleto. |
| “il file parziale era più grande dell’origine; copiato di nuovo dall’inizio” | Il file parziale non corrisponde all’origine. |
| “la sessione di caricamento è scaduta; caricato di nuovo dall’inizio” | Nel frattempo S3 o Google Drive hanno scartato il caricamento incompleto. |

### Limiti {#pausieren-grenzen}

- La **compressione** non può essere sospesa e non sopravvive a un riavvio.
- Le operazioni **dagli archivi** (estrazione) possono essere sospese, ma non vengono riprese dopo l’uscita.
- Senza la password giusta, un’operazione su un server aspetta finché non ti connetti.

## Trascinare e rilasciare {#drag-drop}

### Tra pannelli

1. Seleziona degli elementi e trascinali in un altro pannello.
2. Rilasciali. Manta chiede: nella finestra scegli **Copia** o **Sposta**, la destinazione e cosa succede in caso di conflitti di nome. **Copia** è preselezionato, anche all’interno dello stesso volume del tuo Mac, dove il Finder sposterebbe. Così nulla sparisce per errore dalla sua cartella. **↩** conferma la preselezione.
3. Tieni premuto **⌘** mentre rilasci per preselezionare **Sposta**. **⌥** preseleziona esplicitamente **Copia** (e prevale su ⌘).

Un contrassegno accanto al puntatore mostra cosa è preselezionato (“Copia qui 3 elementi · tieni premuto ⌘ per spostare”, con ⌘: “Sposta qui 3 elementi · tieni premuto ⌥ per copiare”). Trascinare fuori da un archivio significa estrarre. Tra pannelli gli elementi finiscono nella cartella mostrata dal pannello.

Se non vuoi che ti venga chiesto ogni volta, scegli “Copia sempre” o “Sposta sempre” in **Quando rilasci dei file** nelle [Impostazioni](16-einstellungen.md#kopieren-bewegen). L’operazione parte allora senza finestra; per quel singolo rilascio **⌘** sposta comunque e **⌥** copia comunque. Il rilascio sul **Cestino** e sui **Preferiti** non è interessato da questa impostazione.

### Da Manta ad altre app {#drag-raus}

Puoi trascinare file da Manta in altre app: sulla Scrivania, in una finestra del Finder, in Mail o nel caricamento di un browser.

- I **file locali** vengono copiati, anche quando la destinazione è sullo stesso volume. Tieni premuto **⌘** per spostarli.
- I **file su server, in container o in archivi** vengono scaricati nella destinazione quando rilasci. Per i file grandi può volerci un po’.

Trascina gli elementi oltre il bordo della finestra di Manta; da lì subentra macOS. Un messaggio ti avvisa quando inizia un download da un server e quando è terminato o non è riuscito. Se l’altra app ha spostato i file, Manta aggiorna la cartella da cui provenivano.

### Da altre app in Manta {#drag-rein}

Rilascia file dal Finder, dalla Scrivania o da un’altra app su un pannello:

- Su un punto vuoto del pannello: finiscono nella cartella mostrata dal pannello.
- Su una cartella (in ogni vista, nella vista Colonne anche su una colonna): finiscono in quella cartella, che viene evidenziata mentre ci passi sopra.
- Valgono le stesse regole che tra pannelli: Manta chiede con **Copia** preselezionato (con **⌘**: **Sposta**), a meno che tu non abbia scelto “Copia sempre” o “Sposta sempre”.

L’operazione compare in Trasferimenti, dove puoi sospenderla. Gli allegati di Mail e le foto di Foto non possono ancora essere rilasciati su Manta.

### Nei Preferiti

Trascina una cartella sulla sezione **Preferiti** della barra laterale per tenerla lì. Vedi [Preferiti e barra laterale](07-favoriten-seitenleiste.md#favoriten-ziehen).

### Sul Cestino

Trascina gli elementi su **Cestino** nella barra laterale: gli elementi locali vanno nel Cestino senza domande, gli elementi sui server vengono eliminati definitivamente dopo una conferma. Vedi [Cestino](07-favoriten-seitenleiste.md#papierkorb).

## Rinominare {#umbenennen}

1. Seleziona **un** elemento.
2. Premi **F2** (**Rinomina…**) o scegli **Rinomina…** nel menu contestuale.
3. Digita il nuovo nome e premi ↩.

**Rinominare con un clic**, come nel Finder: fai di nuovo clic su un elemento già selezionato e attendi un attimo. Nella vista Elenco il nome diventa modificabile direttamente nella riga (è selezionato il nome senza estensione); ↩ o un clic altrove rinomina, Esc annulla. Nelle altre viste si apre la finestra **Rinomina…**. Un doppio clic apre comunque l’elemento.

Manta controlla subito il nome: non deve essere vuoto, contenere “/” o esistere già nella cartella.

**Rinominare molti elementi in una volta**: selezionane diversi e premi **F2**, oppure premi **⇧⌘R** (**Rinomina in blocco…**). Il plug-in offre regole, un’anteprima in tempo reale e la possibilità di annullare. Vedi [Rinomina in blocco](14-plugins.md#umbenennen).

## Nuova cartella {#neuer-ordner}

1. Premi **⇧⌘N** (**Nuova cartella…**).
2. Digita un nome (viene proposto “cartella senza titolo”) e fai clic su **Crea**.

## Nuovo file {#neue-datei}

1. Premi **⌥⌘N** (**Nuovo file…**) oppure scegli **Nuovo file…** nel menu **Archivio** o nel menu contestuale di un pannello.
2. Digita un nome (viene proposto “file senza titolo.txt”) e fai clic su **Crea**. Manta crea un file vuoto e lo seleziona.

**Dentro una cartella**: se fai clic con il tasto destro su una cartella, le voci diventano **Nuova cartella in “…”** e **Nuovo file in “…”**; la cartella o il file vengono creati in quella cartella e il pannello resta dov’è. **⇧⌘N** e **⌥⌘N** continuano a creare nella cartella mostrata.

## Eliminare {#loeschen}

Manta elimina come il Finder:

| Dove | ⌘⌫ (**Sposta nel Cestino**) | ⌥⌘⌫ (**Elimina immediatamente…**) |
|---|---|---|
| Il tuo Mac (anche unità esterne e cartelle cloud) | Sposta gli elementi nel **Cestino**, senza chiedere. **⌘Z** (**Annulla**) o **Ripristina** del Finder li recuperano. | Elimina definitivamente dopo una conferma. |
| Server (SFTP, FTP, FTPS), S3, container | Non c’è un Cestino: elimina definitivamente dopo una conferma. | Elimina definitivamente dopo una conferma. |
| Archivi | Sola lettura: non si può eliminare nulla. | Sola lettura. |

Per spostare elementi nel Cestino:

1. Selezionali.
2. Premi **⌘⌫** o il tasto di cancellazione in avanti **⌦**, oppure scegli **Sposta nel Cestino** nel menu contestuale o nella barra degli strumenti, oppure trascinali su **Cestino** nella barra laterale.

Un messaggio lo conferma (“3 elementi spostati nel Cestino”). Per recuperare gli elementi, premi **⌘Z** (**Annulla**): Manta li rimette dov’erano, vedi [Annulla](#widerrufen). Per elementi più vecchi o eliminati da un’altra app, apri il Cestino nel Finder, selezionali e scegli **Ripristina**.

Per eliminare definitivamente:

1. Seleziona gli elementi.
2. Premi **⌥⌘⌫**, oppure scegli **Elimina immediatamente…** nella palette dei comandi.
3. Manta chiede: “Vuoi eliminare “nome”?” o “Vuoi eliminare 3 elementi?”, con l’avviso che gli elementi non vanno nel Cestino e che l’operazione non si può annullare.
4. Fai clic su **Elimina**.

Su un **server**, in un **bucket S3** o in un **container** non c’è un Cestino. **⌘⌫**, il menu contestuale (lì **Elimina immediatamente…**) e il pulsante della barra degli strumenti chiedono sempre prima, e la finestra indica in rosso che questa posizione non ha un Cestino e che lì l’eliminazione è definitiva. Le cartelle vengono eliminate con tutto il loro contenuto.

Per aprire il Cestino stesso, fai clic su **Cestino** nella barra laterale. Lì **⌘⌫** (in quel caso: **Elimina immediatamente…**) elimina per sempre dopo una conferma, e **⇧⌘⌫** (**Svuota il Cestino…**) lo svuota. Vedi [Cestino](07-favoriten-seitenleiste.md#papierkorb).

> [!CAUTION]
> Nelle **cartelle cloud** (iCloud Drive, Google Drive, OneDrive, Dropbox) un elemento che sposti nel Cestino sparisce anche dal cloud non appena il tuo Mac si sincronizza. Di solito **Ripristina** lo recupera; alcuni fornitori hanno anche un cestino sul loro sito web. Vedi [Archiviazione cloud](09-cloud.md).

## Annulla {#widerrufen}

**⌘Z** (**Annulla**) annulla la tua ultima operazione sui file, come nel Finder, e **⇧⌘Z** (**Ripeti**) la esegue di nuovo. Entrambi i comandi sono nel menu **Modifica** e nella palette dei comandi. Manta ricorda fino a 100 passaggi, finché non lo chiudi.

| Cosa hai fatto | Cosa fa **⌘Z** |
|---|---|
| **Sposta nel Cestino** (**⌘⌫**, o trascinare su **Cestino** nella barra laterale) | Rimette gli elementi dov’erano. |
| Rinominare (**Rinomina…**, rinominare con un clic, finestra Informazioni) | Ripristina il nome precedente, anche sui server. |
| **Nuova cartella…** o **Nuovo file…** sul tuo Mac | Sposta il nuovo elemento nel Cestino, come il Finder, insieme a tutto ciò che ci hai messo dentro. **⇧⌘Z** lo recupera. |
| **Nuova cartella…** o **Nuovo file…** su un server, in S3 o in un container | Non c’è un Cestino: Manta elimina il nuovo elemento, ma solo finché è ancora vuoto. Altrimenti segnala che non è riuscito a eliminarlo e lo lascia dov’è. |

Non si possono annullare la copia e lo spostamento (**F5**, **F6**, trascinamento) né le operazioni definitive: **Elimina immediatamente…**, l’eliminazione sui server e **Svuota il Cestino…**. **Rinomina in blocco…** ha un proprio comando **Annulla**, vedi [Rinomina in blocco](14-plugins.md#umbenennen).

Se nel frattempo la posizione originale è di nuovo occupata (c’è un elemento con lo stesso nome), Manta non sostituisce nulla. Segnala “Impossibile ripristinare ‘nome’” e conserva il passaggio: appena la posizione è libera, **⌘Z** funziona di nuovo.

- In un **campo di testo** (filtro, percorso, nome che stai rinominando) **⌘Z** e **⇧⌘Z** annullano ciò che hai digitato, non le operazioni sui file. Quando il **terminale** è attivo, **⌘Z** appartiene al terminale.
- Senza [Accesso completo al disco](07-favoriten-seitenleiste.md#papierkorb-zugriff), Manta ripristina gli elementi dal Cestino tramite il Finder, come per **Svuota il Cestino…**: macOS chiede una volta se Manta può controllare il Finder.

## Tag {#tags}

Manta legge e assegna gli stessi tag del Finder (sono memorizzati insieme al file, quindi il Finder, Spotlight e le altre app li vedono). Gli elementi con tag mostrano dei pallini colorati accanto al nome in tutte le viste: nell’elenco e nelle colonne dopo il nome, in Icone e Galleria prima del nome. I tag funzionano per gli elementi su questo Mac, compresi iCloud Drive e le altre cartelle cloud; server e archivi non ne hanno.

**Aggiungere o rimuovere un tag:**

1. Seleziona gli elementi e apri il menu contestuale.
2. Scegli **Tag**. In alto ci sono i sette colori: un clic aggiunge il tag a tutti gli elementi selezionati, oppure lo rimuove se ce l’hanno già tutti (un segno di spunta indica che ce l’hanno tutti, un trattino che ce l’hanno solo alcuni). Sotto sono elencati gli altri tuoi tag.
3. Per creare un nuovo tag, digita il nome in **Nuovo tag…** e premi **↩**.

I tag standard hanno i nomi che il Finder usa sul tuo Mac (ad esempio “Rosso” su un Mac in italiano, “Red” su uno in inglese).

**Trovare gli elementi con tag:** la sezione **Tag** della barra laterale elenca i tuoi tag come li mostra il Finder (se Manta non riesce a leggere l’elenco del Finder, quelli che ha visto sui file). Un clic mostra nel pannello attivo tutto ciò che ha quel tag, trovato da Spotlight. Nel campo del filtro, `tag:Rosso` mostra gli elementi con tag della cartella attuale (vedi [Filtrare i file](04-navigation.md#filtern)).

## Ottieni informazioni {#informationen}

**⌘I** (**Ottieni informazioni**, anche nel menu contestuale e nel menu **Archivio**) apre una finestra Informazioni per la selezione, come nel Finder: una finestra per ogni elemento (richiederla di nuovo la porta in primo piano), una finestra riepilogativa per più elementi. Possono essere aperte fino a sei finestre Informazioni; per aprirne una nuova si chiude la più vecchia. Sono finestre normali: puoi continuare a lavorare nella finestra principale.

Una finestra Informazioni mostra:

- il **nome**: modificalo e premi **↩** per rinominare l’elemento,
- **Tipo**, **Dimensioni** (cartelle ed elementi multipli vengono calcolati in background), **Posizione**, **Creato**, **Modificato** e **Ultima apertura** (la data di Spotlight),
- **Tag**: rimuovine uno con la sua ×, aggiungine uno con **Aggiungi tag**,
- **Bloccato** (l’elemento non può essere modificato, rinominato o spostato nel Cestino) e **Nascosto**,
- **Commenti**: il commento di Spotlight impostato nel Finder (Manta lo mostra ma non può modificarlo: macOS non offre alcun modo per farlo, a parte gli script per il Finder),
- **Apri con**: l’app predefinita, e **Apri con** per aprire il file una volta con un’altra app. macOS non offre ad altre app alcun modo pubblico per cambiare l’app di un singolo file, quindi non c’è “Modifica tutto…”; per questo usa il Finder,
- **Condivisione e permessi**: lettura, scrittura ed esecuzione per proprietario, gruppo e tutti, oltre al valore ottale (come `644`). Su questo Mac e sui server SFTP puoi modificarli direttamente lì (chmod); altrove vengono mostrati in sola lettura.

Gli elementi sui server, nei container e negli archivi mostrano ciò che indica il server (dimensioni, data, permessi).

## Posizioni di sola lettura {#schreibgeschuetzt}

Negli archivi, nei container arrestati e nei container senza shell non puoi modificare nulla. Lo indica un indicatore **Sola lettura** (o **Arrestato**) nella barra del percorso. Copiare fuori con **F5** o trascinando funziona sempre.

Come il Finder, **Applicazioni** mostra anche le app di Apple (Calcolatrice, Mail…) e **Utility** quelle di macOS. Si trovano sul volume di sistema protetto: puoi aprirle e copiarle, ma non rinominarle, spostarle né eliminarle.

## Vedi anche

- [Archivi](10-archive.md): comprimere ed estrarre.
- [Terminale](11-terminal.md#dateiaktionen): vedere ogni azione sui file come comando shell.
