# Navigation {#navigation}

Dieses Kapitel zeigt, wie du dich in Ordnern bewegst: mit der Pfadleiste, mit **Gehe zum Pfad …**, mit dem Verlauf und mit der Tastatur. Am Ende geht es um Filtern, Tippen zum Auswählen und unsichtbare Dateien.

## Ordner und Dateien öffnen {#oeffnen}

- **Doppelklick** oder **↩** öffnet einen Ordner im Bereich oder eine Datei mit ihrer Standard-App.
- **⌘↓** tut dasselbe wie ↩.
- **⌥↩** (**Mit Standard-App öffnen**) öffnet immer mit der Standard-App, auch Archive, die Manta sonst als Ordner öffnen würde (siehe [Archive](10-archive.md)).
- **⌘↑** oder **⌫** (Rückschritttaste) geht zum **übergeordneten Ordner**.

Dateien auf einem Server lädt Manta zum Öffnen in einen temporären Ordner und öffnet dann diese Kopie („‚Name‘ wird zum Öffnen geladen …“).

> [!WARNING]
> Änderungen an einer so geöffneten Server-Datei landen nur in der temporären Kopie, nicht auf dem Server. Willst du eine Datei bearbeiten, kopiere sie in einen lokalen Ordner, bearbeite sie dort und kopiere sie zurück.

## Die Pfadleiste {#pfadleiste}

Oben in jedem Bereich zeigt die Pfadleiste, wo du gerade bist, z. B. `Benutzerordner › Projekte › manta`.

![Pfadleiste mit Ort-Chip, Pfad und Plugin-Chips](images/path-bar.png)

- **Klick auf einen Ordnernamen** springt dorthin.
- **Klick auf „…“** am Anfang zeigt die übergeordneten Ordner („Übergeordnete Ordner anzeigen“), wenn der Pfad gekürzt ist.
- **Doppelklick auf eine leere Stelle** der Pfadleiste (oder **⌘L**, **Pfad bearbeiten**) macht daraus ein Textfeld. Tippe einen Pfad und drücke ↩. **esc** bricht ab.
- **Ort-Chip**: Der Chip ganz links zeigt, wo du bist: **Lokal** auf deinem Mac, z. B. **Docker · shop-web**, **Kubernetes · Pod**, **SFTP · deploy@web-01**, **S3 · Bucket**, **FTP · Host**, **Archiv · backup.zip** – und in Cloud-Ordnern den Anbieter (**Google Drive**, **iCloud Drive**, **OneDrive**, **Dropbox**; das Konto steht im Tooltip). Lange Namen werden mit „…“ gekürzt, in schmalen Bereichen bleibt nur das Symbol; der volle Text steht im Tooltip. Ein Klick springt zum obersten Ordner dieses Orts.
- **Darstellung**: Rechts, vor dem Filterfeld, wechselst du zwischen Symbolen, Liste, Spalten und Galerie; daneben liegen die **Darstellungsoptionen**. Siehe [Ansichten](03-ansichten.md#wechseln).
- **Plugin-Chips** rechts zeigen z. B. den Git-Branch oder das erkannte Projekt. Wird der Bereich schmal, werden sie kürzer und am Ende zum Symbol; der volle Text steht im Tooltip.

### Zurück zu Lokal {#lokal}

Bist du nicht in einem normalen Ordner deines Macs (Server, Container, Bucket, Archiv oder Cloud-Ordner), steht neben dem Ort-Chip ein Pfeil **Zurück zu Lokal**. Er bringt dich zum letzten Ordner auf deinem Mac aus dem Verlauf des Tabs – Cloud-Ordner und Archive werden dabei übersprungen –, sonst zum Benutzerordner. Die Verbindung bleibt bestehen.

## Gehe zum Pfad {#gehe-zum-pfad}

Mit **Gehe zum Pfad …** (**⇧⌘G**) öffnest du einen Ordner, dessen Pfad du kennst oder in der Zwischenablage hast.

1. Drücke **⇧⌘G**.
2. Tippe oder füge einen Pfad ein. Manta versteht `~/Downloads`, relative Pfade, Pfade in Anführungszeichen und `file://`-Adressen.
3. **⇥** ergänzt Ordnernamen. Unter dem Feld siehst du **Passende Ordner** und **Zuletzt benutzt**.
4. Drücke **↩**, um den Ordner in einem **neuen Tab** zu öffnen, oder **⌘↩**, um ihn **im aktuellen Tab** zu öffnen.

Manta prüft den Pfad im Hintergrund und meldet z. B. „Ordner nicht gefunden“ oder „Zugriff verweigert“. Ist der Pfad eine **Datei**, öffnet Manta ihren Ordner und wählt die Datei aus.

Zeigt der aktive Bereich einen Server, gilt der Pfad auf diesem Server („Öffnet in neuem Tab auf …“).

**⇧⌘G** funktioniert überall, auch während du in einem Textfeld tippst.

![Gehe zum Pfad mit Vorschlägen](images/go-to-path.png)

## Zurück und vorwärts {#verlauf}

Jeder Tab hat seinen eigenen Verlauf.

- **⌘[** oder **⌘←**: **Zurück**
- **⌘]** oder **⌘→**: **Vorwärts**
- Die **Daumentasten der Maus** (Taste 4 und 5): **Zurück** und **Vorwärts** im Bereich unter dem Zeiger. Vertauschen oder abschalten unter [Einstellungen → Dateien](16-einstellungen.md#dateien).

Der Verlauf reicht auch in Archive hinein und wieder heraus.

## Mit der Tastatur auswählen {#auswaehlen}

| Taste | Wirkung |
|---|---|
| ↑ / ↓ | Vorheriges / nächstes Objekt |
| ⇧↑ / ⇧↓ | Auswahl erweitern |
| Bild ↑ / Bild ↓ | 20 Zeilen weiter |
| Pos1 / Ende | Erstes / letztes Objekt |
| ⌘A | Alles auswählen |
| esc | Auswahl aufheben |

Mit der Maus: Klick wählt ein Objekt, **⌘-Klick** fügt eines hinzu oder entfernt es, **⇧-Klick** wählt einen Bereich. Ein Klick auf eine leere Stelle hebt die Auswahl auf.

## Tippen zum Auswählen {#tippen}

Wie im Finder: Tippe die ersten Buchstaben eines Namens, und Manta springt zum ersten passenden Objekt. Groß- und Kleinschreibung spielen keine Rolle. Gibt es keinen Namen mit diesem Anfang, springt Manta zum nächsten Namen in der Sortierung.

Tippst du etwa eine Sekunde lang nichts, beginnt die Suche von vorn. Solange du tippst, gehört die Leertaste zum Namen (z. B. „mein ord“) und öffnet nicht die Schnellansicht.

## Dateien filtern {#filtern}

Der Filter zeigt nur Objekte, deren Name einen Text enthält.

1. Drücke **⌘F** (**Dateien filtern**) oder klick in das Feld **Filtern** in der Pfadleiste.
2. Tippe einen Teil des Namens, z. B. `.pdf` oder `rechnung`.

Der Filter gilt nur für den aktuellen Ordner und nur für diesen Tab. Die Statusleiste zeigt „passend zu ‚…‘“. Passt nichts, siehst du „Keine Objekte passen zu ‚…‘“ mit der Taste **Filter löschen**. **esc** im Feld leert den Filter. In schmaleren Bereichen (etwa zwei Bereiche in einem kleinen Fenster) ist das Feld eine Lupe: Ein Klick darauf oder **⌘F** öffnet es; es bleibt offen, solange darin Text steht, und klappt wieder zu, wenn es leer ist und du woanders hinklickst oder **esc** drückst. Damit bleibt Platz für den Ort-Chip und die Pfadleiste; die Darstellungs-Tasten werden dabei, solange das Feld offen ist, zu einer einzigen Menü-Taste.

Nach [Tags](05-dateien.md#tags) filterst du mit `tag:` und dem Namen des Tags, z. B. `tag:Rot` oder `tag:"Sehr wichtig"`; der Name der Farbe geht auch (`tag:rot` findet auch einen roten Tag namens „Dringend“). Das lässt sich mit einem Namen kombinieren: `tag:Rot rechnung`.

## Ordnerübergreifend suchen {#suchen}

Der Filter sieht nur den Ordner, der gerade zu sehen ist. Um darunter oder auf dem ganzen Mac zu suchen, gibt es die Bereichsleiste, die unter der Pfadleiste erscheint, solange du das Feld benutzt:

- **Dieser Ordner** – der oben beschriebene Filter.
- **Unterordner** – der Ordner und alles darin. Auf diesem Mac liest Manta mehrere Ordner gleichzeitig; unsichtbare Objekte werden nur durchsucht, wenn unsichtbare Dateien eingeblendet sind, und der Inhalt von Paketen (Programme, Fotomediatheken) wird übersprungen. Auf Servern, in Containern und in Archiven listet Manta die Ordner nacheinander auf, höchstens 12 Ebenen tief und höchstens 5.000 Ordner.
- **Dieser Mac** – Spotlight: Namen *und* Dateiinhalte, überall, wo Spotlight indiziert (nur lokale Ordner).

1. Drücke **⌘F** und tippe, was du suchst.
2. Klick auf **Unterordner** oder **Dieser Mac**. Die Suche startet kurz nachdem du aufhörst zu tippen (oder sofort mit **↩**).

Ein Name passt, wenn er den Text enthält (Groß- und Kleinschreibung egal). Mit `*` oder `?` ist der Text ein Muster für den ganzen Namen: `*.pdf`, `bericht-202?.xlsx`. `tag:` funktioniert auch hier.

Die Treffer füllen den Bereich als Liste, schon während die Suche läuft: Die Pfadleiste zeigt **Suche: „…“** und wo gesucht wird, die Bereichsleiste, wie viele Objekte gefunden wurden, und die Spalte **Pfad** den Ordner jedes Treffers. Treffer lassen sich wie gewohnt öffnen, in der Schnellansicht ansehen, kopieren, bewegen, umbenennen, taggen und löschen. **Im umgebenden Ordner anzeigen** im Kontextmenü (oder **⌘↑**) öffnet den Ordner des Objekts und wählt es dort aus. Die Stopp-Taste in der Bereichsleiste beendet eine lange Suche (die bisherigen Treffer bleiben); **⌘R** sucht erneut. Angezeigt werden höchstens 10.000 Treffer.

**esc** im Suchfeld, **⌘[** oder ein leeres Feld beendet die Suche und zeigt wieder den Ordner; das Öffnen eines anderen Ordners ebenfalls.

Findet **Unterordner** nichts, bietet der leere Bereich **Diesen Mac durchsuchen** an.

## Unsichtbare Dateien {#unsichtbare-dateien}

Dateien, deren Name mit einem Punkt beginnt (z. B. `.git`, `.env`), und als unsichtbar markierte Dateien blendet Manta normalerweise aus. Nur `.DS_Store` zeigt Manta nie, auch nicht eingeblendet – wie der Finder. macOS legt diese Datei in fast jedem Ordner neu an.

- **⇧⌘.** (**Unsichtbare Dateien einblenden**) blendet sie im aktuellen Tab ein oder aus. Dasselbe tut die Taste mit der gestrichelten Datei in der Symbolleiste (sie ist hervorgehoben, solange unsichtbare Dateien zu sehen sind). Das Auge daneben ist die Schnellansicht.
- Unter **Einstellungen** → **Allgemein** → **Unsichtbare Dateien einblenden** legst du fest, ob neue Tabs und Manta beim Start sie zeigen.

Ist ein Ordner scheinbar leer, enthält aber unsichtbare Objekte, sagt Manta das: „3 unsichtbare Objekte – drücke ⇧⌘. zum Einblenden.“

## Aktualisieren {#aktualisieren}

Lokale Ordner bleiben von selbst aktuell – wie im Finder: Legt eine andere App eine Datei an, löscht, benennt oder ändert sie, erscheint die Änderung nach einem kurzen Moment im Bereich. Auswahl, Cursor, Scrollposition und die Tipp-Auswahl bleiben dabei erhalten. Manta beobachtet die Ordner aller offenen Tabs (in der Spaltenansicht auch die Spalten links davon). Ein Tab im Hintergrund wird neu eingelesen, sobald du zu ihm wechselst – sofort, ohne ⌘R. Viele Änderungen auf einmal – etwa beim Kopieren Tausender Dateien – fasst Manta zusammen und liest den Ordner höchstens etwa einmal pro Sekunde neu ein. Während du einen Namen eingibst (Umbenennen, Neuer Ordner), wartet die Aktualisierung, bis du fertig bist.

Ordner auf **Servern**, in **Containern**, in **S3** und in **Archiven** werden nicht beobachtet. Ändert dort jemand anderes etwas, drücke **⌘R** (**Aktualisieren**), um den Ordner neu einzulesen. ⌘R funktioniert natürlich auch lokal.

## Siehe auch

- [Tags und Informationen](05-dateien.md#tags)
- [Favoriten & Seitenleiste](07-favoriten-seitenleiste.md)
- [Tastenkürzel](18-tastenkuerzel.md#navigation)
