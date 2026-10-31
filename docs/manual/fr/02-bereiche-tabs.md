# Sous-fenêtres et onglets {#bereiche-tabs}

Manta affiche vos fichiers dans des **sous-fenêtres** côte à côte – deux, trois ou autant que votre fenêtre peut en contenir. Chaque sous-fenêtre a ses propres onglets, sa barre du chemin d’accès et son historique.

![Trois sous-fenêtres côte à côte, la sous-fenêtre active mise en évidence](images/panes-three.png)

## La sous-fenêtre active {#aktiver-bereich}

Une seule sous-fenêtre est **active** : elle présente un contour marqué dans la couleur d’accentuation. Les raccourcis et les commandes agissent sur la sous-fenêtre active.

- Cliquez n’importe où dans une sous-fenêtre pour l’activer.
- **⇥** (Tab) passe à la sous-fenêtre suivante (après la dernière, retour à la première).
- **⌥⌘←** et **⌥⌘→** activent la sous-fenêtre de gauche ou de droite.

## Diviser une sous-fenêtre {#teilen}

1. Activez la sous-fenêtre à côté de laquelle la nouvelle doit apparaître.
2. Appuyez sur **⌘D** (**Diviser la sous-fenêtre**).

La nouvelle sous-fenêtre apparaît à droite de la sous-fenêtre active et affiche le même dossier. Vous pouvez aussi cliquer sur le bouton de division à droite de la barre du chemin d’accès de la dernière sous-fenêtre, ou choisir **Diviser la sous-fenêtre** dans la palette de commandes.

Chaque sous-fenêtre a besoin d’environ 280 points de largeur. Si la fenêtre est trop étroite, Manta affiche « Pas de place pour une autre sous-fenêtre ». Élargissez la fenêtre ou fermez une sous-fenêtre.

## Fermer une sous-fenêtre {#schliessen}

- Appuyez sur **⇧⌘D** (**Fermer la sous-fenêtre**), ou
- cliquez sur le **×** à droite de la barre du chemin d’accès de la sous-fenêtre.

La dernière sous-fenêtre ne peut pas être fermée.

> [!NOTE]
> Avec une seule sous-fenêtre, **Copier** et **Déplacer dans l’autre sous-fenêtre**, **Comparer et synchroniser** et le changement de sous-fenêtre sont désactivés. L’infobulle en donne la raison : « Nécessite deux sous-fenêtres – ⌘D divise la sous-fenêtre ».

## Modifier la largeur des sous-fenêtres {#breite}

- Faites glisser le séparateur entre deux sous-fenêtres vers la gauche ou la droite.
- Double-cliquez sur un séparateur pour redonner la même largeur à toutes les sous-fenêtres.

## La sous-fenêtre cible : « l’autre sous-fenêtre » {#zielbereich}

Copier (**F5**) et Déplacer (**F6**) agissent toujours vers la **sous-fenêtre cible**.

- Avec **deux sous-fenêtres**, c’est tout simplement l’autre.
- Avec **trois ou plus**, c’est la sous-fenêtre qui a été active le plus récemment. Tant que la sous-fenêtre active contient une sélection, la sous-fenêtre cible présente un léger contour et une pastille **→** dans sa barre du chemin d’accès (« Copier et Déplacer (F5 / F6) ciblent cette sous-fenêtre »).

Dans la zone de dialogue de copie, vous pouvez choisir une autre sous-fenêtre sous **Vers**. Voir [Gérer les fichiers](05-dateien.md#kopieren).

![Sous-fenêtre cible avec la pastille fléchée](images/panes-target-badge.png)

## Onglets {#tabs}

Chaque sous-fenêtre peut contenir plusieurs onglets. Un onglet affiche un dossier – sur votre Mac, sur un serveur, dans un conteneur ou dans une archive.

| Action | Comment |
|---|---|
| Nouvel onglet | **⌘T** ou le bouton **+** dans la barre du chemin d’accès, ou **Nouvel onglet** dans le menu du Dock (clic droit sur l’icône de Manta dans le Dock) |
| Fermer l’onglet | **⌘W** ou le **×** sur l’onglet (apparaît au survol) |
| Changer d’onglet | Cliquez sur l’onglet |

Le nouvel onglet affiche le même dossier que l’onglet actuel, ou le dossier choisi sous [**Les nouveaux onglets s’ouvrent dans**](16-einstellungen.md#dateien). **Aller au chemin…** (**⇧⌘G**) ouvre un dossier directement dans un nouvel onglet (voir [Navigation](04-navigation.md#gehe-zum-pfad)).

> [!NOTE]
> Comme dans le Finder, **⌘T** ouvre un nouvel onglet. Avant cette version, c’était **⇧⌘T**, et **⌘T** affichait les transferts – c’est désormais **⌥⌘L**.

Dans les sous-fenêtres très étroites, le bouton **+** quitte la barre du chemin d’accès ; **⌘T** fonctionne toujours.

## Sessions : tout revient {#sitzung}

Lorsque vous quittez Manta puis le relancez, vous retrouvez :

- les sous-fenêtres dans leur ordre et avec leur largeur,
- les onglets affichant des dossiers **locaux**,
- la sous-fenêtre qui était active et celle qui était la cible.

Les onglets affichant des dossiers de serveur, de conteneur ou d’archive ne sont pas restaurés, car ils nécessiteraient une nouvelle connexion. Ouvrez-les depuis vos [Favoris](07-favoriten-seitenleiste.md).

Si un dossier enregistré n’existe plus au lancement suivant (parce que son disque a été éjecté, par exemple), l’onglet revient malgré tout. Il affiche une erreur avec **Réessayer** – reconnectez le disque et cliquez sur ce bouton, ou fermez l’onglet ou ouvrez un autre dossier.

## Voir aussi

- [Présentations](03-ansichten.md) : Icônes, Liste, Colonnes, Galerie.
- [Comparer et synchroniser](13-vergleichen-synchronisieren.md) : harmoniser deux sous-fenêtres.
