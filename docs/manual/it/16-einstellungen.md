# Impostazioni {#einstellungen}

Tutte le impostazioni di Manta, sezione per sezione. Tutte le modifiche hanno effetto subito; non devi mai riavviare.

## Aprire le Impostazioni {#oeffnen}

- **⌘,**
- **Manta** → **Impostazioni…** nella barra dei menu
- l’ingranaggio nella barra degli strumenti
- **Impostazioni…** nella palette dei comandi

Le Impostazioni sono una finestra a sé con le sezioni **Generali** e **Plug-in**. Chiudila con il pulsante rosso della finestra o con **⌘W**. **⌘,** la riporta in primo piano.

Finché le Impostazioni sono aperte, la finestra principale resta in attesa: è leggermente oscurata e clic e scorciatoie al suo interno non hanno effetto; un clic riporta le Impostazioni in primo piano con il suono di avviso. Chiudi le Impostazioni per continuare. La finestra dell’aiuto invece può restare aperta mentre lavori.

![Impostazioni → Generali](images/settings-general.png)

## Generali {#allgemein}

### Aspetto {#erscheinungsbild}

| Impostazione | Opzioni |
|---|---|
| **Aspetto** | **Automatico** (segue la modalità chiara e scura del sistema), **Chiaro**, **Scuro**. Anche nella palette dei comandi: “Aspetto: Chiaro/Scuro/Automatico (Sistema)”. |
| **Colore di accento** | **Blu**, **Viola**, **Rosa**, **Rosso**, **Arancione**, **Giallo**, **Verde**, **Grafite**: per selezione, pulsanti e focus. Ogni colore resta leggibile sia in chiaro sia in scuro. |

### Lingua {#sprache}

**Lingua**: **Sistema** (predefinita) oppure una delle lingue dell’app, ognuna con il proprio nome (**Deutsch**, **English**, **简体中文**, **Español**, **Français**, **Italiano**, **Русский**). Il cambio ha effetto subito. “Sistema” segue le lingue preferite in macOS (oppure l’inglese se Manta non ne parla nessuna). Date (“Oggi 14:02”), dimensioni (“1,2 MB”) e numeri seguono la lingua. I plug-in integrati cambiano lingua insieme all’app; nomi dei file, output della shell e le tue azioni rapide restano invariati.

### Gestore file predefinito {#standard-dateimanager}

**Usa Manta come gestore file predefinito**: quando è attivo, “Mostra nel Finder” e le cartelle da altre app si aprono in Manta. Sotto vedi quale app ha il ruolo ora (“Attuale: …”) e quale app lo riceve indietro quando lo disattivi. Cosa permette macOS è spiegato in [Primi passi](01-erste-schritte.md#standard-dateimanager).

Lo stesso interruttore si trova nel menu **Manta** (**Manta come gestore file predefinito**, con un segno di spunta quando è attivo) e nella palette dei comandi (**Usa Manta come gestore file predefinito…**, oppure **Restituisci il gestore file predefinito a …** quando il ruolo è di Manta). Attivandolo da lì compare la stessa domanda del primo avvio, con **Annulla**; disattivandolo il ruolo viene restituito subito.

È disponibile solo quando Manta viene eseguito dalla cartella **Applicazioni** (“Prima sposta Manta nella cartella Applicazioni.”).

### File {#dateien}

| Impostazione | Significato |
|---|---|
| **Mostra file nascosti** | Per le nuove schede e all’avvio. **⇧⌘.** li mostra o li nasconde nella scheda attuale. |
| **Apri gli archivi come cartelle** | A capo o doppio clic sfoglia gli archivi ZIP e TAR nel pannello (in sola lettura). **⌥↩** li apre con l’app predefinita. Vedi [Archivi](10-archive.md). |
| **Le nuove schede si aprono in** | Dove si aprono **⌘T**, **Nuova scheda** nel menu del Dock e un avvio senza sessione salvata: **La stessa cartella** (predefinito; all’avvio la tua cartella Inizio), **Inizio**, **Scrivania**, **Documenti**, **Download** o **Altra…** (una cartella a tua scelta). Come «Le nuove finestre del Finder mostrano» nel Finder. |
| **Ricorda la vista per ogni cartella** | Ogni cartella si apre nella vista in cui è stata mostrata l’ultima volta (Icone, Elenco, Colonne, Galleria). Disattivata (predefinito): la vista appartiene al pannello. Vedi [Viste](03-ansichten.md#pro-ordner). |
| **Tasti laterali del mouse** | I tasti del pollice vanno indietro e avanti nel riquadro sotto il puntatore, come **⌘[** e **⌘]**, anche nell’Aiuto. **Indietro / Avanti** (predefinito), **Avanti / Indietro (invertiti)** o **Disattivati**. |
| **Carica miniature dei file sul server** | Scarica le immagini piccole (sotto i 5 MB) in background. Disattivata (predefinito): i file sui server mostrano l’icona del loro tipo. Vedi [Miniature](03-ansichten.md#server-vorschaubilder). |
| **Icone dei file di macOS** | I file mostrano l’icona che macOS ha per il loro tipo e le app la propria, come nel Finder, anche sui server. Disattivata: i simboli di Manta. Predefinito: attivata. |

### Anteprima {#vorschau}

**Barra spaziatrice**: **Visualizzazione rapida (macOS)** o **Anteprima integrata**. I file sui server e negli archivi usano sempre l’anteprima integrata. Vedi [Visualizzazione rapida e anteprima](06-schnellansicht.md).

### Copia e spostamento {#kopieren-bewegen}

**Quando rilasci dei file**: **Chiedi (Copia preselezionato)** (predefinito), **Copia sempre** o **Sposta sempre**. Vale per il trascinamento tra pannelli e da altre app. “Chiedi” mostra la finestra con Copia / Sposta; le due impostazioni “sempre” partono subito. **⌘** durante il rilascio sposta, **⌥** copia, qualunque sia l’impostazione. Il rilascio sul Cestino e sui Preferiti non è interessato. Vedi [Trascinare e rilasciare](05-dateien.md#drag-drop).

**Se un elemento esiste già**: **Chiedi** (predefinito), **Salta**, **Sostituisci se più recente**, **Sostituisci** o **Mantieni entrambi**. Questa scelta è preselezionata quando copi o sposti (anche quando rilasci); puoi comunque cambiarla ogni volta. Quando un rilascio parte senza finestra (“Copia sempre” / “Sposta sempre”), vale questa impostazione. Con **Chiedi**, un’operazione si ferma a ogni elemento già esistente e ti chiede cosa fare. Le nuove installazioni chiedono; se usavi già Manta, resta l’impostazione salvata (compreso il precedente valore predefinito **Salta**): cambiala qui. Vedi [Conflitti di nome](05-dateien.md#konflikte).

### Cloud {#cloud}

**Google Drive**: indica se Google Drive per desktop è configurato, altrimenti mostra il link per scaricarlo. **Suggerimento nella barra laterale** attiva o disattiva la riga “Collega Google Drive…”. **Esperti: client OAuth Google personale** è solo per utenti avanzati, vedi [Google Drive per esperti](09-cloud.md#experte).

### Aggiornamenti {#updates}

| Impostazione | Significato |
|---|---|
| **Verifica aggiornamenti all’avvio** | Senza disturbare, in background. Nulla viene installato senza chiederti. Predefinito: attivo. |
| **Canale** | **Stabile** o **Beta**. Beta porta prima le nuove funzioni, ma può contenere errori. |
| **Verifica ora** | Verifica subito. Accanto: “Ultima verifica: …”. |
| **Saltata: Manta …** / **Ripristina** | Compare quando hai saltato una versione. Questa versione e quelle precedenti non vengono proposte finché non fai clic su **Ripristina**. |

Se la tua build non ha la chiave di firma degli aggiornamenti, questa sezione indica “Gli aggiornamenti non sono configurati in questa build”. Vedi [Aggiornamenti](17-updates.md).

### Informazioni su Manta {#ueber}

Manta è gratuito e open source. **Sito web e supporto** → **manta-filemanager.com** apre [il sito web](https://manta-filemanager.com/) con novità, aiuto e contatti. **Licenze di terze parti** → **Mostra** apre le note di licenza dei componenti open source su cui si basa Manta. **Codice sorgente** → **GitHub** apre [il repository](https://github.com/web-workers-com/manta); i contributi sono benvenuti. **Licenza MIT** → **Leggi** mostra la licenza: puoi usare, modificare e condividere Manta liberamente. Il sito web, il codice sorgente e le licenze di terze parti sono anche nel menu **Aiuto**. **Segnalazioni** → **Segnala un problema** apre il modulo di GitHub per le segnalazioni di bug con le versioni di Manta e di macOS compilate, vedi [Segnalare un bug](19-fehlerbehebung.md#fehler-melden).

## Plug-in {#plugins}

In alto: **Installa plug-in…**, **Apri cartella plug-in** e **Ricarica**. Sotto ci sono tre sezioni:

- **Integrati**: i plug-in integrati (Git, Terminale, Dimensioni cartelle, Rinomina in blocco, Anteprima Markdown, Azioni rapide, Info progetto, Strumenti immagine, Duplicati, Carico di sistema).
- **Installati**: i plug-in esterni che hai installato.
- **Disponibili**: plug-in forniti con Manta, pronti per l’installazione.

Ogni scheda mostra nome, descrizione, stato (**Attivo**, **Avvio…**, **Disattivato**, **Errore**) e i pulsanti **Attiva**/**Disattiva**, **Impostazioni** (dove disponibili) e **Mostra log**. Altre informazioni in [Plug-in personali](15-eigene-plugins.md).

### Impostazioni dei plug-in {#plugin-einstellungen}

| Plug-in | Impostazioni |
|---|---|
| **Terminale** | Shell, Dimensione font, Cronologia, Usa Opzione come tasto Meta, Il pannello segue “cd” nel terminale, Mostra azioni sui file nel terminale; vedi [Terminale](11-terminal.md#einstellungen). |
| **Dimensioni cartelle** | **Calcola automaticamente le dimensioni delle cartelle** o **Solo su richiesta**; vedi [Dimensioni cartelle](14-plugins.md#ordnergroessen). |
| **Anteprima Markdown** | **Carica immagini dal web (http/https)**: disattivato per impostazione predefinita. |
| **Duplicati** | **Salta i pacchetti (.app, .photoslibrary, …)**, **Includi file nascosti**, **Modifica settings.json…** per le esclusioni e un comando personalizzato per il cestino. |
| **Carico di sistema** | **Mostra il carico della GPU**; vedi [Carico di sistema](14-plugins.md#systemauslastung). |

## Accessibilità {#bedienungshilfen}

Manta segue **Impostazioni di Sistema › Accessibilità**, senza interruttori propri:

- **VoiceOver**: ogni pulsante, campo, elenco e dialogo ha un nome nella tua lingua. Le righe dei file vengono lette per intero, ad es. «README.md, Testo, 214 byte, modificato Oggi 09:41, selezionato», compreso lo stato cloud e Git. Quando ti sposti con le frecce, VoiceOver legge la riga corrente. Messaggi come «3 elementi copiati» ed errori vengono annunciati.
- **Riduci movimento** (Schermo): niente animazioni – il lampeggio di un preferito, la pulsazione del badge di caricamento e le dissolvenze sono disattivati.
- **Aumenta contrasto** (Schermo): bordi e separatori più marcati, testo secondario più scuro (più chiaro in modalità scura), anello di focus più evidente.
- **Riduci trasparenza** (Schermo): niente oscuramenti traslucidi dietro i dialoghi.

Manta applica le modifiche subito, senza riavvio. Anche senza queste impostazioni il testo ha un rapporto di contrasto di almeno 4,5:1. Per l’uso solo da tastiera vedi [Abbreviazioni da tastiera](18-tastenkuerzel.md#tastatur).

## Dove Manta conserva le impostazioni {#dateien-speicherort}

Tutto si trova in `~/Library/Application Support/Manta/`:

| File | Contenuto |
|---|---|
| `settings.json` | Impostazioni generali |
| `favorites.json` | Preferiti e connessioni salvate (senza password: quelle sono nel Portachiavi) |
| `session.json` | Pannelli, schede e larghezze all’ultima uscita |
| `plugins.json` | Quali plug-in sono attivi, stato dei pannelli |
| `plugin-data/` | Dati dei plug-in, come `actions.json` |
| `update-state.json` | Canale di aggiornamento, ultima verifica, versione saltata |
| `default-viewer.json` | Gestore file predefinito e app precedente |
| `folder-views.json` | Vista memorizzata per ogni cartella (con **Ricorda la vista per ogni cartella**) |
| `transfers.json` | Trasferimenti interrotti da riprendere (senza password), vedi [Trasferimenti](05-dateien.md#neustart) |

Le miniature sono conservate a parte in `~/Library/Caches/Manta/thumbs` e possono essere eliminate in qualsiasi momento.

> [!TIP]
> Per trasferire Manta su un altro Mac, copia questa cartella mentre Manta non è in esecuzione.
