# Gérer les fichiers {#dateien}

Copier, déplacer, renommer, supprimer et créer des dossiers – sur votre Mac, sur des serveurs, dans des conteneurs et entre tous ces emplacements. Lisez en particulier la section sur la [suppression](#loeschen) : sur votre Mac, les éléments supprimés vont dans la corbeille ; sur les serveurs, la suppression est définitive.

## Copier des fichiers dans une autre sous-fenêtre {#kopieren}

1. Dans la **sous-fenêtre cible**, ouvrez le dossier dans lequel les fichiers doivent aller.
2. Dans la **sous-fenêtre active**, sélectionnez les fichiers et les dossiers.
3. Appuyez sur **F5** (**Copier dans l’autre sous-fenêtre…**).
4. Vérifiez dans la zone de dialogue ce qui va où, et choisissez ce qui se passe en cas de noms identiques (voir [Conflits de noms](#konflikte)).
5. Cliquez sur **Copier**.

La copie s’exécute en arrière-plan. Vous pouvez continuer à travailler immédiatement.

![Zone de dialogue de copie avec la cible et le choix en cas de conflit](images/copy-dialog.png)

**Déplacer** fonctionne de la même façon avec **F6** (**Déplacer dans l’autre sous-fenêtre…**). Les éléments déplacés disparaissent de leur ancien emplacement.

Les deux commandes se trouvent aussi dans la barre d’outils (« Copier dans la sous-fenêtre de droite », « Déplacer dans la sous-fenêtre de gauche »…) et dans le menu contextuel (clic droit).

### Quelle sous-fenêtre est la cible ? {#ziel}

Avec deux sous-fenêtres, c’est l’autre. Avec trois ou plus, c’est l’autre sous-fenêtre active le plus récemment (avec une pastille **→**, voir [Sous-fenêtre cible](02-bereiche-tabs.md#zielbereich)). Sous **Vers** dans la zone de dialogue, vous pouvez choisir n’importe quelle autre sous-fenêtre.

Manta évite les erreurs courantes :

- « Les deux sous-fenêtres affichent le même dossier — ouvrez un autre dossier dans l’autre sous-fenêtre »
- « Un dossier ne peut pas être copié dans lui-même »

### Entre différents emplacements {#orte}

La copie et le déplacement fonctionnent entre tous les emplacements : de votre Mac vers un serveur SFTP, d’un bucket S3 vers un conteneur Docker, d’une archive vers le Bureau. Manta s’en charge lui-même ; aucun outil supplémentaire n’est nécessaire.

Depuis les emplacements **en lecture seule** (archives, conteneurs arrêtés), un déplacement se contente de copier. Manta vous le signale, par exemple : « Les archives sont en lecture seule : les éléments sont extraits (copiés), pas déplacés ».

## Conflits de noms {#konflikte}

Si un élément portant le même nom existe déjà à la destination, le réglage **Si un élément existe déjà** de la zone de dialogue décide :

| Option | Ce qui se passe |
|---|---|
| **Demander** | Manta s’arrête à chaque élément déjà existant et vous demande quoi faire (par défaut). |
| **Remplacer** | L’élément existant est toujours écrasé. |
| **Ignorer** | Les éléments existants restent intacts. |
| **Remplacer si plus récent** | Écrase uniquement si la source est plus récente ou d’une taille différente. |
| **Garder les deux** | La copie reçoit un nouveau nom (« nom 2 »). |

Les autres options s’appliquent à toute la tâche sans poser de question. Définissez l’option par défaut dans **Réglages** → **Général** → **Copie et déplacement**.

### Demander {#nachfragen}

Avec **Demander**, la tâche s’arrête au premier élément qui existe déjà à la destination. Manta affiche « « nom » existe déjà dans « dossier » » avec les deux éléments – taille, date de modification et une étiquette **Plus récent** sur le plus récent des deux – et vous choisissez :

| Bouton | Ce qui se passe |
|---|---|
| **Garder les deux** | La copie reçoit un nouveau nom (« nom 2 »). C’est le choix par défaut : **↩** le sélectionne. |
| **Remplacer** | L’élément existant est écrasé. |
| **Remplacer si plus récent** | Écrase uniquement si la source est plus récente ; sinon l’élément existant est conservé. |
| **Ignorer** | L’élément existant est conservé et celui-ci n’est pas transféré (un déplacement le laisse à sa place). |
| **Annuler** (**esc**) | La tâche s’arrête. Ce qui a déjà été transféré reste à la destination. |

Si un **dossier** portant ce nom existe déjà, Manta propose **Fusionner** (comme le Finder) : le contenu est placé dans le dossier existant, et pour les fichiers déjà présents, la question est posée un par un. Sont aussi proposés **Garder les deux** (« photos 2 ») et **Ignorer**. Les sous-dossiers sont toujours fusionnés.

Avec **Appliquer à tous les autres conflits de cette tâche**, la réponse vaut pour le reste de la tâche et Manta ne repose plus la question. Lorsque plusieurs tâches posent une question en même temps, elles le font à tour de rôle. Les autres transferts continuent pendant qu’une tâche attend ; la liste des transferts l’indique par « En attente de votre décision ». Si vous quittez Manta alors qu’une question est ouverte, la tâche repose la question dès qu’elle reprend.

## Suivre les transferts {#uebertragungen}

**⌥⌘L** (**Afficher les transferts**) ou le bouton **Transferts** de la barre d’outils affiche toutes les tâches en cours : copie, déplacement, suppression, extraction et compression.

- Chaque tâche indique sa progression, sa vitesse et le temps restant.
- **Suspendre** interrompt une tâche, **Reprendre** la poursuit (voir ci-dessous).
- **Arrêter** annule une tâche. Les fichiers copiés à moitié sont supprimés.
- **Effacer les terminés** fait le ménage dans la liste.
- En cas de problème, la tâche affiche « N problèmes ». Le message « Terminé avec des problèmes » renvoie aussi ici.

Lorsque vous masquez le panneau, la barre d’état affiche en bas à droite une petite pastille avec un anneau de progression : vous voyez ainsi toujours où en est une tâche – y compris pour les fichiers déposés depuis d’autres apps ou copiés dans des dossiers cloud :

- une tâche : « Copie de 3 éléments… 42 % », « Déplacement de « Vacances.mov »… 80 % »,
- plusieurs : « 2 transferts · 37 % »,
- suspendue ou en attente : « … · suspendu », « … · en attente », « … · en attente de connexion »,
- ensuite **Terminé** (ou **Arrêté**) pendant quelques secondes,
- en cas de problème, la pastille rouge (« Échec du transfert », « Terminé avec des erreurs ») reste affichée jusqu’à ce que vous cliquiez dessus.

Cliquez sur la pastille pour ouvrir les transferts ; son infobulle liste chaque tâche en cours avec sa progression et le temps restant.

Lorsque vous copiez dans un dossier cloud (Google Drive, iCloud Drive, OneDrive, Dropbox), le transfert est terminé dès que les fichiers sont sur votre Mac – l’app du fournisseur les téléverse ensuite. La barre d’état l’indique aussi, par exemple « Google Drive téléverse 2 éléments » ou, en rouge, « Google Drive : 1 erreur de téléversement » (voir [Stockage cloud](09-cloud.md#abzeichen)).

![Transferts avec une tâche en cours](images/transfers-panel.png)

## Suspendre et reprendre les transferts {#pausieren}

Toute tâche de copie, de déplacement ou de suppression peut être suspendue puis reprise au même point – même au milieu d’un gros fichier.

- **Une tâche** : cliquez sur l’icône de pause dans sa ligne (**Suspendre**). La ligne affiche alors par exemple « Suspendu · 1,2 Go sur 3,0 Go ». L’icône de lecture (**Reprendre**) la relance.
- **Toutes à la fois** : en haut de **Transferts**, vous trouvez **Tout suspendre** et **Tout reprendre** dès qu’il y a quelque chose à suspendre ou à reprendre.

Une tâche reprise ignore les fichiers terminés et poursuit le fichier en cours là où il s’était arrêté (« Reprise à partir de 1,2 Go »).

La **compression** ne peut pas être suspendue.

### Quand la connexion est coupée {#verbindung-weg}

Si le réseau tombe ou si le serveur disparaît brièvement, Manta n’abandonne pas la tâche. Il **attend** et réessaie de lui-même :

- La ligne affiche « En attente de connexion… (tentative 3, suivante dans 8 s) ». Les pauses s’allongent (1, 2, 4… secondes, au maximum 1 minute) ; Manta continue d’essayer.
- Dès que votre Mac est de nouveau en ligne, la tâche reprend immédiatement – là où elle s’était arrêtée.
- Si vous ne voulez pas attendre, cliquez sur **Réessayer maintenant** (icône de flèche).
- Lorsque votre Mac n’a plus du tout de réseau, **Transferts** affiche le bandeau **Aucune connexion à Internet** – « Les transferts reprennent là où ils se sont arrêtés dès que la connexion est rétablie. »

Une tâche en attente peut être suspendue ou arrêtée comme n’importe quelle autre.

![Transferts : une tâche suspendue et une en attente, bandeau « Aucune connexion à Internet »](images/transfers-paused-waiting.png)

### Fichiers partiels (`.manta-part`) {#teildateien}

Pendant la copie d’un fichier, Manta l’écrit d’abord dans un **fichier partiel masqué** à côté de la cible : `film.mov` devient `.film.mov.manta-part`. Ce n’est qu’une fois le fichier complet qu’il reçoit son vrai nom. Ainsi, la cible ne contient jamais un fichier à moitié copié sous son vrai nom.

- Vous ne voyez les fichiers partiels que si les fichiers masqués sont affichés (**⇧⌘.**).
- Ne supprimez pas un fichier partiel pendant que la tâche est suspendue ou en attente – sinon la copie de ce fichier recommence depuis le début.
- **Arrêter** et **Abandonner** (voir ci-dessous) les nettoient pour vous.
- Les téléversements vers **S3** et **Google Drive** n’ont pas besoin de fichier partiel : le serveur garde le fichier inachevé invisible jusqu’à ce qu’il soit complet.

### Après avoir quitté l’app ou après un plantage {#neustart}

Manta mémorise les transferts inachevés même si vous quittez l’app, si elle plante ou si le Mac redémarre. Au lancement suivant, il demande :

**Reprendre les transferts interrompus ?** – par exemple « 2 transferts ne se sont pas terminés la dernière fois. » En dessous figurent les tâches avec leur progression et, si nécessaire, le serveur manquant (« nécessite files.example.com (SFTP) »).

- **Reprendre** (ou ↩) : les tâches reprennent là où elles s’étaient arrêtées.
- **Ignorer** : les tâches sont abandonnées et leurs fichiers partiels supprimés – dans la mesure où Manta peut y accéder. Sur un serveur non connecté, un fichier partiel peut subsister ; vous pourrez le supprimer vous-même plus tard.
- **esc** (fermer la fenêtre) : décider plus tard. Manta repose la question au lancement suivant.

![La question « Reprendre les transferts interrompus ? »](images/resume-prompt.png)

#### « Nécessite une connexion » {#benoetigt-verbindung}

Lorsqu’une tâche reprise a besoin d’un serveur auquel vous n’êtes pas encore connecté, sa ligne affiche « Nécessite une connexion à files.example.com (SFTP) » et le bouton **Reconnecter**.

- Les **favoris de serveur** dont le mot de passe est mémorisé (ou utilisant ssh-agent) se reconnectent d’eux-mêmes en arrière-plan ; vous n’avez rien à faire.
- Sinon, cliquez sur **Reconnecter**. Manta ouvre la zone de dialogue de connexion avec les informations du serveur ; saisissez le mot de passe. Dès que la connexion est établie, la tâche reprend.

Manta n’enregistre **jamais** de mots de passe ni de phrases de passe à cet effet : la liste des transferts interrompus (`transfers.json`) ne contient que des serveurs, des noms d’utilisateur et des chemins.

#### Remarques « copié de nouveau depuis le début » {#von-vorn}

Il arrive que Manta ne puisse pas poursuivre un fichier en toute sécurité et le copie de nouveau. La tâche affiche alors « 1 remarque » ; cliquez dessus pour afficher les remarques :

| Remarque | Ce qui s’est passé |
|---|---|
| « la source a changé depuis l’interruption ; copié de nouveau depuis le début » | Le fichier d’origine a désormais une taille ou une date de modification différente. Le poursuivre mélangerait l’ancien et le nouveau. |
| « ce serveur ne peut pas compléter les fichiers partiels ; copié de nouveau depuis le début » | La cible ne permet pas d’ajouter des données à un fichier inachevé. |
| « le fichier partiel était plus grand que la source ; copié de nouveau depuis le début » | Le fichier partiel ne correspond pas à la source. |
| « la session de téléversement a expiré ; téléversé de nouveau depuis le début » | S3 ou Google Drive a abandonné entre-temps le téléversement inachevé. |

### Limites {#pausieren-grenzen}

- La **compression** ne peut pas être suspendue et ne survit pas à un redémarrage.
- Les tâches **depuis des archives** (extraction) peuvent être suspendues, mais ne sont pas reprises après avoir quitté l’app.
- Sans le bon mot de passe, une tâche sur un serveur attend que vous vous connectiez.

## Glisser-déposer {#drag-drop}

### Entre sous-fenêtres

1. Sélectionnez des éléments et faites-les glisser dans une autre sous-fenêtre.
2. Relâchez. Manta vous demande : dans la zone de dialogue, vous choisissez **Copier** ou **Déplacer**, la destination et ce qui se passe en cas de conflit de noms. **Copier** est présélectionné – même au sein d’un même volume de votre Mac, où le Finder déplacerait. Ainsi, rien ne disparaît par mégarde de son dossier. **↩** confirme la présélection.
3. Maintenez **⌘** en relâchant pour présélectionner **Déplacer**. **⌥** présélectionne explicitement **Copier** (et l’emporte sur ⌘).

Une pastille près du pointeur indique ce qui est présélectionné (« Copier 3 éléments ici · maintenez ⌘ pour déplacer », avec ⌘ : « Déplacer 3 éléments ici · maintenez ⌥ pour copier »). Faire glisser hors d’une archive revient à extraire. Entre sous-fenêtres, les éléments vont dans le dossier affiché par la sous-fenêtre.

Si vous ne voulez pas que la question soit posée à chaque fois, choisissez « Toujours copier » ou « Toujours déplacer » sous **Lorsque vous déposez des fichiers** dans les [Réglages](16-einstellungen.md#kopieren-bewegen). La tâche démarre alors sans zone de dialogue ; **⌘** déplace et **⌥** copie toujours pour ce dépôt précis. Le dépôt sur la **Corbeille** et sur les **Favoris** n’est pas concerné.

### De Manta vers d’autres apps {#drag-raus}

Vous pouvez faire glisser des fichiers de Manta vers d’autres apps : sur le Bureau, dans une fenêtre du Finder, dans Mail ou dans le formulaire de téléversement d’un navigateur.

- Les **fichiers locaux** sont copiés – même si la destination se trouve sur le même volume. Maintenez **⌘** pour les déplacer.
- Les **fichiers sur des serveurs, dans des conteneurs ou des archives** sont téléchargés vers la destination au moment où vous relâchez. Cela peut prendre un moment pour les gros fichiers.

Faites glisser les éléments au-delà du bord de la fenêtre de Manta ; macOS prend alors le relais. Un message vous indique quand un téléchargement depuis un serveur commence et quand il est terminé ou a échoué. Si l’autre app a déplacé les fichiers, Manta met à jour le dossier dont ils proviennent.

### D’autres apps vers Manta {#drag-rein}

Déposez des fichiers depuis le Finder, le Bureau ou une autre app sur une sous-fenêtre :

- Sur un emplacement vide de la sous-fenêtre : ils vont dans le dossier affiché par la sous-fenêtre.
- Sur un dossier (dans toutes les présentations, et en présentation Colonnes aussi sur une colonne) : ils vont dans ce dossier. Celui-ci est mis en évidence pendant le survol.
- Les mêmes règles qu’entre sous-fenêtres s’appliquent : Manta demande avec **Copier** présélectionné (avec **⌘** : **Déplacer**), sauf si vous avez choisi « Toujours copier » ou « Toujours déplacer ».

La tâche apparaît dans Transferts, où vous pouvez la suspendre. Les pièces jointes de Mail et les photos de Photos ne peuvent pas encore être déposées sur Manta.

### Dans les Favoris

Faites glisser un dossier sur la section **Favoris** de la barre latérale pour l’y conserver. Voir [Favoris et barre latérale](07-favoriten-seitenleiste.md#favoriten-ziehen).

### Sur la Corbeille

Faites glisser des éléments sur **Corbeille** dans la barre latérale : les éléments locaux sont placés dans la corbeille sans confirmation, les éléments sur des serveurs sont supprimés définitivement après confirmation. Voir [Corbeille](07-favoriten-seitenleiste.md#papierkorb).

## Renommer {#umbenennen}

1. Sélectionnez **un seul** élément.
2. Appuyez sur **F2** (**Renommer…**) ou choisissez **Renommer…** dans le menu contextuel.
3. Tapez le nouveau nom et appuyez sur ↩.

**Renommer d’un clic**, comme dans le Finder : cliquez de nouveau sur un élément déjà sélectionné et patientez un instant. Dans la présentation par liste, le nom devient modifiable directement dans la ligne (le nom sans l’extension est sélectionné) ; ↩ ou un clic ailleurs renomme, Échap annule. Dans les autres présentations, la zone de dialogue **Renommer…** s’ouvre. Un double-clic ouvre toujours l’élément.

Manta vérifie immédiatement le nom : il ne doit pas être vide, contenir « / » ni exister déjà dans le dossier.

**Renommer de nombreux éléments à la fois** : sélectionnez-en plusieurs et appuyez sur **F2**, ou appuyez sur **⇧⌘R** (**Renommer par lot…**). Le plug-in propose des règles, un aperçu en direct et une fonction d’annulation. Voir [Renommage par lot](14-plugins.md#umbenennen).

## Nouveau dossier {#neuer-ordner}

1. Appuyez sur **⇧⌘N** (**Nouveau dossier…**).
2. Tapez un nom (« dossier sans titre » est proposé) et cliquez sur **Créer**.

## Nouveau fichier {#neue-datei}

1. Appuyez sur **⌥⌘N** (**Nouveau fichier…**) ou choisissez **Nouveau fichier…** dans le menu **Fichier** ou dans le menu contextuel d’une sous-fenêtre.
2. Tapez un nom (« fichier sans titre.txt » est proposé) et cliquez sur **Créer**. Manta crée un fichier vide et le sélectionne.

**Dans un dossier** : si vous cliquez avec le bouton droit sur un dossier, les commandes deviennent **Nouveau dossier dans « … »** et **Nouveau fichier dans « … »** ; le dossier ou le fichier est alors créé dans ce dossier et la sous-fenêtre reste où elle est. **⇧⌘N** et **⌥⌘N** créent toujours dans le dossier affiché.

## Supprimer {#loeschen}

Manta supprime comme le Finder :

| Où | ⌘⌫ (**Placer dans la corbeille**) | ⌥⌘⌫ (**Supprimer immédiatement…**) |
|---|---|---|
| Votre Mac (y compris les disques externes et les dossiers cloud) | Place les éléments dans la **corbeille**, sans confirmation. **⌘Z** (**Annuler**) ou la commande **Remettre** du Finder les restaure. | Supprime définitivement après confirmation. |
| Serveurs (SFTP, FTP, FTPS), S3, conteneurs | Il n’y a pas de corbeille : supprime définitivement après confirmation. | Supprime définitivement après confirmation. |
| Archives | Lecture seule : rien ne peut être supprimé. | Lecture seule. |

Pour placer des éléments dans la corbeille :

1. Sélectionnez-les.
2. Appuyez sur **⌘⌫** ou sur la touche de suppression vers l’avant **⌦**, ou choisissez **Placer dans la corbeille** dans le menu contextuel ou la barre d’outils – ou faites-les glisser sur **Corbeille** dans la barre latérale.

Un message le confirme (« 3 éléments placés dans la corbeille »). Pour récupérer les éléments, appuyez sur **⌘Z** (**Annuler**) : Manta les remet à leur place, voir [Annuler](#widerrufen). Pour des éléments plus anciens ou supprimés dans une autre app, ouvrez la corbeille dans le Finder, sélectionnez-les et choisissez **Remettre**.

Pour supprimer définitivement :

1. Sélectionnez les éléments.
2. Appuyez sur **⌥⌘⌫**, ou choisissez **Supprimer immédiatement…** dans la palette de commandes.
3. Manta demande : « Supprimer 1 élément (« nom ») ? » ou « Supprimer 3 éléments ? », en précisant que les éléments ne sont pas placés dans la corbeille et que l’opération est irréversible.
4. Cliquez sur **Supprimer**.

Sur un **serveur**, dans un **bucket S3** ou dans un **conteneur**, il n’y a pas de corbeille. **⌘⌫**, le menu contextuel (qui y affiche **Supprimer immédiatement…**) et le bouton de la barre d’outils demandent toujours confirmation, et la zone de dialogue indique en rouge que cet emplacement n’a pas de corbeille et que la suppression y est définitive. Les dossiers sont supprimés avec tout leur contenu.

Pour ouvrir la corbeille elle-même, cliquez sur **Corbeille** dans la barre latérale. À cet endroit, **⌘⌫** (qui y correspond à **Supprimer immédiatement…**) supprime définitivement après confirmation, et **⇧⌘⌫** (**Vider la corbeille…**) la vide. Voir [Corbeille](07-favoriten-seitenleiste.md#papierkorb).

> [!CAUTION]
> Dans les **dossiers cloud** (iCloud Drive, Google Drive, OneDrive, Dropbox), un élément placé dans la corbeille disparaît aussi du cloud dès que votre Mac se synchronise. **Remettre** le restaure généralement ; certains fournisseurs conservent aussi une corbeille sur leur site web. Voir [Stockage cloud](09-cloud.md).

## Annuler {#widerrufen}

**⌘Z** (**Annuler**) annule votre dernière opération sur les fichiers, comme dans le Finder, et **⇧⌘Z** (**Rétablir**) la refait. Les deux commandes se trouvent dans le menu **Édition** et dans la palette de commandes. Manta mémorise jusqu’à 100 étapes, jusqu’à ce que vous le quittiez.

| Ce que vous avez fait | Ce que fait **⌘Z** |
|---|---|
| **Placer dans la corbeille** (**⌘⌫**, ou glisser sur **Corbeille** dans la barre latérale) | Remet les éléments à leur emplacement d’origine. |
| Renommer (**Renommer…**, renommer d’un clic, fenêtre d’informations) | Rétablit l’ancien nom, y compris sur les serveurs. |
| **Nouveau dossier…** ou **Nouveau fichier…** sur votre Mac | Place le nouvel élément dans la corbeille, comme le Finder, avec tout ce que vous y avez mis. **⇧⌘Z** le récupère. |
| **Nouveau dossier…** ou **Nouveau fichier…** sur un serveur, dans S3 ou dans un conteneur | Il n’y a pas de corbeille : Manta supprime le nouvel élément, mais seulement tant qu’il est vide. Sinon, Manta indique qu’il n’a pas pu le supprimer et le laisse en place. |

Ne peuvent pas être annulés : la copie et le déplacement (**F5**, **F6**, glisser-déposer) et tout ce qui est définitif : **Supprimer immédiatement…**, la suppression sur les serveurs et **Vider la corbeille…**. **Renommer par lot…** a sa propre fonction d’annulation, voir [Renommage par lot](14-plugins.md#umbenennen).

Si l’emplacement d’origine est de nouveau occupé entre-temps (un élément du même nom s’y trouve), Manta ne remplace rien. Il affiche « Impossible de remettre « nom » à sa place » et conserve l’étape : dès que la place est libre, **⌘Z** fonctionne de nouveau.

- Dans un **champ de texte** (filtre, chemin, nom en cours de modification), **⌘Z** et **⇧⌘Z** annulent la saisie, pas les opérations sur les fichiers. Tant que le **terminal** est actif, **⌘Z** s’adresse au terminal.
- Sans [Accès complet au disque](07-favoriten-seitenleiste.md#papierkorb-zugriff), Manta remet les éléments de la corbeille à leur place via le Finder, comme pour **Vider la corbeille…** : macOS demande une fois si Manta peut contrôler le Finder.

## Tags {#tags}

Manta lit et attribue les mêmes tags que le Finder (ils sont enregistrés avec le fichier : le Finder, Spotlight et les autres apps les voient donc). Les éléments dotés de tags affichent des pastilles de couleur à côté de leur nom dans toutes les présentations – après le nom en Liste et en Colonnes, avant le nom en Icônes et en Galerie. Les tags fonctionnent pour les éléments de ce Mac, y compris iCloud Drive et les autres dossiers cloud ; les serveurs et les archives n’en ont pas.

**Ajouter ou supprimer un tag :**

1. Sélectionnez les éléments et ouvrez le menu contextuel.
2. Choisissez **Tags**. Les sept couleurs se trouvent en haut : un clic ajoute le tag à tous les éléments sélectionnés, ou le supprime si tous l’ont déjà (une coche signifie que tous l’ont, un tiret que certains l’ont). Vos autres tags sont listés en dessous.
3. Pour créer un tag, tapez son nom dans **Nouveau tag…** et appuyez sur **↩**.

Les tags standard portent les noms que le Finder utilise sur votre Mac (par exemple « Rouge » sur un Mac en français, « Red » sur un Mac en anglais).

**Trouver les éléments dotés d’un tag :** la section **Tags** de la barre latérale liste vos tags tels que le Finder les affiche (si Manta ne peut pas lire la liste du Finder, ce sont les tags qu’il a rencontrés sur des fichiers). Un clic affiche dans la sous-fenêtre active tous les éléments dotés de ce tag, trouvés par Spotlight. Dans le champ de filtre, `tag:Rouge` affiche les éléments du dossier actuel qui portent ce tag (voir [Filtrer les fichiers](04-navigation.md#filtern)).

## Lire les informations {#informationen}

**⌘I** (**Lire les informations**, également dans le menu contextuel et le menu Fichier) ouvre une fenêtre d’informations sur la sélection, comme dans le Finder : une fenêtre par élément (la redemander la ramène au premier plan), une fenêtre récapitulative pour plusieurs éléments. Six fenêtres d’informations au maximum peuvent être ouvertes ; la plus ancienne se ferme pour laisser place à une nouvelle. Ce sont des fenêtres ordinaires : vous pouvez continuer à travailler dans la fenêtre principale.

Une fenêtre d’informations affiche :

- le **nom** – modifiez-le et appuyez sur **↩** pour renommer l’élément,
- **Type**, **Taille** (les dossiers et les sélections multiples sont calculés en arrière-plan), **Emplacement**, **Création**, **Modification** et **Dernière ouverture** (la date de Spotlight),
- **Tags** – supprimez-en un avec son ×, ajoutez-en un avec **Ajouter un tag**,
- **Verrouillé** (l’élément ne peut être ni modifié, ni renommé, ni placé dans la corbeille) et **Masqué**,
- **Commentaires** – le commentaire Spotlight défini dans le Finder (Manta l’affiche mais ne peut pas le modifier : macOS n’offre aucun moyen de le faire, hormis en pilotant le Finder par script),
- **Ouvrir avec** – l’app par défaut, et le menu **Ouvrir avec** pour ouvrir le fichier une fois avec une autre app. macOS ne permet pas aux autres apps de changer, par une interface publique, l’app d’un seul fichier ; il n’y a donc pas de bouton « Tout modifier… » : utilisez le Finder pour cela,
- **Partage et permissions** – lecture, écriture et exécution pour le propriétaire, le groupe et tout le monde, ainsi que la valeur octale (par exemple `644`). Sur ce Mac et sur les serveurs SFTP, vous pouvez les modifier directement (chmod) ; ailleurs, elles sont affichées en lecture seule.

Pour les éléments situés sur des serveurs, dans des conteneurs et dans des archives, la fenêtre affiche ce que le serveur indique (taille, date, permissions).

## Emplacements en lecture seule {#schreibgeschuetzt}

Dans les archives, les conteneurs arrêtés et les conteneurs sans shell, vous ne pouvez rien modifier. Une pastille **Lecture seule** (ou **Arrêté**) dans la barre du chemin d’accès l’indique. La copie vers l’extérieur avec **F5** ou par glisser-déposer fonctionne toujours.

Comme le Finder, **Applications** affiche aussi les apps d’Apple (Calculette, Mail…) et **Utilitaires** celles de macOS. Elles se trouvent sur le volume système protégé : vous pouvez les ouvrir et les copier, mais pas les renommer, les déplacer ni les supprimer.

## Voir aussi

- [Archives](10-archive.md) : compresser et extraire.
- [Terminal](11-terminal.md#dateiaktionen) : voir chaque action sur fichier sous forme de commande shell.
