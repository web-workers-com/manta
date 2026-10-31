# Viste {#ansichten}

Come nel Finder, ogni pannello può mostrare i suoi elementi come **Icone**, come **Elenco**, in **Colonne** o come **Galleria**. Cambia vista con **⌘1**–**⌘4** o con il selettore nella barra del percorso.

![Le quattro viste a confronto](images/views-overview.png)

## Cambiare vista {#wechseln}

| Vista | Scorciatoia | Comando | Ideale per … |
|---|---|---|---|
| Icone | **⌘1** | **Come icone** | Immagini e cartelle con pochi elementi a colpo d’occhio |
| Elenco | **⌘2** | **Come elenco** | Ordinare per dimensione, data, permessi |
| Colonne | **⌘3** | **Come colonne** | Muoversi in strutture di cartelle profonde vedendo sempre il percorso |
| Galleria | **⌘4** | **Come galleria** | Guardare immagini, video e documenti in un’anteprima grande |

Hai tre possibilità:

- **Tastiera**: **⌘1**–**⌘4** agiscono sul pannello attivo.
- **Barra del percorso**: a destra nella barra del percorso di ogni pannello c’è il selettore **Vista** con quattro icone. Quando il pannello è stretto diventa un unico pulsante con un menu.
- **Palette dei comandi** (**⇧⌘P**): digita ad esempio “colonne” o “galleria”. I comandi si trovano nel gruppo **Vista**.

La vista appartiene al **pannello**: tutte le schede di un pannello la condividono e Manta la ricorda fino all’avvio successivo. Due pannelli possono mostrare viste diverse, ad esempio Colonne a sinistra ed Elenco a destra.

La selezione (clic, clic con **⌘**, clic con **⇧**, **⌘A**), la selezione digitando, Visualizzazione rapida, la rinomina, il menu contestuale, il trascinamento e i contrassegni dei plug-in funzionano allo stesso modo in tutte le viste.

## Opzioni vista {#optionen}

Nelle viste Icone, Colonne e Galleria, accanto al selettore c’è il pulsante **Opzioni vista** (icona con i cursori). Apre un piccolo menu:

- **Ordina per**: **Nome**, **Dimensione**, **Data di modifica** o **Permessi**, più **Crescente** o **Decrescente**.
- **Dimensione icone** (solo vista Icone): un cursore da 48 a 256 punti.

![Selettore di vista e Opzioni vista nella barra del percorso](images/view-options.png)

La vista Elenco non ha questo pulsante: lì ordini con le intestazioni delle colonne. In un pannello stretto le opzioni si trovano nel menu del selettore.

## Vista Elenco {#liste}

L’elenco mostra le colonne **Nome**, **Dimensione**, **Data di modifica** e **Permessi**.

- **Ordinare**: fai clic sull’intestazione di una colonna. Un secondo clic inverte l’ordine. Le cartelle vengono sempre prima; le app e gli altri pacchetti sono ordinati come file, come nel Finder.
- **Larghezza delle colonne**: trascina il bordo sinistro di una colonna.
- **Dimensioni delle cartelle**: con il plug-in [Dimensioni cartelle](14-plugins.md#ordnergroessen) la colonna **Dimensione** mostra le dimensioni reali anche per le cartelle.

![Vista Elenco ordinata per dimensione](images/view-list.png)

## Vista Icone {#symbole}

La vista Icone mostra una griglia di icone e miniature grandi, con il nome sotto.

- **Cambiare la dimensione**: con il cursore in **Opzioni vista**, oppure tieni premuto **⌘** e scorri, oppure pizzica sul trackpad. Ogni pannello ha la propria dimensione delle icone.
- **Selezionare più elementi**: trascina un rettangolo di selezione partendo da un punto vuoto. Con **⌘** o **⇧** la selezione precedente rimane.
- **Tastiera**: ← → ↑ ↓ ti spostano nella griglia, **Pagina su**/**Pagina giù** di una pagina alla volta.

## Vista Colonne {#spalten}

Nella vista Colonne ogni cartella del percorso è una colonna a sé. Quando selezioni una cartella, la colonna alla sua destra ne mostra il contenuto.

- **Tastiera**: ↑ ↓ selezionano nella colonna attuale, **→** apre la cartella selezionata come colonna successiva, **←** torna alla colonna a sinistra.
- **Colonna di anteprima**: quando selezioni un file, l’ultima colonna mostra un’anteprima con la miniatura o l’icona e le **Informazioni**: **Tipo**, **Dimensione**, **Modificato**, **Permessi** e, per gli alias, l’**Originale**. Con più elementi selezionati indica ad esempio “3 elementi selezionati”.
- **Larghezza delle colonne**: trascina il bordo destro di una colonna (dalla maniglia in basso). Fai doppio clic sul bordo per ripristinare la larghezza.
- Le cartelle vuote mostrano “Nessun elemento”. Quando le colonne sono più larghe del pannello, la vista scorre lateralmente e mantiene visibile la colonna attiva.

Se con **⌘↑** vai oltre la prima colonna, oppure salti dalla barra laterale o dalla barra del percorso a una cartella esterna alle colonne mostrate, le colonne ripartono da quella cartella.

![Vista Colonne](images/view-columns.png)

## Vista Galleria {#galerie}

La galleria mostra in grande l’elemento selezionato, con il nome e sotto una riga con tipo, dimensione e data di modifica. In basso scorre una **pellicola** con tutti gli elementi della cartella.

- Fai clic su un elemento nella pellicola o scorri con ← → per vederlo in grande. **Pagina su**/**Pagina giù** saltano di una larghezza della pellicola.
- Se non è selezionato nulla, la galleria mostra il primo elemento.
- Le immagini piccole vengono ingrandite al massimo al doppio della loro dimensione, così non risultano sgranate.

![Vista Galleria con pellicola](images/view-gallery.png)

## Miniature {#vorschaubilder}

Nelle viste Icone, Colonne e Galleria Manta mostra le miniature. Vengono create in background, così la vista resta reattiva; finché una miniatura non è pronta vedi l’icona del tipo di file. Senza miniatura, i file mostrano l’icona che macOS ha per il loro tipo (anche nelle viste Elenco e Colonne) e le app (`.app`) la propria, come nel Finder (si può disattivare in **Impostazioni**: **Icone dei file di macOS**).

- Le **immagini** (PNG, JPEG, GIF, BMP, WebP) vengono lette direttamente da Manta e ruotate secondo l’orientamento della fotocamera (EXIF).
- **PDF, video, HEIC, documenti** e così via sono forniti da macOS (come in Visualizzazione rapida). I file senza anteprima mantengono l’icona del loro tipo.
- Le miniature pronte vengono salvate in `~/Library/Caches/Manta/thumbs`. Quando un file cambia, viene creata una nuova miniatura. Puoi eliminare la cartella in qualsiasi momento.

### Miniature per i file sui server {#server-vorschaubilder}

Per i file **sui server** (SFTP, FTP, S3, container) Manta mostra di norma solo l’icona del tipo di file. Se vuoi le miniature anche lì, attiva **Carica miniature dei file sul server** in **Impostazioni** → **Generali** → **File**. Manta scarica allora le immagini piccole (sotto i 5 MB) in background. I file più grandi mantengono la loro icona.

> [!NOTE]
> Le miniature dai server consumano dati e un po’ di tempo. Con connessioni lente o a consumo, lascia l’opzione disattivata.

## Ricordare la vista per ogni cartella {#pro-ordner}

Di norma la vista appartiene al **pannello**: quando cambi cartella, la vista resta la stessa.

Se attivi **Ricorda la vista per ogni cartella** in **Impostazioni** → **Generali** → **File** (impostazione predefinita: disattivata), ogni cartella si apre nella vista in cui è stata mostrata l’ultima volta: le tue foto sempre come galleria, il tuo progetto come elenco. Funziona anche con le cartelle sui server e negli archivi. Le cartelle per cui non hai scelto una vista mantengono la vista attuale del pannello.

Manta ricorda le ultime 2000 cartelle in `~/Library/Application Support/Manta/folder-views.json`.

## Vedi anche

- [Visualizzazione rapida e anteprima](06-schnellansicht.md)
- [La barra del percorso](04-navigation.md#pfadleiste)
- [Impostazioni](16-einstellungen.md#dateien)
- [Scorciatoie da tastiera](18-tastenkuerzel.md#ansichten)
