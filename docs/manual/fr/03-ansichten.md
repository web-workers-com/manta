# Présentations {#ansichten}

Comme dans le Finder, chaque sous-fenêtre peut présenter ses éléments par **Icônes**, en **Liste**, en **Colonnes** ou en **Galerie**. Changez de présentation avec **⌘1** à **⌘4** ou avec le sélecteur de la barre du chemin d’accès.

![Les quatre présentations comparées](images/views-overview.png)

## Changer de présentation {#wechseln}

| Présentation | Raccourci | Commande | Idéale pour… |
|---|---|---|---|
| Icônes | **⌘1** | **Par icônes** | Voir d’un coup d’œil des images et des dossiers contenant peu d’éléments |
| Liste | **⌘2** | **Par liste** | Trier par taille, date, autorisations |
| Colonnes | **⌘3** | **Par colonnes** | Parcourir des arborescences profondes tout en voyant le chemin |
| Galerie | **⌘4** | **Par galerie** | Regarder des images, des vidéos et des documents dans un grand aperçu |

Il existe trois façons de procéder :

- **Clavier** : **⌘1** à **⌘4** agissent sur la sous-fenêtre active.
- **Barre du chemin d’accès** : à droite de la barre du chemin d’accès de chaque sous-fenêtre se trouve le sélecteur **Présentation** avec quatre icônes. Lorsque la sous-fenêtre est étroite, il se transforme en un seul bouton avec un menu.
- **Palette de commandes** (**⇧⌘P**) : tapez par exemple « colonnes » ou « galerie ». Les commandes se trouvent dans le groupe **Présentation**.

La présentation appartient à la **sous-fenêtre** : tous les onglets d’une sous-fenêtre la partagent, et Manta la mémorise jusqu’au lancement suivant. Deux sous-fenêtres peuvent utiliser des présentations différentes, par exemple Colonnes à gauche et Liste à droite.

La sélection (clic, **⌘**-clic, **⇧**-clic, **⌘A**), la saisie pour sélectionner, Coup d’œil, le renommage, le menu contextuel, le glisser-déposer et les pastilles des plug-ins fonctionnent de la même façon dans toutes les présentations.

## Options de présentation {#optionen}

Dans les présentations Icônes, Colonnes et Galerie, le bouton **Options de présentation** (icône de curseurs) se trouve à côté du sélecteur. Il ouvre un petit menu :

- **Trier par** : **Nom**, **Taille**, **Date de modification** ou **Autorisations**, ainsi que **Croissant** ou **Décroissant**.
- **Taille des icônes** (présentation Icônes uniquement) : un curseur de 48 à 256 points.

![Sélecteur de présentation et Options de présentation dans la barre du chemin d’accès](images/view-options.png)

La présentation Liste n’a pas ce bouton – vous y triez à l’aide des en-têtes de colonne. Dans une sous-fenêtre étroite, les options se trouvent dans le menu du sélecteur.

## Présentation Liste {#liste}

La liste affiche les colonnes **Nom**, **Taille**, **Date de modification** et **Autorisations**.

- **Trier** : cliquez sur un en-tête de colonne. Un deuxième clic inverse l’ordre. Les dossiers viennent toujours en premier ; les apps et autres paquets sont triés comme des fichiers, comme dans le Finder.
- **Largeur des colonnes** : faites glisser le bord gauche d’une colonne.
- **Taille des dossiers** : avec le plug-in [Tailles des dossiers](14-plugins.md#ordnergroessen), la colonne **Taille** affiche aussi la taille réelle des dossiers.

![Présentation Liste triée par taille](images/view-list.png)

## Présentation Icônes {#symbole}

La présentation Icônes affiche une grille de grandes icônes et de vignettes, avec le nom en dessous.

- **Modifier la taille** : avec le curseur des **Options de présentation**, ou en faisant défiler tout en maintenant **⌘**, ou en pinçant sur le trackpad. Chaque sous-fenêtre a sa propre taille d’icônes.
- **Sélectionner plusieurs éléments** : tracez un rectangle de sélection en partant d’un emplacement vide. Avec **⌘** ou **⇧**, la sélection précédente est conservée.
- **Clavier** : ← → ↑ ↓ permettent de se déplacer dans la grille, **Page précédente**/**Page suivante** d’une page à la fois.

## Présentation Colonnes {#spalten}

Dans la présentation Colonnes, chaque dossier du chemin occupe sa propre colonne. Lorsque vous sélectionnez un dossier, la colonne à sa droite affiche son contenu.

- **Clavier** : ↑ ↓ sélectionnent dans la colonne actuelle, **→** ouvre le dossier sélectionné dans la colonne suivante, **←** revient à la colonne de gauche.
- **Colonne d’aperçu** : lorsque vous sélectionnez un fichier, la dernière colonne affiche un aperçu avec sa vignette ou son icône et les **Informations** : **Type**, **Taille**, **Modifié**, **Autorisations** et, pour les alias, l’**Original**. Si plusieurs éléments sont sélectionnés, elle indique par exemple « 3 éléments sélectionnés ».
- **Largeur des colonnes** : faites glisser le bord droit d’une colonne (au niveau de la poignée en bas). Double-cliquez sur le bord pour réinitialiser la largeur.
- Les dossiers vides affichent « Aucun élément ». Lorsque les colonnes sont plus larges que la sous-fenêtre, la présentation défile latéralement et garde la colonne active visible.

Si vous remontez au-delà de la première colonne avec **⌘↑**, ou si vous accédez via la barre latérale ou la barre du chemin d’accès à un dossier situé en dehors des colonnes affichées, les colonnes repartent de ce dossier.

![Présentation Colonnes](images/view-columns.png)

## Présentation Galerie {#galerie}

La galerie affiche l’élément sélectionné en grand, avec son nom et, en dessous, une ligne indiquant le type, la taille et la date de modification. En bas défile un **bandeau** contenant tous les éléments du dossier.

- Cliquez sur un élément du bandeau ou parcourez-les avec ← → pour l’afficher en grand. **Page précédente**/**Page suivante** avancent d’une largeur de bandeau.
- Si rien n’est sélectionné, la galerie affiche le premier élément.
- Les petites images sont agrandies au maximum au double de leur taille afin de ne pas devenir pixelisées.

![Présentation Galerie avec le bandeau](images/view-gallery.png)

## Vignettes {#vorschaubilder}

Dans les présentations Icônes, Colonnes et Galerie, Manta affiche des vignettes. Elles sont créées en arrière-plan, la présentation reste donc réactive ; tant qu’une vignette n’est pas prête, vous voyez l’icône du type de fichier. Sans vignette, les fichiers affichent l’icône que macOS associe à leur type (aussi en présentations Liste et Colonnes) et les apps (`.app`) la leur, comme dans le Finder (désactivable dans **Réglages** : **Icônes de fichiers de macOS**).

- Les **images** (PNG, JPEG, GIF, BMP, WebP) sont lues par Manta lui-même et pivotées selon l’orientation de l’appareil photo (EXIF).
- Les **PDF, vidéos, HEIC, documents**, etc. sont fournis par macOS (comme dans Coup d’œil). Les fichiers sans aperçu conservent l’icône de leur type.
- Les vignettes terminées sont stockées dans `~/Library/Caches/Manta/thumbs`. Lorsqu’un fichier change, une nouvelle vignette est créée. Vous pouvez supprimer ce dossier à tout moment.

### Vignettes des fichiers sur serveur {#server-vorschaubilder}

Pour les fichiers **sur des serveurs** (SFTP, FTP, S3, conteneurs), Manta n’affiche par défaut que l’icône du type de fichier. Si vous souhaitez aussi des vignettes à cet endroit, activez **Charger les vignettes des fichiers sur serveur** dans **Réglages** → **Général** → **Fichiers**. Manta télécharge alors les petites images (moins de 5 Mo) en arrière-plan. Les fichiers plus volumineux conservent leur icône.

> [!NOTE]
> Les vignettes de serveur consomment des données et un peu de temps. Sur les connexions lentes ou facturées au volume, laissez l’option désactivée.

## Mémoriser la présentation par dossier {#pro-ordner}

Par défaut, la présentation appartient à la **sous-fenêtre** : lorsque vous changez de dossier, la présentation reste la même.

Si vous activez **Mémoriser la présentation par dossier** dans **Réglages** → **Général** → **Fichiers** (désactivé par défaut), chaque dossier s’ouvre dans la présentation dans laquelle il a été affiché pour la dernière fois – vos photos toujours en galerie, votre projet en liste. Cela fonctionne aussi pour les dossiers sur des serveurs et dans des archives. Les dossiers pour lesquels vous n’avez pas choisi de présentation conservent la présentation actuelle de la sous-fenêtre.

Manta mémorise les 2000 derniers dossiers dans `~/Library/Application Support/Manta/folder-views.json`.

## Voir aussi

- [Coup d’œil et aperçu](06-schnellansicht.md)
- [La barre du chemin d’accès](04-navigation.md#pfadleiste)
- [Réglages](16-einstellungen.md#dateien)
- [Raccourcis clavier](18-tastenkuerzel.md#ansichten)
