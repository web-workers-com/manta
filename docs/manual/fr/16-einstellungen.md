# Réglages {#einstellungen}

Tous les réglages de Manta, section par section. Toutes les modifications s’appliquent immédiatement ; vous n’avez jamais besoin de redémarrer.

## Ouvrir les Réglages {#oeffnen}

- **⌘,**
- **Manta** → **Réglages…** dans la barre des menus
- la roue dentée dans la barre d’outils
- **Réglages…** dans la palette de commandes

Les Réglages s’affichent dans une fenêtre à part entière comprenant les sections **Général** et **Plug-ins**. Fermez-la avec le bouton rouge de la fenêtre ou **⌘W**. **⌘,** la ramène au premier plan.

Tant que les Réglages sont ouverts, la fenêtre principale attend : elle est légèrement assombrie, et les clics et raccourcis n’y ont aucun effet – un clic ramène les Réglages au premier plan avec le son d’alerte. Fermez les Réglages pour continuer. La fenêtre d’aide, elle, peut rester ouverte pendant que vous travaillez.

![Réglages → Général](images/settings-general.png)

## Général {#allgemein}

### Apparence {#erscheinungsbild}

| Réglage | Options |
|---|---|
| **Apparence** | **Automatique** (suit le mode clair ou sombre du système), **Clair**, **Sombre**. Aussi dans la palette de commandes : « Apparence : Clair/Sombre/Automatique (Système) ». |
| **Couleur d’accentuation** | **Bleu**, **Violet**, **Rose**, **Rouge**, **Orange**, **Jaune**, **Vert**, **Graphite** – pour la sélection, les boutons et le focus. Chaque couleur reste lisible en mode clair comme en mode sombre. |

### Langue {#sprache}

**Langue** : **Système** (par défaut) ou l’une des langues proposées, chacune sous son propre nom (**Deutsch**, **English**, **简体中文**, **Español**, **Français**, **Italiano**, **Русский**). Le changement s’applique immédiatement. « Système » suit vos langues préférées dans macOS (ou l’anglais si Manta n’en parle aucune). Les dates (« Aujourd’hui 14:02 »), les tailles (« 1,2 Mo ») et les nombres suivent la langue. Les plug-ins intégrés changent de langue en même temps ; les noms de fichier, la sortie du shell et vos propres actions rapides restent tels quels.

### Gestionnaire de fichiers par défaut {#standard-dateimanager}

**Utiliser Manta comme gestionnaire de fichiers par défaut** : lorsque cette option est activée, « Afficher dans le Finder » et les dossiers ouverts par d’autres apps s’ouvrent dans Manta. En dessous, vous voyez quelle app a actuellement ce rôle (« Actuellement : … ») et quelle app le récupère si vous désactivez l’option. Ce que macOS permet est expliqué dans [Premiers pas](01-erste-schritte.md#standard-dateimanager).

Le même interrupteur se trouve dans le menu **Manta** (**Manta comme gestionnaire de fichiers par défaut**, coché lorsque l’option est active) et dans la palette de commandes (**Utiliser Manta comme gestionnaire de fichiers par défaut…**, ou **Rendre le rôle de gestionnaire de fichiers par défaut à…** lorsque Manta l’a). L’activation depuis ces emplacements pose la même question qu’au premier lancement, avec **Annuler** ; la désactivation rend le rôle immédiatement.

Disponible uniquement si Manta est exécuté depuis le dossier **Applications** (« Déplacez d’abord Manta dans le dossier Applications »).

### Fichiers {#dateien}

| Réglage | Signification |
|---|---|
| **Afficher les fichiers masqués** | Pour les nouveaux onglets et au lancement. **⇧⌘.** les affiche ou les masque dans l’onglet actuel. |
| **Ouvrir les archives comme des dossiers** | Retour ou un double-clic parcourt les archives ZIP et TAR dans la sous-fenêtre (en lecture seule). **⌥↩** les ouvre avec l’app par défaut. Voir [Archives](10-archive.md). |
| **Les nouveaux onglets s’ouvrent dans** | Où s’ouvrent **⌘T**, **Nouvel onglet** dans le menu du Dock et un démarrage sans session enregistrée : **Le même dossier** (par défaut ; au démarrage, votre dossier de départ), **Départ**, **Bureau**, **Documents**, **Téléchargements** ou **Autre…** (un dossier de votre choix). Comme « Les nouvelles fenêtres de Finder affichent » dans le Finder. |
| **Mémoriser la présentation par dossier** | Chaque dossier s’ouvre dans la présentation dans laquelle il a été affiché pour la dernière fois (Icônes, Liste, Colonnes, Galerie). Désactivé (par défaut) : la présentation appartient à la sous-fenêtre. Voir [Présentations](03-ansichten.md#pro-ordner). |
| **Boutons latéraux de la souris** | Les boutons du pouce reculent et avancent dans la sous-fenêtre sous le pointeur, comme **⌘[** et **⌘]**, aussi dans l’Aide. **Précédent / Suivant** (par défaut), **Suivant / Précédent (inversés)** ou **Désactivés**. |
| **Charger les vignettes des fichiers sur serveur** | Télécharge les petites images (moins de 5 Mo) en arrière-plan. Désactivé (par défaut) : les fichiers de serveur affichent l’icône de leur type. Voir [Vignettes](03-ansichten.md#server-vorschaubilder). |
| **Icônes de fichiers de macOS** | Les fichiers affichent l’icône que macOS associe à leur type, les apps la leur, comme dans le Finder, aussi sur les serveurs. Désactivé : les symboles de Manta. Par défaut : activé. |

### Aperçu {#vorschau}

**La barre d’espace ouvre** : **Coup d’œil (macOS)** ou **Aperçu intégré**. Les fichiers sur des serveurs et dans des archives utilisent toujours l’aperçu intégré. Voir [Coup d’œil et aperçu](06-schnellansicht.md).

### Copie et déplacement {#kopieren-bewegen}

**Lorsque vous déposez des fichiers** : **Demander (Copier présélectionné)** (par défaut), **Toujours copier** ou **Toujours déplacer**. S’applique au glisser-déposer entre sous-fenêtres et depuis d’autres apps. « Demander » affiche la zone de dialogue avec Copier / Déplacer ; les deux réglages « Toujours » démarrent immédiatement. **⌘** pendant le dépôt déplace, **⌥** copie, quel que soit le réglage. Le dépôt sur la corbeille et sur les Favoris n’est pas concerné. Voir [Glisser-déposer](05-dateien.md#drag-drop).

**Si un élément existe déjà** : **Demander** (par défaut), **Ignorer**, **Remplacer si plus récent**, **Remplacer** ou **Garder les deux**. Cette option est présélectionnée lorsque vous copiez ou déplacez (y compris lors d’un dépôt) ; vous pouvez toujours la modifier à chaque fois. Lorsqu’un dépôt démarre sans zone de dialogue (« Toujours copier » / « Toujours déplacer »), ce réglage s’applique. Avec **Demander**, une tâche s’arrête à chaque élément déjà existant et vous demande quoi faire. Les nouvelles installations demandent ; si vous utilisiez déjà Manta, votre réglage enregistré est conservé (y compris l’ancienne valeur par défaut **Ignorer**) – modifiez-le ici. Voir [Conflits de noms](05-dateien.md#konflikte).

### Cloud {#cloud}

**Google Drive** : indique si Google Drive pour ordinateur est configuré, sinon affiche le lien de téléchargement. **Suggestion dans la barre latérale** active ou désactive la ligne « Connecter Google Drive… ». **Expert : votre propre client OAuth Google** est réservé aux utilisateurs avancés, voir [Google Drive pour les experts](09-cloud.md#experte).

### Mises à jour {#updates}

| Réglage | Signification |
|---|---|
| **Rechercher les mises à jour au lancement** | Discrètement, en arrière-plan. Rien n’est installé sans votre accord. Par défaut : activé. |
| **Canal** | **Stable** ou **Bêta**. La version bêta propose les nouvelles fonctionnalités plus tôt, mais peut contenir des bogues. |
| **Rechercher** | Lance immédiatement la recherche. À côté : « Dernière recherche : … ». |
| **Ignorée : Manta…** / **Réinitialiser** | Apparaît lorsque vous avez ignoré une version. Cette version et les plus anciennes ne sont plus proposées tant que vous ne cliquez pas sur **Réinitialiser**. |

Si votre version ne contient pas la clé de signature des mises à jour, cette section indique « Les mises à jour ne sont pas configurées dans cette version ». Voir [Mises à jour](17-updates.md).

### À propos de Manta {#ueber}

Manta est gratuit et open source. **Site web et assistance** → **manta-filemanager.com** ouvre [le site web](https://manta-filemanager.com/) avec actualités, aide et contact. **Licences de tiers** → **Afficher** ouvre les mentions de licence des composants open source sur lesquels repose Manta. **Code source** → **GitHub** ouvre [le dépôt](https://github.com/web-workers-com/manta) ; les contributions sont les bienvenues. **Licence MIT** → **Lire** affiche la licence : vous pouvez utiliser, modifier et partager Manta librement. Le site web, le code source et les licences de tiers figurent aussi dans le menu **Aide**. **Commentaires** → **Signaler un problème** ouvre le formulaire de rapport de bug de GitHub avec les versions de Manta et de macOS renseignées, voir [Signaler un bug](19-fehlerbehebung.md#fehler-melden).

## Plug-ins {#plugins}

En haut : **Installer un plug-in…**, **Ouvrir le dossier des plug-ins** et **Recharger**. En dessous, trois sections :

- **Intégrés** – les plug-ins intégrés (Git, Terminal, Tailles des dossiers, Renommage par lot, Aperçu Markdown, Actions rapides, Infos projet, Outils d’image, Doublons, Charge du système).
- **Installés** – les plug-ins externes que vous avez installés.
- **Disponibles** – les plug-ins fournis avec Manta, prêts à être installés.

Chaque carte affiche le nom, la description, l’état (**Actif**, **Démarrage…**, **Désactivé**, **Erreur**) et les boutons **Activer**/**Désactiver**, **Réglages** (si disponible) et **Afficher le journal**. Plus d’informations dans [Vos propres plug-ins](15-eigene-plugins.md).

### Réglages des plug-ins {#plugin-einstellungen}

| Plug-in | Réglages |
|---|---|
| **Terminal** | Shell, Taille de police, Historique, Utiliser Option comme touche Méta, La sous-fenêtre suit « cd » du terminal, Afficher les actions sur fichier dans le terminal – voir [Terminal](11-terminal.md#einstellungen). |
| **Tailles des dossiers** | **Calculer automatiquement la taille des dossiers** ou **Uniquement sur demande** – voir [Tailles des dossiers](14-plugins.md#ordnergroessen). |
| **Aperçu Markdown** | **Charger les images depuis le Web (http/https)** – désactivé par défaut. |
| **Doublons** | **Ignorer les paquets (.app, .photoslibrary, …)**, **Inclure les fichiers masqués**, **Modifier settings.json…** pour les exclusions et une commande de corbeille personnalisée. |
| **Charge du système** | **Afficher la charge de la carte graphique** – voir [Charge du système](14-plugins.md#systemauslastung). |

## Accessibilité {#bedienungshilfen}

Manta suit **Réglages Système › Accessibilité**, sans réglage propre :

- **VoiceOver** : chaque bouton, champ, liste et dialogue porte un nom dans ta langue. Les lignes de fichiers sont lues en entier, p. ex. « README.md, Texte, 214 octets, modifié Aujourd’hui 09:41, sélectionné », état du cloud et de Git compris. Quand tu te déplaces avec les flèches, VoiceOver lit la ligne courante. Les messages comme « 3 éléments copiés » et les erreurs sont annoncés.
- **Réduire les animations** (Affichage) : plus d’animations – le clignotement d’un favori, la pulsation du badge d’envoi et les fondus disparaissent.
- **Augmenter le contraste** (Affichage) : bordures et séparateurs plus marqués, texte secondaire plus foncé (plus clair en mode sombre), anneau de focus plus visible.
- **Réduire la transparence** (Affichage) : plus d’assombrissement translucide derrière les dialogues.

Manta applique les changements aussitôt, sans redémarrage. Même sans ces réglages, le texte atteint un rapport de contraste d’au moins 4,5:1. Pour l’utilisation au clavier seul, voir [Raccourcis clavier](18-tastenkuerzel.md#tastatur).

## Où Manta conserve ses réglages {#dateien-speicherort}

Tout se trouve dans `~/Library/Application Support/Manta/` :

| Fichier | Contenu |
|---|---|
| `settings.json` | Réglages généraux |
| `favorites.json` | Favoris et connexions enregistrées (sans mots de passe – ceux-ci sont dans le trousseau) |
| `session.json` | Sous-fenêtres, onglets et largeurs lors de la dernière fermeture |
| `plugins.json` | Plug-ins activés, état des panneaux |
| `plugin-data/` | Données des plug-ins, comme `actions.json` |
| `update-state.json` | Canal de mise à jour, dernière recherche, version ignorée |
| `default-viewer.json` | Gestionnaire de fichiers par défaut et app précédente |
| `folder-views.json` | Présentation mémorisée par dossier (avec **Mémoriser la présentation par dossier**) |
| `transfers.json` | Transferts interrompus à reprendre (sans mots de passe), voir [Transferts](05-dateien.md#neustart) |

Les vignettes sont conservées séparément dans `~/Library/Caches/Manta/thumbs` et peuvent être supprimées à tout moment.

> [!TIP]
> Pour transférer Manta sur un autre Mac, copiez ce dossier pendant que Manta n’est pas ouvert.
