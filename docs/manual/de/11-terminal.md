# Terminal {#terminal}

Unter jedem Bereich kannst du ein Terminal einblenden, das dem Ordner des Bereichs folgt – und umgekehrt. Es läuft dort, wo der Bereich gerade ist: lokal, auf einem SFTP-Server oder in einem Container.

![Bereich mit Terminal darunter](images/terminal.png)

## So blendest du das Terminal ein {#einblenden}

- Drücke **⌃`** (**Terminal ein-/ausblenden**).
- Oder wähle **Terminal ein-/ausblenden** in der Befehlspalette.

Das Terminal erscheint unter dem aktiven Bereich. Die Höhe änderst du, indem du an seinem oberen Rand ziehst.

Weitere Befehle in der Befehlspalette:

- **Neue Terminal-Sitzung** – startet eine frische Shell.
- **Terminal leeren** – leert die Anzeige.

Beendest du die Shell (z. B. mit `exit`), zeigt das Terminal „[Prozess beendet – zum Neustarten den Zeilenschalter drücken]“.

## Bereich und Terminal folgen einander {#folgen}

**Vom Bereich zum Terminal:** Wechselst du im Bereich den Ordner, wechselt die Shell mit `cd` dorthin – sobald sie bereit ist, also wenn gerade kein Befehl läuft. Bis dahin zeigt ein Hinweis „Wechselt nach „Ordner“, sobald die Shell bereit ist“. Was du schon halb eingetippt hast, bleibt erhalten.

**Vom Terminal zum Bereich:** Tippst du im Terminal `cd ~/Projekte`, zeigt der Bereich darüber diesen Ordner. Das schaltest du unter **Bereich folgt „cd“ im Terminal** aus (siehe [Einstellungen](#einstellungen)).

Dein Prompt und deine Shell-Konfiguration bleiben dabei unverändert.

## Terminal in Containern und auf Servern {#container-ssh}

Das Terminal läuft dort, wo der Bereich ist:

| Bereich zeigt … | Terminal |
|---|---|
| einen lokalen Ordner (auch Cloud-Ordner) | deine normale Shell auf dem Mac |
| einen Docker-Container | eine Shell im Container (`docker exec`) |
| einen Kubernetes-Pod | eine Shell im Pod (`kubectl exec`) |
| einen SFTP-Server | `ssh` zum Server, mit deiner SSH-Konfiguration und deinem ssh-agent |
| S3, FTP, ein Archiv | kein Terminal („Für diesen Ort gibt es kein Terminal“) |

- Manta installiert dafür nichts im Container oder auf dem Server.
- Bei SFTP-Verbindungen mit Passwort tippst du das Passwort im Terminal ein. Eine Schlüsseldatei, die nicht in deiner SSH-Konfiguration oder im Agent steckt, fragt ebenfalls im Terminal.
- Jeder Bereich hält bis zu vier Shells (lokal, Container, Server). Gehst du zurück, ist die frühere Shell mit ihrem Verlauf wieder da.
- Ist ein Container gestoppt, zeigt das Terminal „Container „Name“ ist gestoppt“. Starte ihn und klicke auf den Hinweis.

Mit Rechtsklick → **Shell im Container öffnen** startest du gezielt eine Shell im angezeigten Container-Ordner.

### `ssh` im lokalen Terminal {#ssh-erkennen}

Tippst du im lokalen Terminal selbst `ssh` (z. B. `ssh anna@server` oder einen Host aus deiner `~/.ssh/config`), erkennt Manta das. Solange die Verbindung läuft, erscheint oben rechts im Terminal **anna@server im Bereich öffnen**. Ein Klick zeigt den Server per SFTP im Bereich: Eine offene Verbindung zu ihm nutzt Manta wieder, sonst einen passenden Favoriten, sonst öffnet sich der Verbindungsdialog fertig ausgefüllt (du gibst nur noch das Passwort ein). Danach folgen Bereich und Terminal einander wie bei einem Container, und du kannst zwischen den Bereichen kopieren und bewegen.

- Das Terminal startet dann eine eigene Shell auf dem Server; bei Anmeldung mit Passwort fragt sie es dort noch einmal. Deine getippte `ssh`-Sitzung bleibt im Hintergrund erhalten, bis du zum lokalen Ordner zurückgehst.
- Erkannt wird `ssh` in der zsh (der Standard-Shell von macOS), auch über einen Alias. `ssh` mit einem Befehl dahinter (`ssh server uptime`) oder nur für einen Tunnel (`-N`) bietet nichts an.

## Dateiaktionen im Terminal {#dateiaktionen}

Kopierst, bewegst, löschst oder benennst du in Manta etwas um, zeigt das Terminal den passenden Shell-Befehl – z. B. `cp -R …`, `scp …`, `docker cp …` oder `aws s3 cp …` – und danach das Ergebnis („fertig“, „fehlgeschlagen: …“). In den Papierkorb legen erscheint als `mv … ~/.Trash/`, endgültiges Löschen als `rm -r …`.

> [!NOTE]
> Das ist nur eine Anzeige. Manta erledigt die Arbeit selbst; in der Shell wird nichts ausgeführt. Du kannst die Befehle aber kopieren und z. B. in eigene Skripte übernehmen.

Auf entfernten Shells (Container, SSH) zeigt Manta diese Befehle als Karte über dem Terminal, statt sie in die Shell zu schreiben.

Ausschalten: **Dateiaktionen im Terminal zeigen** in den Einstellungen.

## Einstellungen {#einstellungen}

Öffne **Einstellungen** (**⌘,**) → **Plugins** → **Terminal** → **Einstellungen**.

| Einstellung | Bedeutung |
|---|---|
| **Shell** | Welche Shell startet (z. B. `/bin/zsh`). |
| **Schriftgröße** | Größe der Terminalschrift. |
| **Verlauf** | Wie viele Zeilen du zurückscrollen kannst. |
| **Wahltaste als Meta-Taste verwenden** | Wahltaste+Taste sendet ESC+Taste (für Kurzbefehle wie in Emacs), statt Sonderzeichen wie „@“ oder „€“ zu tippen. |
| **Bereich folgt „cd“ im Terminal** | Ein `cd` im Terminal ändert den Ordner des Bereichs. |
| **Dateiaktionen im Terminal zeigen** | Zeigt Dateiaktionen als Shell-Befehle (siehe oben). |

> [!TIP]
> Auf einer deutschen Tastatur brauchst du die Wahltaste für `@`, `{`, `[` und `|`. Lass **Wahltaste als Meta-Taste verwenden** deshalb ausgeschaltet, wenn du nicht mit Emacs-Kürzeln arbeitest.

## Tastatur im Terminal {#tastatur}

Solange das Terminal den Fokus hat, gehören diese Kürzel dem Terminal statt Manta: **⌘C**, **⌘V**, **⌘A**, **⌘K**, **⌘←**, **⌘→**, **⌘⌫** und **⌘Z**. Andere Manta-Kürzel wie **⇧⌘P** funktionieren weiter. Klicke in die Dateiliste, um Manta wieder die Tastatur zu geben.

## Grenzen {#grenzen}

- busybox-`sh` (Alpine) und bash 3 können halb getippte Eingaben nicht behalten, wenn das Terminal dem Bereich folgt (bash ab 4 und zsh können es).
- In Containern mit einer anderen Shell als bash oder POSIX-`sh` folgt das Terminal nur in eine Richtung.
- Kein Terminal für S3, FTP und Archive.
