# Einstellungen {#einstellungen}

Hier findest du jede Einstellung von Manta, Abschnitt für Abschnitt. Alle Änderungen wirken sofort; einen Neustart brauchst du nie.

## So öffnest du die Einstellungen {#oeffnen}

- **⌘,**
- **Manta** → **Einstellungen …** in der Menüleiste
- das Zahnrad in der Symbolleiste
- **Einstellungen …** in der Befehlspalette

Die Einstellungen sind ein eigenes Fenster mit den Bereichen **Allgemein** und **Plugins**. Schließen: rote Fenstertaste oder **⌘W**. **⌘,** holt das Fenster wieder nach vorn.

Solange die Einstellungen offen sind, ruht das Hauptfenster: Es ist leicht abgedunkelt, und Klicks und Tastenkürzel dort wirken nicht – ein Klick holt die Einstellungen mit einem Hinweiston nach vorn. Schließ sie, um weiterzuarbeiten. Das Hilfefenster bleibt dagegen frei nutzbar.

![Einstellungen → Allgemein](images/settings-general.png)

## Allgemein {#allgemein}

### Erscheinungsbild {#erscheinungsbild}

| Einstellung | Optionen |
|---|---|
| **Erscheinungsbild** | **Automatisch** (folgt dem hellen oder dunklen Modus des Systems), **Hell**, **Dunkel**. Auch per Befehlspalette: „Erscheinungsbild: Hell/Dunkel/Automatisch (System)“. |
| **Akzentfarbe** | **Blau**, **Lila**, **Rosa**, **Rot**, **Orange**, **Gelb**, **Grün**, **Graphit** – für Auswahl, Tasten und Fokus. Jede Farbe ist in hell und dunkel gut lesbar. |

### Sprache {#sprache}

**Sprache**: **System** (Standard) oder eine der Sprachen der App (**Deutsch**, **English**, **简体中文**, **Español**, **Français**, **Italiano**, **Русский**), jede in ihrem eigenen Namen. Der Wechsel wirkt sofort. „System“ folgt den bevorzugten Sprachen in macOS (oder Englisch, wenn Manta keine davon kann). Datumsangaben („Heute 14:02“), Größen („1,2 MB“) und Zahlen folgen der Sprache. Die eingebauten Plugins wechseln mit; Dateinamen, Shell-Ausgaben und deine eigenen Schnellaktionen bleiben, wie sie sind.

### Standard-Dateimanager {#standard-dateimanager}

**Manta als Standard-Dateimanager**: Ist der Schalter an, öffnen sich „Im Finder anzeigen“ und Ordner aus anderen Apps in Manta. Darunter steht, welche App die Rolle gerade hat („Derzeit: …“) und an wen sie beim Ausschalten zurückgeht. Was macOS dabei erlaubt, steht in [Erste Schritte](01-erste-schritte.md#standard-dateimanager).

Denselben Schalter findest du im Menü **Manta** (**Manta als Standard-Dateimanager**, mit Häkchen, wenn er an ist) und in der Befehlspalette (**Manta als Standard-Dateimanager verwenden …**, bzw. **Standard-Dateimanager an … zurückgeben**, wenn Manta die Rolle hat). Beim Einschalten stellt Manta dort dieselbe Frage wie beim ersten Start, mit **Abbrechen**; das Ausschalten gibt die Rolle sofort zurück.

Nur verfügbar, wenn Manta aus dem Ordner **Programme** läuft („Bewege Manta zuerst in den Ordner ‚Programme‘“).

### Dateien {#dateien}

| Einstellung | Bedeutung |
|---|---|
| **Unsichtbare Dateien einblenden** | Für neue Tabs und beim Start. **⇧⌘.** blendet sie im aktuellen Tab ein oder aus. |
| **Archive als Ordner öffnen** | ↩ oder Doppelklick zeigt ZIP- und TAR-Archive im Bereich an (schreibgeschützt). **⌥↩** öffnet sie mit der Standard-App. Siehe [Archive](10-archive.md). |
| **Neue Tabs öffnen in** | Wo **⌘T**, **Neuer Tab** im Dock-Menü und ein Start ohne gespeicherte Sitzung öffnen: **Gleicher Ordner** (Standard; beim Start der Benutzerordner), **Benutzerordner**, **Schreibtisch**, **Dokumente**, **Downloads** oder **Anderer …** (ein Ordner deiner Wahl). Wie „Neue Finder-Fenster zeigen“ im Finder. |
| **Ansicht pro Ordner merken** | Jeder Ordner öffnet sich in der Darstellung, in der er zuletzt gezeigt wurde (Symbole, Liste, Spalten, Galerie). Aus (Standard): Die Ansicht gehört zum Bereich. Siehe [Ansichten](03-ansichten.md#pro-ordner). |
| **Maus-Seitentasten** | Die Daumentasten gehen im Bereich unter dem Zeiger zurück und vorwärts, wie **⌘[** und **⌘]**, auch in der Hilfe. **Zurück / Vorwärts** (Standard), **Vorwärts / Zurück (vertauscht)** oder **Aus**. |
| **Vorschaubilder für Server-Dateien laden** | Lädt kleine Bilder (unter 5 MB) im Hintergrund. Aus (Standard): Server-Dateien zeigen ihr Typsymbol. Siehe [Vorschaubilder](03-ansichten.md#server-vorschaubilder). |
| **Dateisymbole von macOS** | Dateien zeigen das Symbol, das macOS für ihren Typ hat, Apps ihr eigenes – wie im Finder, auch auf Servern. Aus: Mantas Symbole. Standard: an. |

### Vorschau {#vorschau}

**Leertaste öffnet**: **Schnellansicht (macOS)** oder **Integrierte Vorschau**. Dateien auf Servern und in Archiven verwenden immer die integrierte Vorschau. Siehe [Schnellansicht & Vorschau](06-schnellansicht.md).

### Kopieren und Bewegen {#kopieren-bewegen}

**Beim Ablegen von Dateien**: **Nachfragen (Kopieren vorausgewählt)** (Standard), **Immer kopieren** oder **Immer bewegen**. Gilt fürs Ziehen zwischen Bereichen und aus anderen Apps. „Nachfragen“ zeigt den Dialog mit Kopieren / Bewegen; die beiden „Immer“-Einstellungen starten sofort. **⌘** beim Ablegen bewegt, **⌥** kopiert, egal was eingestellt ist. Ablegen auf Papierkorb und Favoriten ist nicht betroffen. Siehe [Drag & Drop](05-dateien.md#drag-drop).

**Wenn ein Objekt schon existiert**: **Nachfragen** (Standard), **Überspringen**, **Ersetzen, wenn neuer**, **Ersetzen** oder **Beide behalten**. Das ist die Vorauswahl im Kopieren- und Bewegen-Dialog (auch beim Ablegen); du kannst sie jedes Mal ändern. Startet ein Ablegen ohne Dialog („Immer kopieren“ / „Immer bewegen“), gilt diese Einstellung. Mit **Nachfragen** hält ein Vorgang bei jedem schon vorhandenen Objekt an und fragt dich. Neue Installationen fragen nach; wer Manta schon benutzt hat, behält seine gespeicherte Einstellung (auch den früheren Standard **Überspringen**) und stellt hier um. Siehe [Namenskonflikte](05-dateien.md#konflikte).

### Cloud {#cloud}

**Google Drive**: Zeigt, ob Google Drive für den Desktop eingerichtet ist, sonst den Link zum Herunterladen. **Hinweis in der Seitenleiste** schaltet die Zeile „Google Drive verbinden …“ ein oder aus. **Experte: eigener Google-OAuth-Client** ist nur für Fortgeschrittene, siehe [Google Drive für Experten](09-cloud.md#experte).

### Updates {#updates}

| Einstellung | Bedeutung |
|---|---|
| **Beim Start nach Updates suchen** | Leise im Hintergrund. Installiert wird nur nach deiner Bestätigung. Standard: an. |
| **Kanal** | **Stabil** oder **Beta**. Beta bringt neue Funktionen früher, kann aber Fehler enthalten. |
| **Jetzt suchen** | Sucht sofort. Daneben: „Zuletzt gesucht: …“. |
| **Übersprungen: Manta …** / **Zurücksetzen** | Erscheint, wenn du eine Version übersprungen hast. Diese und ältere Versionen werden nicht angeboten, bis du **Zurücksetzen** klickst. |

Fehlt in deinem Build der Signaturschlüssel für Updates, steht hier „Updates sind in diesem Build nicht eingerichtet“. Siehe [Updates](17-updates.md).

### Über Manta {#ueber}

Manta ist kostenlos und Open Source. **Website & Support** → **manta-filemanager.com** öffnet [die Website](https://manta-filemanager.com/) mit Neuigkeiten, Hilfe und Kontakt. **Lizenzen von Drittanbietern** → **Anzeigen** öffnet die Lizenzhinweise der Open-Source-Komponenten, auf denen Manta aufbaut. **Quellcode** → **GitHub** öffnet [das Repository](https://github.com/web-workers-com/manta); Beiträge sind willkommen. **MIT-Lizenz** → **Lesen** zeigt die Lizenz: Du darfst Manta frei nutzen, ändern und weitergeben. Website, Quellcode und Lizenzen von Drittanbietern stehen auch im Menü **Hilfe**. **Rückmeldung** → **Fehler melden** öffnet das Formular für Fehlerberichte auf GitHub, mit Manta- und macOS-Version, siehe [Fehler melden](19-fehlerbehebung.md#fehler-melden).

## Plugins {#plugins}

Oben: **Plugin installieren …**, **Plugin-Ordner öffnen** und **Neu laden**. Darunter drei Abschnitte:

- **Integriert** – die eingebauten Plugins (Git, Terminal, Ordnergrößen, Stapelweise umbenennen, Markdown-Vorschau, Schnellaktionen, Projektinfo, Bildwerkzeuge, Duplikate, Systemauslastung).
- **Installiert** – externe Plugins, die du installiert hast.
- **Verfügbar** – mit Manta gelieferte Plugins zum Installieren.

Jede Karte zeigt Name, Beschreibung, Status (**Aktiv**, **Wird gestartet …**, **Aus**, **Fehler**) und die Tasten **Einschalten**/**Ausschalten**, **Einstellungen** (wenn vorhanden) und **Protokoll einblenden**. Mehr in [Eigene Plugins](15-eigene-plugins.md).

### Plugin-Einstellungen {#plugin-einstellungen}

| Plugin | Einstellungen |
|---|---|
| **Terminal** | Shell, Schriftgröße, Verlauf, Wahltaste als Meta-Taste verwenden, Bereich folgt „cd“ im Terminal, Dateiaktionen im Terminal zeigen – siehe [Terminal](11-terminal.md#einstellungen). |
| **Ordnergrößen** | **Ordnergrößen automatisch berechnen** oder **Nur bei Bedarf** – siehe [Ordnergrößen](14-plugins.md#ordnergroessen). |
| **Markdown-Vorschau** | **Bilder aus dem Internet laden (http/https)** – standardmäßig aus. |
| **Duplikate** | **Pakete überspringen (.app, .photoslibrary, …)**, **Unsichtbare Dateien einbeziehen**, **settings.json bearbeiten …** für Ausschlüsse und einen eigenen Papierkorb-Befehl. |
| **Systemauslastung** | **GPU-Last anzeigen** – siehe [Systemauslastung](14-plugins.md#systemauslastung). |

## Bedienungshilfen {#bedienungshilfen}

Manta folgt den Einstellungen unter **Systemeinstellungen › Bedienungshilfen** – ohne eigene Schalter:

- **VoiceOver**: Alle Tasten, Felder, Listen und Dialoge haben Namen in deiner Sprache. Dateizeilen werden vollständig vorgelesen, z. B. „README.md, Textdatei, 214 Byte, geändert Heute 09:41, ausgewählt“ – samt Cloud- und Git-Status. Beim Bewegen mit den Pfeiltasten liest VoiceOver die aktuelle Zeile. Meldungen wie „3 Objekte kopiert“ oder Fehler werden angesagt.
- **Bewegung reduzieren** (Anzeige): keine Animationen – das Aufleuchten eines Favoriten, das Pulsieren des Upload-Symbols und Einblendungen entfallen.
- **Kontrast erhöhen** (Anzeige): kräftigere Rahmen und Trennlinien, dunklerer (im dunklen Modus hellerer) Zweittext, deutlicherer Fokusring.
- **Transparenz reduzieren** (Anzeige): keine durchscheinenden Abdunklungen hinter Dialogen.

Manta übernimmt Änderungen sofort, ohne Neustart. Auch ohne diese Einstellungen erreicht Text mindestens ein Kontrastverhältnis von 4,5 : 1. Zur Bedienung ohne Maus siehe [Tastenkürzel](18-tastenkuerzel.md#tastatur).

## Wo Manta Einstellungen speichert {#dateien-speicherort}

Alles liegt in `~/Library/Application Support/Manta/`:

| Datei | Inhalt |
|---|---|
| `settings.json` | Allgemeine Einstellungen |
| `favorites.json` | Favoriten und gespeicherte Verbindungen (ohne Passwörter – die liegen im Schlüsselbund) |
| `session.json` | Bereiche, Tabs und Breiten vom letzten Beenden |
| `plugins.json` | Welche Plugins an sind, Panel-Zustände |
| `plugin-data/` | Daten der Plugins, z. B. `actions.json` |
| `update-state.json` | Update-Kanal, letzte Suche, übersprungene Version |
| `default-viewer.json` | Standard-Dateimanager und vorherige App |
| `folder-views.json` | Gemerkte Ansicht je Ordner (bei **Ansicht pro Ordner merken**) |
| `transfers.json` | Unterbrochene Übertragungen zum Fortsetzen (ohne Passwörter), siehe [Übertragungen](05-dateien.md#neustart) |

Vorschaubilder liegen getrennt davon in `~/Library/Caches/Manta/thumbs` und dürfen jederzeit gelöscht werden.

> [!TIP]
> Um Manta auf einen anderen Mac umzuziehen, kopiere diesen Ordner, während Manta beendet ist.
