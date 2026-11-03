# Fehlerbehebung {#fehlerbehebung}

Hier findest du Lösungen für häufige Probleme und erfährst, wo Manta Fehler protokolliert. Wenn du einen Fehler meldest, hilft die Protokolldatei sehr.

## Die Protokolldatei {#protokoll}

Manta schreibt Fehler und wichtige Ereignisse in:

`~/Library/Logs/Manta/manta.log`

So findest du sie:

1. Wähle in der Menüleiste **Hilfe** → **Protokolldatei im Finder zeigen** – oder öffne die Befehlspalette mit **⇧⌘P** und wähle dort **Protokolldatei im Finder zeigen**.
2. Der Finder öffnet den Protokollordner (`~/Library/Logs/Manta`) mit der Datei `manta.log` – im Finder selbst, auch wenn Manta dein Standard-Dateimanager ist.

Jede Fehlermeldung, die Manta dir anzeigt, steht dort mit dem technischen Fehler, dem Pfad und dem Ort (lokal, Server …). Die Datei wird bei 5 MB rotiert; eine ältere Kopie bleibt erhalten.

> [!TIP]
> Für ausführlichere Protokolle starte Manta im Terminal mit `MANTA_LOG=debug /Applications/Manta.app/Contents/MacOS/manta`.

> [!NOTE]
> Die Protokolldatei kann Datei- und Servernamen enthalten. Prüfe sie, bevor du sie weitergibst. Passwörter, Passphrasen und geheime Schlüssel stehen nicht darin – auch nicht bei SFTP, FTP und S3.

Plugins haben eigene Protokolle: **Einstellungen** → **Plugins** → **Protokoll einblenden** auf der Karte des Plugins.

## Fehler melden {#fehler-melden}

Hast du einen Fehler gefunden? Melde ihn auf GitHub – am einfachsten mit **Hilfe** → **Fehler melden …** (auch in der Befehlspalette, Stichwort „Fehler“, und unter **Einstellungen** → **Allgemein** → **Über Manta** → **Rückmeldung**). Manta öffnet das Formular für Fehlerberichte im Browser und trägt die Manta-Version, die macOS-Version und den Chip (Apple Silicon oder Intel) schon ein. Manta selbst sendet dabei nichts: Du siehst das Formular und entscheidest, was du abschickst. Du brauchst ein kostenloses GitHub-Konto.

Das hilft beim Finden des Fehlers:

- **Manta- und macOS-Version** – trägt Manta ein; sonst unter **Manta** → **Über Manta** und Apple-Menü → „Über diesen Mac“.
- **Schritte zum Nachstellen** – was hast du der Reihe nach getan?
- **Erwartet und tatsächlich** – was sollte passieren, und was ist stattdessen passiert? Die genaue Fehlermeldung hilft.
- **Ein Bildschirmfoto** (⇧⌘4) oder eine kurze Bildschirmaufnahme (⇧⌘5).
- **Die passenden Zeilen aus der Protokolldatei** `~/Library/Logs/Manta/manta.log` – öffnen mit **Hilfe** → **Protokolldatei im Finder zeigen**, siehe [Die Protokolldatei](#protokoll).

> [!IMPORTANT]
> Fehlerberichte auf GitHub sind öffentlich. Entferne oder ersetze vor dem Einfügen private Daten aus Protokoll und Bildschirmfotos: Benutzernamen und Pfade wie `/Users/anna/…`, Server- und Hostnamen, IP-Adressen, Bucket-Namen.

> [!CAUTION]
> Sicherheitslücken meldest du bitte nie in einem öffentlichen Issue, sondern vertraulich über [Report a vulnerability](https://github.com/web-workers-com/manta/security/advisories/new) auf GitHub, wie in der [Sicherheitsrichtlinie](https://github.com/web-workers-com/manta/blob/main/SECURITY.md) beschrieben.

Eine Idee oder ein Verbesserungsvorschlag? **Hilfe** → **Idee vorschlagen …** öffnet das Formular für Vorschläge. Alle Wege findest du auch unter [github.com/web-workers-com/manta/issues/new/choose](https://github.com/web-workers-com/manta/issues/new/choose).

## Wenn Manta unerwartet beendet wird {#absturz}

Stürzt Manta ab, speichert es einen Absturzbericht in `~/Library/Logs/Manta` – eine Datei, deren Name mit `crash-` beginnt, gefolgt von Datum und Uhrzeit (UTC). Der Bericht enthält nur, was zum Finden des Fehlers nötig ist: die Manta- und macOS-Version, den Chip, die Fehlermeldung und die Programmadressen im Moment des Absturzes (den „Backtrace“). Dein Benutzerordner erscheint als `~`, Benutzer-, Server- und Hostnamen sind durch Platzhalter wie `<user>` und `<redacted>` ersetzt.

Beim nächsten Start zeigt Manta unten rechts einmal den Hinweis **Manta wurde beim letzten Mal unerwartet beendet**.

- **Auf GitHub melden** öffnet das Formular für Fehlerberichte mit den Versionen und einer kurzen Zusammenfassung des Berichts. Hänge die Berichtsdatei im Formular an (in das Textfeld ziehen) – ein Link kann keine Datei mitnehmen. Abgeschickt wird erst, wenn du das Formular auf GitHub selbst absendest.
- **Bericht zeigen** öffnet den Bericht in deinem Texteditor, damit du genau siehst, was darin steht.
- **Ignorieren** schließt den Hinweis. Der Bericht bleibt im Ordner; Manta behält die 10 neuesten.

> [!NOTE]
> Manta sendet Absturzberichte nie von selbst und enthält kein Tracking. Willst du einen Absturz nicht melden, ignoriere den Hinweis – oder lösche die Dateien `crash-…` in `~/Library/Logs/Manta`.

## Manta lässt sich nicht öffnen (Gatekeeper) {#gatekeeper}

**Problem:** Beim ersten Start meldet macOS „‚Manta‘ nicht geöffnet“ („Apple konnte nicht überprüfen, ob ‚Manta‘ frei von Schadsoftware ist …“) oder unter älteren macOS-Versionen „… kann nicht geöffnet werden, da Apple es nicht auf Schadsoftware überprüfen kann“.

**Ursache:** Manta ist nicht von Apple beglaubigt (notarisiert). macOS blockiert solche Apps beim ersten Start.

**Lösung:**

1. Klicke in der Meldung auf **Fertig** (nicht auf **In den Papierkorb legen**).
2. Öffne **Systemeinstellungen** → **Datenschutz & Sicherheit** und scrolle zum Abschnitt **Sicherheit**.
3. Klicke bei „‚Manta‘ wurde blockiert …“ auf **Dennoch öffnen**, in der Rückfrage noch einmal auf **Dennoch öffnen**, und bestätige mit Passwort oder Touch ID.

![Systemeinstellungen, Datenschutz & Sicherheit: die Taste Dennoch öffnen neben dem Hinweis zu Manta](images/gatekeeper-open-anyway.png)

Fehlt **Dennoch öffnen**, starte Manta noch einmal und schau gleich danach nach – die Taste bleibt nur etwa eine Stunde da. Alle Schritte mit Bildern und den Weg über das Terminal findest du unter [Der erste Start: Gatekeeper](01-erste-schritte.md#gatekeeper).

Vor macOS 15: Rechtsklick (oder Control-Klick) auf Manta im Ordner **Programme** → **Öffnen** → **Öffnen**.

## „macOS hat den Zugriff blockiert“ {#berechtigungen}

**Problem:** Ein Ordner lässt sich nicht öffnen, oder Manta meldet „macOS hat den Zugriff auf „…“ blockiert (Vorgang nicht erlaubt)“.

**Ursache:** Du hast Manta den Zugriff auf diesen Ordner nicht erlaubt (oder versehentlich „Nicht erlauben“ geklickt), oder der Ordner braucht Festplattenvollzugriff.

**Lösung:**

1. Öffne **Systemeinstellungen** → **Datenschutz & Sicherheit** → **Dateien und Ordner**.
2. Suche **Manta** und schalte den Ordner (Schreibtisch, Dokumente, Downloads, Wechseldatenträger, Netzwerkvolumes) ein.
3. Für Ordner wie `~/Library/Mail`: **Festplattenvollzugriff** → **Manta** einschalten.
4. Beende Manta und starte es neu.

Taucht Manta nicht in der Liste auf, hat macOS noch nicht gefragt. Öffne den Ordner einmal in Manta, dann erscheint die Frage.

„Du hast keine Berechtigung, diesen Ordner zu öffnen“ ist etwas anderes: Hier fehlen die normalen Unix-Zugriffsrechte. Frag den Eigentümer des Ordners oder ändere die Rechte im Finder (**Informationen**).

## Funktionen, die Finder oder Terminal steuern {#automation}

**Problem:** „Im Finder zeigen“, „In Terminal.app öffnen“ oder das Entfernen von Duplikaten funktioniert nicht.

**Lösung:** **Systemeinstellungen** → **Datenschutz & Sicherheit** → **Automation** → bei **Manta** **Finder** und **Terminal** einschalten.

## FTP: Ordner laden nicht, „TLS“-Fehler {#ftp-datenkanal}

**Problem:** Die Anmeldung bei einem FTPS-Server klappt, aber Ordner bleiben leer oder laden ewig, oder Manta meldet „Das Netzwerk blockiert verschlüsselte FTP-Datenverbindungen“.

**Ursache:** Manche Firmennetze und Firewalls setzen verschlüsselte FTP-Datenverbindungen zurück.

**Lösung:**

1. Rechtsklick auf den Favoriten → **Verbindung bearbeiten …** (oder **⌘K** für eine neue Verbindung).
2. Stelle **Datenkanal** auf **Automatisch**.
3. Verbinde neu.

Manta überträgt dann Ordnerinhalte und Dateien unverschlüsselt; die Anmeldung bleibt verschlüsselt. Ein Warnschild am Ort-Chip zeigt das an.

> [!WARNING]
> Für vertrauliche Daten ist das keine gute Lösung. Nutze dann SFTP, ein anderes Netz oder ein VPN, und stelle den Datenkanal auf **Erforderlich**.

Weitere FTP-Hinweise:

- „Es konnte keine sichere Verbindung hergestellt werden“: Der Server kann auf diesem Port kein FTPS (versuche **FTP**, falls das in deinem Netz vertretbar ist), oder sein Zertifikat ist nicht vertrauenswürdig.
- Implizites FTPS (Port 990) unterstützt Manta nicht.

## Container ohne Shell {#container-ohne-shell}

**Problem:** Ein Container öffnet sich schreibgeschützt, oder Manta meldet „Der Container hat keine Shell“.

**Ursache:** Manta braucht `sh` im Container, um Dateien aufzulisten und zu ändern. Distroless- und Scratch-Images haben keine Shell.

**Lösung:**

- Lesen und Herauskopieren (**F5**, Ziehen) funktioniert trotzdem – über `docker cp`.
- Zum Ändern brauchst du ein Image mit Shell (z. B. eine Debug-Variante des Images).
- Kubernetes-Pods brauchen `sh` auch zum Lesen.
- Gestoppte Container sind ebenfalls schreibgeschützt: Starte sie zum Ändern.

## Docker oder Kubernetes erscheinen nicht {#container-fehlen}

- **„Docker läuft nicht“**: Starte Docker Desktop.
- **„Docker ist nicht installiert“**: Installiere Docker Desktop, oder sorge dafür, dass der Befehl `docker` im PATH ist. Manta sucht auch in `/opt/homebrew/bin`, `/usr/local/bin` und im Docker-Desktop-Programm.
- **„kubectl ist nicht installiert“**: `brew install kubectl`.
- **„Der Kubernetes-Cluster ist nicht erreichbar“**: Prüfe den Kontext und dein VPN oder starte deinen lokalen Cluster.

## Server-Verbindungen {#server}

| Meldung | Lösung |
|---|---|
| „Der Schlüssel des Servers hat sich geändert“ | Der Schlüssel passt nicht zu `~/.ssh/known_hosts`; Manta verbindet sich nicht (möglicher Man-in-the-Middle-Angriff). Frag den Administrator nach dem Fingerabdruck. Ist der Server wirklich neu installiert, entferne den alten Eintrag mit dem Befehl aus dem Dialog (`ssh-keygen -R host`). |
| „Unbekannter Server“ | Erste Verbindung zu diesem Server. Vergleiche den Fingerabdruck und klicke **Verbinden und merken**, siehe [Verbindungen](08-verbindungen.md). |
| „Das gesicherte Passwort wurde nicht im Schlüsselbund gefunden“ | Der Eintrag wurde z. B. in der Schlüsselbundverwaltung gelöscht. Gib das Passwort ein und lass **Passwort merken** an. |
| „Kein verwendbarer Schlüssel im ssh-agent“ | `ssh-add ~/.ssh/id_ed25519` |
| „Die Uhr deines Mac geht falsch“ | **Systemeinstellungen** → **Allgemein** → **Datum & Uhrzeit** → automatisch einstellen. |
| „Der Bucket liegt in einer anderen Region“ | Region im Dialog auf die genannte ändern. |
| „Die Verbindung zum Server wurde unterbrochen“ | Netzwerk prüfen, dann **Erneut versuchen**. Manta meldet sich dabei selbst neu am Server an. |

## Eine Übertragung kommt nicht weiter {#uebertragungen}

| Was du siehst | Lösung |
|---|---|
| „Wartet auf Verbindung … (Versuch 5, nächster in 16 s)“ | Das Netzwerk oder der Server ist weg. Manta versucht es weiter und macht an derselben Stelle weiter. Prüfe WLAN/VPN; **Jetzt erneut versuchen** startet den nächsten Versuch sofort. |
| **Keine Internetverbindung** oben im Bereich **Übertragungen** | Dein Mac ist offline. Sobald er wieder online ist, laufen die Übertragungen von selbst weiter. |
| „Benötigt eine Verbindung zu …“ | Nach einem Neustart fehlt die Server-Verbindung. Klicke auf **Verbinden** und gib das Passwort ein. |
| „Pausiert · …“ | Du (oder **Alle pausieren**) hast den Vorgang angehalten. Klicke auf **Fortsetzen**. |
| Unsichtbare Dateien `.NAME.manta-part` im Ziel | Teildateien eines unterbrochenen Vorgangs. Lass sie liegen, solange der Vorgang fortgesetzt werden soll; sonst darfst du sie löschen. |

Mehr in [Übertragungen pausieren und fortsetzen](05-dateien.md#pausieren).

## Ein Update klappt nicht {#updates}

Siehe [Updates](17-updates.md#blockiert). Meist läuft Manta nicht aus dem Ordner **Programme**.

## Cloud: „Download entfernen“ geht nicht {#cloud}

Bei Google Drive, OneDrive, Dropbox und anderen entscheidet die App des Anbieters. Nutze im Finder Rechtsklick → **Download entfernen**. Siehe [Cloud-Speicher](09-cloud.md#download-entfernen).

## Ein Ordner zeigt veraltete Inhalte {#aktualisieren}

Drücke **⌘R**. Änderungen, die andere Apps oder der Server machen, sieht Manta erst nach dem Aktualisieren.

## Manta zurücksetzen {#zuruecksetzen}

Als letzte Möglichkeit:

1. Beende Manta.
2. Benenne `~/Library/Application Support/Manta` um (z. B. in `Manta alt`).
3. Starte Manta. Es beginnt wie beim ersten Start.

Deine Favoriten und Schnellaktionen findest du dann im umbenannten Ordner.
