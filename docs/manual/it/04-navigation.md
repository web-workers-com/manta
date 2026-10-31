# Navigazione {#navigation}

Questo capitolo spiega come spostarti tra le cartelle: con la barra del percorso, con **Vai al percorso…**, con la cronologia e con la tastiera. Alla fine parla di filtro, selezione digitando e file nascosti.

## Aprire cartelle e file {#oeffnen}

- **Doppio clic** o **↩** apre una cartella nel pannello, oppure un file con la sua app predefinita.
- **⌘↓** fa lo stesso di ↩.
- **⌥↩** (**Apri con l’app predefinita**) apre sempre con l’app predefinita, anche gli archivi, che altrimenti Manta apre come cartelle (vedi [Archivi](10-archive.md)).
- **⌘↑** o **⌫** (Elimina/Backspace) va alla **cartella superiore**.

Per aprire un file su un server, Manta lo scarica in una cartella temporanea e apre quella copia (“Download di “nome” per aprirlo…”).

> [!WARNING]
> Le modifiche a un file del server aperto in questo modo finiscono solo nella copia temporanea, non sul server. Per modificare un file, copialo in una cartella locale, modificalo lì e ricopialo sul server.

## La barra del percorso {#pfadleiste}

La barra del percorso in cima a ogni pannello mostra dove ti trovi, ad esempio `Inizio › Progetti › manta`.

![Barra del percorso con indicatore della posizione, percorso e indicatori dei plug-in](images/path-bar.png)

- **Fai clic sul nome di una cartella** per andarci.
- **Fai clic su “…”** all’inizio per vedere le cartelle superiori (“Mostra le cartelle superiori”) quando il percorso è abbreviato.
- **Fai doppio clic su un punto vuoto** della barra del percorso (o premi **⌘L**, **Modifica percorso**) per trasformarla in un campo di testo. Digita un percorso e premi ↩. **esc** annulla.
- **Indicatore della posizione**: l’indicatore all’estrema sinistra mostra dove ti trovi: **Locale** sul tuo Mac, oppure ad esempio **Docker · shop-web**, **Kubernetes · pod**, **SFTP · deploy@web-01**, **S3 · bucket**, **FTP · host**, **Archivio · backup.zip**, e nelle cartelle cloud il fornitore (**Google Drive**, **iCloud Drive**, **OneDrive**, **Dropbox**; l’account è nel suggerimento). I nomi lunghi vengono abbreviati con “…”, i pannelli stretti mostrano solo l’icona; il testo completo è nel suggerimento. Un clic porta alla cartella principale di quella posizione.
- **Vista**: a destra, prima del campo del filtro, passi tra Icone, Elenco, Colonne e Galleria; accanto c’è **Opzioni vista**. Vedi [Viste](03-ansichten.md#wechseln).
- Gli **indicatori dei plug-in** a destra mostrano ad esempio il branch Git o il progetto riconosciuto. Quando il pannello si restringe si accorciano e alla fine diventano un’icona; il testo completo è nel suggerimento.

### Torna a Locale {#lokal}

Quando non sei in una normale cartella del tuo Mac (un server, un container, un bucket, un archivio o una cartella cloud), accanto all’indicatore della posizione compare una freccia **Torna a Locale**. Ti porta all’ultima cartella del tuo Mac nella cronologia della scheda, saltando cartelle cloud e archivi, oppure alla tua cartella Inizio. La connessione resta aperta.

## Vai al percorso {#gehe-zum-pfad}

**Vai al percorso…** (**⇧⌘G**) apre una cartella di cui conosci il percorso o che hai negli appunti.

1. Premi **⇧⌘G**.
2. Digita o incolla un percorso. Manta capisce `~/Downloads`, percorsi relativi, percorsi tra virgolette e URL `file://`.
3. **⇥** completa i nomi delle cartelle. Sotto il campo vedi **Cartelle corrispondenti** e **Recenti**.
4. Premi **↩** per aprire la cartella in una **nuova scheda**, oppure **⌘↩** per aprirla **nella scheda attuale**.

Manta controlla il percorso in background e segnala ad esempio “Cartella non trovata” o “Permesso negato”. Se il percorso è un **file**, Manta apre la sua cartella e seleziona il file.

Se il pannello attivo mostra un server, il percorso si riferisce a quel server (“Si apre in una nuova scheda su …”).

**⇧⌘G** funziona ovunque, anche mentre stai scrivendo in un campo di testo.

![Vai al percorso con suggerimenti](images/go-to-path.png)

## Indietro e avanti {#verlauf}

Ogni scheda ha la propria cronologia.

- **⌘[** o **⌘←**: **Indietro**
- **⌘]** o **⌘→**: **Avanti**
- I **tasti del pollice del mouse** (tasti 4 e 5): **Indietro** e **Avanti** nel riquadro sotto il puntatore. Si possono invertire o disattivare in [Impostazioni → File](16-einstellungen.md#dateien).

La cronologia entra negli archivi ed esce di nuovo.

## Selezionare con la tastiera {#auswaehlen}

| Tasto | Effetto |
|---|---|
| ↑ / ↓ | Elemento precedente / successivo |
| ⇧↑ / ⇧↓ | Estendi la selezione |
| Pagina su / Pagina giù | 20 righe più in là |
| Inizio / Fine | Primo / ultimo elemento |
| ⌘A | Seleziona tutto |
| esc | Annulla la selezione |

Con il mouse: un clic seleziona un elemento, **clic con ⌘** ne aggiunge o toglie uno, **clic con ⇧** seleziona un intervallo. Un clic su un punto vuoto annulla la selezione.

## Selezionare digitando {#tippen}

Come nel Finder: digita le prime lettere di un nome e Manta salta al primo elemento corrispondente. Maiuscole e minuscole non contano. Se nessun nome inizia con quelle lettere, Manta salta al nome successivo nell’ordinamento.

Dopo circa un secondo senza digitare, la ricerca ricomincia. Mentre digiti, lo spazio fa parte del nome (“mia ca”) e non apre Visualizzazione rapida.

## Filtrare i file {#filtern}

Il filtro mostra solo gli elementi il cui nome contiene un certo testo.

1. Premi **⌘F** (**Filtra file**) o fai clic sul campo **Filtra** nella barra del percorso.
2. Digita una parte del nome, ad esempio `.pdf` o `fattura`.

Il filtro vale solo per la cartella attuale e per questa scheda. La barra di stato mostra “corrispondenti a “…””. Se nessun elemento corrisponde vedi “Nessun elemento corrisponde a “…”” con un pulsante **Cancella filtro**. **esc** nel campo cancella il filtro. Nei pannelli più stretti (ad esempio due pannelli in una finestra piccola) il campo è una lente: fai clic su di essa o premi **⌘F** per aprirlo; resta aperto finché contiene del testo e si richiude quando è vuoto e fai clic altrove o premi **esc**. Così resta spazio per l’indicatore della posizione e per il percorso; mentre il campo è aperto, i pulsanti della vista si raccolgono in un unico pulsante con menu.

Per filtrare per [tag](05-dateien.md#tags), digita `tag:` seguito dal nome del tag, ad esempio `tag:Rosso` o `tag:"Molto importante"`; funziona anche il nome del colore (`tag:rosso` trova anche un tag rosso chiamato “Urgente”). Si può combinare con un nome: `tag:Rosso fattura`.

## Cercare in più cartelle {#suchen}

Il filtro guarda solo la cartella che vedi. Per cercare anche al suo interno, o in tutto il Mac, usa la barra dell’ambito che compare sotto la barra del percorso mentre usi il campo:

- **Questa cartella**: il filtro descritto sopra.
- **Sottocartelle**: la cartella e tutto ciò che contiene. Su questo Mac vengono lette più cartelle contemporaneamente; gli elementi nascosti vengono cercati solo quando i file nascosti sono visibili, e il contenuto dei pacchetti (app, librerie di Foto) viene saltato. Sui server, nei container e negli archivi Manta elenca le cartelle una per una, fino a un massimo di 12 livelli di profondità e 5.000 cartelle.
- **Questo Mac**: Spotlight; nomi *e* contenuto dei file, ovunque Spotlight indicizzi (solo cartelle locali).

1. Premi **⌘F** e digita ciò che cerchi.
2. Fai clic su **Sottocartelle** o **Questo Mac**. La ricerca parte un istante dopo che hai smesso di digitare (o subito con **↩**).

Un nome corrisponde quando contiene il testo (maiuscole e minuscole non contano). Con `*` o `?` il testo diventa un modello per l’intero nome: `*.pdf`, `report-202?.xlsx`. Anche qui funzionano i termini `tag:`.

Mentre la ricerca è in corso, i risultati riempiono il pannello come elenco: la barra del percorso indica **Ricerca: “…”** e dove si cerca, la barra dell’ambito mostra quanti elementi sono stati trovati e una colonna **Percorso** mostra la cartella in cui si trova ciascuno. Puoi aprire, visualizzare con Visualizzazione rapida, copiare, spostare, rinominare, contrassegnare con tag ed eliminare i risultati come al solito. **Mostra nella cartella superiore** nel menu contestuale (o **⌘↑**) apre la cartella dell’elemento con l’elemento selezionato. Il pulsante di interruzione nella barra dell’ambito ferma una ricerca lunga (i risultati trovati fino a quel momento restano); **⌘R** la esegue di nuovo. Vengono mostrati al massimo 10.000 risultati.

**esc** nel campo di ricerca, **⌘[** o la cancellazione del testo chiudono la ricerca e mostrano di nuovo la cartella; lo stesso avviene se apri un’altra cartella.

Se Sottocartelle non trova nulla, il pannello vuoto propone **Cerca su questo Mac**.


## File nascosti {#unsichtbare-dateien}

I file il cui nome inizia con un punto (come `.git`, `.env`) e i file contrassegnati come nascosti di solito non sono visibili. Solo `.DS_Store` non viene mai mostrato, nemmeno con i file nascosti visibili, come nel Finder: macOS lo ricrea in quasi ogni cartella.

- **⇧⌘.** (**Mostra file nascosti**) li mostra o li nasconde nella scheda attuale. Il pulsante della barra degli strumenti con il file tratteggiato fa lo stesso (è evidenziato mentre i file nascosti sono visibili). L’occhio accanto è Visualizzazione rapida.
- **Impostazioni** → **Generali** → **Mostra file nascosti** decide se le nuove schede e Manta all’avvio li mostrano.

Se una cartella sembra vuota ma contiene elementi nascosti, Manta te lo dice: “3 elementi nascosti — premi ⇧⌘. per mostrarli.”

## Aggiornare {#aktualisieren}

Le cartelle locali restano aggiornate da sole, come nel Finder: quando un’altra app crea, elimina, rinomina o modifica un file, la modifica compare nel pannello dopo un istante. Selezione, cursore, posizione di scorrimento e selezione digitando restano invariati. Manta tiene d’occhio le cartelle di tutte le schede aperte (nella vista Colonne anche le colonne a sinistra). Una scheda in secondo piano viene riletta appena ci passi, subito e senza ⌘R. Molte modifiche contemporanee, come la copia di migliaia di file, vengono raggruppate: la cartella viene riletta al massimo circa una volta al secondo. Mentre digiti un nome (Rinomina, Nuova cartella), l’aggiornamento aspetta che tu abbia finito.

Le cartelle sui **server**, nei **container**, in **S3** e negli **archivi** non vengono sorvegliate. Se qualcun altro cambia qualcosa lì, premi **⌘R** (**Aggiorna**) per rileggere la cartella. Naturalmente ⌘R funziona anche in locale.

## Vedi anche

- [Tag e Ottieni informazioni](05-dateien.md#tags)
- [Preferiti e barra laterale](07-favoriten-seitenleiste.md)
- [Scorciatoie da tastiera](18-tastenkuerzel.md#navigation)
