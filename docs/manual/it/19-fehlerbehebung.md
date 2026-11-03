# Risoluzione dei problemi {#fehlerbehebung}

Soluzioni ai problemi più comuni e dove Manta registra gli errori. Se segnali un bug, il file di log è di grande aiuto.

## Il file di log {#protokoll}

Manta scrive errori ed eventi importanti in:

`~/Library/Logs/Manta/manta.log`

Per trovarlo:

1. Scegli **Aiuto** → **Mostra il file di log nel Finder** nella barra dei menu, oppure apri la palette dei comandi con **⇧⌘P** e scegli lì **Mostra il file di log nel Finder**.
2. Il Finder apre la cartella dei log (`~/Library/Logs/Manta`) con il file `manta.log`, proprio nel Finder, anche quando Manta è il tuo gestore file predefinito.

Ogni messaggio di errore che Manta ti mostra si trova lì con l’errore tecnico, il percorso e la posizione (locale, server …). Il file ruota a 5 MB; viene conservata una copia precedente.

> [!TIP]
> Per log più dettagliati, avvia Manta dal Terminale con `MANTA_LOG=debug /Applications/Manta.app/Contents/MacOS/manta`.

> [!NOTE]
> Il file di log può contenere nomi di file e di server. Controllalo prima di condividerlo. Password, passphrase e chiavi segrete non ci sono, nemmeno per SFTP, FTP e S3.

I plug-in hanno log propri: **Impostazioni** → **Plug-in** → **Mostra log** sulla scheda del plug-in.

## Segnalare un bug {#fehler-melden}

Hai trovato un bug? Segnalalo su GitHub: il modo più semplice è **Aiuto** → **Segnala un problema…** (anche nella palette dei comandi, digitando “bug” o “errore”, e in **Impostazioni** → **Generali** → **Informazioni su Manta** → **Segnalazioni**). Manta apre nel browser il modulo per le segnalazioni di bug con la versione di Manta, la versione di macOS e il chip (Apple silicon o Intel) già compilati. Manta di per sé non invia nulla: vedi il modulo e decidi tu cosa inviare. Ti serve un account GitHub gratuito.

Cosa aiuta a trovare il bug:

- **Versione di Manta e di macOS**: le inserisce Manta; altrimenti in **Manta** → **Informazioni su Manta** e menu Apple → “Informazioni su questo Mac”.
- **Passaggi per riprodurlo**: cosa hai fatto, passo per passo?
- **Comportamento atteso ed effettivo**: cosa doveva succedere e cosa è successo invece? Il messaggio di errore esatto aiuta.
- **Uno screenshot** (⇧⌘4) o una breve registrazione dello schermo (⇧⌘5).
- **Le righe rilevanti del file di log** `~/Library/Logs/Manta/manta.log`: aprilo con **Aiuto** → **Mostra il file di log nel Finder**, vedi [Il file di log](#protokoll).

> [!IMPORTANT]
> Le segnalazioni di bug su GitHub sono pubbliche. Prima di incollare un log o uno screenshot, rimuovi o sostituisci i dati privati: nomi utente e percorsi come `/Users/anna/…`, nomi di server e host, indirizzi IP, nomi dei bucket.

> [!CAUTION]
> Non segnalare mai le vulnerabilità di sicurezza in una issue pubblica. Segnalale in privato tramite [Report a vulnerability](https://github.com/web-workers-com/manta/security/advisories/new) su GitHub, come descritto nella [politica di sicurezza](https://github.com/web-workers-com/manta/blob/main/SECURITY.md).

Hai un’idea o un suggerimento? **Aiuto** → **Suggerisci un’idea…** apre il modulo per le idee. Tutte le opzioni sono anche su [github.com/web-workers-com/manta/issues/new/choose](https://github.com/web-workers-com/manta/issues/new/choose).

## Se Manta si chiude inaspettatamente {#absturz}

Se Manta si arresta in modo anomalo, salva un rapporto in `~/Library/Logs/Manta`: un file il cui nome inizia con `crash-` seguito da data e ora (UTC). Il rapporto contiene solo ciò che serve per trovare il bug: la versione di Manta e di macOS, il chip, il messaggio di errore e gli indirizzi del programma al momento dell’arresto (il «backtrace»). La tua cartella Inizio compare come `~`, e i nomi di utenti, server e host sono sostituiti da segnaposto come `<user>` e `<redacted>`.

Al successivo avvio, Manta mostra una sola volta un avviso in basso a destra: **L’ultima volta Manta si è chiuso inaspettatamente**.

- **Segnala su GitHub** apre il modulo per la segnalazione di bug con le versioni e un breve riepilogo del rapporto già compilati. Allega il file del rapporto al modulo (trascinalo nel campo di testo): un link non può trasportare un file. Non viene inviato nulla finché non invii tu stesso il modulo su GitHub.
- **Mostra rapporto** apre il rapporto nel tuo editor di testo, così vedi esattamente cosa contiene.
- **Ignora** chiude l’avviso. Il rapporto resta nella cartella; Manta conserva i 10 più recenti.

> [!NOTE]
> Manta non invia mai rapporti sugli arresti anomali da solo e non contiene alcun tracciamento. Se non vuoi segnalare un arresto anomalo, ignora l’avviso, oppure elimina i file `crash-…` in `~/Library/Logs/Manta`.

## Manta non si apre (Gatekeeper) {#gatekeeper}

**Problema:** al primo avvio macOS segnala *Manta non è stata aperta* (*Apple non è in grado di verificare che Manta non contenga malware…*) oppure, nelle versioni meno recenti di macOS, che *“…” non può essere aperto perché Apple non può verificare che non contenga malware*.

**Causa:** Manta non è autenticato da Apple. macOS blocca queste app al primo avvio.

**Soluzione:**

1. Nel messaggio fai clic su **Fine** (non su **Sposta nel Cestino**).
2. Apri **Impostazioni di Sistema** → **Privacy e sicurezza** e scorri fino alla sezione **Sicurezza**.
3. Fai clic su **Apri comunque** accanto a *Il Mac ha bloccato Manta…*, di nuovo su **Apri comunque** quando macOS lo chiede, e conferma con la password o con Touch ID.

![Impostazioni di Sistema, Privacy e sicurezza: il pulsante Apri comunque accanto all’avviso su Manta](images/gatekeeper-open-anyway.png)

Se **Apri comunque** non c’è, avvia di nuovo Manta e guarda subito dopo: il pulsante resta solo per circa un’ora. Tutti i passaggi con le immagini, e la via dal Terminale, sono in [Il primo avvio: Gatekeeper](01-erste-schritte.md#gatekeeper).

Prima di macOS 15: clic destro (o clic con Ctrl) su Manta nella cartella **Applicazioni** → **Apri** → **Apri**.

## “macOS ha bloccato l’accesso” {#berechtigungen}

**Problema:** una cartella non si apre, oppure Manta segnala “macOS ha bloccato l’accesso a “…” (operazione non consentita)”.

**Causa:** non hai consentito a Manta di accedere a questa cartella (o hai fatto clic per errore su “Non consentire”), oppure la cartella richiede l’Accesso completo al disco.

**Soluzione:**

1. Apri **Impostazioni di Sistema** → **Privacy e sicurezza** → **File e cartelle**.
2. Trova **Manta** e attiva la cartella (Scrivania, Documenti, Download, volumi rimovibili, volumi di rete).
3. Per cartelle come `~/Library/Mail`: **Accesso completo al disco** → attiva **Manta**.
4. Esci da Manta e riavvialo.

Se Manta non compare nell’elenco, macOS non ha ancora fatto la domanda. Apri una volta la cartella in Manta e la domanda comparirà.

“Non hai il permesso di aprire questa cartella” è un’altra cosa: mancano i normali permessi Unix. Chiedi al proprietario della cartella oppure modifica i permessi nel Finder (**Ottieni informazioni**).

## Funzioni che controllano il Finder o il Terminale {#automation}

**Problema:** “Mostra nel Finder”, “Apri in Terminal.app” o lo spostamento dei duplicati nel Cestino non funzionano.

**Soluzione:** **Impostazioni di Sistema** → **Privacy e sicurezza** → **Automazione** → sotto **Manta**, attiva **Finder** e **Terminale**.

## FTP: le cartelle non si caricano, errori “TLS” {#ftp-datenkanal}

**Problema:** l’accesso a un server FTPS funziona, ma le cartelle restano vuote o si caricano all’infinito, oppure Manta segnala che “la rete blocca le connessioni dati FTP codificate”.

**Causa:** alcune reti e firewall aziendali reimpostano le connessioni dati FTP codificate.

**Soluzione:**

1. Clic destro sul preferito → **Modifica connessione…** (o **⌘K** per una nuova connessione).
2. Imposta **Canale dati** su **Automatico**.
3. Connettiti di nuovo.

Manta invia allora elenchi e file senza codifica; l’accesso resta codificato. Lo indica uno scudo di avviso sull’indicatore della posizione.

> [!WARNING]
> Non è una buona soluzione per i dati riservati. Usa invece SFTP, un’altra rete o una VPN e imposta il canale dati su **Obbligatorio**.

Altri suggerimenti per FTP:

- “Impossibile stabilire una connessione sicura”: il server non supporta FTPS su questa porta (prova **FTP**, se è accettabile nella tua rete), oppure il suo certificato non è attendibile.
- Manta non supporta l’FTPS implicito (porta 990).

## Un container senza shell {#container-ohne-shell}

**Problema:** un container si apre in sola lettura, oppure Manta segnala “Il container non ha una shell”.

**Causa:** Manta ha bisogno di `sh` nel container per elencare e modificare i file. Le immagini distroless e scratch non hanno una shell.

**Soluzione:**

- Leggere e copiare all’esterno (**F5**, trascinamento) funziona comunque, tramite `docker cp`.
- Per apportare modifiche ti serve un’immagine con una shell (ad esempio una variante di debug dell’immagine).
- I pod Kubernetes richiedono `sh` anche solo per la lettura.
- Anche i container arrestati sono di sola lettura: avviali per apportare modifiche.

## Docker o Kubernetes non compaiono {#container-fehlen}

- **“Docker non è in esecuzione”**: avvia Docker Desktop.
- **“Docker non è installato”**: installa Docker Desktop, oppure assicurati che il comando `docker` sia nel tuo PATH. Manta cerca anche in `/opt/homebrew/bin`, `/usr/local/bin` e all’interno dell’app Docker Desktop.
- **“kubectl non è installato”**: `brew install kubectl`.
- **“Il cluster Kubernetes non è raggiungibile”**: controlla il contesto e la VPN, oppure avvia il tuo cluster locale.

## Connessioni ai server {#server}

| Messaggio | Soluzione |
|---|---|
| “La chiave del server è cambiata” | La chiave non corrisponde a `~/.ssh/known_hosts`; Manta non si connette (possibile attacco man-in-the-middle). Chiedi l’impronta all’amministratore. Se il server è stato davvero reinstallato, rimuovi la vecchia voce con il comando indicato nella finestra (`ssh-keygen -R host`). |
| “Server sconosciuto” | Prima connessione a questo server. Confronta l’impronta e fai clic su **Connetti e ricorda**, vedi [Connessioni](08-verbindungen.md). |
| “La password salvata non è stata trovata nel Portachiavi” | L’elemento è stato eliminato, ad esempio in Accesso Portachiavi. Inserisci la password e lascia attivo **Ricorda password**. |
| “Nessuna chiave utilizzabile in ssh-agent” | `ssh-add ~/.ssh/id_ed25519` |
| “L’orologio del Mac non è corretto” | **Impostazioni di Sistema** → **Generali** → **Data e ora** → impostazione automatica. |
| “Il bucket si trova in un’altra regione” | Nella finestra imposta la regione indicata. |
| “La connessione al server è stata persa” | Controlla la rete, poi **Riprova**. Manta accede di nuovo al server da solo. |

## Un trasferimento non va avanti {#uebertragungen}

| Cosa vedi | Soluzione |
|---|---|
| “In attesa di connessione… (tentativo 5, prossimo tra 16 s)” | La rete o il server non sono raggiungibili. Manta continua a provare e riprende dallo stesso punto. Controlla Wi-Fi/VPN; **Riprova ora** avvia subito il tentativo successivo. |
| **Nessuna connessione a Internet** in cima a **Trasferimenti** | Il tuo Mac è offline. Non appena torna online, i trasferimenti riprendono da soli. |
| “Richiede una connessione a …” | Dopo un riavvio manca la connessione al server. Fai clic su **Riconnetti** e inserisci la password. |
| “In pausa · …” | Hai sospeso l’operazione (tu o **Metti tutto in pausa**). Fai clic su **Riprendi**. |
| File nascosti `.NAME.manta-part` nella destinazione | File parziali di un’operazione interrotta. Lasciali finché l’operazione deve essere ripresa; altrimenti puoi eliminarli. |

Vedi [Sospendere e riprendere i trasferimenti](05-dateien.md#pausieren).

## Un aggiornamento non riesce {#updates}

Vedi [Aggiornamenti](17-updates.md#blockiert). Di solito Manta non viene eseguito dalla cartella **Applicazioni**.

## Cloud: “Rimuovi download” non funziona {#cloud}

Per Google Drive, OneDrive, Dropbox e altri decide l’app del fornitore. Usa il Finder: clic destro → **Rimuovi download**. Vedi [Archiviazione cloud](09-cloud.md#download-entfernen).

## Una cartella mostra contenuti non aggiornati {#aktualisieren}

Premi **⌘R**. Manta vede le modifiche fatte da altre app o dal server solo dopo un aggiornamento.

## Ripristinare Manta {#zuruecksetzen}

Come ultima risorsa:

1. Esci da Manta.
2. Rinomina `~/Library/Application Support/Manta` (ad esempio in `Manta old`).
3. Avvia Manta. Partirà come se fosse la prima volta.

I tuoi Preferiti e le tue azioni rapide si trovano ancora nella cartella rinominata.
