# Ansichten {#ansichten}

Wie im Finder kann jeder Bereich seine Objekte als **Symbole**, als **Liste**, in **Spalten** oder als **Galerie** zeigen. Die Ansicht wechselst du mit **⌘1** bis **⌘4** oder mit dem Umschalter in der Pfadleiste.

![Die vier Ansichten im Vergleich](images/views-overview.png)

## So wechselst du die Ansicht {#wechseln}

| Ansicht | Kürzel | Befehl | Gut für … |
|---|---|---|---|
| Symbole | **⌘1** | **Als Symbole** | Bilder und Ordner mit wenigen Objekten auf einen Blick |
| Liste | **⌘2** | **Als Liste** | Sortieren nach Größe, Datum, Zugriffsrechten |
| Spalten | **⌘3** | **Als Spalten** | Durch tiefe Ordnerstrukturen klicken und dabei den Weg sehen |
| Galerie | **⌘4** | **Als Galerie** | Bilder, Videos und Dokumente groß ansehen |

Es gibt drei Wege:

- **Tastatur**: **⌘1** bis **⌘4** wirken auf den aktiven Bereich.
- **Pfadleiste**: Rechts in der Pfadleiste jedes Bereichs sitzt der Umschalter **Darstellung** mit vier Symbolen. Ist der Bereich schmal, wird daraus eine einzelne Taste mit Menü.
- **Befehlspalette** (**⇧⌘P**): Tippe z. B. „spalten“ oder „galerie“. Die Befehle stehen in der Gruppe **Darstellung**.

Die Ansicht gehört zum **Bereich**: Alle Tabs eines Bereichs teilen sie, und Manta merkt sie sich bis zum nächsten Start. Zwei Bereiche können verschiedene Ansichten zeigen, z. B. links Spalten, rechts Liste.

In allen Ansichten funktionieren Auswahl (Klick, **⌘**-Klick, **⇧**-Klick, **⌘A**), Tippen zum Auswählen, Schnellansicht, Umbenennen, Kontextmenü, Drag & Drop und die Abzeichen der Plugins gleich.

## Darstellungsoptionen {#optionen}

In der Symbol-, Spalten- und Galerieansicht steht neben dem Umschalter die Taste **Darstellungsoptionen** (Regler-Symbol). Sie öffnet ein kleines Menü:

- **Sortieren nach**: **Name**, **Größe**, **Änderungsdatum** oder **Zugriffsrechte**, dazu **Aufsteigend** oder **Absteigend**.
- **Symbolgröße** (nur Symbolansicht): ein Regler von 48 bis 256 Punkt.

![Umschalter „Darstellung“ und Darstellungsoptionen in der Pfadleiste](images/view-options.png)

In der Listenansicht gibt es die Taste nicht – dort sortierst du über die Spaltenüberschriften. Bei einem schmalen Bereich stehen die Optionen im Menü des Umschalters.

## Die Listenansicht {#liste}

Die Liste zeigt die Spalten **Name**, **Größe**, **Änderungsdatum** und **Zugriffsrechte**.

- **Sortieren**: Klicke auf eine Spaltenüberschrift. Ein zweiter Klick kehrt die Reihenfolge um. Ordner stehen immer oben; Apps und andere Pakete sortiert Manta wie Dateien, wie der Finder.
- **Spaltenbreite**: Zieh am linken Rand einer Spalte.
- **Ordnergrößen**: Mit dem Plugin [Ordnergrößen](14-plugins.md#ordnergroessen) zeigt die Spalte **Größe** auch für Ordner die echte Größe.

![Listenansicht mit sortierter Größenspalte](images/view-list.png)

## Die Symbolansicht {#symbole}

Die Symbolansicht zeigt ein Raster aus großen Symbolen und Vorschaubildern mit dem Namen darunter.

- **Größe ändern**: mit dem Regler in den **Darstellungsoptionen**, oder halte **⌘** gedrückt und scrolle bzw. zieh auf dem Trackpad zwei Finger auseinander oder zusammen. Jeder Bereich hat seine eigene Symbolgröße.
- **Mehrere auswählen**: Zieh auf einer freien Stelle ein Auswahlrechteck auf. Mit **⌘** oder **⇧** bleibt die bisherige Auswahl erhalten.
- **Tastatur**: ← → ↑ ↓ bewegen die Auswahl im Raster, **Bild ↑**/**Bild ↓** seitenweise.

## Die Spaltenansicht {#spalten}

In der Spaltenansicht siehst du jeden Ordner des Pfads als eigene Spalte. Wählst du einen Ordner aus, zeigt die Spalte rechts daneben seinen Inhalt.

- **Tastatur**: ↑ ↓ wählen in der aktuellen Spalte, **→** öffnet den ausgewählten Ordner als nächste Spalte, **←** geht zurück in die Spalte links davon.
- **Vorschau-Spalte**: Wählst du eine Datei aus, zeigt die letzte Spalte eine Vorschau mit Vorschaubild oder Symbol und den **Informationen**: **Art**, **Größe**, **Geändert**, **Zugriffsrechte** und bei Aliassen das **Originalobjekt**. Bei mehreren ausgewählten Objekten steht dort z. B. „3 Objekte ausgewählt“.
- **Spaltenbreite**: Zieh am rechten Rand einer Spalte (am Griff unten). Ein Doppelklick auf den Rand setzt die Breite zurück.
- Leere Ordner zeigen „Keine Objekte“. Sind die Spalten breiter als der Bereich, scrollt die Ansicht zur Seite und hält die aktive Spalte sichtbar.

Gehst du mit **⌘↑** über die erste Spalte hinaus oder springst du über die Seitenleiste oder die Pfadleiste in einen Ordner außerhalb der angezeigten Spalten, beginnen die Spalten neu bei diesem Ordner.

![Spaltenansicht](images/view-columns.png)

## Die Galerieansicht {#galerie}

Die Galerie zeigt das ausgewählte Objekt groß, darunter seinen Namen und eine Zeile mit Art, Größe und Änderungsdatum. Unten läuft ein **Filmstreifen** mit allen Objekten des Ordners.

- Klicke im Filmstreifen auf ein Objekt oder blättere mit ← →, um es groß zu sehen. **Bild ↑**/**Bild ↓** springen um eine Streifenbreite.
- Ohne Auswahl zeigt die Galerie das erste Objekt.
- Kleine Bilder werden höchstens auf die doppelte Größe vergrößert, damit sie nicht verpixeln.

![Galerieansicht mit Filmstreifen](images/view-gallery.png)

## Vorschaubilder {#vorschaubilder}

In der Symbol-, Spalten- und Galerieansicht zeigt Manta Vorschaubilder (Miniaturen). Sie entstehen im Hintergrund, die Ansicht bleibt dabei bedienbar; bis ein Bild fertig ist, siehst du das Symbol des Dateityps. Ohne Vorschaubild zeigen Dateien – auch in der Listen- und Spaltenansicht – das Symbol, das macOS für ihren Typ hat, Apps (`.app`) ihr eigenes, wie im Finder (abschaltbar unter **Einstellungen**: **Dateisymbole von macOS**).

- **Bilder** (PNG, JPEG, GIF, BMP, WebP) liest Manta selbst und dreht sie nach der Kamera-Ausrichtung (EXIF).
- **PDFs, Videos, HEIC, Dokumente** usw. liefert macOS (wie in der Schnellansicht). Dateien ohne Vorschau behalten ihr Typsymbol.
- Fertige Vorschaubilder speichert Manta in `~/Library/Caches/Manta/thumbs`. Ändert sich eine Datei, entsteht ein neues Bild. Den Ordner darfst du jederzeit löschen.

### Vorschaubilder für Server-Dateien {#server-vorschaubilder}

Für Dateien **auf Servern** (SFTP, FTP, S3, Container) zeigt Manta standardmäßig nur das Symbol des Dateityps. Willst du auch dort Vorschaubilder, schalte unter **Einstellungen** → **Allgemein** → **Dateien** die Option **Vorschaubilder für Server-Dateien laden** ein. Manta lädt dann kleine Bilder (unter 5 MB) im Hintergrund herunter. Größere Dateien behalten ihr Symbol.

> [!NOTE]
> Server-Vorschaubilder kosten Datenvolumen und etwas Zeit. Bei langsamen oder getakteten Verbindungen lässt du die Option besser aus.

## Ansicht pro Ordner merken {#pro-ordner}

Standardmäßig gehört die Ansicht zum **Bereich**: Wechselst du den Ordner, bleibt die Ansicht gleich.

Wenn du unter **Einstellungen** → **Allgemein** → **Dateien** die Option **Ansicht pro Ordner merken** einschaltest (Standard: aus), öffnet sich jeder Ordner in der Ansicht, in der du ihn zuletzt gesehen hast – z. B. dein Fotoordner immer als Galerie, dein Projektordner als Liste. Das gilt auch für Ordner auf Servern und in Archiven. Ordner, für die du noch keine Ansicht gewählt hast, behalten die aktuelle Ansicht des Bereichs.

Manta merkt sich die letzten 2000 Ordner in `~/Library/Application Support/Manta/folder-views.json`.

## Siehe auch

- [Schnellansicht & Vorschau](06-schnellansicht.md)
- [Die Pfadleiste](04-navigation.md#pfadleiste)
- [Einstellungen](16-einstellungen.md#dateien)
- [Tastenkürzel](18-tastenkuerzel.md#ansichten)
