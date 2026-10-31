# Pannelli e schede {#bereiche-tabs}

Manta mostra i tuoi file in **pannelli** affiancati: due, tre o quanti ne entrano nella finestra. Ogni pannello ha le proprie schede, la propria barra del percorso e la propria cronologia.

![Tre pannelli affiancati, con il pannello attivo evidenziato](images/panes-three.png)

## Il pannello attivo {#aktiver-bereich}

Esattamente un pannello è **attivo**: ha un bordo marcato nel colore di accento. Le scorciatoie e i comandi agiscono sul pannello attivo.

- Fai clic in un punto qualsiasi di un pannello per attivarlo.
- **⇥** (Tab) passa al pannello successivo (dopo l’ultimo torna al primo).
- **⌥⌘←** e **⌥⌘→** attivano il pannello a sinistra o a destra.

## Dividere un pannello {#teilen}

1. Attiva il pannello accanto al quale deve comparire quello nuovo.
2. Premi **⌘D** (**Dividi pannello**).

Il nuovo pannello compare a destra di quello attivo e mostra la stessa cartella. Puoi anche fare clic sul pulsante di divisione a destra nella barra del percorso dell’ultimo pannello, oppure scegliere **Dividi pannello** nella palette dei comandi.

Ogni pannello ha bisogno di circa 280 punti di larghezza. Se la finestra è troppo stretta, Manta segnala “Non c’è spazio per un altro pannello”. Allarga la finestra o chiudi un pannello.

## Chiudere un pannello {#schliessen}

- Premi **⇧⌘D** (**Chiudi pannello**), oppure
- fai clic sulla **×** a destra nella barra del percorso del pannello.

L’ultimo pannello non può essere chiuso.

> [!NOTE]
> Con un solo pannello, **Copia** e **Sposta nell’altro pannello**, **Confronta e sincronizza** e il cambio di pannello sono disattivati. Il suggerimento spiega il motivo: “Servono due pannelli – ⌘D divide il pannello”.

## Cambiare la larghezza dei pannelli {#breite}

- Trascina il divisore tra due pannelli verso sinistra o verso destra.
- Fai doppio clic su un divisore per rendere di nuovo tutti i pannelli larghi uguali.

## Il pannello di destinazione: “l’altro pannello” {#zielbereich}

Copia (**F5**) e Sposta (**F6**) agiscono sempre verso il **pannello di destinazione**.

- Con **due pannelli** è semplicemente l’altro.
- Con **tre o più** è il pannello che è stato attivo per ultimo. Mentre nel pannello attivo c’è una selezione, il pannello di destinazione ha un bordo leggero e un contrassegno **→** nella barra del percorso (“Copia e Sposta (F5 / F6) vanno in questo pannello”).

Nella finestra di copia puoi scegliere un altro pannello in **In**. Vedi [Gestire i file](05-dateien.md#kopieren).

![Pannello di destinazione con il contrassegno a freccia](images/panes-target-badge.png)

## Schede {#tabs}

Ogni pannello può avere più schede. Una scheda mostra una cartella: sul tuo Mac, su un server, in un container o in un archivio.

| Azione | Come |
|---|---|
| Nuova scheda | **⌘T** o il pulsante **+** nella barra del percorso, oppure **Nuova scheda** nel menu del Dock (clic destro sull’icona di Manta nel Dock) |
| Chiudere una scheda | **⌘W** o la **×** sulla scheda (compare al passaggio del puntatore) |
| Cambiare scheda | Fai clic sulla scheda |

La nuova scheda mostra la stessa cartella di quella attuale, oppure la cartella scelta in [**Le nuove schede si aprono in**](16-einstellungen.md#dateien). **Vai al percorso…** (**⇧⌘G**) apre una cartella direttamente in una nuova scheda (vedi [Navigazione](04-navigation.md#gehe-zum-pfad)).

> [!NOTE]
> Come nel Finder, **⌘T** apre una nuova scheda. Prima di questa versione era **⇧⌘T**, mentre **⌘T** mostrava i trasferimenti: ora questo è **⌥⌘L**.

Nei pannelli molto stretti il pulsante **+** scompare dalla barra del percorso; **⌘T** funziona comunque.

## Sessioni: tutto torna com’era {#sitzung}

Quando esci da Manta e lo riavvii, ritrovi:

- i pannelli nel loro ordine e con la loro larghezza,
- le schede che mostrano cartelle **locali**,
- quale pannello era attivo e quale è la destinazione.

Le schede con cartelle di server, container o archivi non vengono ripristinate, perché richiederebbero una nuova connessione. Aprile dai tuoi [Preferiti](07-favoriten-seitenleiste.md).

Se all’avvio successivo una cartella salvata non esiste più (ad esempio perché la sua unità è stata espulsa), la scheda viene comunque ripristinata. Mostra un errore con **Riprova**: ricollega l’unità e fai clic sul pulsante, oppure chiudi la scheda o apri un’altra cartella.

## Vedi anche

- [Viste](03-ansichten.md): Icone, Elenco, Colonne, Galleria.
- [Confronta e sincronizza](13-vergleichen-synchronisieren.md): allineare due pannelli.
