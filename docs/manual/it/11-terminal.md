# Terminale {#terminal}

Sotto ogni pannello puoi mostrare un terminale che segue la cartella del pannello, e viceversa. Funziona dove si trova il pannello: sul tuo Mac, su un server SFTP o in un container.

![Un pannello con un terminale sotto](images/terminal.png)

## Mostrare il terminale {#einblenden}

- Premi **⌃`** (**Mostra/nascondi Terminale**).
- Oppure scegli **Mostra/nascondi Terminale** nella palette dei comandi.

Il terminale compare sotto il pannello attivo. Trascina il suo bordo superiore per cambiarne l’altezza.

Altri comandi nella palette dei comandi:

- **Nuova sessione Terminale**: avvia una nuova shell.
- **Pulisci Terminale**: pulisce lo schermo.

Se chiudi la shell (ad esempio con `exit`), il terminale mostra “[Processo terminato – premi A capo per riavviare]”.

## Pannello e terminale si seguono a vicenda {#folgen}

**Dal pannello al terminale:** quando cambi cartella nel pannello, la shell esegue `cd` verso di essa non appena è pronta, cioè quando nessun comando è in esecuzione. Fino ad allora un avviso indica “Passerà a “cartella” quando la shell è pronta”. Quello che hai digitato a metà viene conservato.

**Dal terminale al pannello:** digita `cd ~/Projects` nel terminale e il pannello sopra mostra quella cartella. Puoi disattivarlo con **Il pannello segue “cd” nel terminale** (vedi [Impostazioni](#einstellungen)).

Il prompt e la configurazione della shell restano invariati.

## Terminale nei container e sui server {#container-ssh}

Il terminale funziona dove si trova il pannello:

| Il pannello mostra … | Terminale |
|---|---|
| una cartella locale (anche le cartelle cloud) | la tua normale shell sul Mac |
| un container Docker | una shell all’interno del container (`docker exec`) |
| un pod Kubernetes | una shell all’interno del pod (`kubectl exec`) |
| un server SFTP | `ssh` verso il server, con la tua configurazione SSH e ssh-agent |
| S3, FTP, un archivio | nessun terminale (“Nessun terminale per questa posizione”) |

- Per questo Manta non installa nulla nel container o sul server.
- Per le connessioni SFTP con password, digiti la password nel terminale. Anche un file chiave che non è nella tua configurazione SSH o nell’agent viene richiesto nel terminale.
- Ogni pannello conserva fino a quattro shell (locale, container, server). Quando torni indietro, la shell precedente è di nuovo lì con la sua cronologia.
- Se un container è arrestato, il terminale indica “Il container “nome” è arrestato”. Avvialo e fai clic sull’avviso.

Clic destro → **Apri shell nel container** avvia una shell nella cartella del container che stai guardando.

### `ssh` nel terminale locale {#ssh-erkennen}

Se digiti tu stesso `ssh` nel terminale locale (ad es. `ssh anna@server` o un host del tuo `~/.ssh/config`), Manta se ne accorge. Finché la connessione è attiva, in alto a destra nel terminale compare **Apri anna@server nel pannello**. Un clic mostra il server via SFTP nel pannello: Manta riusa una connessione aperta verso di esso, altrimenti un preferito corrispondente, altrimenti si apre la finestra di connessione già compilata (inserisci solo la password). Da lì pannello e terminale si seguono come con un container, e puoi copiare e spostare tra i pannelli.

- Il terminale avvia quindi una propria shell sul server; con l’accesso tramite password, la richiede lì ancora una volta. La sessione `ssh` che hai digitato resta in background finché non torni alla cartella locale.
- `ssh` viene riconosciuto in zsh (la shell predefinita di macOS), anche tramite un alias. `ssh` seguito da un comando (`ssh server uptime`) o solo per un tunnel (`-N`) non propone nulla.

## Azioni sui file nel terminale {#dateiaktionen}

Quando copi, sposti, elimini o rinomini qualcosa in Manta, il terminale mostra il comando shell corrispondente, ad esempio `cp -R …`, `scp …`, `docker cp …` o `aws s3 cp …`, seguito dal risultato (“done”, “failed: …”). Lo spostamento nel Cestino compare come `mv … ~/.Trash/`, un’eliminazione definitiva come `rm -r …`.

> [!NOTE]
> È solo una visualizzazione. Il lavoro lo fa Manta stesso; nella shell non viene eseguito nulla. Puoi però copiare i comandi, ad esempio per i tuoi script.

Nelle shell remote (container, SSH) Manta mostra questi comandi come una scheda sopra il terminale invece di scriverli nella shell.

Per disattivarlo: **Mostra azioni sui file nel terminale** nelle impostazioni.

## Impostazioni {#einstellungen}

Apri **Impostazioni** (**⌘,**) → **Plug-in** → **Terminale** → **Impostazioni**.

| Impostazione | Significato |
|---|---|
| **Shell** | Quale shell viene avviata (ad esempio `/bin/zsh`). |
| **Dimensione font** | Dimensione del carattere del terminale. |
| **Cronologia** | Quante righe puoi scorrere all’indietro. |
| **Usa Opzione come tasto Meta** | Opzione+tasto invia ESC+tasto (per le scorciatoie in stile Emacs) invece di digitare caratteri speciali. |
| **Il pannello segue “cd” nel terminale** | Un `cd` nel terminale cambia la cartella del pannello. |
| **Mostra azioni sui file nel terminale** | Mostra le azioni sui file come comandi shell (vedi sopra). |

> [!TIP]
> Su molte tastiere non statunitensi, inclusa quella italiana, ti serve Opzione per `@`, `{`, `[` e `|`. Lascia disattivato **Usa Opzione come tasto Meta**, a meno che tu non usi scorciatoie in stile Emacs.

## Tastiera nel terminale {#tastatur}

Quando il terminale è attivo, queste scorciatoie appartengono al terminale invece che a Manta: **⌘C**, **⌘V**, **⌘A**, **⌘K**, **⌘←**, **⌘→**, **⌘⌫** e **⌘Z**. Le altre scorciatoie di Manta, come **⇧⌘P**, continuano a funzionare. Fai clic nell’elenco dei file per restituire la tastiera a Manta.

## Limiti {#grenzen}

- `sh` di busybox (Alpine) e bash 3 non riescono a conservare l’input digitato a metà quando il terminale segue il pannello (bash 4 o successivo e zsh sì).
- Nei container con una shell diversa da bash o da `sh` POSIX, il terminale segue in una sola direzione.
- Nessun terminale per S3, FTP e archivi.
