# Dateien verwalten {#dateien}

Kopieren, Bewegen, Umbenennen, Löschen und neue Ordner – lokal, auf Servern, in Containern und zwischen all diesen Orten. Lies vor allem den Abschnitt [Löschen](#loeschen): Auf deinem Mac landet Gelöschtes im Papierkorb, auf Servern ist Löschen endgültig.

## So kopierst du Dateien in einen anderen Bereich {#kopieren}

1. Öffne im **Zielbereich** den Ordner, in den die Dateien sollen.
2. Wähle im **aktiven Bereich** die Dateien und Ordner aus.
3. Drücke **F5** (**In anderen Bereich kopieren …**).
4. Prüfe im Dialog, was wohin geht, und wähle, was bei gleichen Namen passieren soll (siehe [Namenskonflikte](#konflikte)).
5. Klicke auf **Kopieren**.

Das Kopieren läuft im Hintergrund weiter. Du kannst sofort weiterarbeiten.

![Kopieren-Dialog mit Ziel und Konfliktwahl](images/copy-dialog.png)

**Bewegen** funktioniert genauso mit **F6** (**In anderen Bereich bewegen …**). Beim Bewegen verschwinden die Objekte am alten Ort.

Du findest beide Befehle auch in der Symbolleiste („In rechten Bereich kopieren“, „In linken Bereich bewegen“ …) und im Kontextmenü (Rechtsklick).

### Welcher Bereich ist das Ziel? {#ziel}

Bei zwei Bereichen ist es der andere. Bei drei oder mehr Bereichen ist es der zuletzt aktive andere Bereich (mit **→**-Abzeichen, siehe [Zielbereich](02-bereiche-tabs.md#zielbereich)). Im Dialog kannst du unter **Nach** jeden anderen Bereich wählen.

Manta verhindert typische Fehler:

- „Beide Bereiche zeigen denselben Ordner – öffne einen anderen Ordner im anderen Bereich“
- „Ein Ordner kann nicht in sich selbst kopiert werden“

### Zwischen verschiedenen Orten {#orte}

Kopieren und Bewegen funktionieren zwischen allen Orten: von deinem Mac auf einen SFTP-Server, von einem S3-Bucket in einen Docker-Container, aus einem Archiv auf den Schreibtisch. Manta erledigt das selbst; du brauchst keine Zusatzprogramme.

Aus **schreibgeschützten** Orten (Archive, gestoppte Container) wird beim Bewegen nur kopiert. Manta sagt dir das, z. B. „Archive sind schreibgeschützt: Die Objekte werden entpackt (kopiert), nicht bewegt“.

## Namenskonflikte {#konflikte}

Gibt es am Ziel schon ein Objekt mit gleichem Namen, entscheidet die Einstellung **Wenn ein Objekt schon existiert** im Dialog:

| Option | Was passiert |
|---|---|
| **Nachfragen** | Manta hält bei jedem Objekt an, das es schon gibt, und fragt dich (Standard). |
| **Ersetzen** | Das vorhandene Objekt wird immer überschrieben. |
| **Überspringen** | Vorhandene Objekte bleiben unverändert. |
| **Ersetzen, wenn neuer** | Nur überschreiben, wenn die Quelle neuer ist oder eine andere Größe hat. |
| **Beide behalten** | Die Kopie bekommt einen neuen Namen („Name 2“). |

Die anderen Optionen gelten für den ganzen Vorgang, ohne Rückfrage. Die Vorauswahl stellst du unter **Einstellungen** → **Allgemein** → **Kopieren und Bewegen** ein.

### Nachfragen {#nachfragen}

Mit **Nachfragen** hält der Vorgang beim ersten Objekt an, das es am Ziel schon gibt. Manta zeigt „‚Name‘ ist bereits in ‚Ordner‘ vorhanden“ mit beiden Objekten – Größe, Änderungsdatum und einem Hinweis **Neuer** beim jüngeren – und du wählst:

| Taste | Was passiert |
|---|---|
| **Beide behalten** | Die Kopie bekommt einen neuen Namen („Name 2“). Das ist die Vorauswahl: **↩** wählt sie. |
| **Ersetzen** | Das vorhandene Objekt wird überschrieben. |
| **Ersetzen, wenn neuer** | Nur überschreiben, wenn die Quelle neuer ist; sonst bleibt das vorhandene. |
| **Überspringen** | Das vorhandene Objekt bleibt, dieses wird nicht übertragen (beim Bewegen bleibt es am Ursprung). |
| **Abbrechen** (**esc**) | Der Vorgang stoppt. Was schon übertragen ist, bleibt am Ziel. |

Gibt es am Ziel schon einen **Ordner** mit dem Namen, bietet Manta **Zusammenführen** an (wie der Finder): Der Inhalt kommt in den vorhandenen Ordner, Dateien, die es dort schon gibt, werden einzeln erfragt. Daneben **Beide behalten** („photos 2“) und **Überspringen**. Unterordner werden immer zusammengeführt.

Mit **Für alle weiteren Konflikte dieses Auftrags übernehmen** gilt die Antwort für den Rest des Vorgangs, Manta fragt dann nicht mehr. Fragen mehrere Vorgänge gleichzeitig, kommen sie nacheinander dran. Andere Übertragungen laufen weiter, während einer wartet; in den Übertragungen steht er als „Wartet auf deine Entscheidung“. Beendest du Manta, während eine Frage offen ist, fragt der Vorgang nach dem Fortsetzen erneut.

## Übertragungen beobachten {#uebertragungen}

Mit **⌥⌘L** (**Übertragungen einblenden**) oder der Taste **Übertragungen** in der Symbolleiste siehst du alle laufenden Vorgänge: Kopieren, Bewegen, Löschen, Entpacken und Komprimieren.

- Jeder Vorgang zeigt Fortschritt, Geschwindigkeit und Restzeit.
- **Pausieren** hält einen Vorgang an, **Fortsetzen** lässt ihn weiterlaufen (siehe unten).
- **Stoppen** bricht einen Vorgang ab. Halb kopierte Dateien werden dabei entfernt.
- **Erledigte entfernen** räumt die Liste auf.
- Ging etwas schief, zeigt der Vorgang „N Probleme“. Die Meldung „Mit Problemen abgeschlossen“ verweist ebenfalls hierher.

Blendest du den Bereich aus, zeigt die Statusleiste unten rechts eine kleine Plakette mit Fortschrittsring – so siehst du immer, wie weit ein Vorgang ist, auch beim Ablegen von Dateien aus anderen Apps oder beim Kopieren in Cloud-Ordner:

- ein Vorgang: „Kopiere 3 Objekte … 42 %“, „Bewege „Urlaub.mov“ … 80 %“,
- mehrere: „2 Übertragungen · 37 %“,
- angehalten oder wartend: „… · pausiert“, „… · wartet“, „… · wartet auf Verbindung“,
- danach einige Sekunden **Fertig** (bzw. **Gestoppt**),
- ging etwas schief, bleibt die rote Plakette („Übertragung fehlgeschlagen“, „Fertig mit Fehlern“) stehen, bis du sie anklickst.

Ein Klick auf die Plakette öffnet die Übertragungen; der Tooltip nennt jeden laufenden Vorgang mit Stand und Restzeit.

Kopierst du in einen Cloud-Ordner (Google Drive, iCloud Drive, OneDrive, Dropbox), ist die Übertragung fertig, sobald die Dateien auf deinem Mac liegen – hochladen tut danach die App des Anbieters. Das zeigt die Statusleiste ebenfalls, z. B. „Google Drive lädt 2 Objekte hoch“ oder rot „Google Drive: 1 Upload-Fehler“ (siehe [Cloud-Speicher](09-cloud.md#abzeichen)).

![Übertragungen mit einem laufenden Vorgang](images/transfers-panel.png)

## Übertragungen pausieren und fortsetzen {#pausieren}

Jeder Kopier-, Bewege- und Löschvorgang lässt sich anhalten und später an derselben Stelle fortsetzen – auch mitten in einer großen Datei.

- **Ein Vorgang**: Klicke in seiner Zeile auf das Pause-Symbol (**Pausieren**). Die Zeile zeigt dann z. B. „Pausiert · 1,2 GB von 3,0 GB“. Das Abspiel-Symbol (**Fortsetzen**) lässt ihn weiterlaufen.
- **Alle auf einmal**: Oben im Bereich **Übertragungen** stehen **Alle pausieren** und **Alle fortsetzen**, sobald es etwas zu pausieren bzw. fortzusetzen gibt.

Ein fortgesetzter Vorgang überspringt fertige Dateien und macht die angefangene Datei an der Stelle weiter, an der er aufgehört hat („Wird fortgesetzt ab 1,2 GB“).

**Komprimieren** lässt sich nicht pausieren.

### Wenn die Verbindung abbricht {#verbindung-weg}

Reißt die Netzwerkverbindung ab oder ist der Server kurz weg, bricht Manta den Vorgang nicht ab. Er **wartet** und versucht es selbst wieder:

- Die Zeile zeigt „Wartet auf Verbindung … (Versuch 3, nächster in 8 s)“. Die Abstände werden länger (1, 2, 4 … Sekunden, höchstens 1 Minute); Manta gibt nicht auf.
- Sobald dein Mac wieder online ist, geht es sofort weiter – an der unterbrochenen Stelle.
- Willst du nicht warten, klicke auf **Jetzt erneut versuchen** (Pfeil-Symbol).
- Ist dein Mac ganz ohne Netz, zeigt der Bereich **Übertragungen** oben den Hinweis **Keine Internetverbindung** – „Übertragungen laufen an der unterbrochenen Stelle weiter, sobald die Verbindung zurück ist.“

Ein wartender Vorgang lässt sich wie jeder andere pausieren oder stoppen.

![Übertragungen: ein pausierter und ein wartender Vorgang, Hinweis „Keine Internetverbindung“](images/transfers-paused-waiting.png)

### Teildateien (`.manta-part`) {#teildateien}

Während eine Datei kopiert wird, schreibt Manta sie zuerst in eine **unsichtbare Teildatei** neben dem Ziel: aus `film.mov` wird `.film.mov.manta-part`. Erst wenn die Datei vollständig ist, bekommt sie ihren richtigen Namen. So liegt im Ziel nie eine halbe Datei unter dem echten Namen.

- Teildateien siehst du nur, wenn unsichtbare Dateien eingeblendet sind (**⇧⌘.**).
- Lösche eine Teildatei nicht, solange der Vorgang pausiert ist oder wartet – sonst beginnt diese Datei von vorn.
- Beim **Stoppen** und beim **Verwerfen** (siehe unten) räumt Manta sie selbst weg.
- Uploads zu **S3** und **Google Drive** brauchen keine Teildatei: Der Server hält die angefangene Datei unsichtbar zurück, bis sie fertig ist.

### Nach dem Beenden oder einem Absturz {#neustart}

Manta merkt sich unfertige Übertragungen, auch wenn du die App beendest, sie abstürzt oder der Mac neu startet. Beim nächsten Start fragt es:

**Unterbrochene Übertragungen fortsetzen?** – z. B. „2 Übertragungen wurden beim letzten Mal nicht beendet.“ Darunter stehen die Vorgänge mit ihrem Stand und, falls nötig, welcher Server fehlt („benötigt files.example.com (SFTP)“).

- **Fortsetzen** (oder ↩): Die Vorgänge laufen dort weiter, wo sie aufgehört haben.
- **Verwerfen**: Die Vorgänge werden gelöscht, ihre Teildateien entfernt – soweit Manta den Ort erreicht. Auf einem Server ohne Verbindung bleibt eine Teildatei liegen; du kannst sie später selbst löschen.
- **esc** (Fenster schließen): Später entscheiden. Manta fragt beim nächsten Start wieder.

![Dialog „Unterbrochene Übertragungen fortsetzen?“](images/resume-prompt.png)

#### „Benötigt eine Verbindung“ {#benoetigt-verbindung}

Braucht ein fortgesetzter Vorgang einen Server, mit dem du noch nicht verbunden bist, zeigt seine Zeile „Benötigt eine Verbindung zu files.example.com (SFTP)“ und die Taste **Verbinden**.

- **Server-Favoriten** mit gemerktem Passwort (oder ssh-agent) verbinden sich von selbst im Hintergrund; du musst nichts tun.
- Sonst klicke auf **Verbinden**. Manta öffnet den Verbindungsdialog mit den Serverdaten; gib das Passwort ein. Sobald die Verbindung steht, läuft der Vorgang weiter.

Passwörter und Passphrasen speichert Manta für diesen Zweck **nie**: Die Liste der unterbrochenen Übertragungen (`transfers.json`) enthält nur Server, Benutzernamen und Pfade.

#### Hinweise „wird von vorn kopiert“ {#von-vorn}

Manchmal kann Manta eine angefangene Datei nicht sicher fortsetzen und kopiert sie neu. Der Vorgang zeigt dann „1 Hinweis“; ein Klick klappt die Hinweise auf:

| Hinweis | Was passiert ist |
|---|---|
| „Die Quelle hat sich seit der Unterbrechung geändert; wird von vorn kopiert“ | Die Originaldatei hat jetzt eine andere Größe oder ein anderes Änderungsdatum. Fortsetzen würde altes und neues mischen. |
| „Dieser Server kann Teildateien nicht fortsetzen; wird von vorn kopiert“ | Das Ziel erlaubt kein Anhängen an eine angefangene Datei. |
| „Die Teildatei war größer als die Quelle; wird von vorn kopiert“ | Die Teildatei passt nicht zur Quelle. |
| „Die Upload-Sitzung ist abgelaufen; wird von vorn hochgeladen“ | S3 oder Google Drive hat den angefangenen Upload inzwischen verworfen. |

### Grenzen {#pausieren-grenzen}

- **Komprimieren** kann nicht pausiert werden und übersteht keinen Neustart.
- Vorgänge **aus Archiven** (Entpacken) lassen sich pausieren, werden nach dem Beenden aber nicht fortgesetzt.
- Ohne passendes Passwort wartet ein Vorgang auf einem Server, bis du dich verbindest.

## Drag & Drop {#drag-drop}

### Zwischen Bereichen

1. Wähle Objekte aus und zieh sie in einen anderen Bereich.
2. Lass sie los. Manta fragt nach: Im Dialog wählst du **Kopieren** oder **Bewegen**, dazu das Ziel und was bei Namenskonflikten passiert. **Kopieren** ist vorausgewählt – auch auf demselben Volume deines Macs, wo der Finder bewegen würde. So verschwindet nichts versehentlich aus seinem Ordner. **↩** bestätigt die Vorauswahl.
3. Hältst du beim Loslassen **⌘** gedrückt, ist **Bewegen** vorausgewählt. **⌥** wählt ausdrücklich **Kopieren** (und gewinnt gegen ⌘).

Ein Hinweis am Mauszeiger zeigt, was vorausgewählt ist („3 Objekte hierher kopieren · ⌘ halten zum Bewegen“, mit ⌘: „3 Objekte hierher bewegen · ⌥ halten zum Kopieren“). Ziehst du aus einem Archiv, wird entpackt. Zwischen Bereichen landen die Objekte im angezeigten Ordner des Bereichs.

Möchtest du nicht jedes Mal gefragt werden, stell in den [Einstellungen](16-einstellungen.md#kopieren-bewegen) unter **Beim Ablegen von Dateien** „Immer kopieren“ oder „Immer bewegen“ ein. Dann startet der Vorgang ohne Dialog; **⌘** bewegt und **⌥** kopiert trotzdem für diesen einen Vorgang. Ablegen auf **Papierkorb** und **Favoriten** ist davon nicht betroffen.

### Aus Manta in andere Apps {#drag-raus}

Du kannst Dateien aus Manta in andere Apps ziehen, z. B. auf den Schreibtisch, in ein Finder-Fenster, in Mail oder in einen Browser-Upload.

- **Lokale Dateien** werden kopiert – auch wenn das Ziel auf demselben Volume liegt. Hältst du **⌘** gedrückt, werden sie bewegt.
- **Dateien von Servern, aus Containern oder Archiven** lädt Manta beim Loslassen an den Zielort herunter. Das kann bei großen Dateien dauern.

Zieh die Objekte über den Rand des Manta-Fensters hinaus; ab dort übernimmt macOS. Eine Meldung zeigt, wann ein Download vom Server beginnt und ob er geklappt hat. Hat die andere App die Dateien bewegt, aktualisiert Manta den Ordner, aus dem sie kamen.

### Aus anderen Apps in Manta {#drag-rein}

Leg Dateien aus dem Finder, vom Schreibtisch oder aus einer anderen App auf einem Bereich ab:

- Auf eine freie Stelle im Bereich: Sie landen im angezeigten Ordner.
- Auf einen Ordner (in jeder Darstellung, in der Spaltenansicht auch auf eine Spalte): Sie landen in diesem Ordner. Er wird beim Darüberziehen hervorgehoben.
- Es gelten dieselben Regeln wie zwischen Bereichen: Manta fragt mit **Kopieren** vorausgewählt (mit **⌘**: **Bewegen**), außer du hast „Immer kopieren“ oder „Immer bewegen“ eingestellt.

Der Vorgang erscheint unter „Übertragungen“, wo du es pausieren kannst. Mail-Anhänge und Bilder aus Fotos lassen sich noch nicht auf Manta ablegen.

### In die Favoriten

Zieh einen Ordner auf den Abschnitt **Favoriten** der Seitenleiste, um ihn dort abzulegen. Siehe [Favoriten & Seitenleiste](07-favoriten-seitenleiste.md#favoriten-ziehen).

### In den Papierkorb

Zieh Objekte auf **Papierkorb** in der Seitenleiste: Lokale Objekte landen ohne Rückfrage im Papierkorb, Objekte auf Servern werden nach Rückfrage endgültig gelöscht. Siehe [Papierkorb](07-favoriten-seitenleiste.md#papierkorb).

## Umbenennen {#umbenennen}

1. Wähle **ein** Objekt aus.
2. Drücke **F2** (**Umbenennen …**) oder wähle **Umbenennen …** im Kontextmenü.
3. Tippe den neuen Namen und drücke ↩.

**Per Klick umbenennen**, wie im Finder: Klicke ein bereits ausgewähltes Objekt noch einmal an und warte kurz. In der Listenansicht wird der Name direkt in der Zeile editierbar (markiert ist der Name ohne Endung); ↩ oder ein Klick daneben benennt um, Esc bricht ab. In den anderen Ansichten öffnet sich der Dialog **Umbenennen …**. Ein Doppelklick öffnet das Objekt wie gewohnt.

Manta prüft den Namen sofort: Er darf nicht leer sein, kein „/“ enthalten und nicht schon im Ordner existieren.

**Mehrere Objekte auf einmal umbenennen**: Wähle mehrere aus und drücke **F2**, oder drücke **⇧⌘R** (**Stapelweise umbenennen …**). Das Plugin bietet Regeln, eine Live-Vorschau und **Widerrufen**. Siehe [Stapelweise umbenennen](14-plugins.md#umbenennen).

## Neuer Ordner {#neuer-ordner}

1. Drücke **⇧⌘N** (**Neuer Ordner …**).
2. Tippe einen Namen (vorgeschlagen ist „Neuer Ordner“) und klicke auf **Erstellen**.

## Neue Datei {#neue-datei}

1. Drücke **⌥⌘N** (**Neue Datei …**) oder wähle **Neue Datei …** im Menü **Ablage** oder im Kontextmenü eines Bereichs.
2. Tippe einen Namen (vorgeschlagen ist „Neue Datei.txt“) und klicke auf **Erstellen**. Manta legt eine leere Datei an und wählt sie aus.

**In einem Ordner anlegen**: Klickst du mit der rechten Maustaste auf einen Ordner, heißen die Einträge **Neuer Ordner in „…“** und **Neue Datei in „…“** – Ordner bzw. Datei entstehen dann in diesem Ordner, der Bereich bleibt, wo er ist. **⇧⌘N** und **⌥⌘N** legen weiterhin im angezeigten Ordner an.

## Löschen {#loeschen}

Manta löscht wie der Finder:

| Wo | ⌘⌫ (**In den Papierkorb legen**) | ⌥⌘⌫ (**Sofort löschen …**) |
|---|---|---|
| Dein Mac (auch externe Laufwerke und Cloud-Ordner) | Legt die Objekte ohne Rückfrage in den **Papierkorb**. **⌘Z** (**Widerrufen**) oder **Zurücklegen** im Finder holt sie zurück. | Löscht nach Rückfrage endgültig. |
| Server (SFTP, FTP, FTPS), S3, Container | Es gibt keinen Papierkorb: löscht nach Rückfrage endgültig. | Löscht nach Rückfrage endgültig. |
| Archive | Schreibgeschützt: Es lässt sich nichts löschen. | Schreibgeschützt. |

So legst du Objekte in den Papierkorb:

1. Wähle die Objekte aus.
2. Drücke **⌘⌫** oder die Entfernen-Taste **⌦**, oder wähle **In den Papierkorb legen** im Kontextmenü oder in der Symbolleiste – oder zieh sie auf **Papierkorb** in der Seitenleiste.

Eine Meldung bestätigt es („3 Objekte in den Papierkorb gelegt“). Um die Objekte zurückzuholen, drücke **⌘Z** (**Widerrufen**): Manta legt sie dorthin zurück, wo sie waren, siehe [Widerrufen](#widerrufen). Ältere Objekte oder solche, die du in einer anderen App gelöscht hast, holst du im Finder zurück: Öffne dort den Papierkorb, wähle sie aus und wähle **Zurücklegen**.

Wo macOS Manta das Ablegen verweigert, obwohl der Finder es darf (etwa in iCloud Drive), legt Manta das Objekt über den Finder in den Papierkorb. macOS fragt beim ersten Mal, ob Manta den Finder steuern darf.

So löschst du endgültig:

1. Wähle die Objekte aus.
2. Drücke **⌥⌘⌫** oder wähle **Sofort löschen …** in der Befehlspalette.
3. Manta fragt nach: „‚Name‘ löschen?“ bzw. „3 Objekte löschen?“ – mit dem Hinweis, dass die Objekte nicht in den Papierkorb kommen und du das nicht widerrufen kannst.
4. Klicke auf **Löschen**.

Auf einem **Server**, in einem **S3-Bucket** oder in einem **Container** gibt es keinen Papierkorb. **⌘⌫**, das Kontextmenü (dort **Sofort löschen …**) und der Knopf in der Symbolleiste fragen immer nach, und der Dialog sagt in Rot, dass dieser Ort keinen Papierkorb hat und Löschen dort endgültig ist. Ordner werden mit ihrem gesamten Inhalt gelöscht.

Den Papierkorb selbst öffnest du über **Papierkorb** in der Seitenleiste. Darin löscht **⌘⌫** (dort **Sofort löschen …**) nach Rückfrage endgültig, und **⇧⌘⌫** (**Papierkorb entleeren …**) leert ihn ganz. Siehe [Papierkorb](07-favoriten-seitenleiste.md#papierkorb).

> [!CAUTION]
> In **Cloud-Ordnern** (iCloud Drive, Google Drive, OneDrive, Dropbox) verschwindet ein Objekt, das du in den Papierkorb legst, auch aus der Cloud, sobald dein Mac synchronisiert. **Zurücklegen** holt es meist zurück; manche Anbieter haben zusätzlich einen Papierkorb auf ihrer Website. Siehe [Cloud-Speicher](09-cloud.md).

## Widerrufen {#widerrufen}

**⌘Z** (**Widerrufen**) macht deinen letzten Dateivorgang rückgängig, wie im Finder, und **⇧⌘Z** (**Wiederholen**) führt ihn erneut aus. Beide findest du im Menü **Bearbeiten** und in der Befehlspalette. Manta merkt sich bis zu 100 Schritte, bis du es beendest.

| Was du getan hast | Was **⌘Z** macht |
|---|---|
| **In den Papierkorb legen** (**⌘⌫** oder Ziehen auf **Papierkorb** in der Seitenleiste) | Legt die Objekte dorthin zurück, wo sie waren. |
| Umbenennen (**Umbenennen …**, per Klick umbenennen, Informationsfenster) | Stellt den alten Namen wieder her, auch auf Servern. |
| **Neuer Ordner …** oder **Neue Datei …** auf deinem Mac | Legt das neue Objekt in den Papierkorb, wie der Finder, samt allem, was du inzwischen hineingelegt hast. **⇧⌘Z** holt es zurück. |
| **Neuer Ordner …** oder **Neue Datei …** auf einem Server, in S3 oder in einem Container | Es gibt keinen Papierkorb: Manta löscht das neue Objekt, aber nur, solange es noch leer ist. Sonst meldet Manta, dass es sich nicht löschen ließ, und lässt es stehen. |

Nicht widerrufen lassen sich Kopieren und Bewegen (**F5**, **F6**, Drag & Drop) und alles Endgültige: **Sofort löschen …**, Löschen auf Servern und **Papierkorb entleeren …**. **Stapelweise umbenennen …** hat ein eigenes **Widerrufen**, siehe [Stapelweise umbenennen](14-plugins.md#umbenennen).

Ist der ursprüngliche Ort inzwischen wieder belegt (dort liegt ein Objekt mit demselben Namen), ersetzt Manta nichts. Es meldet „‚Name‘ konnte nicht zurückgelegt werden“ und behält den Schritt: Sobald der Platz frei ist, funktioniert **⌘Z** wieder.

- In einem **Textfeld** (Filter, Pfad, Name beim Umbenennen) widerrufen **⌘Z** und **⇧⌘Z** deine Eingabe, nicht Dateivorgänge. Solange das **Terminal** den Fokus hat, gehört **⌘Z** dem Terminal.
- Ohne [Festplattenvollzugriff](07-favoriten-seitenleiste.md#papierkorb-zugriff) legt Manta Objekte über den Finder aus dem Papierkorb zurück, wie bei **Papierkorb entleeren …**: macOS fragt beim ersten Mal, ob Manta den Finder steuern darf.

## Tags {#tags}

Manta liest und setzt dieselben Tags wie der Finder (sie gehören zur Datei, Finder, Spotlight und andere Apps sehen sie also auch). Objekte mit Tags zeigen farbige Punkte bei ihrem Namen, in jeder Darstellung – in der Liste und in den Spalten hinter dem Namen, in Symbolen und Galerie davor. Tags gibt es für Objekte auf diesem Mac, auch in iCloud Drive und anderen Cloud-Ordnern; Server und Archive haben keine.

**Tag hinzufügen oder entfernen:**

1. Wähle die Objekte aus und öffne das Kontextmenü.
2. Wähle **Tags**. Oben stehen die sieben Farben: Ein Klick gibt allen ausgewählten Objekten den Tag oder entfernt ihn, wenn alle ihn haben (ein Häkchen heißt: alle haben ihn, ein Strich: einige). Darunter stehen deine weiteren Tags.
3. Für einen neuen Tag tippst du seinen Namen in **Neuer Tag …** und drückst **↩**.

Die Standard-Tags tragen die Namen, die der Finder auf deinem Mac verwendet (z. B. „Rot“ auf einem deutschen, „Red“ auf einem englischen Mac).

**Objekte mit Tags finden:** Der Abschnitt **Tags** der Seitenleiste zeigt deine Tags wie der Finder (kann Manta die Liste des Finders nicht lesen, die Tags, die es an Dateien gesehen hat). Ein Klick zeigt im aktiven Bereich alles mit diesem Tag, gefunden von Spotlight. Im Filterfeld zeigt `tag:Rot` die Objekte des aktuellen Ordners mit diesem Tag (siehe [Dateien filtern](04-navigation.md#filtern)).

## Informationen {#informationen}

**⌘I** (**Informationen**, auch im Kontextmenü und im Menü „Ablage“) öffnet wie im Finder ein Informationsfenster zur Auswahl: eines pro Objekt (erneutes Aufrufen holt es nach vorn), ein Übersichtsfenster für mehrere Objekte. Bis zu sechs Informationsfenster können offen sein; für ein neues schließt sich das älteste. Es sind gewöhnliche Fenster: Du kannst im Hauptfenster weiterarbeiten.

Ein Informationsfenster zeigt:

- den **Namen** – ändere ihn und drücke **↩**, um das Objekt umzubenennen,
- **Art**, **Größe** (Ordner und mehrere Objekte werden im Hintergrund gezählt), **Ort**, **Erstellt**, **Geändert** und **Zuletzt geöffnet** (das Datum von Spotlight),
- **Tags** – entferne einen mit seinem ×, füge mit **Tag hinzufügen** einen hinzu,
- **Geschützt** (das Objekt kann nicht geändert, umbenannt oder in den Papierkorb gelegt werden) und **Unsichtbar**,
- **Kommentare** – den Spotlight-Kommentar aus dem Finder (Manta zeigt ihn, kann ihn aber nicht ändern: macOS bietet dafür nur das Fernsteuern des Finders),
- **Öffnen mit** – das Standardprogramm und **Öffnen mit**, um die Datei einmal mit einem anderen Programm zu öffnen. macOS bietet anderen Apps keinen öffentlichen Weg, das Programm für eine einzelne Datei zu ändern, deshalb gibt es kein „Alle ändern …“; dafür nimmst du den Finder,
- **Freigabe & Zugriffsrechte** – Lesen, Schreiben und Ausführen für Eigentümer, Gruppe und Jeder sowie der Oktalwert (z. B. `644`). Auf diesem Mac und auf SFTP-Servern änderst du sie direkt dort (chmod); anderswo werden sie nur angezeigt.

Objekte auf Servern, in Containern und in Archiven zeigen, was der Server meldet (Größe, Datum, Zugriffsrechte).

## Schreibgeschützte Orte {#schreibgeschuetzt}

In Archiven, gestoppten Containern und Containern ohne Shell kannst du nichts ändern. Ein Chip **Schreibgeschützt** (bzw. **Gestoppt**) in der Pfadleiste zeigt das an. Kopieren mit **F5** oder per Drag & Drop heraus geht immer.

Im Ordner **Programme** zeigt Manta wie der Finder auch die Apps von Apple (Rechner, Mail …) und in **Dienstprogramme** die von macOS. Sie liegen auf dem geschützten Systemvolume: Öffnen und Kopieren geht, Umbenennen, Verschieben und Löschen nicht.

## Siehe auch

- [Archive](10-archive.md): Komprimieren und Entpacken.
- [Terminal](11-terminal.md#dateiaktionen): Jede Dateiaktion als Shell-Befehl sehen.
