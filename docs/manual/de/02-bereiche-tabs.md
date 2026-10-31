# Bereiche & Tabs {#bereiche-tabs}

Manta zeigt deine Dateien in **Bereichen** nebeneinander – zwei, drei oder so viele, wie in dein Fenster passen. Jeder Bereich hat eigene Tabs, eine eigene Pfadleiste und einen eigenen Verlauf.

![Drei Bereiche nebeneinander, der aktive Bereich hervorgehoben](images/panes-three.png)

## Der aktive Bereich {#aktiver-bereich}

Immer genau ein Bereich ist **aktiv**: Er hat einen deutlichen Rahmen in der Akzentfarbe. Tastenkürzel und Befehle wirken auf den aktiven Bereich.

- Klicke irgendwo in einen Bereich, um ihn zu aktivieren.
- **⇥** (Tabulatortaste) wechselt zum nächsten Bereich (nach dem letzten wieder zum ersten).
- **⌥⌘←** und **⌥⌘→** aktivieren den Bereich links bzw. rechts.

## So teilst du einen Bereich {#teilen}

1. Aktiviere den Bereich, neben dem ein neuer erscheinen soll.
2. Drücke **⌘D** (**Bereich teilen**).

Der neue Bereich erscheint rechts neben dem aktiven und zeigt denselben Ordner. Alternativ klickst du auf die Teilen-Taste rechts in der Pfadleiste des letzten Bereichs oder wählst **Bereich teilen** in der Befehlspalette.

Jeder Bereich braucht etwa 280 Punkte Breite. Ist das Fenster zu schmal, meldet Manta „Kein Platz für einen weiteren Bereich“. Mach dann das Fenster breiter oder schließe einen Bereich.

## So schließt du einen Bereich {#schliessen}

- Drücke **⇧⌘D** (**Bereich schließen**), oder
- klicke auf das **×** rechts in der Pfadleiste des Bereichs.

Der letzte Bereich lässt sich nicht schließen.

> [!NOTE]
> Mit nur einem Bereich sind **Kopieren** und **Bewegen in den anderen Bereich**, **Vergleichen und synchronisieren** und das Wechseln zwischen Bereichen ausgeschaltet. Der Tooltip sagt dir warum: „Benötigt zwei Bereiche – ⌘D teilt den Bereich“.

## Breite der Bereiche ändern {#breite}

- Zieh die Trennlinie zwischen zwei Bereichen nach links oder rechts.
- Doppelklicke auf eine Trennlinie, um alle Bereiche wieder gleich breit zu machen.

## Der Zielbereich: „der andere Bereich“ {#zielbereich}

Kopieren (**F5**) und Bewegen (**F6**) gehen immer in den **Zielbereich**.

- Bei **zwei Bereichen** ist das einfach der andere.
- Bei **drei oder mehr** ist es der Bereich, den du zuletzt aktiv hattest. Solange im aktiven Bereich etwas ausgewählt ist, hat der Zielbereich einen feinen Rahmen und ein **→**-Abzeichen in der Pfadleiste („Kopieren und Bewegen (F5 / F6) gehen in diesen Bereich“).

Im Kopieren-Dialog kannst du unter **Nach** einen anderen Bereich wählen. Mehr dazu in [Dateien verwalten](05-dateien.md#kopieren).

![Zielbereich mit Pfeil-Abzeichen](images/panes-target-badge.png)

## Tabs {#tabs}

Jeder Bereich kann mehrere Tabs haben. Ein Tab zeigt einen Ordner – lokal, auf einem Server, in einem Container oder in einem Archiv.

| Aktion | So geht's |
|---|---|
| Neuer Tab | **⌘T**, die **+**-Taste in der Pfadleiste oder **Neuer Tab** im Dock-Menü (Rechtsklick auf das Manta-Symbol im Dock) |
| Tab schließen | **⌘W** oder das **×** auf dem Tab (erscheint beim Darüberfahren) |
| Tab wechseln | Auf den Tab klicken |

Der neue Tab zeigt denselben Ordner wie der aktuelle – oder den Ordner, den du unter [**Neue Tabs öffnen in**](16-einstellungen.md#dateien) gewählt hast. **Gehe zum Pfad …** (**⇧⌘G**) öffnet einen Ordner direkt in einem neuen Tab (siehe [Navigation](04-navigation.md#gehe-zum-pfad)).

> [!NOTE]
> Wie im Finder öffnet **⌘T** einen neuen Tab. Vor dieser Version war das **⇧⌘T**, und **⌘T** blendete die Übertragungen ein – dafür gibt es jetzt **⌥⌘L**.

In sehr schmalen Bereichen verschwindet die **+**-Taste aus der Pfadleiste; **⌘T** funktioniert trotzdem.

## Sitzungen: Alles ist wieder da {#sitzung}

Wenn du Manta beendest und wieder startest, kommen zurück:

- die Bereiche in ihrer Reihenfolge und Breite,
- die Tabs mit **lokalen** Ordnern,
- welcher Bereich aktiv war und welcher der Zielbereich ist.

Tabs mit Server-, Container- oder Archiv-Ordnern werden nicht wiederhergestellt, weil dafür eine neue Verbindung nötig wäre. Öffne sie über die [Favoriten](07-favoriten-seitenleiste.md).

Gibt es einen gespeicherten Ordner beim nächsten Start nicht mehr (z. B. weil das Laufwerk ausgeworfen ist), kommt der Tab trotzdem wieder. Er zeigt dann eine Fehlermeldung mit **Erneut versuchen** – steck das Laufwerk wieder an und klicke darauf, oder schließ den Tab bzw. öffne einen anderen Ordner.

## Siehe auch

- [Ansichten](03-ansichten.md): Symbole, Liste, Spalten, Galerie.
- [Vergleichen & Synchronisieren](13-vergleichen-synchronisieren.md): zwei Bereiche abgleichen.
