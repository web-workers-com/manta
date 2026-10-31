# Navigation {#navigation}

Ce chapitre explique comment vous déplacer dans les dossiers : avec la barre du chemin d’accès, avec **Aller au chemin…**, avec l’historique et avec le clavier. Il aborde pour finir le filtrage, la saisie pour sélectionner et les fichiers masqués.

## Ouvrir des dossiers et des fichiers {#oeffnen}

- Un **double-clic** ou **↩** ouvre un dossier dans la sous-fenêtre, ou un fichier avec son app par défaut.
- **⌘↓** fait la même chose que ↩.
- **⌥↩** (**Ouvrir avec l’app par défaut**) ouvre toujours avec l’app par défaut, y compris les archives, que Manta ouvre sinon comme des dossiers (voir [Archives](10-archive.md)).
- **⌘↑** ou **⌫** (Suppr. arrière) remonte au **dossier parent**.

Pour ouvrir un fichier situé sur un serveur, Manta le télécharge dans un dossier temporaire et ouvre cette copie (« Téléchargement de « nom » pour l’ouvrir… »).

> [!WARNING]
> Les modifications apportées à un fichier de serveur ouvert de cette façon ne sont enregistrées que dans la copie temporaire, pas sur le serveur. Pour modifier un fichier, copiez-le dans un dossier local, modifiez-le à cet endroit, puis recopiez-le.

## La barre du chemin d’accès {#pfadleiste}

La barre du chemin d’accès, en haut de chaque sous-fenêtre, indique où vous vous trouvez, par exemple `Départ › Projets › manta`.

![Barre du chemin d’accès avec la pastille d’emplacement, le chemin et les pastilles des plug-ins](images/path-bar.png)

- **Cliquez sur un nom de dossier** pour vous y rendre.
- **Cliquez sur « … »** au début pour voir les dossiers parents (« Afficher les dossiers parents ») lorsque le chemin est raccourci.
- **Double-cliquez sur un emplacement vide** de la barre du chemin d’accès (ou appuyez sur **⌘L**, **Modifier le chemin**) pour la transformer en champ de texte. Tapez un chemin et appuyez sur ↩. **esc** annule.
- **Pastille d’emplacement** : la pastille tout à gauche indique où vous vous trouvez : **Local** sur votre Mac, ou par exemple **Docker · shop-web**, **Kubernetes · pod**, **SFTP · deploy@web-01**, **S3 · bucket**, **FTP · host**, **Archive · backup.zip** – et, dans les dossiers cloud, le fournisseur (**Google Drive**, **iCloud Drive**, **OneDrive**, **Dropbox** ; le compte figure dans l’infobulle). Les noms longs sont abrégés par « … », les sous-fenêtres étroites n’affichent que l’icône ; le texte complet figure dans l’infobulle. Un clic sur la pastille mène au dossier de plus haut niveau de cet emplacement.
- **Présentation** : à droite, avant le champ de filtre, vous passez d’Icônes à Liste, Colonnes et Galerie ; à côté se trouve **Options de présentation**. Voir [Présentations](03-ansichten.md#wechseln).
- Les **pastilles des plug-ins** à droite indiquent par exemple la branche Git ou le projet détecté. Lorsque la sous-fenêtre se rétrécit, elles raccourcissent puis deviennent une simple icône ; le texte complet figure dans l’infobulle.

### Revenir au local {#lokal}

Lorsque vous n’êtes pas dans un dossier ordinaire de votre Mac (serveur, conteneur, bucket, archive ou dossier cloud), une flèche **Revenir au local** apparaît à côté de la pastille d’emplacement. Elle vous ramène au dernier dossier de votre Mac dans l’historique de l’onglet – en ignorant les dossiers cloud et les archives – ou, à défaut, à votre dossier de départ. La connexion reste ouverte.

## Aller au chemin {#gehe-zum-pfad}

**Aller au chemin…** (**⇧⌘G**) ouvre un dossier dont vous connaissez le chemin ou dont le chemin se trouve dans le presse-papiers.

1. Appuyez sur **⇧⌘G**.
2. Tapez ou collez un chemin. Manta comprend `~/Downloads`, les chemins relatifs, les chemins entre guillemets et les URL `file://`.
3. **⇥** complète les noms de dossier. Sous le champ, vous voyez **Dossiers correspondants** et **Récents**.
4. Appuyez sur **↩** pour ouvrir le dossier dans un **nouvel onglet**, ou sur **⌘↩** pour l’ouvrir **dans l’onglet actuel**.

Manta vérifie le chemin en arrière-plan et signale par exemple « Dossier introuvable » ou « Autorisation refusée ». Si le chemin désigne un **fichier**, Manta ouvre son dossier et sélectionne le fichier.

Si la sous-fenêtre active affiche un serveur, le chemin se rapporte à ce serveur (« S’ouvre dans un nouvel onglet sur… »).

**⇧⌘G** fonctionne partout, même pendant que vous tapez dans un champ de texte.

![Aller au chemin avec des suggestions](images/go-to-path.png)

## Précédent et suivant {#verlauf}

Chaque onglet a son propre historique.

- **⌘[** ou **⌘←** : **Précédent**
- **⌘]** ou **⌘→** : **Suivant**
- Les **boutons du pouce de la souris** (boutons 4 et 5) : **Précédent** et **Suivant** dans la sous-fenêtre sous le pointeur. Inversez-les ou désactivez-les dans [Réglages → Fichiers](16-einstellungen.md#dateien).

L’historique entre dans les archives et en ressort.

## Sélectionner au clavier {#auswaehlen}

| Touche | Effet |
|---|---|
| ↑ / ↓ | Élément précédent / suivant |
| ⇧↑ / ⇧↓ | Étendre la sélection |
| Page précédente / Page suivante | 20 lignes plus loin |
| Début / Fin | Premier / dernier élément |
| ⌘A | Tout sélectionner |
| esc | Annuler la sélection |

À la souris : un clic sélectionne un élément, **⌘-clic** en ajoute ou en retire un, **⇧-clic** sélectionne une plage. Un clic sur un emplacement vide annule la sélection.

## Saisir pour sélectionner {#tippen}

Comme dans le Finder : tapez les premières lettres d’un nom et Manta passe au premier élément correspondant. La casse n’a pas d’importance. Si aucun nom ne commence par ces lettres, Manta passe au nom suivant dans l’ordre de tri.

Après environ une seconde sans frappe, la recherche repart de zéro. Pendant la saisie, l’Espace fait partie du nom (« mon do ») et n’ouvre pas Coup d’œil.

## Filtrer les fichiers {#filtern}

Le filtre n’affiche que les éléments dont le nom contient un certain texte.

1. Appuyez sur **⌘F** (**Filtrer les fichiers**) ou cliquez sur le champ **Filtrer** de la barre du chemin d’accès.
2. Tapez une partie du nom, par exemple `.pdf` ou `facture`.

Le filtre s’applique uniquement au dossier actuel et à cet onglet. La barre d’état affiche « correspondant à « … » ». Si rien ne correspond, vous voyez « Aucun élément ne correspond à « … » » avec un bouton **Effacer le filtre**. **esc** dans le champ efface le filtre. Dans les sous-fenêtres plus étroites (par exemple deux sous-fenêtres dans une petite fenêtre), le champ prend la forme d’une loupe : cliquez dessus ou appuyez sur **⌘F** pour l’ouvrir ; il reste ouvert tant qu’il contient du texte et se replie lorsqu’il est vide et que vous cliquez ailleurs ou appuyez sur **esc**. Cela laisse de la place pour la pastille d’emplacement et le chemin ; tant que le champ est ouvert, les boutons de présentation se replient en un seul bouton de menu.

Pour filtrer par [tag](05-dateien.md#tags), tapez `tag:` suivi du nom du tag, par exemple `tag:Rouge` ou `tag:"Très important"` ; le nom de la couleur fonctionne aussi (`tag:rouge` trouve également un tag rouge nommé « Urgent »). Vous pouvez le combiner avec un nom : `tag:Rouge facture`.

## Rechercher dans plusieurs dossiers {#suchen}

Le filtre ne porte que sur le dossier affiché. Pour chercher dans ses sous-dossiers, ou sur tout le Mac, utilisez la barre d’étendue qui apparaît sous la barre du chemin d’accès tant que le champ est utilisé :

- **Ce dossier** – le filtre décrit ci-dessus.
- **Sous-dossiers** – le dossier et tout ce qu’il contient. Sur ce Mac, plusieurs dossiers sont lus en même temps ; les éléments masqués ne sont parcourus que si les fichiers masqués sont affichés, et le contenu des paquets (apps, photothèques) est ignoré. Sur les serveurs, dans les conteneurs et dans les archives, Manta parcourt les dossiers un par un, sur 12 niveaux de profondeur et 5 000 dossiers au maximum.
- **Ce Mac** – Spotlight : les noms *et* le contenu des fichiers, partout où Spotlight indexe (dossiers locaux uniquement).

1. Appuyez sur **⌘F** et tapez ce que vous cherchez.
2. Cliquez sur **Sous-dossiers** ou **Ce Mac**. La recherche démarre un instant après la dernière frappe (ou immédiatement avec **↩**).

Un nom correspond s’il contient le texte (majuscules et minuscules sont indifférentes). Avec `*` ou `?`, le texte devient un motif pour le nom entier : `*.pdf`, `report-202?.xlsx`. Les termes `tag:` fonctionnent ici aussi.

Les résultats remplissent la sous-fenêtre sous forme de liste pendant la recherche : la barre du chemin d’accès indique **Recherche : « … »** et l’emplacement, la barre d’étendue indique le nombre d’éléments trouvés, et une colonne **Chemin** montre le dossier de chaque élément. Vous pouvez ouvrir les résultats, les afficher avec Coup d’œil, les copier, les déplacer, les renommer, leur attribuer des tags et les supprimer comme d’habitude. **Afficher dans le dossier parent** dans le menu contextuel (ou **⌘↑**) ouvre le dossier de l’élément avec celui-ci sélectionné. Le bouton d’arrêt de la barre d’étendue interrompt une longue recherche (les résultats déjà trouvés restent) ; **⌘R** la relance. 10 000 résultats au maximum sont affichés.

**esc** dans le champ de recherche, **⌘[** ou l’effacement du texte ferme la recherche et affiche de nouveau le dossier ; ouvrir un autre dossier a le même effet.

Si **Sous-dossiers** ne trouve rien, la sous-fenêtre vide propose **Rechercher sur ce Mac**.

## Fichiers masqués {#unsichtbare-dateien}

Les fichiers dont le nom commence par un point (comme `.git`, `.env`) et les fichiers marqués comme masqués sont normalement cachés. Seul `.DS_Store` n’apparaît jamais, même avec les fichiers masqués affichés, comme dans le Finder : macOS le recrée dans presque chaque dossier.

- **⇧⌘.** (**Afficher les fichiers masqués**) les affiche ou les masque dans l’onglet actuel. Le bouton de la barre d’outils représentant un fichier en pointillés fait de même (il est mis en évidence tant que les fichiers masqués sont affichés). L’œil à côté correspond à Coup d’œil.
- **Réglages** → **Général** → **Afficher les fichiers masqués** détermine si les nouveaux onglets et Manta au lancement les affichent.

Si un dossier semble vide mais contient des éléments masqués, Manta vous le signale : « 3 éléments masqués — appuyez sur ⇧⌘. pour les afficher. »

## Actualiser {#aktualisieren}

Les dossiers locaux restent à jour d’eux-mêmes – comme dans le Finder : lorsqu’une autre app crée, supprime, renomme ou modifie un fichier, le changement apparaît dans la sous-fenêtre au bout d’un instant. La sélection, le curseur, la position de défilement et la saisie pour sélectionner restent inchangés. Manta surveille les dossiers de tous les onglets ouverts (dans la présentation Colonnes, également les colonnes de gauche). Un onglet en arrière-plan est relu dès que vous y revenez, immédiatement et sans ⌘R. De nombreux changements simultanés – la copie de milliers de fichiers, par exemple – sont regroupés ; le dossier est relu au maximum une fois par seconde environ. Pendant que vous tapez un nom (Renommer, Nouveau dossier), l’actualisation attend que vous ayez terminé.

Les dossiers sur des **serveurs**, dans des **conteneurs**, dans **S3** et dans des **archives** ne sont pas surveillés. Si quelqu’un d’autre y modifie quelque chose, appuyez sur **⌘R** (**Actualiser**) pour relire le dossier. ⌘R fonctionne bien sûr aussi en local.

## Voir aussi

- [Tags et informations](05-dateien.md#tags)
- [Favoris et barre latérale](07-favoriten-seitenleiste.md)
- [Raccourcis clavier](18-tastenkuerzel.md#navigation)
