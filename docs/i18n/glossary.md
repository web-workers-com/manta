# UI glossary

Generated from the string catalogs (`crates/app/src/i18n/catalog`, `crates/plugin-*/src/i18n`) by the test `i18n::glossary::glossary_is_up_to_date` — do not edit by hand; run `MANTA_UPDATE_GLOSSARY=1 cargo test -p manta glossary`.

Short UI terms only (menus, commands, buttons, settings labels, dialog titles, sidebar sections, columns); sentences, tooltips and errors are left out. Quote these exactly in the manual. `{name}`-style placeholders are filled in by the app. Plural keys (`.one`, `.other`) show the form for that key; Russian also has `.few` / `.many` forms in the catalog.

## App: Common

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `common.ok` | OK | OK | 好 | Aceptar | OK | OK | ОК |
| `common.cancel` | Cancel | Abbrechen | 取消 | Cancelar | Annuler | Annulla | Отменить |
| `common.close` | Close | Schließen | 关闭 | Cerrar | Fermer | Chiudi | Закрыть |
| `language.system` | System | System | 系统 | Sistema | Système | Sistema | Системный |
| `appearance.system` | Auto | Automatisch | 自动 | Automático | Automatique | Automatico | Авто |
| `appearance.light` | Light | Hell | 浅色 | Claro | Clair | Chiaro | Светлое |
| `appearance.dark` | Dark | Dunkel | 深色 | Oscuro | Sombre | Scuro | Темное |
| `accent.blue` | Blue | Blau | 蓝色 | Azul | Bleu | Blu | Синий |
| `accent.purple` | Purple | Lila | 紫色 | Morado | Violet | Viola | Фиолетовый |
| `accent.pink` | Pink | Rosa | 粉色 | Rosa | Rose | Rosa | Розовый |
| `accent.red` | Red | Rot | 红色 | Rojo | Rouge | Rosso | Красный |
| `accent.orange` | Orange | Orange | 橙色 | Naranja | Orange | Arancione | Оранжевый |
| `accent.yellow` | Yellow | Gelb | 黄色 | Amarillo | Jaune | Giallo | Желтый |
| `accent.green` | Green | Grün | 绿色 | Verde | Vert | Verde | Зеленый |
| `accent.graphite` | Graphite | Graphit | 石墨色 | Grafito | Graphite | Grafite | Графитовый |
| `date.today` | Today {time} | Heute {time} | 今天 {time} | Hoy {time} | Aujourd’hui {time} | Oggi {time} | Сегодня {time} |
| `date.yesterday` | Yesterday {time} | Gestern {time} | 昨天 {time} | Ayer {time} | Hier {time} | Ieri {time} | Вчера {time} |

## App: Commands and menus

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `cmd.open` | Open | Öffnen | 打开 | Abrir | Ouvrir | Apri | Открыть |
| `cmd.quick_look` | Quick Look | Schnellansicht | 快速查看 | Vista Rápida | Coup d’œil | Visualizzazione rapida | Быстрый просмотр |
| `cmd.get_info` | Get Info | Informationen | 显示简介 | Obtener información | Lire les informations | Ottieni informazioni | Свойства |
| `cmd.enclosing_folder` | Enclosing Folder | Übergeordneter Ordner | 上层文件夹 | Carpeta contenedora | Dossier parent | Cartella superiore | Папка уровнем выше |
| `cmd.back` | Back | Zurück | 返回 | Atrás | Précédent | Indietro | Назад |
| `cmd.forward` | Forward | Vorwärts | 前进 | Adelante | Suivant | Avanti | Вперед |
| `cmd.refresh` | Refresh | Aktualisieren | 刷新 | Actualizar | Actualiser | Aggiorna | Обновить |
| `cmd.copy_to_other` | Copy to Other Pane… | In anderen Bereich kopieren … | 拷贝到另一个面板… | Copiar al otro panel… | Copier dans l’autre sous-fenêtre… | Copia nell’altro pannello… | Копировать в другую панель… |
| `cmd.move_to_other` | Move to Other Pane… | In anderen Bereich bewegen … | 移到另一个面板… | Mover al otro panel… | Déplacer dans l’autre sous-fenêtre… | Sposta nell’altro pannello… | Переместить в другую панель… |
| `cmd.rename` | Rename… | Umbenennen … | 重新命名… | Renombrar… | Renommer… | Rinomina… | Переименовать… |
| `cmd.new_folder` | New Folder… | Neuer Ordner … | 新建文件夹… | Nueva carpeta… | Nouveau dossier… | Nuova cartella… | Новая папка… |
| `cmd.new_file` | New File… | Neue Datei … | 新建文件… | Nuevo archivo… | Nouveau fichier… | Nuovo file… | Новый файл… |
| `cmd.move_to_trash` | Move to Trash | In den Papierkorb legen | 移到废纸篓 | Trasladar a la papelera | Placer dans la corbeille | Sposta nel Cestino | Переместить в Корзину |
| `cmd.delete_permanently` | Delete Immediately… | Sofort löschen … | 立即删除… | Eliminar inmediatamente… | Supprimer immédiatement… | Elimina immediatamente… | Удалить немедленно… |
| `cmd.empty_trash` | Empty Trash… | Papierkorb entleeren … | 清倒废纸篓… | Vaciar papelera… | Vider la corbeille… | Svuota il Cestino… | Очистить Корзину… |
| `cmd.undo` | Undo | Widerrufen | 撤销 | Deshacer | Annuler | Annulla | Отменить |
| `cmd.redo` | Redo | Wiederholen | 重做 | Rehacer | Rétablir | Ripeti | Повторить |
| `cmd.select_all` | Select All | Alles auswählen | 全选 | Seleccionar todo | Tout sélectionner | Seleziona tutto | Выбрать все |
| `cmd.toggle_hidden` | Show Hidden Files | Unsichtbare Dateien einblenden | 显示隐藏文件 | Mostrar archivos ocultos | Afficher les fichiers masqués | Mostra file nascosti | Показать скрытые файлы |
| `cmd.new_tab` | New Tab | Neuer Tab | 新建标签页 | Nueva pestaña | Nouvel onglet | Nuova scheda | Новая вкладка |
| `cmd.close_tab` | Close Tab | Tab schließen | 关闭标签页 | Cerrar pestaña | Fermer l’onglet | Chiudi scheda | Закрыть вкладку |
| `cmd.switch_pane` | Switch Pane | Bereich wechseln | 切换面板 | Cambiar de panel | Changer de sous-fenêtre | Cambia pannello | Сменить панель |
| `cmd.edit_path` | Edit Path | Pfad bearbeiten | 编辑路径 | Editar ruta | Modifier le chemin | Modifica percorso | Изменить путь |
| `cmd.go_to_path` | Go to Path… | Gehe zum Pfad … | 前往路径… | Ir a la ruta… | Aller au chemin… | Vai al percorso… | Перейти по пути… |
| `cmd.filter` | Filter Files | Dateien filtern | 过滤文件 | Filtrar archivos | Filtrer les fichiers | Filtra file | Фильтровать файлы |
| `cmd.add_to_favorites` | Add Folder to Favorites | Ordner zu Favoriten hinzufügen | 将文件夹添加到个人收藏 | Añadir carpeta a Favoritos | Ajouter le dossier aux Favoris | Aggiungi cartella ai Preferiti | Добавить папку в Избранное |
| `cmd.connect` | Connect to Server… | Mit Server verbinden … | 连接服务器… | Conectarse al servidor… | Se connecter au serveur… | Connessione al server… | Подключение к серверу… |
| `cmd.transfers` | Show Transfers | Übertragungen einblenden | 显示传输 | Mostrar transferencias | Afficher les transferts | Mostra trasferimenti | Показать передачи |
| `cmd.compare_sync` | Compare & Sync… | Vergleichen und synchronisieren … | 比较与同步… | Comparar y sincronizar… | Comparer et synchroniser… | Confronta e sincronizza… | Сравнить и синхронизировать… |
| `cmd.command_palette` | Command Palette | Befehlspalette | 命令面板 | Paleta de comandos | Palette de commandes | Palette dei comandi | Палитра команд |
| `cmd.settings` | Settings… | Einstellungen … | 设置… | Ajustes… | Réglages… | Impostazioni… | Настройки… |
| `cmd.help` | Open Manual | Handbuch öffnen | 打开手册 | Abrir el manual | Ouvrir le manuel | Apri il manuale | Открыть руководство |
| `cmd.support` | Manta Website & Support… | Manta-Website & Support … | Manta 网站与支持… | Sitio web y soporte de Manta… | Site web et assistance Manta… | Sito web e supporto di Manta… | Сайт и поддержка Manta… |
| `cmd.source_code` | Source Code on GitHub… | Quellcode auf GitHub … | GitHub 上的源代码… | Código fuente en GitHub… | Code source sur GitHub… | Codice sorgente su GitHub… | Исходный код на GitHub… |
| `cmd.report_issue` | Report an Issue… | Fehler melden … | 报告问题… | Informar de un problema… | Signaler un problème… | Segnala un problema… | Сообщить о проблеме… |
| `cmd.suggest_idea` | Suggest an Idea… | Idee vorschlagen … | 提出建议… | Sugerir una idea… | Suggérer une idée… | Suggerisci un’idea… | Предложить идею… |
| `cmd.third_party_licenses` | Third-Party Licences | Lizenzen von Drittanbietern | 第三方许可 | Licencias de terceros | Licences de tiers | Licenze di terze parti | Лицензии сторонних компонентов |
| `cmd.reload_plugins` | Reload Plugins | Plugins neu laden | 重新载入插件 | Recargar plugins | Recharger les plug-ins | Ricarica i plug-in | Перезагрузить плагины |
| `cmd.browse_plugins` | Browse Plugins… | Plugins entdecken … | 浏览插件… | Explorar plugins… | Parcourir les plug-ins… | Sfoglia i plug-in… | Каталог плагинов… |
| `cmd.compress` | Compress… | Komprimieren … | 压缩… | Comprimir… | Compresser… | Comprimi… | Сжать… |
| `cmd.extract_here` | Extract Here | Hier entpacken | 在此处解压缩 | Extraer aquí | Extraire ici | Estrai qui | Извлечь сюда |
| `cmd.open_with_app` | Open With Default App | Mit Standard-App öffnen | 使用默认 App 打开 | Abrir con la app predeterminada | Ouvrir avec l’app par défaut | Apri con l’app predefinita | Открыть в программе по умолчанию |
| `cmd.open_shell_in_container` | Open Shell in Container | Shell im Container öffnen | 在容器中打开 Shell | Abrir shell en el contenedor | Ouvrir un shell dans le conteneur | Apri shell nel container | Открыть оболочку в контейнере |
| `cmd.appearance_light` | Appearance: Light | Erscheinungsbild: Hell | 外观：浅色 | Apariencia: Claro | Apparence : Clair | Aspetto: Chiaro | Оформление: светлое |
| `cmd.appearance_dark` | Appearance: Dark | Erscheinungsbild: Dunkel | 外观：深色 | Apariencia: Oscuro | Apparence : Sombre | Aspetto: Scuro | Оформление: темное |
| `cmd.appearance_system` | Appearance: Auto (System) | Erscheinungsbild: Automatisch (System) | 外观：自动（系统） | Apariencia: Automático (sistema) | Apparence : Automatique (Système) | Aspetto: Automatico (Sistema) | Оформление: авто (системное) |
| `cmd.shortcut.space` | Space | Leertaste | 空格键 | Espacio | Espace | Spazio | Пробел |
| `cmd.cat.navigate` | Navigate | Navigieren | 导航 | Navegar | Navigation | Naviga | Навигация |
| `cmd.cat.file` | File | Ablage | 文件 | Archivo | Fichier | File | Файл |
| `cmd.cat.select` | Select | Auswahl | 选择 | Selección | Sélection | Selezione | Выбор |
| `cmd.cat.view` | View | Darstellung | 显示 | Visualización | Présentation | Vista | Вид |
| `cmd.cat.go` | Go | Gehe zu | 前往 | Ir | Aller | Vai | Переход |
| `cmd.cat.tools` | Tools | Werkzeuge | 工具 | Herramientas | Outils | Strumenti | Инструменты |

## App: Main window

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `app.local` | Local | Lokal | 本地 | Local | Local | Locale | Локально |
| `app.disconnected` | Disconnected | Getrennt | 已断开连接 | Desconectado | Déconnecté | Disconnesso | Отключено |
| `app.backend_archive` | {name} (read-only archive) | {name} (schreibgeschütztes Archiv) | {name}（只读归档） | {name} (archivo comprimido de solo lectura) | {name} (archive en lecture seule) | {name} (archivio di sola lettura) | {name} (архив только для чтения) |
| `app.chip.stopped` | Stopped | Gestoppt | 已停止 | Detenido | Arrêté | Arrestato | Остановлен |
| `app.chip.read_only` | Read-only | Schreibgeschützt | 只读 | Solo lectura | Lecture seule | Sola lettura | Только чтение |
| `toolbar.copy_right` | Copy to Right Pane | In rechten Bereich kopieren | 拷贝到右侧面板 | Copiar al panel derecho | Copier dans la sous-fenêtre de droite | Copia nel pannello destro | Копировать в правую панель |
| `toolbar.move_right` | Move to Right Pane | In rechten Bereich bewegen | 移到右侧面板 | Mover al panel derecho | Déplacer dans la sous-fenêtre de droite | Sposta nel pannello destro | Переместить в правую панель |
| `toolbar.copy_left` | Copy to Left Pane | In linken Bereich kopieren | 拷贝到左侧面板 | Copiar al panel izquierdo | Copier dans la sous-fenêtre de gauche | Copia nel pannello sinistro | Копировать в левую панель |
| `toolbar.move_left` | Move to Left Pane | In linken Bereich bewegen | 移到左侧面板 | Mover al panel izquierdo | Déplacer dans la sous-fenêtre de gauche | Sposta nel pannello sinistro | Переместить в левую панель |
| `toolbar.hide_hidden` | Hide Hidden Files | Unsichtbare Dateien ausblenden | 不显示隐藏文件 | Ocultar archivos ocultos | Masquer les fichiers masqués | Nascondi file nascosti | Скрыть скрытые файлы |
| `menu.about` | About {app} | Über {app} | 关于 {app} | Acerca de {app} | À propos de {app} | Informazioni su {app} | О программе {app} |
| `menu.services` | Services | Dienste | 服务 | Servicios | Services | Servizi | Службы |
| `menu.hide` | Hide {app} | {app} ausblenden | 隐藏 {app} | Ocultar {app} | Masquer {app} | Nascondi {app} | Скрыть {app} |
| `menu.hide_others` | Hide Others | Andere ausblenden | 隐藏其他 | Ocultar otros | Masquer les autres | Nascondi altre | Скрыть остальные |
| `menu.show_all` | Show All | Alle einblenden | 全部显示 | Mostrar todo | Tout afficher | Mostra tutte | Показать все |
| `menu.quit` | Quit {app} | {app} beenden | 退出 {app} | Salir de {app} | Quitter {app} | Esci da {app} | Завершить {app} |
| `menu.quit_keep` | Quit and Keep Windows | Beenden und Fenster behalten | 退出并保留窗口 | Salir y mantener ventanas | Quitter et garder les fenêtres | Esci e mantieni le finestre | Завершить, сохранив окна |
| `toolbar.commands` | Commands | Befehle | 命令 | Comandos | Commandes | Comandi | Команды |
| `toolbar.transfers_active` | Show Transfers – {n} active | Übertragungen einblenden – {n} aktiv | 显示传输（{n} 个进行中） | Mostrar transferencias – {n} activas | Afficher les transferts – actifs : {n} | Mostra trasferimenti – {n} attivi | Показать передачи – активно: {n} |
| `toolbar.connect` | Connect | Verbinden | 连接 | Conectar | Se connecter | Connetti | Подключиться |
| `a11y.modified` | modified {when} | geändert {when} | 修改于 {when} | modificado {when} | modifié {when} | modificato {when} | изменено {when} |
| `a11y.file_list` | Files in {folder} | Dateien in {folder} | “{folder}”中的文件 | Archivos en {folder} | Fichiers dans {folder} | File in {folder} | Файлы в «{folder}» |
| `a11y.connected` | connected | verbunden | 已连接 | conectado | connecté | connesso | подключено |
| `a11y.pane_divider` | Pane divider | Trennlinie zwischen Bereichen | 面板分隔线 | Divisor de paneles | Séparateur de sous-fenêtres | Divisore dei pannelli | Разделитель панелей |
| `a11y.resize_panel` | Resize panel | Bereichsgröße ändern | 调整面板大小 | Cambiar tamaño del panel | Redimensionner le panneau | Ridimensiona pannello | Изменить размер панели |
| `a11y.toolbar` | Toolbar | Symbolleiste | 工具栏 | Barra de herramientas | Barre d’outils | Barra strumenti | Панель инструментов |
| `a11y.sidebar` | Sidebar | Seitenleiste | 边栏 | Barra lateral | Barre latérale | Barra laterale | Боковое меню |
| `a11y.status_bar` | Status bar | Statusleiste | 状态栏 | Barra de estado | Barre d’état | Barra di stato | Строка состояния |
| `a11y.pane` | Pane: {folder} | Bereich: {folder} | 面板：{folder} | Panel: {folder} | Sous-fenêtre : {folder} | Pannello: {folder} | Панель: {folder} |
| `a11y.active` | active | aktiv | 当前 | activo | sélectionnée | attivo | активна |
| `a11y.sections` | Sections of {chapter} | Abschnitte von {chapter} | “{chapter}”的小节 | Secciones de {chapter} | Sections de {chapter} | Sezioni di {chapter} | Разделы главы «{chapter}» |
| `crash.title` | Manta quit unexpectedly last time | Manta wurde beim letzten Mal unerwartet beendet | Manta 上次意外退出 | Manta se cerró inesperadamente la última vez | Manta a quitté de façon inattendue la dernière fois | L’ultima volta Manta si è chiuso inaspettatamente | В прошлый раз Manta неожиданно завершила работу |
| `crash.report` | Report on GitHub | Auf GitHub melden | 在 GitHub 上报告 | Informar en GitHub | Signaler sur GitHub | Segnala su GitHub | Сообщить на GitHub |
| `crash.show` | Show Report | Bericht zeigen | 显示报告 | Mostrar informe | Afficher le rapport | Mostra rapporto | Показать отчёт |
| `crash.ignore` | Ignore | Ignorieren | 忽略 | Ignorar | Ignorer | Ignora | Игнорировать |
| `app.job.copy_to` | Copying {what} to {dst} | {what} nach {dst} kopieren | 正在将 {what} 拷贝到 {dst} | Copiando {what} a {dst} | Copie de {what} vers {dst} | Copia di {what} in {dst} | Копирование {what} в {dst} |
| `app.job.move_to` | Moving {what} to {dst} | {what} nach {dst} bewegen | 正在将 {what} 移到 {dst} | Moviendo {what} a {dst} | Déplacement de {what} vers {dst} | Spostamento di {what} in {dst} | Перемещение {what} в {dst} |
| `app.job.extract_to` | Extracting {what} to {dst} | {what} nach {dst} entpacken | 正在将 {what} 解压缩到 {dst} | Extrayendo {what} en {dst} | Extraction de {what} vers {dst} | Estrazione di {what} in {dst} | Извлечение {what} в {dst} |
| `app.job.compress_to` | Compressing {what} to {dst} | {what} in {dst} komprimieren | 正在将 {what} 压缩到 {dst} | Comprimiendo {what} en {dst} | Compression de {what} dans {dst} | Compressione di {what} in {dst} | Сжатие {what} в {dst} |
| `app.job.copy` | Copying {what} | {what} kopieren | 正在拷贝 {what} | Copiando {what} | Copie de {what} | Copia di {what} | Копирование {what} |
| `app.job.move` | Moving {what} | {what} bewegen | 正在移动 {what} | Moviendo {what} | Déplacement de {what} | Spostamento di {what} | Перемещение {what} |
| `app.job.delete` | Deleting {what} | {what} löschen | 正在删除 {what} | Eliminando {what} | Suppression de {what} | Eliminazione di {what} | Удаление {what} |
| `app.job.extract` | Extracting {what} | {what} entpacken | 正在解压缩 {what} | Extrayendo {what} | Extraction de {what} | Estrazione di {what} | Извлечение {what} |
| `app.job.compress` | Compressing {what} | {what} komprimieren | 正在压缩 {what} | Comprimiendo {what} | Compression de {what} | Compressione di {what} | Сжатие {what} |
| `app.job.items.one` | {count} item | {count} Objekt | {count} 项 | {count} ítem | {count} élément | {count} elemento | {count} объекта |
| `app.job.items.other` | {count} items | {count} Objekte | {count} 项 | {count} ítems | {count} éléments | {count} elementi | {count} объектов |
| `palette.no_match` | No matching commands | Keine passenden Befehle | 没有匹配的命令 | No hay comandos coincidentes | Aucune commande correspondante | Nessun comando corrispondente | Нет подходящих команд |
| `palette.show_panel` | Show “{title}” Panel | Panel „{title}“ einblenden | 显示“{title}”面板 | Mostrar el panel “{title}” | Afficher le panneau « {title} » | Mostra il pannello “{title}” | Показать панель «{title}» |
| `palette.hide_panel` | Hide “{title}” Panel | Panel „{title}“ ausblenden | 隐藏“{title}”面板 | Ocultar el panel “{title}” | Masquer le panneau « {title} » | Nascondi il pannello “{title}” | Скрыть панель «{title}» |
| `palette.show_log` | Show Log File in Finder | Protokolldatei im Finder zeigen | 在访达中显示日志文件 | Mostrar archivo de registro en Finder | Afficher le fichier journal dans le Finder | Mostra il file di log nel Finder | Показать файл журнала в Finder |
| `palette.cat.help` | Help | Hilfe | 帮助 | Ayuda | Aide | Aiuto | Справка |
| `archive.cant_rename` | Can't rename items in “{name}” | In „{name}“ können keine Objekte umbenannt werden | 无法重新命名“{name}”中的项目 | No se pueden renombrar ítems en “{name}” | Impossible de renommer des éléments dans « {name} » | Impossibile rinominare elementi in “{name}” | Нельзя переименовывать объекты в «{name}» |
| `archive.cant_new_folder` | Can't create folders in “{name}” | In „{name}“ können keine Ordner erstellt werden | 无法在“{name}”中创建文件夹 | No se pueden crear carpetas en “{name}” | Impossible de créer des dossiers dans « {name} » | Impossibile creare cartelle in “{name}” | Нельзя создавать папки в «{name}» |
| `archive.cant_new_file` | Can't create files in “{name}” | In „{name}“ können keine Dateien erstellt werden | 无法在“{name}”中创建文件 | No se pueden crear archivos en “{name}” | Impossible de créer des fichiers dans « {name} » | Impossibile creare file in “{name}” | Нельзя создавать файлы в «{name}» |
| `archive.cant_delete` | Can't delete items in “{name}” | In „{name}“ können keine Objekte gelöscht werden | 无法删除“{name}”中的项目 | No se pueden eliminar ítems en “{name}” | Impossible de supprimer des éléments dans « {name} » | Impossibile eliminare elementi in “{name}” | Нельзя удалять объекты в «{name}» |
| `archive.cant_paste` | Can't paste or copy items in “{name}” | In „{name}“ können keine Objekte eingesetzt oder kopiert werden | 无法在“{name}”中粘贴或拷贝项目 | No se pueden pegar ni copiar ítems en “{name}” | Impossible de coller ou de copier des éléments dans « {name} » | Impossibile incollare o copiare elementi in “{name}” | Нельзя вставлять или копировать объекты в «{name}» |
| `archive.cant_compress` | Can't create an archive in “{name}” | In „{name}“ kann kein Archiv erstellt werden | 无法在“{name}”中创建归档 | No se puede crear un archivo comprimido en “{name}” | Impossible de créer une archive dans « {name} » | Impossibile creare un archivio in “{name}” | Нельзя создать архив в «{name}» |
| `archive.contents_folder` | {name} contents | Inhalt von {name} | {name} 的内容 | Contenido de {name} | Contenu de {name} | Contenuto di {name} | Содержимое {name} |
| `archive.skipped_unsafe.one` | Skipped {count} unsafe entry in “{name}” | {count} unsicherer Eintrag in „{name}“ übersprungen | 已跳过“{name}”中 {count} 个不安全的条目 | Se ha omitido {count} entrada no segura en “{name}” | {count} entrée non sûre ignorée dans « {name} » | {count} elemento non sicuro in “{name}” saltato | Пропущен {count} небезопасный объект в «{name}» |
| `archive.empty` | “{name}” is empty | „{name}“ ist leer | “{name}”为空 | “{name}” está vacío | « {name} » est vide | “{name}” è vuoto | Архив «{name}» пуст |
| `archive.nothing` | Nothing to extract | Nichts zu entpacken | 没有可解压缩的内容 | No hay nada que extraer | Rien à extraire | Niente da estrarre | Нечего извлекать |
| `archive.select_one` | Select one archive to extract it | Wähle ein Archiv zum Entpacken aus | 选择一个归档以将其解压缩 | Selecciona un archivo comprimido para extraerlo | Sélectionnez une archive pour l’extraire | Seleziona un archivio da estrarre | Выберите один архив, чтобы извлечь его |
| `archive.select_compress` | Select the items you want to compress | Wähle die Objekte aus, die du komprimieren möchtest | 选择要压缩的项目 | Selecciona los ítems que quieras comprimir | Sélectionnez les éléments à compresser | Seleziona gli elementi da comprimere | Выберите объекты, которые нужно сжать |

## App: File pane and sidebar

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `pane.filter` | Filter | Filtern | 过滤 | Filtrar | Filtrer | Filtra | Фильтр |
| `pane.local` | Local | Lokal | 本地 | Local | Local | Locale | Локально |
| `pane.back_local` | Back to Local | Zurück zu Lokal | 返回本地 | Volver a Local | Revenir au local | Torna a Locale | Назад к локальным |
| `place.archive` | Archive | Archiv | 归档 | Archivo comprimido | Archive | Archivio | Архив |
| `pane.enclosing_folders` | Show enclosing folders | Übergeordnete Ordner anzeigen | 显示上层文件夹 | Mostrar carpetas contenedoras | Afficher les dossiers parents | Mostra le cartelle superiori | Показать вышестоящие папки |
| `pane.try_again` | Try Again | Erneut versuchen | 重试 | Reintentar | Réessayer | Riprova | Повторить |
| `pane.menu.new_folder_in` | New Folder in “{name}” | Neuer Ordner in „{name}“ | 在“{name}”中新建文件夹 | Nueva carpeta en “{name}” | Nouveau dossier dans « {name} » | Nuova cartella in “{name}” | Новая папка в «{name}» |
| `pane.menu.new_file_in` | New File in “{name}” | Neue Datei in „{name}“ | 在“{name}”中新建文件 | Nuevo archivo en “{name}” | Nouveau fichier dans « {name} » | Nuovo file in “{name}” | Новый файл в «{name}» |
| `pane.start_container` | Start Container | Container starten | 启动容器 | Iniciar contenedor | Démarrer le conteneur | Avvia container | Запустить контейнер |
| `pane.starting_container` | Starting… | Wird gestartet … | 正在启动… | Iniciando… | Démarrage… | Avvio… | Запуск… |
| `pane.go_back` | Go Back | Zurück | 返回 | Volver | Revenir | Torna indietro | Назад |
| `pane.loading` | Loading “{name}”… | „{name}“ wird geladen … | 正在载入“{name}”… | Cargando “{name}”… | Chargement de « {name} »… | Caricamento di “{name}”… | Загрузка «{name}»… |
| `pane.no_match` | No items match “{filter}” | Keine Objekte passen zu „{filter}“ | 没有与“{filter}”匹配的项目 | Ningún ítem coincide con “{filter}” | Aucun élément ne correspond à « {filter} » | Nessun elemento corrisponde a “{filter}” | Нет объектов, соответствующих «{filter}» |
| `pane.clear_filter` | Clear Filter | Filter löschen | 清除过滤条件 | Borrar filtro | Effacer le filtre | Cancella filtro | Сбросить фильтр |
| `pane.new_folder` | New Folder | Neuer Ordner | 新建文件夹 | Nueva carpeta | Nouveau dossier | Nuova cartella | Новая папка |
| `pane.empty_title` | This folder is empty | Dieser Ordner ist leer | 此文件夹为空 | Esta carpeta está vacía | Ce dossier est vide | Questa cartella è vuota | Эта папка пуста |
| `pane.alias_of` | Alias of {target} | Alias von {target} | {target} 的替身 | Alias de {target} | Alias de {target} | Alias di {target} | Псевдоним для {target} |
| `pane.opening` | Opening “{name}”… | „{name}“ wird geöffnet … | 正在打开“{name}”… | Abriendo “{name}”… | Ouverture de « {name} »… | Apertura di “{name}”… | Открытие «{name}»… |
| `pane.downloading_of` | Downloading · {done} of {total} | Wird geladen · {done} von {total} | 正在下载 · {done}/{total} | Descargando · {done} de {total} | Téléchargement · {done} sur {total} | Download · {done} di {total} | Загрузка · {done} из {total} |
| `pane.downloading` | Downloading · {done} | Wird geladen · {done} | 正在下载 · {done} | Descargando · {done} | Téléchargement · {done} | Download · {done} | Загрузка · {done} |
| `pane.reading_archive` | Reading the archive’s contents | Inhalt des Archivs wird gelesen | 正在读取归档的内容 | Leyendo el contenido del archivo comprimido | Lecture du contenu de l’archive | Lettura del contenuto dell’archivio | Чтение содержимого архива |
| `pane.col.name` | Name | Name | 名称 | Nombre | Nom | Nome | Имя |
| `pane.col.size` | Size | Größe | 大小 | Tamaño | Taille | Dimensione | Размер |
| `pane.col.modified` | Modified | Änderungsdatum | 修改日期 | Fecha de modificación | Date de modification | Data di modifica | Дата изменения |
| `pane.col.permissions` | Permissions | Zugriffsrechte | 权限 | Permisos | Autorisations | Permessi | Права доступа |
| `pane.sort_by` | Sort by {column} | Nach „{column}“ sortieren | 按“{column}”排序 | Ordenar por “{column}” | Trier par {column} | Ordina per “{column}” | Сортировать по столбцу «{column}» |
| `pane.size.files.one` | {count} file | {count} Datei | {count} 个文件 | {count} archivo | {count} fichier | {count} file | {count} файл |
| `pane.size.files.other` | {count} files | {count} Dateien | {count} 个文件 | {count} archivos | {count} fichiers | {count} file | {count} файла |
| `pane.size.on_disk` | {size} on disk | {size} auf dem Volume | 占用磁盘 {size} | {size} en disco | {size} sur le disque | {size} su disco | {size} на диске |
| `pane.size.counting` | Still counting… | Wird noch gezählt … | 仍在计算… | Calculando… | Calcul en cours… | Conteggio in corso… | Подсчет… |
| `pane.size.unreadable.one` | {count} folder couldn't be read | {count} Ordner konnte nicht gelesen werden | {count} 个文件夹无法读取 | No se ha podido leer {count} carpeta | Impossible de lire {count} dossier | Impossibile leggere {count} cartella | Не удалось прочитать {count} папку |
| `pane.size.unreadable.other` | {count} folders couldn't be read | {count} Ordner konnten nicht gelesen werden | {count} 个文件夹无法读取 | No se han podido leer {count} carpetas | Impossible de lire {count} dossiers | Impossibile leggere {count} cartelle | Не удалось прочитать {count} папки |
| `pane.menu.browse_archive` | Browse Archive | Archiv durchsuchen | 浏览归档 | Explorar archivo comprimido | Parcourir l’archive | Sfoglia archivio | Просмотреть архив |
| `pane.menu.extract_to_other_ellipsis` | Extract to Other Pane… | In anderen Bereich entpacken … | 解压缩到另一个面板… | Extraer al otro panel… | Extraire dans l’autre sous-fenêtre… | Estrai nell’altro pannello… | Извлечь в другую панель… |
| `pane.menu.extract_to_other` | Extract to Other Pane | In anderen Bereich entpacken | 解压缩到另一个面板 | Extraer al otro panel | Extraire dans l’autre sous-fenêtre | Estrai nell’altro pannello | Извлечь в другую панель |
| `pane.items.one` | {count} item | {count} Objekt | {count} 项 | {count} ítem | {count} élément | {count} elemento | {count} объект |
| `pane.items.other` | {count} items | {count} Objekte | {count} 项 | {count} ítems | {count} éléments | {count} elementi | {count} объекта |
| `pane.drag.extract` | Extract {what} | {what} entpacken | 解压缩 {what} | Extraer {what} | Extraire {what} | Estrai {what} | Извлечь {what} |
| `pane.drag.move` | Move {what} | {what} bewegen | 移动 {what} | Mover {what} | Déplacer {what} | Sposta {what} | Переместить {what} |
| `pane.drag.copy` | Copy {what} | {what} kopieren | 拷贝 {what} | Copiar {what} | Copier {what} | Copia {what} | Скопировать {what} |
| `pane.drag.fav_add.one` | Add {count} folder to Favorites | {count} Ordner zu den Favoriten hinzufügen | 将 {count} 个文件夹添加到个人收藏 | Añadir {count} carpeta a Favoritos | Ajouter {count} dossier aux Favoris | Aggiungi {count} cartella ai Preferiti | Добавить {count} папку в Избранное |
| `pane.drag.fav_add.other` | Add {count} folders to Favorites | {count} Ordner zu den Favoriten hinzufügen | 将 {count} 个文件夹添加到个人收藏 | Añadir {count} carpetas a Favoritos | Ajouter {count} dossiers aux Favoris | Aggiungi {count} cartelle ai Preferiti | Добавить {count} папки в Избранное |
| `pane.drag.trash.one` | Move {count} item to the Trash | {count} Objekt in den Papierkorb legen | 将 {count} 项移到废纸篓 | Trasladar {count} ítem a la papelera | Placer {count} élément dans la corbeille | Sposta {count} elemento nel Cestino | Переместить {count} объект в Корзину |
| `pane.drag.trash.other` | Move {count} items to the Trash | {count} Objekte in den Papierkorb legen | 将 {count} 项移到废纸篓 | Trasladar {count} ítems a la papelera | Placer {count} éléments dans la corbeille | Sposta {count} elementi nel Cestino | Переместить {count} объекта в Корзину |
| `pane.drag.trash_delete.one` | Delete {count} item immediately… | {count} Objekt sofort löschen … | 立即删除 {count} 项… | Eliminar {count} ítem inmediatamente… | Supprimer immédiatement {count} élément… | Elimina immediatamente {count} elemento… | Удалить {count} объект немедленно… |
| `pane.drag.trash_delete.other` | Delete {count} items immediately… | {count} Objekte sofort löschen … | 立即删除 {count} 项… | Eliminar {count} ítems inmediatamente… | Supprimer immédiatement {count} éléments… | Elimina immediatamente {count} elementi… | Удалить {count} объекта немедленно… |
| `pane.drag.trash_already` | Already in the Trash | Bereits im Papierkorb | 已在废纸篓中 | Ya está en la papelera | Déjà dans la corbeille | Già nel Cestino | Уже в Корзине |
| `pane.drag.fav_folders_only` | Only folders can be favorites | Nur Ordner können Favoriten sein | 只有文件夹才能添加到个人收藏 | Solo las carpetas pueden ser favoritos | Seuls les dossiers peuvent être des favoris | Solo le cartelle possono essere tra i Preferiti | В Избранное можно добавлять только папки |
| `pane.drop.read_only` | Read-only archive — can't drop here | Schreibgeschütztes Archiv – Ablegen nicht möglich | 只读归档：无法拖放到此处 | Archivo comprimido de solo lectura — no se puede soltar aquí | Archive en lecture seule — impossible de déposer ici | Archivio di sola lettura — impossibile rilasciare qui | Архив только для чтения — сюда нельзя перетаскивать |
| `pane.drop.extract` | Extract {what} here | {what} hier entpacken | 将 {what} 解压缩到此处 | Extraer {what} aquí | Extraire {what} ici | Estrai qui {what} | Извлечь {what} сюда |
| `pane.drop.move` | Move {what} here  ·  hold ⌥ to copy | {what} hierher bewegen  ·  ⌥ halten zum Kopieren | 将 {what} 移到此处  ·  按住 ⌥ 以拷贝 | Mover {what} aquí  ·  mantén ⌥ para copiar | Déplacer {what} ici  ·  maintenez ⌥ pour copier | Sposta qui {what}  ·  tieni premuto ⌥ per copiare | Переместить {what} сюда  ·  удерживайте ⌥, чтобы скопировать |
| `pane.drop.copy` | Copy {what} here  ·  hold ⌘ to move | {what} hierher kopieren  ·  ⌘ halten zum Bewegen | 将 {what} 拷贝到此处  ·  按住 ⌘ 以移动 | Copiar {what} aquí  ·  mantén ⌘ para mover | Copier {what} ici  ·  maintenez ⌘ pour déplacer | Copia qui {what}  ·  tieni premuto ⌘ per spostare | Скопировать {what} сюда  ·  удерживайте ⌘, чтобы переместить |
| `pane.drag.downloading` | Downloading “{name}” from {backend}… | „{name}“ wird von {backend} geladen … | 正在从 {backend} 下载“{name}”… | Descargando “{name}” de {backend}… | Téléchargement de « {name} » depuis {backend}… | Download di “{name}” da {backend}… | Загрузка «{name}» с {backend}… |
| `pane.drag.downloaded` | “{name}” downloaded | „{name}“ geladen | “{name}”已下载 | “{name}” descargado | « {name} » téléchargé | “{name}” scaricato | Загружено: «{name}» |
| `pane.drag.download_failed` | Couldn't download “{name}” | „{name}“ konnte nicht geladen werden | 无法下载“{name}” | No se ha podido descargar “{name}” | Impossible de télécharger « {name} » | Impossibile scaricare “{name}” | Не удалось загрузить «{name}» |
| `pane.drag.failed` | Couldn't drag the items out of Manta | Die Objekte konnten nicht aus Manta gezogen werden | 无法将项目拖出 Manta | No se han podido arrastrar los ítems fuera de Manta | Impossible de faire glisser les éléments hors de Manta | Impossibile trascinare gli elementi fuori da Manta | Не удалось перетащить объекты из Manta |
| `sidebar.disconnect` | Disconnect | Verbindung trennen | 断开连接 | Desconectar | Déconnecter | Disconnetti | Отключить |
| `sidebar.eject` | Eject | Auswerfen | 推出 | Expulsar | Éjecter | Espelli | Извлечь |
| `sidebar.eject_named` | Eject “{name}” | „{name}“ auswerfen | 推出“{name}” | Expulsar “{name}” | Éjecter « {name} » | Espelli “{name}” | Извлечь «{name}» |
| `sidebar.ejectable` | Ejectable | Auswerfbar | 可推出 | Expulsables | Éjectables | Espellibili | Извлекаемые |
| `sidebar.open_new_tab` | Open in New Tab | In neuem Tab öffnen | 在新标签页中打开 | Abrir en una pestaña nueva | Ouvrir dans un nouvel onglet | Apri in una nuova scheda | Открыть в новой вкладке |
| `sidebar.show_enclosing` | Show in Enclosing Folder | Im umgebenden Ordner anzeigen | 在所在文件夹中显示 | Mostrar en la carpeta contenedora | Afficher dans le dossier parent | Mostra nella cartella superiore | Показать в папке |
| `sidebar.favorites` | Favorites | Favoriten | 个人收藏 | Favoritos | Favoris | Preferiti | Избранное |
| `sidebar.add_current` | Add Current Folder | Aktuellen Ordner hinzufügen | 添加当前文件夹 | Añadir carpeta actual | Ajouter le dossier actuel | Aggiungi cartella attuale | Добавить текущую папку |
| `sidebar.all_buckets` | all buckets | alle Buckets | 所有 Bucket | todos los buckets | tous les buckets | tutti i bucket | все бакеты |
| `sidebar.docker_container` | Docker container {name} | Docker-Container {name} | Docker 容器 {name} | Contenedor de Docker {name} | Conteneur Docker {name} | Container Docker {name} | Контейнер Docker {name} |
| `sidebar.kubernetes_pod` | Kubernetes pod {name} · {namespace} | Kubernetes-Pod {name} · {namespace} | Kubernetes Pod {name} · {namespace} | Pod de Kubernetes {name} · {namespace} | Pod Kubernetes {name} · {namespace} | Pod Kubernetes {name} · {namespace} | Под Kubernetes {name} · {namespace} |
| `sidebar.open` | Open | Öffnen | 打开 | Abrir | Ouvrir | Apri | Открыть |
| `sidebar.edit_connection` | Edit Connection… | Verbindung bearbeiten … | 编辑连接… | Editar conexión… | Modifier la connexion… | Modifica connessione… | Изменить подключение… |
| `sidebar.remove_favorite` | Remove from Favorites | Aus Favoriten entfernen | 从个人收藏中移除 | Eliminar de Favoritos | Supprimer des Favoris | Rimuovi dai Preferiti | Удалить из Избранного |
| `sidebar.locations` | Locations | Orte | 位置 | Ubicaciones | Emplacements | Posizioni | Места |
| `sidebar.devices` | Devices | Geräte | 设备 | Dispositivos | Appareils | Dispositivi | Устройства |
| `sidebar.rescan` | Rescan Volumes | Volumes neu einlesen | 重新扫描宗卷 | Volver a buscar volúmenes | Analyser à nouveau les volumes | Rileggi volumi | Обновить список томов |
| `sidebar.servers` | Servers | Server | 服务器 | Servidores | Serveurs | Server | Серверы |
| `sidebar.cloud` | Cloud | Cloud | 云端 | Nube | Cloud | Cloud | Облако |
| `sidebar.rescan_cloud` | Look for Cloud Folders Again | Erneut nach Cloud-Ordnern suchen | 重新查找云端文件夹 | Volver a buscar carpetas en la nube | Rechercher à nouveau les dossiers cloud | Cerca di nuovo cartelle cloud | Снова найти облачные папки |
| `sidebar.trash` | Trash | Papierkorb | 废纸篓 | Papelera | Corbeille | Cestino | Корзина |
| `sidebar.trash.empty` | The Trash is empty | Der Papierkorb ist leer | 废纸篓为空 | La papelera está vacía | La corbeille est vide | Il Cestino è vuoto | Корзина пуста |
| `sidebar.trash.show_in_finder` | Open in Finder | Im Finder öffnen | 在访达中打开 | Abrir en Finder | Ouvrir dans le Finder | Apri nel Finder | Открыть в Finder |
| `sidebar.trash.remove_favorite` | Remove from Favorites | Aus Favoriten entfernen | 从个人收藏中移除 | Eliminar de Favoritos | Supprimer des Favoris | Rimuovi dai Preferiti | Удалить из Избранного |
| `trash.fda.open_settings` | Open Full Disk Access | Festplattenvollzugriff öffnen | 打开“完全磁盘访问权限” | Abrir Acceso total al disco | Ouvrir Accès complet au disque | Apri Accesso completo al disco | Открыть «Полный доступ к диску» |
| `trash.fda.show_in_finder` | Open in Finder | Im Finder öffnen | 在访达中打开 | Abrir en Finder | Ouvrir dans le Finder | Apri nel Finder | Открыть в Finder |
| `cloud.personal` | Personal | Persönlich | 个人 | Personal | Personnel | Personale | Личное |
| `cloud.shared_libraries` | Shared Libraries | Geteilte Bibliotheken | 共享资源库 | Bibliotecas compartidas | Bibliothèques partagées | Librerie condivise | Общие библиотеки |
| `cloud.state.cloud_only` | Only in the cloud – not downloaded | Nur in der Cloud – nicht geladen | 仅在云端，未下载 | Solo en la nube – no descargado | Uniquement dans le cloud – non téléchargé | Solo nel cloud – non scaricato | Только в облаке – не загружено |
| `cloud.state.downloading` | Downloading… | Wird geladen … | 正在下载… | Descargando… | Téléchargement… | Download in corso… | Загрузка… |
| `cloud.state.uploading` | Uploading… | Wird hochgeladen … | 正在上传… | Subiendo… | Téléversement… | Caricamento in corso… | Выгрузка… |
| `cloud.state.uploading_percent` | Uploading… {percent}% | Wird hochgeladen … {percent} % | 正在上传… {percent}% | Subiendo… {percent} % | Téléversement… {percent} % | Caricamento in corso… {percent}% | Выгрузка… {percent} % |
| `cloud.state.upload_waiting` | Waiting to upload | Wartet auf Upload | 正在等待上传 | Esperando para subir | En attente de téléversement | In attesa di caricamento | Ожидание выгрузки |
| `cloud.download_started.one` | Downloading 1 item from the cloud | 1 Objekt wird aus der Cloud geladen | 正在从云端下载 {count} 项 | Descargando 1 ítem de la nube | Téléchargement de {count} élément depuis le cloud | Download di 1 elemento dal cloud | Загрузка {count} объекта из облака |
| `cloud.download_started.other` | Downloading {count} items from the cloud | {count} Objekte werden aus der Cloud geladen | 正在从云端下载 {count} 项 | Descargando {count} ítems de la nube | Téléchargement de {count} éléments depuis le cloud | Download di {count} elementi dal cloud | Загрузка {count} объектов из облака |
| `cloud.evicted.one` | Removed the download of 1 item | Download von 1 Objekt entfernt | 已移除 {count} 项的下载 | Se ha eliminado la descarga de 1 ítem | Téléchargement de {count} élément supprimé | Download di 1 elemento rimosso | Загрузка {count} объекта удалена |
| `cloud.evicted.other` | Removed the download of {count} items | Download von {count} Objekten entfernt | 已移除 {count} 项的下载 | Se ha eliminado la descarga de {count} ítems | Téléchargement de {count} éléments supprimé | Download di {count} elementi rimosso | Загрузка {count} объектов удалена |
| `cloud.download_failed` | Couldn't download from the cloud | Laden aus der Cloud fehlgeschlagen | 无法从云端下载 | No se ha podido descargar de la nube | Impossible de télécharger depuis le cloud | Impossibile scaricare dal cloud | Не удалось загрузить из облака |
| `cloud.evict_failed` | Couldn't remove the download | Download konnte nicht entfernt werden | 无法移除下载 | No se ha podido eliminar la descarga | Impossible de supprimer le téléchargement | Impossibile rimuovere il download | Не удалось удалить загрузку |
| `pane.menu.cloud_download` | Download Now | Jetzt laden | 立即下载 | Descargar ahora | Télécharger maintenant | Scarica ora | Загрузить сейчас |
| `pane.menu.cloud_evict` | Remove Download | Download entfernen | 移除下载 | Eliminar descarga | Supprimer le téléchargement | Rimuovi download | Удалить загрузку |
| `sidebar.connected` | Connected · {name} | Verbunden · {name} | 已连接 · {name} | Conectado · {name} | Connecté · {name} | Connesso · {name} | Подключено · {name} |
| `sidebar.containers` | Containers | Container | 容器 | Contenedores | Conteneurs | Container | Контейнеры |
| `sidebar.refresh_containers` | Refresh Containers | Container aktualisieren | 刷新容器 | Actualizar contenedores | Actualiser les conteneurs | Aggiorna container | Обновить контейнеры |
| `sidebar.place.home` | Home | Benutzerordner | 个人文件夹 | Inicio | Départ | Inizio | Домой |
| `sidebar.place.desktop` | Desktop | Schreibtisch | 桌面 | Escritorio | Bureau | Scrivania | Рабочий стол |
| `sidebar.place.documents` | Documents | Dokumente | 文稿 | Documentos | Documents | Documenti | Документы |
| `sidebar.place.downloads` | Downloads | Downloads | 下载 | Descargas | Téléchargements | Download | Загрузки |
| `sidebar.place.applications` | Applications | Programme | 应用程序 | Aplicaciones | Applications | Applicazioni | Программы |
| `preview.unreadable` | This file couldn't be read | Diese Datei konnte nicht gelesen werden | 无法读取此文件 | No se ha podido leer este archivo | Impossible de lire ce fichier | Impossibile leggere questo file | Не удалось прочитать этот файл |
| `preview.no_preview` | No preview for this file | Keine Vorschau für diese Datei | 此文件没有预览 | No hay vista previa para este archivo | Aucun aperçu pour ce fichier | Nessuna anteprima per questo file | Нет просмотра для этого файла |
| `ql.unavailable` | Quick Look isn't available | Die Schnellansicht ist nicht verfügbar | “快速查看”不可用 | Vista Rápida no está disponible | Coup d’œil n’est pas disponible | Visualizzazione rapida non è disponibile | Быстрый просмотр недоступен |
| `kind.folder` | Folder | Ordner | 文件夹 | Carpeta | Dossier | Cartella | Папка |
| `kind.image` | Image | Bild | 图像 | Imagen | Image | Immagine | Изображение |
| `kind.video` | Video | Videodatei | 视频 | Archivo de vídeo | Vidéo | Video | Видео |
| `kind.audio` | Audio | Audiodatei | 音频 | Archivo de audio | Audio | Audio | Аудио |
| `kind.archive` | Archive | Archiv | 归档 | Archivo comprimido | Archive | Archivio | Архив |
| `kind.pdf` | PDF document | PDF-Dokument | PDF 文稿 | Documento PDF | Document PDF | Documento PDF | Документ PDF |
| `kind.document` | Document | Dokument | 文稿 | Documento | Document | Documento | Документ |
| `kind.spreadsheet` | Spreadsheet | Tabelle | 电子表格 | Hoja de cálculo | Feuille de calcul | Foglio di calcolo | Таблица |
| `kind.presentation` | Presentation | Präsentation | 演示文稿 | Presentación | Présentation | Presentazione | Презентация |
| `kind.code` | Source code | Quellcode | 源代码 | Código fuente | Code source | Codice sorgente | Исходный код |
| `kind.text` | Text | Textdatei | 文本 | Archivo de texto | Texte | Testo | Текст |
| `kind.script` | Shell script | Shell-Skript | Shell 脚本 | Script de shell | Script shell | Script shell | Сценарий оболочки |
| `kind.file` | File | Datei | 文件 | Archivo | Fichier | File | Файл |
| `widgets.clear` | Clear | Löschen | 清除 | Borrar | Effacer | Cancella | Очистить |

## App: Panes

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `cmd.split_pane` | Split Pane | Bereich teilen | 拆分面板 | Dividir panel | Diviser la sous-fenêtre | Dividi pannello | Разделить панель |
| `cmd.close_pane` | Close Pane | Bereich schließen | 关闭面板 | Cerrar panel | Fermer la sous-fenêtre | Chiudi pannello | Закрыть панель |
| `cmd.focus_pane_left` | Focus Pane on the Left | Bereich links aktivieren | 聚焦左侧面板 | Activar el panel de la izquierda | Activer la sous-fenêtre de gauche | Attiva pannello a sinistra | Перейти к панели слева |
| `cmd.focus_pane_right` | Focus Pane on the Right | Bereich rechts aktivieren | 聚焦右侧面板 | Activar el panel de la derecha | Activer la sous-fenêtre de droite | Attiva pannello a destra | Перейти к панели справа |
| `panes.needs_two` | Needs two panes – ⌘D splits the pane | Benötigt zwei Bereiche – ⌘D teilt den Bereich | 需要两个面板（按 ⌘D 拆分面板） | Se necesitan dos paneles – ⌘D divide el panel | Nécessite deux sous-fenêtres – ⌘D divise la sous-fenêtre | Servono due pannelli – ⌘D divide il pannello | Нужны две панели – ⌘D разделяет панель |
| `panes.last_pane` | The last pane can't be closed | Der letzte Bereich kann nicht geschlossen werden | 无法关闭最后一个面板 | No se puede cerrar el último panel | La dernière sous-fenêtre ne peut pas être fermée | L’ultimo pannello non può essere chiuso | Последнюю панель нельзя закрыть |
| `panes.no_room` | No room for another pane | Kein Platz für einen weiteren Bereich | 没有空间再添加面板 | No hay espacio para otro panel | Pas de place pour une autre sous-fenêtre | Non c’è spazio per un altro pannello | Нет места для еще одной панели |
| `panes.sync_pick_a` | Compare this pane as A | Diesen Bereich als A vergleichen | 将此面板作为 A 进行比较 | Comparar este panel como A | Comparer cette sous-fenêtre comme A | Confronta questo pannello come A | Сравнить эту панель как A |
| `panes.sync_pick_b` | Compare this pane as B | Diesen Bereich als B vergleichen | 将此面板作为 B 进行比较 | Comparar este panel como B | Comparer cette sous-fenêtre comme B | Confronta questo pannello come B | Сравнить эту панель как B |
| `panes.pane_n` | Pane {n} | Bereich {n} | 面板 {n} | Panel {n} | Sous-fenêtre {n} | Pannello {n} | Панель {n} |
| `toolbar.copy_to_pane` | Copy to Pane {n} ({name}) | In Bereich {n} ({name}) kopieren | 拷贝到面板 {n}（{name}） | Copiar al panel {n} ({name}) | Copier dans la sous-fenêtre {n} ({name}) | Copia nel pannello {n} ({name}) | Копировать в панель {n} ({name}) |
| `toolbar.move_to_pane` | Move to Pane {n} ({name}) | In Bereich {n} ({name}) bewegen | 移到面板 {n}（{name}） | Mover al panel {n} ({name}) | Déplacer dans la sous-fenêtre {n} ({name}) | Sposta nel pannello {n} ({name}) | Переместить в панель {n} ({name}) |
| `dialog.transfer.to` | To | Nach | 目标 | A | Vers | In | Куда |
| `sync.pane_a` | Pane A | Bereich A | 面板 A | Panel A | Sous-fenêtre A | Pannello A | Панель A |
| `sync.pane_b` | Pane B | Bereich B | 面板 B | Panel B | Sous-fenêtre B | Pannello B | Панель B |

## App: Views

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `view.icons` | Icons | Symbole | 图标 | Iconos | Icônes | Icone | Значки |
| `view.list` | List | Liste | 列表 | Lista | Liste | Elenco | Список |
| `view.columns` | Columns | Spalten | 分栏 | Columnas | Colonnes | Colonne | Колонки |
| `view.gallery` | Gallery | Galerie | 画廊 | Galería | Galerie | Galleria | Галерея |
| `view.options` | View Options | Darstellungsoptionen | 显示选项 | Opciones de visualización | Options de présentation | Opzioni vista | Параметры вида |
| `view.sort_by` | Sort By | Sortieren nach | 排序方式 | Ordenar por | Trier par | Ordina per | Сортировка |
| `view.ascending` | Ascending | Aufsteigend | 升序 | Ascendente | Croissant | Crescente | По возрастанию |
| `view.icon_size` | Icon Size | Symbolgröße | 图标大小 | Tamaño de los iconos | Taille des icônes | Dimensione icone | Размер значков |
| `view.column_empty` | No items | Keine Objekte | 无项目 | No hay ítems | Aucun élément | Nessun elemento | Нет объектов |
| `view.items.one` | {count} item | {count} Objekt | {count} 项 | {count} ítem | {count} élément | {count} elemento | {count} объект |
| `view.items.other` | {count} items | {count} Objekte | {count} 项 | {count} ítems | {count} éléments | {count} elementi | {count} объекта |
| `view.selected_n.one` | {count} item selected | {count} Objekt ausgewählt | 已选择 {count} 项 | {count} ítem seleccionado | {count} élément sélectionné | {count} elemento selezionato | Выбран {count} объект |
| `view.selected_n.other` | {count} items selected | {count} Objekte ausgewählt | 已选择 {count} 项 | {count} ítems seleccionados | {count} éléments sélectionnés | {count} elementi selezionati | Выбрано {count} объекта |
| `view.information` | Information | Informationen | 信息 | Información | Informations | Informazioni | Информация |
| `view.info.kind` | Kind | Art | 种类 | Tipo | Type | Tipo | Тип |
| `view.info.size` | Size | Größe | 大小 | Tamaño | Taille | Dimensione | Размер |
| `view.info.modified` | Modified | Geändert | 修改日期 | Modificado | Modifié | Modificato | Изменен |
| `view.info.permissions` | Permissions | Zugriffsrechte | 权限 | Permisos | Autorisations | Permessi | Права доступа |
| `view.info.original` | Original | Originalobjekt | 原身 | Original | Original | Originale | Оригинал |
| `cmd.view_icons` | View as Icons | Als Symbole | 为图标 | Como iconos | Par icônes | Come icone | Как значки |
| `cmd.view_list` | View as List | Als Liste | 为列表 | Como lista | Par liste | Come elenco | Как список |
| `cmd.view_columns` | View as Columns | Als Spalten | 为分栏 | Como columnas | Par colonnes | Come colonne | В колонках |
| `cmd.view_gallery` | View as Gallery | Als Galerie | 为画廊 | Como galería | Par galerie | Come galleria | Как галерея |
| `settings.view_per_folder` | Remember view per folder | Ansicht pro Ordner merken | 为每个文件夹记住显示方式 | Recordar la visualización por carpeta | Mémoriser la présentation par dossier | Ricorda la vista per ogni cartella | Запоминать вид для каждой папки |
| `settings.remote_thumbnails` | Load thumbnails for server files | Vorschaubilder für Server-Dateien laden | 为服务器文件载入缩略图 | Cargar miniaturas de archivos del servidor | Charger les vignettes des fichiers sur serveur | Carica miniature dei file sul server | Загружать миниатюры файлов на сервере |
| `settings.system_icons` | File icons from macOS | Dateisymbole von macOS | 使用 macOS 文件图标 | Iconos de archivo de macOS | Icônes de fichiers de macOS | Icone dei file di macOS | Значки файлов macOS |

## App: Dialogs

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `connect.title` | Connect to Server | Mit Server verbinden | 连接服务器 | Conectarse al servidor | Se connecter au serveur | Connessione al server | Подключение к серверу |
| `connect.title.edit` | Edit Connection | Verbindung bearbeiten | 编辑连接 | Editar conexión | Modifier la connexion | Modifica connessione | Изменить подключение |
| `connect.protocol` | Protocol | Protokoll | 协议 | Protocolo | Protocole | Protocollo | Протокол |
| `connect.protocol.container` | Container | Container | 容器 | Contenedor | Conteneur | Container | Контейнер |
| `connect.server` | Server | Server | 服务器 | Servidor | Serveur | Server | Сервер |
| `connect.port` | Port | Port | 端口 | Puerto | Port | Porta | Порт |
| `connect.user` | User | Benutzer | 用户 | Usuario | Utilisateur | Utente | Пользователь |
| `connect.user.anonymous` | anonymous | anonym | 匿名 | anónimo | anonyme | anonimo | анонимно |
| `connect.sign_in_with` | Sign in with | Anmelden mit | 登录方式 | Iniciar sesión con | S’identifier avec | Accedi con | Способ входа |
| `connect.password` | Password | Passwort | 密码 | Contraseña | Mot de passe | Password | Пароль |
| `connect.key_file` | Key file | Schlüsseldatei | 密钥文件 | Archivo de clave | Fichier de clé | File chiave | Файл ключа |
| `connect.passphrase` | Passphrase | Passphrase | 密码短语 | Frase de contraseña | Phrase de passe | Passphrase | Парольная фраза |
| `connect.folder` | Folder | Ordner | 文件夹 | Carpeta | Dossier | Cartella | Папка |
| `connect.gdrive.account` | Account | Konto | 账户 | Cuenta | Compte | Account | Учетная запись |
| `connect.ftps.data` | Data channel | Datenkanal | 数据通道 | Canal de datos | Canal de données | Canale dati | Канал данных |
| `connect.ftps.data.auto` | Automatic | Automatisch | 自动 | Automático | Automatique | Automatico | Автоматически |
| `connect.ftps.data.require` | Required | Erforderlich | 必需 | Obligatorio | Obligatoire | Obbligatorio | Обязательно |
| `connect.ftps.data.off` | Off | Aus | 关闭 | Desactivado | Désactivé | Disattivato | Выкл. |
| `connect.webdav.security` | Security | Sicherheit | 安全性 | Seguridad | Sécurité | Sicurezza | Безопасность |
| `connect.webdav.http` | HTTP (unencrypted) | HTTP (unverschlüsselt) | HTTP（未加密） | HTTP (sin cifrar) | HTTP (non chiffré) | HTTP (non crittografato) | HTTP (без шифрования) |
| `connect.webdav.preset` | Server type | Servertyp | 服务器类型 | Tipo de servidor | Type de serveur | Tipo di server | Тип сервера |
| `connect.webdav.preset.generic` | WebDAV server | WebDAV-Server | WebDAV 服务器 | Servidor WebDAV | Serveur WebDAV | Server WebDAV | Сервер WebDAV |
| `connect.webdav.path` | WebDAV path | WebDAV-Pfad | WebDAV 路径 | Ruta WebDAV | Chemin WebDAV | Percorso WebDAV | Путь WebDAV |
| `connect.smb.share` | Share | Freigabe | 共享点 | Recurso compartido | Partage | Condivisione | Общий ресурс |
| `connect.smb.browse` | Browse… | Durchsuchen … | 浏览… | Explorar… | Parcourir… | Sfoglia… | Обзор… |
| `connect.smb.loading` | Looking for shares… | Freigaben werden gesucht … | 正在查找共享点… | Buscando recursos compartidos… | Recherche des partages… | Ricerca delle condivisioni… | Поиск общих ресурсов… |
| `connect.smb.none` | No shares found | Keine Freigaben gefunden | 未找到共享点 | No se han encontrado recursos compartidos | Aucun partage trouvé | Nessuna condivisione trovata | Общие ресурсы не найдены |
| `connect.smb.list_failed` | Couldn't list the shares: {error} | Die Freigaben konnten nicht angezeigt werden: {error} | 无法列出共享点：{error} | No se han podido mostrar los recursos compartidos: {error} | Impossible d’afficher les partages : {error} | Impossibile mostrare le condivisioni: {error} | Не удалось показать общие ресурсы: {error} |
| `connect.auth.password` | Password | Passwort | 密码 | Contraseña | Mot de passe | Password | Пароль |
| `connect.auth.key_file` | Key File | Schlüsseldatei | 密钥文件 | Archivo de clave | Fichier de clé | File chiave | Файл ключа |
| `connect.auth.agent` | SSH Agent | SSH-Agent | SSH 代理 | Agente SSH | Agent SSH | Agente SSH | Агент SSH |
| `connect.auth.anonymous` | Anonymous | Anonym | 匿名 | Anónimo | Anonyme | Anonimo | Анонимно |
| `connect.auth.aws_profile` | AWS Profile | AWS-Profil | AWS 配置文件 | Perfil de AWS | Profil AWS | Profilo AWS | Профиль AWS |
| `connect.auth.access_key` | Access Key | Zugriffsschlüssel | 访问密钥 | Clave de acceso | Clé d’accès | Chiave di accesso | Ключ доступа |
| `connect.auth.public` | Public | Öffentlich | 公开 | Público | Public | Pubblico | Публичный |
| `connect.auth.guest` | Guest | Gast | 客人 | Invitado | Invité | Ospite | Гость |
| `connect.add_favorite` | Add to Favorites | Zu Favoriten hinzufügen | 添加到个人收藏 | Añadir a Favoritos | Ajouter aux Favoris | Aggiungi ai Preferiti | Добавить в Избранное |
| `connect.remember.password` | Remember password | Passwort merken | 记住密码 | Recordar contraseña | Mémoriser le mot de passe | Ricorda password | Запомнить пароль |
| `connect.remember.passphrase` | Remember passphrase | Passphrase merken | 记住密码短语 | Recordar frase de contraseña | Mémoriser la phrase de passe | Ricorda passphrase | Запомнить парольную фразу |
| `connect.remember.secret` | Remember secret key | Geheimen Schlüssel merken | 记住私密密钥 | Recordar clave secreta | Mémoriser la clé secrète | Ricorda chiave segreta | Запомнить секретный ключ |
| `connect.secret.stored` | saved in Keychain | im Schlüsselbund gesichert | 已存储在钥匙串中 | guardada en el Llavero | enregistré dans le trousseau | salvata nel Portachiavi | сохранено в Связке ключей |
| `connect.test` | Test Connection | Verbindung testen | 测试连接 | Probar conexión | Tester la connexion | Prova connessione | Проверить подключение |
| `connect.testing` | Testing… | Wird getestet … | 正在测试… | Probando… | Test en cours… | Prova in corso… | Проверка… |
| `connect.connect` | Connect | Verbinden | 连接 | Conectar | Se connecter | Connetti | Подключиться |
| `connect.connecting` | Connecting to {target}… | Verbindung mit {target} wird hergestellt … | 正在连接到 {target}… | Conectando a {target}… | Connexion à {target}… | Connessione a {target}… | Подключение к {target}… |
| `connect.test.works` | Connection works | Verbindung funktioniert | 连接正常 | La conexión funciona | La connexion fonctionne | La connessione funziona | Подключение работает |
| `connect.test.home` | Signed in. Home folder: {home} | Angemeldet. Benutzerordner: {home} | 已登录。个人文件夹：{home} | Sesión iniciada. Carpeta de inicio: {home} | Connecté. Dossier de départ : {home} | Accesso eseguito. Cartella Inizio: {home} | Вход выполнен. Папка пользователя: {home} |
| `connect.problem.host` | Enter a server name or IP address | Gib einen Servernamen oder eine IP-Adresse ein. | 输入服务器名称或 IP 地址 | Introduce un nombre de servidor o una dirección IP | Saisissez un nom de serveur ou une adresse IP. | Inserisci un nome server o un indirizzo IP | Введите имя сервера или IP-адрес. |
| `connect.problem.key_file` | Choose a private key file | Wähle eine private Schlüsseldatei aus. | 选择私钥文件 | Elige un archivo de clave privada | Choisissez un fichier de clé privée. | Scegli un file di chiave privata | Выберите файл закрытого ключа. |
| `connect.problem.account` | Enter your Cloudflare account ID | Gib deine Cloudflare-Account-ID ein. | 输入你的 Cloudflare 账户 ID | Introduce el ID de tu cuenta de Cloudflare | Saisissez votre ID de compte Cloudflare. | Inserisci l’ID account Cloudflare | Введите ID учетной записи Cloudflare. |
| `connect.problem.endpoint` | Enter the endpoint URL | Gib die Endpunkt-URL ein. | 输入端点 URL | Introduce la URL del endpoint | Saisissez l’URL du point de terminaison. | Inserisci l’URL dell’endpoint | Введите URL конечной точки. |
| `connect.problem.access_key` | Enter the access key ID | Gib die Zugriffsschlüssel-ID ein. | 输入访问密钥 ID | Introduce el ID de la clave de acceso | Saisissez l’ID de clé d’accès. | Inserisci l’ID della chiave di accesso | Введите ID ключа доступа. |
| `connect.problem.secret_key` | Enter the secret access key | Gib den geheimen Zugriffsschlüssel ein. | 输入私密访问密钥 | Introduce la clave de acceso secreta | Saisissez la clé d’accès secrète. | Inserisci la chiave di accesso segreta | Введите секретный ключ доступа. |
| `connect.problem.container` | Choose a container | Wähle einen Container aus. | 选择容器 | Elige un contenedor | Choisissez un conteneur. | Scegli un container | Выберите контейнер. |
| `connect.problem.pod` | Choose a pod | Wähle einen Pod aus. | 选择 Pod | Elige un pod | Choisissez un pod. | Scegli un pod | Выберите под. |
| `connect.problem.share` | Enter the name of the share | Gib den Namen der Freigabe ein. | 输入共享点的名称 | Introduce el nombre del recurso compartido | Saisissez le nom du partage. | Inserisci il nome della condivisione | Введите имя общего ресурса. |
| `connect.s3.provider` | Provider | Anbieter | 提供商 | Proveedor | Fournisseur | Fornitore | Поставщик |
| `connect.s3.provider.custom` | Other S3-compatible | Anderer S3-kompatibler | 其他 S3 兼容服务 | Otro compatible con S3 | Autre service compatible S3 | Altro compatibile S3 | Другой, совместимый с S3 |
| `connect.s3.account` | Account ID | Account-ID | 账户 ID | ID de cuenta | ID de compte | ID account | ID учетной записи |
| `connect.s3.region` | Region | Region | 区域 | Región | Région | Regione | Регион |
| `connect.s3.region.from_profile` | from the profile | aus dem Profil | 来自配置文件 | del perfil | selon le profil | dal profilo | из профиля |
| `connect.s3.endpoint` | Endpoint | Endpunkt | 端点 | Endpoint | Point de terminaison | Endpoint | Конечная точка |
| `connect.s3.endpoint.follows_region` | follows the region | folgt der Region | 跟随区域 | según la región | selon la région | segue la regione | по региону |
| `connect.s3.profile` | Profile | Profil | 配置文件 | Perfil | Profil | Profilo | Профиль |
| `connect.s3.access_key_id` | Access key ID | Schlüssel-ID | 访问密钥 ID | ID de clave de acceso | ID de clé d’accès | ID chiave di accesso | ID ключа доступа |
| `connect.s3.secret_key` | Secret key | Geheimer Schlüssel | 私密密钥 | Clave secreta | Clé secrète | Chiave segreta | Секретный ключ |
| `connect.s3.secret_access_key` | Secret access key | Geheimer Zugriffsschlüssel | 私密访问密钥 | Clave de acceso secreta | Clé d’accès secrète | Chiave di accesso segreta | Секретный ключ доступа |
| `connect.s3.bucket` | Bucket | Bucket | Bucket | Bucket | Bucket | Bucket | Бакет |
| `connect.s3.path_style` | Path-style addressing | Pfadbasierte Adressierung | 路径样式寻址 | Direccionamiento por ruta | Adressage par chemin | Indirizzamento basato sul percorso | Адресация по пути |
| `connect.engine` | Engine | Laufzeit | 引擎 | Motor | Moteur | Motore | Среда |
| `connect.container` | Container | Container | 容器 | Contenedor | Conteneur | Container | Контейнер |
| `connect.containers.loading` | Looking for containers… | Container werden gesucht … | 正在查找容器… | Buscando contenedores… | Recherche de conteneurs… | Ricerca dei container… | Поиск контейнеров… |
| `connect.containers.stopped` | {image} · stopped, read-only | {image} · gestoppt, schreibgeschützt | {image} · 已停止，只读 | {image} · detenido, solo lectura | {image} · arrêté, lecture seule | {image} · arrestato, sola lettura | {image} · остановлен, только чтение |
| `connect.k8s.context` | Context | Kontext | 上下文 | Contexto | Contexte | Contesto | Контекст |
| `connect.k8s.current_context` | Current context | Aktueller Kontext | 当前上下文 | Contexto actual | Contexte actuel | Contesto attuale | Текущий контекст |
| `connect.k8s.current_context_lower` | current context | aktueller Kontext | 当前上下文 | contexto actual | contexte actuel | contesto attuale | текущий контекст |
| `connect.k8s.current_suffix` | {name} (current) | {name} (aktuell) | {name}（当前） | {name} (actual) | {name} (actuel) | {name} (attuale) | {name} (текущий) |
| `connect.k8s.namespace` | Namespace | Namespace | 命名空间 | Namespace | Espace de noms | Namespace | Пространство имен |
| `connect.k8s.loading_pods` | Loading pods… | Pods werden geladen … | 正在载入 Pod… | Cargando pods… | Chargement des pods… | Caricamento dei pod… | Загрузка подов… |
| `connect.k8s.pod` | Pod | Pod | Pod | Pod | Pod | Pod | Под |
| `connect.k8s.phase.running` | Running | Läuft | 正在运行 | En ejecución | En cours | In esecuzione | Работает |
| `connect.k8s.phase.pending` | Pending | Ausstehend | 待处理 | Pendiente | En attente | In sospeso | Ожидание |
| `connect.k8s.phase.succeeded` | Succeeded | Abgeschlossen | 已成功 | Completado | Terminé | Completato | Завершен |
| `connect.k8s.phase.failed` | Failed | Fehlgeschlagen | 已失败 | Fallido | Échec | Non riuscito | Сбой |
| `connect.k8s.phase.unknown` | Unknown | Unbekannt | 未知 | Desconocido | Inconnu | Sconosciuto | Неизвестно |
| `hostkey.unknown.title` | Unknown server | Unbekannter Server | 未知服务器 | Servidor desconocido | Serveur inconnu | Server sconosciuto | Неизвестный сервер |
| `hostkey.host` | Server | Server | 服务器 | Servidor | Serveur | Server | Сервер |
| `hostkey.key_type` | Key type | Schlüsseltyp | 密钥类型 | Tipo de clave | Type de clé | Tipo di chiave | Тип ключа |
| `hostkey.fingerprint` | Fingerprint | Fingerabdruck | 指纹 | Huella | Empreinte | Impronta | Отпечаток |
| `hostkey.accept` | Connect and Remember | Verbinden und merken | 连接并记住 | Conectar y recordar | Se connecter et mémoriser | Connetti e ricorda | Подключиться и запомнить |
| `hostkey.changed.title` | The server's key has changed | Der Schlüssel des Servers hat sich geändert | 服务器的密钥已更改 | La clave del servidor ha cambiado | La clé du serveur a changé | La chiave del server è cambiata | Ключ сервера изменился |
| `hostkey.copy_command` | Copy Command | Befehl kopieren | 拷贝命令 | Copiar comando | Copier la commande | Copia comando | Скопировать команду |
| `hostkey.close` | Close | Schließen | 关闭 | Cerrar | Fermer | Chiudi | Закрыть |
| `dialog.transfer.copy` | Copy | Kopieren | 拷贝 | Copiar | Copier | Copia | Копировать |
| `dialog.transfer.move` | Move | Bewegen | 移动 | Mover | Déplacer | Sposta | Переместить |
| `dialog.transfer.extract` | Extract | Entpacken | 解压缩 | Extraer | Extraire | Estrai | Извлечь |
| `dialog.transfer.mode` | Action | Aktion | 操作 | Acción | Action | Azione | Действие |
| `dialog.conflict.label` | If an item already exists | Wenn ein Objekt schon existiert | 如果项目已存在 | Si un ítem ya existe | Si un élément existe déjà | Se un elemento esiste già | Если объект уже существует |
| `dialog.conflict.replace` | Replace | Ersetzen | 替换 | Reemplazar | Remplacer | Sostituisci | Заменить |
| `dialog.conflict.skip` | Skip | Überspringen | 跳过 | Omitir | Ignorer | Salta | Пропустить |
| `dialog.conflict.if_newer` | Replace if newer | Ersetzen, wenn neuer | 较新时替换 | Reemplazar si es más reciente | Remplacer si plus récent | Sostituisci se più recente | Заменить, если новее |
| `dialog.conflict.keep_both` | Keep both | Beide behalten | 保留两者 | Conservar ambos | Garder les deux | Mantieni entrambi | Оставить оба |
| `dialog.conflict.ask` | Ask | Nachfragen | 询问 | Preguntar | Demander | Chiedi | Спрашивать |
| `conflict.title` | “{name}” already exists in “{folder}” | „{name}“ ist bereits in „{folder}“ vorhanden | “{folder}”中已存在“{name}” | “{name}” ya existe en “{folder}” | « {name} » existe déjà dans « {folder} » | “{name}” esiste già in “{folder}” | «{name}» уже есть в папке «{folder}» |
| `conflict.incoming.copy` | Being copied | Wird kopiert | 正在拷贝 | Se está copiando | En cours de copie | In copia | Копируется |
| `conflict.incoming.move` | Being moved | Wird bewegt | 正在移动 | Se está moviendo | En cours de déplacement | In spostamento | Перемещается |
| `conflict.incoming.extract` | Being extracted | Wird entpackt | 正在解压缩 | Se está extrayendo | En cours d’extraction | In estrazione | Извлекается |
| `conflict.existing` | Already there | Bereits vorhanden | 已存在 | Ya existente | Déjà présent | Già presente | Уже существует |
| `conflict.newer` | Newer | Neuer | 较新 | Más reciente | Plus récent | Più recente | Новее |
| `conflict.folder` | Folder | Ordner | 文件夹 | Carpeta | Dossier | Cartella | Папка |
| `conflict.merge` | Merge | Zusammenführen | 合并 | Combinar | Fusionner | Unisci | Объединить |
| `dialog.route.from` | From | Von | 从 | De | De | Da | Откуда |
| `dialog.route.on` | on {location} | auf {location} | 位于 {location} | en {location} | sur {location} | su {location} | на {location} |
| `dialog.items.more.one` | and {count} more | und {count} weiteres | 及另外 {count} 项 | y {count} más | et {count} autre | e {count} in più | и еще {count} |
| `dialog.items.more.other` | and {count} more | und {count} weitere | 及另外 {count} 项 | y {count} más | et {count} autres | e altri {count} | и еще {count} |
| `dialog.delete.button` | Delete | Löschen | 删除 | Eliminar | Supprimer | Elimina | Удалить |
| `dialog.empty_trash.measuring` | measuring size… | Größe wird berechnet … | 正在计算大小… | calculando tamaño… | calcul de la taille… | calcolo dimensioni… | вычисление размера… |
| `dialog.empty_trash.button` | Empty Trash | Papierkorb entleeren | 清倒废纸篓 | Vaciar papelera | Vider la corbeille | Svuota il Cestino | Очистить Корзину |
| `whats_new.title` | What's New in Manta | Neu in Manta | Manta 的新功能 | Novedades de Manta | Nouveautés de Manta | Novità di Manta | Что нового в Manta |
| `whats_new.drop` | Dragging now copies | Ziehen kopiert jetzt | 拖移现在默认拷贝 | Arrastrar ahora copia | Glisser copie désormais | Trascinare ora copia | Перетаскивание теперь копирует |
| `whats_new.conflicts` | Ask on name conflicts | Nachfragen bei Namenskonflikten | 名称冲突时询问 | Preguntar en conflictos de nombre | Demander en cas de conflit de noms | Chiedi in caso di conflitti di nome | Вопрос при совпадении имен |
| `whats_new.languages` | 7 languages | 7 Sprachen | 7 种语言 | 7 idiomas | 7 langues | 7 lingue | 7 языков |
| `whats_new.location` | See where you are | Sehen, wo du bist | 随时知道你在哪里 | Siempre sabes dónde estás | Vous savez où vous êtes | Sai sempre dove ti trovi | Видно, где вы находитесь |
| `whats_new.menu_bar` | Complete menu bar | Vollständige Menüleiste | 完整的菜单栏 | Barra de menús completa | Barre des menus complète | Barra dei menu completa | Полная строка меню |
| `whats_new.ok` | Got It | Verstanden | 知道了 | Entendido | J’ai compris | Ho capito | Понятно |
| `dialog.rename.title` | Rename “{name}” | „{name}“ umbenennen | 重新命名“{name}” | Renombrar “{name}” | Renommer « {name} » | Rinomina “{name}” | Переименовать «{name}» |
| `dialog.rename.button` | Rename | Umbenennen | 重新命名 | Renombrar | Renommer | Rinomina | Переименовать |
| `dialog.rename.field` | New name | Neuer Name | 新名称 | Nombre nuevo | Nouveau nom | Nuovo nome | Новое имя |
| `dialog.new_folder.title` | New Folder | Neuer Ordner | 新建文件夹 | Nueva carpeta | Nouveau dossier | Nuova cartella | Новая папка |
| `dialog.new_folder.title_in` | New Folder in “{name}” | Neuer Ordner in „{name}“ | 在“{name}”中新建文件夹 | Nueva carpeta en “{name}” | Nouveau dossier dans « {name} » | Nuova cartella in “{name}” | Новая папка в «{name}» |
| `dialog.new_folder.button` | Create | Erstellen | 创建 | Crear | Créer | Crea | Создать |
| `dialog.new_folder.field` | Folder name | Ordnername | 文件夹名称 | Nombre de la carpeta | Nom du dossier | Nome cartella | Имя папки |
| `dialog.new_folder.default_name` | untitled folder | Neuer Ordner | 未命名文件夹 | carpeta sin título | dossier sans titre | cartella senza titolo | новая папка |
| `dialog.new_file.title` | New File | Neue Datei | 新建文件 | Nuevo archivo | Nouveau fichier | Nuovo file | Новый файл |
| `dialog.new_file.title_in` | New File in “{name}” | Neue Datei in „{name}“ | 在“{name}”中新建文件 | Nuevo archivo en “{name}” | Nouveau fichier dans « {name} » | Nuovo file in “{name}” | Новый файл в «{name}» |
| `dialog.new_file.button` | Create | Erstellen | 创建 | Crear | Créer | Crea | Создать |
| `dialog.new_file.field` | File name | Dateiname | 文件名称 | Nombre del archivo | Nom du fichier | Nome del file | Имя файла |
| `dialog.new_file.default_name` | untitled.txt | Neue Datei.txt | 未命名文件.txt | archivo sin título.txt | fichier sans titre.txt | file senza titolo.txt | новый файл.txt |
| `dialog.name.in_folder` | In “{folder}” | In „{folder}“ | 位于“{folder}” | En “{folder}” | Dans « {folder} » | In “{folder}” | В папке «{folder}» |
| `dialog.name.exists` | “{name}” already exists in this folder | „{name}“ ist in diesem Ordner bereits vorhanden. | 此文件夹中已存在“{name}” | “{name}” ya existe en esta carpeta | « {name} » existe déjà dans ce dossier. | “{name}” esiste già in questa cartella | «{name}» уже есть в этой папке. |
| `compress.title.single` | Compress “{name}” | „{name}“ komprimieren | 压缩“{name}” | Comprimir “{name}” | Compresser « {name} » | Comprimi “{name}” | Сжать «{name}» |
| `compress.title.one` | Compress {count} item | {count} Objekt komprimieren | 压缩 {count} 项 | Comprimir {count} ítem | Compresser {count} élément | Comprimi {count} elemento | Сжать {count} объект |
| `compress.title.other` | Compress {count} items | {count} Objekte komprimieren | 压缩 {count} 项 | Comprimir {count} ítems | Compresser {count} éléments | Comprimi {count} elementi | Сжать {count} объекта |
| `compress.format` | Format | Format | 格式 | Formato | Format | Formato | Формат |
| `compress.name` | Name | Name | 名称 | Nombre | Nom | Nome | Имя |
| `compress.name.a11y` | Archive name | Archivname | 归档名称 | Nombre del archivo comprimido | Nom de l’archive | Nome archivio | Имя архива |
| `compress.default_stem` | Archive | Archiv | 归档 | Archivo | Archive | Archivio | Архив |
| `compress.level` | Level | Stufe | 级别 | Nivel | Niveau | Livello | Уровень |
| `compress.faster` | Faster | Schneller | 更快 | Más rápido | Plus rapide | Più veloce | Быстрее |
| `compress.smaller` | Smaller | Kleiner | 更小 | Más pequeño | Plus petit | Più piccolo | Меньше |
| `compress.exclude_junk` | Exclude .DS_Store and __MACOSX | .DS_Store und __MACOSX auslassen | 排除 .DS_Store 和 __MACOSX | Excluir .DS_Store y __MACOSX | Exclure .DS_Store et __MACOSX | Escludi .DS_Store e __MACOSX | Исключить .DS_Store и __MACOSX |
| `compress.button` | Compress | Komprimieren | 压缩 | Comprimir | Compresser | Comprimi | Сжать |
| `goto.a11y` | Go to path | Gehe zum Pfad | 前往路径 | Ir a la ruta | Aller au chemin | Vai al percorso | Перейти по пути |
| `goto.recent` | Recent | Zuletzt benutzt | 最近使用 | Recientes | Récents | Recenti | Недавние |
| `goto.matching` | Matching folders | Passende Ordner | 匹配的文件夹 | Carpetas coincidentes | Dossiers correspondants | Cartelle corrispondenti | Подходящие папки |
| `goto.no_folders` | No folders here | Hier gibt es keine Ordner | 此处没有文件夹 | Aquí no hay carpetas | Aucun dossier ici | Nessuna cartella qui | Здесь нет папок |
| `goto.no_match` | No folders start with “{prefix}” | Keine Ordner beginnen mit „{prefix}“ | 没有以“{prefix}”开头的文件夹 | Ninguna carpeta empieza por “{prefix}” | Aucun dossier ne commence par « {prefix} » | Nessuna cartella inizia con “{prefix}” | Нет папок, начинающихся с «{prefix}» |
| `goto.footer.new_tab_on` | Opens in a new tab on  | Öffnet in neuem Tab auf  | 在新标签页中打开，位于： | Se abre en una pestaña nueva en  | S’ouvre dans un nouvel onglet sur  | Si apre in una nuova scheda su  | Откроется в новой вкладке на  |

## App: Panels

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugins.choose_folder` | Choose a Plugin Folder | Plugin-Ordner auswählen | 选择插件文件夹 | Elige una carpeta de plugin | Choisir un dossier de plug-in | Scegli una cartella di plug-in | Выберите папку плагина |
| `plugins.choosing` | Choosing… | Auswählen … | 正在选择… | Eligiendo… | Sélection… | Selezione… | Выбор… |
| `plugins.install_plugin` | Install Plugin… | Plugin installieren … | 安装插件… | Instalar plugin… | Installer un plug-in… | Installa plug-in… | Установить плагин… |
| `plugins.open_folder` | Open Plugins Folder | Plugin-Ordner öffnen | 打开插件文件夹 | Abrir carpeta de plugins | Ouvrir le dossier des plug-ins | Apri cartella plug-in | Открыть папку плагинов |
| `plugins.reload` | Reload | Neu laden | 重新载入 | Recargar | Recharger | Ricarica | Обновить |
| `plugins.section.builtin` | Built-in | Integriert | 内建 | Integrados | Intégrés | Integrati | Встроенные |
| `plugins.section.installed` | Installed | Installiert | 已安装 | Instalados | Installés | Installati | Установленные |
| `plugins.section.available` | Available | Verfügbar | 可用 | Disponibles | Disponibles | Disponibili | Доступные |
| `plugins.none_installed` | No plugins installed yet | Noch keine Plugins installiert | 尚未安装插件 | Aún no hay plugins instalados | Aucun plug-in installé pour l’instant | Nessun plug-in ancora installato | Плагины еще не установлены |
| `plugins.kind.builtin` | Built-in | Integriert | 内建 | Integrado | Intégré | Integrato | Встроенный |
| `plugins.kind.external` | External | Extern | 外部 | Externo | Externe | Esterno | Внешний |
| `plugins.kind.invalid` | Can't load | Nicht ladbar | 无法载入 | No se puede cargar | Chargement impossible | Impossibile caricare | Не загружается |
| `plugins.kind.replaced` | Replaced by built-in | Durch integrierte Version ersetzt | 已被内建版本取代 | Sustituido por el integrado | Remplacé par la version intégrée | Sostituito dalla versione integrata | Заменен встроенным |
| `plugins.by` | by {author} | von {author} | 作者：{author} | de {author} | par {author} | di {author} | автор: {author} |
| `plugins.turn_on` | Turn On | Einschalten | 打开 | Activar | Activer | Attiva | Включить |
| `plugins.turn_off` | Turn Off | Ausschalten | 关闭 | Desactivar | Désactiver | Disattiva | Выключить |
| `plugins.settings` | Settings | Einstellungen | 设置 | Ajustes | Réglages | Impostazioni | Настройки |
| `plugins.show_log` | Show Log | Protokoll einblenden | 显示日志 | Mostrar registro | Afficher le journal | Mostra log | Показать журнал |
| `plugins.show_log_n` | Show Log ({n}) | Protokoll einblenden ({n}) | 显示日志（{n}） | Mostrar registro ({n}) | Afficher le journal ({n}) | Mostra log ({n}) | Показать журнал ({n}) |
| `plugins.uninstall` | Uninstall | Deinstallieren | 卸载 | Desinstalar | Désinstaller | Disinstalla | Удалить |
| `plugins.show_folder` | Show Folder | Ordner zeigen | 显示文件夹 | Mostrar carpeta | Afficher le dossier | Mostra cartella | Показать папку |
| `plugins.runs` | Runs | Befehl | 命令 | Comando | Commande | Comando | Команда |
| `plugins.in` | In | Ordner | 文件夹 | Carpeta | Dossier | Cartella | Папка |
| `plugins.replace` | Replace | Ersetzen | 替换 | Reemplazar | Remplacer | Sostituisci | Заменить |
| `plugins.update.available` | Version {version} comes with this Manta | Version {version} ist in diesem Manta enthalten | 此版本的 Manta 附带版本 {version} | Esta versión de Manta incluye la versión {version} | La version {version} est fournie avec ce Manta | La versione {version} è inclusa in questo Manta | Версия {version} входит в эту версию Manta |
| `plugins.update.installed` | installed: {version} | installiert: {version} | 已安装：{version} | instalada: {version} | installée : {version} | installata: {version} | установлена: {version} |
| `plugins.update` | Update | Aktualisieren | 更新 | Actualizar | Mettre à jour | Aggiorna | Обновить |
| `plugins.updating` | Updating… | Aktualisieren … | 正在更新… | Actualizando… | Mise à jour… | Aggiornamento… | Обновление… |
| `plugins.install` | Install | Installieren | 安装 | Instalar | Installer | Installa | Установить |
| `plugins.installing` | Installing… | Installieren … | 正在安装… | Instalando… | Installation… | Installazione… | Установка… |
| `plugins.choose_file` | Choose a Plugin File | Plugin-Datei auswählen | 选择插件文件 | Elige un archivo de plugin | Choisir un fichier de plug-in | Scegli un file di plug-in | Выберите файл плагина |
| `plugins.file_filter` | Manta plugins | Manta-Plugins | Manta 插件 | Plugins de Manta | Plug-ins Manta | Plug-in di Manta | Плагины Manta |
| `plugins.install_from_file` | From a File (.mantaplugin, .zip)… | Aus einer Datei (.mantaplugin, .zip) … | 从文件安装（.mantaplugin、.zip）… | Desde un archivo (.mantaplugin, .zip)… | À partir d’un fichier (.mantaplugin, .zip)… | Da un file (.mantaplugin, .zip)… | Из файла (.mantaplugin, .zip)… |
| `plugins.install_from_folder` | From a Folder… | Aus einem Ordner … | 从文件夹安装… | Desde una carpeta… | À partir d’un dossier… | Da una cartella… | Из папки… |
| `plugins.kind.dev` | Development | Entwicklung | 开发 | Desarrollo | Développement | Sviluppo | Разработка |
| `plugins.review.permissions` | What it may do | Was es darf | 允许的操作 | Lo que puede hacer | Ce qu’il peut faire | Cosa può fare | Что ему разрешено |
| `plugins.review.no_permissions` | Nothing beyond its own folder and data | Nichts außer dem eigenen Ordner und den eigenen Daten | 仅限其自身的文件夹和数据 | Nada más allá de su propia carpeta y sus datos | Rien au-delà de son propre dossier et de ses données | Nient’altro che la propria cartella e i propri dati | Ничего, кроме собственной папки и данных |
| `plugins.perm.read_files` | Reads your files | Liest deine Dateien | 读取你的文件 | Lee tus archivos | Lit vos fichiers | Legge i tuoi file | Читает ваши файлы |
| `plugins.perm.write_files` | Changes or deletes files | Ändert oder löscht Dateien | 更改或删除文件 | Modifica o elimina archivos | Modifie ou supprime des fichiers | Modifica o elimina file | Изменяет или удаляет файлы |
| `plugins.perm.network` | Uses the network | Greift auf das Netzwerk zu | 使用网络 | Usa la red | Utilise le réseau | Usa la rete | Использует сеть |
| `plugins.perm.run_programs` | Runs other programs | Startet andere Programme | 运行其他程序 | Ejecuta otros programas | Lance d’autres programmes | Esegue altri programmi | Запускает другие программы |
| `plugins.perm.terminal` | Runs commands in the Terminal | Führt Befehle im Terminal aus | 在终端中运行命令 | Ejecuta comandos en el Terminal | Exécute des commandes dans le Terminal | Esegue comandi nel Terminale | Выполняет команды в Терминале |
| `plugins.review.from` | From | Quelle | 来源 | Origen | Source | Origine | Источник |
| `plugins.review.source_code` | Source code | Quellcode | 源代码 | Código fuente | Code source | Codice sorgente | Исходный код |
| `plugins.review.website` | Website | Webseite | 网站 | Sitio web | Site web | Sito web | Сайт |
| `plugins.review.license` | Licence | Lizenz | 许可 | Licencia | Licence | Licenza | Лицензия |
| `plugins.review.license_unknown` | Not specified | Nicht angegeben | 未指定 | Sin especificar | Non précisée | Non specificata | Не указана |
| `plugins.review.contents` | Contents | Inhalt | 内容 | Contenido | Contenu | Contenuto | Содержимое |
| `plugins.review.size.one` | {count} file, {size} | {count} Datei, {size} | {count} 个文件，{size} | {count} archivo, {size} | {count} fichier, {size} | {size} in {count} file | {count} файл, {size} |
| `plugins.review.size.other` | {count} files, {size} | {count} Dateien, {size} | {count} 个文件，{size} | {count} archivos, {size} | {count} fichiers, {size} | {size} in {count} file | {count} файла, {size} |
| `plugins.review.installed` | Installed | Installiert | 已安装 | Instalado | Installé | Installato | Установлено |
| `plugins.review.replaces` | Version {version} (will be replaced) | Version {version} (wird ersetzt) | 版本 {version}（将被替换） | Versión {version} (se reemplazará) | Version {version} (sera remplacée) | Versione {version} (verrà sostituita) | Версия {version} (будет заменена) |
| `plugins.page.installed` | Installed | Installiert | 已安装 | Instalados | Installés | Installati | Установленные |
| `plugins.page.discover` | Discover | Entdecken | 发现 | Descubrir | Découvrir | Scopri | Обзор |
| `plugins.discover.refresh` | Refresh | Aktualisieren | 刷新 | Actualizar | Actualiser | Aggiorna | Обновить |
| `plugins.discover.web` | On the Web | Im Web | 在网页上查看 | En la web | Sur le web | Sul web | В интернете |
| `plugins.discover.loading` | Loading the plugin directory… | Plugin-Verzeichnis wird geladen … | 正在载入插件目录… | Cargando el directorio de plugins… | Chargement de l’annuaire des plug-ins… | Caricamento della directory dei plug-in… | Загрузка каталога плагинов… |
| `plugins.discover.failed` | Couldn't load the plugin directory | Das Plugin-Verzeichnis konnte nicht geladen werden | 无法载入插件目录 | No se pudo cargar el directorio de plugins | Impossible de charger l’annuaire des plug-ins | Impossibile caricare la directory dei plug-in | Не удалось загрузить каталог плагинов |
| `plugins.discover.empty` | No community plugins yet | Noch keine Community-Plugins | 暂无社区插件 | Aún no hay plugins de la comunidad | Aucun plug-in de la communauté pour l’instant | Ancora nessun plug-in della community | Плагинов сообщества пока нет |
| `plugins.discover.min_version` | Manta {version} or newer | ab Manta {version} | Manta {version} 及以上 | Manta {version} o posterior | Manta {version} ou ultérieur | Manta {version} o successiva | Manta {version} и новее |
| `plugins.discover.installed_version` | Installed: version {version} | Installiert: Version {version} | 已安装：版本 {version} | Instalado: versión {version} | Installé : version {version} | Installato: versione {version} | Установлено: версия {version} |
| `plugins.discover.needs_manta` | Needs Manta {version} or newer | Benötigt Manta {version} oder neuer | 需要 Manta {version} 或更高版本 | Requiere Manta {version} o posterior | Nécessite Manta {version} ou ultérieur | Richiede Manta {version} o successiva | Требуется Manta {version} или новее |
| `plugins.discover.wrong_platform` | Doesn't run on this Mac | Läuft nicht auf diesem Mac | 无法在此 Mac 上运行 | No funciona en este Mac | Ne fonctionne pas sur ce Mac | Non funziona su questo Mac | Не работает на этом Mac |
| `plugins.discover.downloading` | Downloading… | Wird geladen … | 正在下载… | Descargando… | Téléchargement… | Download in corso… | Загрузка… |
| `plugins.review.checksum` | Checksum | Prüfsumme | 校验和 | Suma de control | Somme de contrôle | Somma di controllo | Контрольная сумма |
| `plugins.review.checksum_ok` | SHA-256 matches the plugin directory | SHA-256 stimmt mit dem Plugin-Verzeichnis überein | SHA-256 与插件目录一致 | El SHA-256 coincide con el directorio de plugins | Le SHA-256 correspond à l’annuaire des plug-ins | Lo SHA-256 corrisponde alla directory dei plug-in | SHA-256 совпадает с каталогом плагинов |
| `plugins.sandboxed` | Sandboxed | Sandbox | 沙盒 | Aislado | Isolé | In sandbox | В песочнице |
| `plugins.not_sandboxed` | Not sandboxed | Ohne Sandbox | 未使用沙盒 | Sin aislar | Non isolé | Senza sandbox | Без песочницы |
| `plugin_ui.close_panel` | Close {title} | {title} schließen | 关闭“{title}” | Cerrar {title} | Fermer {title} | Chiudi {title} | Закрыть «{title}» |
| `plugin_ui.drag_resize` | Drag to resize | Zum Ändern der Größe ziehen | 拖移以调整大小 | Arrastra para cambiar el tamaño | Faites glisser pour redimensionner | Trascina per ridimensionare | Перетащите, чтобы изменить размер |
| `plugin_ui.image.too_large` | too large to preview ({size} MB) | zu groß für die Vorschau ({size} MB) | 太大，无法预览（{size} MB） | demasiado grande para previsualizar ({size} MB) | trop volumineux pour l’aperçu ({size} Mo) | troppo grande per l’anteprima ({size} MB) | слишком большой для просмотра ({size} МБ) |
| `plugin_ui.image.not_found` | Image not found | Bild nicht gefunden | 未找到图像 | No se ha encontrado la imagen | Image introuvable | Immagine non trovata | Изображение не найдено |
| `plugin_ui.image.cant_preview` | Can't preview: {error} | Keine Vorschau möglich: {error} | 无法预览：{error} | No se puede previsualizar: {error} | Aperçu impossible : {error} | Anteprima non disponibile: {error} | Просмотр невозможен: {error} |
| `plugin_ui.waiting` | Waiting for the plugin… | Warten auf das Plugin … | 正在等待插件… | Esperando al plugin… | En attente du plug-in… | In attesa del plug-in… | Ожидание плагина… |
| `transfers.title` | Transfers | Übertragungen | 传输 | Transferencias | Transferts | Trasferimenti | Передачи |
| `transfers.active.one` | {count} active | {count} aktiv | {count} 个进行中 | {count} activa | {count} actif | {count} attivo | Активно: {count} |
| `transfers.active.other` | {count} active | {count} aktiv | {count} 个进行中 | {count} activas | {count} actifs | {count} attivi | Активно: {count} |
| `transfers.all_done` | All done | Alles erledigt | 全部完成 | Todo terminado | Tout est terminé | Tutto completato | Все готово |
| `transfers.hide` | Hide Transfers | Übertragungen ausblenden | 隐藏传输 | Ocultar transferencias | Masquer les transferts | Nascondi trasferimenti | Скрыть передачи |
| `transfers.clear_finished` | Clear Finished | Erledigte entfernen | 清除已完成 | Borrar terminadas | Effacer les terminés | Rimuovi completati | Убрать завершенные |
| `transfers.empty` | No transfers | Keine Übertragungen | 无传输 | No hay transferencias | Aucun transfert | Nessun trasferimento | Нет передач |
| `transfers.stop` | Stop | Stoppen | 停止 | Detener | Arrêter | Interrompi | Остановить |
| `transfers.issues.one` | {count} issue | {count} Problem | {count} 个问题 | {count} problema | {count} problème | {count} problema | {count} проблема |
| `transfers.issues.other` | {count} issues | {count} Probleme | {count} 个问题 | {count} problemas | {count} problèmes | {count} problemi | {count} проблемы |
| `transfers.waiting_size` | Waiting · {size} | Wartet · {size} | 正在等待 · {size} | En espera · {size} | En attente · {size} | In attesa · {size} | Ожидание · {size} |
| `transfers.waiting` | Waiting to start | Wartet auf Start | 正在等待开始 | Esperando para empezar | En attente de démarrage | In attesa di avvio | Ожидание запуска |
| `transfers.deleted_of` | {done} of {total} deleted | {done} von {total} gelöscht | 已删除 {done}/{total} | {done} de {total} eliminados | {done} sur {total} supprimés | {done} di {total} eliminati | Удалено {done} из {total} |
| `transfers.speed_left` |  · {speed} · {eta} left |  · {speed} · noch {eta} |  · {speed} · 剩余 {eta} |  · {speed} · quedan {eta} |  · {speed} · {eta} restant(s) |  · {speed} · ancora {eta} |  · {speed} · осталось {eta} |
| `transfers.items.one` | {count} item | {count} Objekt | {count} 项 | {count} ítem | {count} élément | {count} elemento | {count} объект |
| `transfers.items.other` | {count} items | {count} Objekte | {count} 项 | {count} ítems | {count} éléments | {count} elementi | {count} объекта |
| `transfers.files.one` | {count} file | {count} Datei | {count} 个文件 | {count} archivo | {count} fichier | {count} file | {count} файл |
| `transfers.files.other` | {count} files | {count} Dateien | {count} 个文件 | {count} archivos | {count} fichiers | {count} file | {count} файла |
| `transfers.done` | Done · {what} | Fertig · {what} | 完成 · {what} | Terminado · {what} | Terminé · {what} | Completato · {what} | Готово · {what} |
| `transfers.failed` | Failed · {error} | Fehlgeschlagen · {error} | 失败 · {error} | Error · {error} | Échec · {error} | Non riuscito · {error} | Сбой · {error} |
| `transfers.stopped` | Stopped | Gestoppt | 已停止 | Detenido | Arrêté | Interrotto | Остановлено |
| `sync.title` | Compare & Sync | Vergleichen und synchronisieren | 比较与同步 | Comparar y sincronizar | Comparer et synchroniser | Confronta e sincronizza | Сравнение и синхронизация |
| `sync.left` | Left | Links | 左侧 | Izquierda | Gauche | Sinistra | Слева |
| `sync.right` | Right | Rechts | 右侧 | Derecha | Droite | Destra | Справа |
| `sync.mirror_ltr` | Mirror left to right | Links nach rechts spiegeln | 从左侧镜像到右侧 | Reflejar de izquierda a derecha | Copier en miroir de gauche à droite | Rispecchia da sinistra a destra | Зеркалировать слева направо |
| `sync.mirror_rtl` | Mirror right to left | Rechts nach links spiegeln | 从右侧镜像到左侧 | Reflejar de derecha a izquierda | Copier en miroir de droite à gauche | Rispecchia da destra a sinistra | Зеркалировать справа налево |
| `sync.newer_wins` | Newer file wins | Neuere Datei gewinnt | 保留较新的文件 | Gana el archivo más reciente | Le fichier le plus récent l’emporte | Prevale il file più recente | Более новый файл побеждает |
| `sync.click_hide` | Click to hide | Zum Ausblenden klicken | 点按以隐藏 | Haz clic para ocultar | Cliquez pour masquer | Fai clic per nascondere | Нажмите, чтобы скрыть |
| `sync.click_show` | Click to show | Zum Einblenden klicken | 点按以显示 | Haz clic para mostrar | Cliquez pour afficher | Fai clic per mostrare | Нажмите, чтобы показать |
| `sync.compare_failed` | Couldn't compare: {error} | Vergleich nicht möglich: {error} | 无法比较：{error} | No se ha podido comparar: {error} | Comparaison impossible : {error} | Impossibile confrontare: {error} | Не удалось сравнить: {error} |
| `sync.include_subfolders` | Include subfolders | Mit Unterordnern | 包括子文件夹 | Incluir subcarpetas | Inclure les sous-dossiers | Includi sottocartelle | С подпапками |
| `sync.hidden_files` | Hidden files | Unsichtbare Dateien | 隐藏文件 | Archivos ocultos | Fichiers masqués | File nascosti | Скрытые файлы |
| `sync.delete_extra` | Delete extra files on target | Überzählige im Ziel löschen | 删除目标位置中多余的文件 | Eliminar archivos sobrantes en el destino | Supprimer les fichiers en trop dans la cible | Elimina file in più nella destinazione | Удалять лишние файлы в цели |
| `sync.delete_extra.one_way` | Only for one-way sync | Nur beim Synchronisieren in eine Richtung | 仅适用于单向同步 | Solo para sincronización en un sentido | Uniquement pour la synchronisation unidirectionnelle | Solo per la sincronizzazione unidirezionale | Только при односторонней синхронизации |
| `sync.sync` | Sync | Synchronisieren | 同步 | Sincronizar | Synchroniser | Sincronizza | Синхронизировать |
| `sync.sync_n.one` | Sync {count} Item | {count} Objekt synchronisieren | 同步 {count} 项 | Sincronizar {count} ítem | Synchroniser {count} élément | Sincronizza {count} elemento | Синхронизировать {count} объект |
| `sync.sync_n.other` | Sync {count} Items | {count} Objekte synchronisieren | 同步 {count} 项 | Sincronizar {count} ítems | Synchroniser {count} éléments | Sincronizza {count} elementi | Синхронизировать {count} объекта |
| `sync.compare_again` | Compare Again | Erneut vergleichen | 重新比较 | Volver a comparar | Comparer à nouveau | Confronta di nuovo | Сравнить снова |
| `sync.not_compared` | Not compared yet | Noch nicht verglichen | 尚未比较 | Aún no se ha comparado | Pas encore comparé | Non ancora confrontati | Еще не сравнивалось |
| `sync.no_match` | Nothing matches the filter | Nichts entspricht dem Filter | 没有与过滤条件匹配的内容 | Nada coincide con el filtro | Aucun élément ne correspond au filtre | Nessun elemento corrisponde al filtro | Ничто не соответствует фильтру |
| `sync.in_sync` | Both folders are in sync | Beide Ordner sind synchron | 两个文件夹已同步 | Ambas carpetas están sincronizadas | Les deux dossiers sont synchronisés | Le due cartelle sono sincronizzate | Обе папки синхронизированы |
| `sync.col.status` | Status | Status | 状态 | Estado | État | Stato | Статус |
| `sync.col.path` | Path | Pfad | 路径 | Ruta | Chemin | Percorso | Путь |
| `sync.col.action` | Sync will | Aktion | 同步操作 | Acción | Action | Azione | Действие |
| `sync.action.copy_right` | Copy → | Kopieren → | 拷贝 → | Copiar → | Copier → | Copia → | Копировать → |
| `sync.action.copy_left` | ← Copy | ← Kopieren | ← 拷贝 | ← Copiar | ← Copier | ← Copia | ← Копировать |
| `sync.action.delete_right` | Delete right | Rechts löschen | 删除右侧 | Eliminar a la derecha | Supprimer à droite | Elimina a destra | Удалить справа |
| `sync.action.delete_left` | Delete left | Links löschen | 删除左侧 | Eliminar a la izquierda | Supprimer à gauche | Elimina a sinistra | Удалить слева |
| `compare.only_left` | Only left | Nur links | 仅左侧 | Solo a la izquierda | Seulement à gauche | Solo a sinistra | Только слева |
| `compare.only_right` | Only right | Nur rechts | 仅右侧 | Solo a la derecha | Seulement à droite | Solo a destra | Только справа |
| `compare.only_a` | Only in A | Nur in A | 仅在 A 中 | Solo en A | Seulement dans A | Solo in A | Только в A |
| `compare.only_b` | Only in B | Nur in B | 仅在 B 中 | Solo en B | Seulement dans B | Solo in B | Только в B |
| `compare.a_newer` | A newer | A neuer | A 较新 | A más reciente | A plus récent | A più recente | A новее |
| `compare.b_newer` | B newer | B neuer | B 较新 | B más reciente | B plus récent | B più recente | B новее |
| `compare.identical` | Identical | Identisch | 相同 | Idénticos | Identique | Identici | Одинаковые |
| `compare.left_newer` | Left newer | Links neuer | 左侧较新 | Izquierda más reciente | Gauche plus récent | Sinistra più recente | Слева новее |
| `compare.right_newer` | Right newer | Rechts neuer | 右侧较新 | Derecha más reciente | Droite plus récent | Destra più recente | Справа новее |
| `compare.different` | Different | Verschieden | 不同 | Diferentes | Différent | Diversi | Различаются |
| `containers.opens_read_only` | Opens read-only | Wird schreibgeschützt geöffnet | 以只读方式打开 | Se abre en modo de solo lectura | S’ouvre en lecture seule | Si apre in sola lettura | Открывается только для чтения |
| `containers.start` | Start | Starten | 启动 | Iniciar | Démarrer | Avvia | Запустить |
| `containers.restart` | Restart | Neu starten | 重新启动 | Reiniciar | Redémarrer | Riavvia | Перезапустить |
| `containers.stop` | Stop | Beenden | 停止 | Detener | Arrêter | Arresta | Остановить |
| `containers.starting` | Starting… | Wird gestartet … | 正在启动… | Iniciando… | Démarrage… | Avvio… | Запуск… |
| `containers.restarting` | Restarting… | Wird neu gestartet … | 正在重新启动… | Reiniciando… | Redémarrage… | Riavvio… | Перезапуск… |
| `containers.stopping` | Stopping… | Wird beendet … | 正在停止… | Deteniendo… | Arrêt… | Arresto… | Остановка… |
| `containers.looking` | Looking for containers… | Suche nach Containern … | 正在查找容器… | Buscando contenedores… | Recherche de conteneurs… | Ricerca dei container… | Поиск контейнеров… |
| `containers.group.stopped` | stopped | gestoppt | 已停止 | detenidos | arrêté | arrestati | остановлены |
| `containers.group.running` | {n} running | {n} aktiv | {n} 个正在运行 | {n} en ejecución | {n} en cours | {n} in esecuzione | работает: {n} |
| `containers.group.running_of` | {n}/{total} running | {n}/{total} aktiv | {n}/{total} 个正在运行 | {n}/{total} en ejecución | {n}/{total} en cours | {n}/{total} in esecuzione | работает: {n}/{total} |
| `containers.stopped_n` | Stopped ({n}) | Gestoppt ({n}) | 已停止（{n}） | Detenidos ({n}) | Arrêtés ({n}) | Arrestati ({n}) | Остановлены ({n}) |
| `containers.namespace` | Namespace | Namespace | 命名空间 | Namespace | Espace de noms | Namespace | Пространство имен |
| `containers.loading_pods` | Loading pods… | Pods werden geladen … | 正在载入 Pod… | Cargando pods… | Chargement des pods… | Caricamento dei pod… | Загрузка подов… |
| `containers.n_containers.one` | {count} container | {count} Container | {count} 个容器 | {count} contenedor | {count} conteneur | {count} container | {count} контейнер |
| `containers.n_containers.other` | {count} containers | {count} Container | {count} 个容器 | {count} contenedores | {count} conteneurs | {count} container | {count} контейнера |
| `logging.no_log_dir` | No log directory on this system | Auf diesem System gibt es keinen Protokollordner. | 此系统上没有日志文件夹 | No hay carpeta de registros en este sistema | Aucun dossier de journaux sur ce système. | Nessuna cartella di log su questo sistema | В этой системе нет папки журналов. |

## App: Transfers

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `transfers.pause` | Pause | Pausieren | 暂停 | Pausar | Suspendre | Pausa | Пауза |
| `transfers.resume` | Resume | Fortsetzen | 继续 | Reanudar | Reprendre | Riprendi | Продолжить |
| `transfers.retry_now` | Try Again Now | Jetzt erneut versuchen | 立即重试 | Reintentar ahora | Réessayer maintenant | Riprova ora | Повторить сейчас |
| `transfers.pause_all` | Pause All | Alle pausieren | 全部暂停 | Pausar todo | Tout suspendre | Metti tutto in pausa | Приостановить все |
| `transfers.resume_all` | Resume All | Alle fortsetzen | 全部继续 | Reanudar todo | Tout reprendre | Riprendi tutto | Продолжить все |
| `transfers.paused` | Paused · {done} of {total} | Pausiert · {done} von {total} | 已暂停 · {done}/{total} | En pausa · {done} de {total} | Suspendu · {done} sur {total} | In pausa · {done} di {total} | Приостановлено · {done} из {total} |
| `transfers.paused_plain` | Paused | Pausiert | 已暂停 | En pausa | Suspendu | In pausa | Приостановлено |
| `transfers.resuming_from` | Resuming from {size} | Wird fortgesetzt ab {size} | 从 {size} 处继续 | Reanudando desde {size} | Reprise à partir de {size} | Ripresa da {size} | Продолжение с {size} |
| `transfers.needs_connection` | Needs a connection to {server} | Benötigt eine Verbindung zu {server} | 需要连接到 {server} | Requiere una conexión con {server} | Nécessite une connexion à {server} | Richiede una connessione a {server} | Требуется подключение к {server} |
| `transfers.reconnect` | Reconnect | Verbinden | 重新连接 | Reconectar | Reconnecter | Riconnetti | Подключиться |
| `transfers.offline` | No internet connection | Keine Internetverbindung | 无互联网连接 | No hay conexión a internet | Aucune connexion à Internet | Nessuna connessione a Internet | Нет подключения к интернету |
| `pill.copy` | Copying {what} | Kopiere {what} | 正在拷贝 {what} | Copiando {what} | Copie de {what} | Copia di {what} | Копирование: {what} |
| `pill.move` | Moving {what} | Bewege {what} | 正在移动 {what} | Moviendo {what} | Déplacement de {what} | Spostamento di {what} | Перемещение: {what} |
| `pill.delete` | Deleting {what} | Lösche {what} | 正在删除 {what} | Eliminando {what} | Suppression de {what} | Eliminazione di {what} | Удаление: {what} |
| `pill.extract` | Extracting {what} | Entpacke {what} | 正在解压缩 {what} | Extrayendo {what} | Extraction de {what} | Estrazione di {what} | Извлечение: {what} |
| `pill.compress` | Compressing {what} | Komprimiere {what} | 正在压缩 {what} | Comprimiendo {what} | Compression de {what} | Compressione di {what} | Сжатие: {what} |
| `pill.many.one` | {count} transfer | {count} Übertragung | {count} 个传输 | {count} transferencia | {count} transfert | {count} trasferimento | {count} передача |
| `pill.many.other` | {count} transfers | {count} Übertragungen | {count} 个传输 | {count} transferencias | {count} transferts | {count} trasferimenti | {count} передачи |
| `pill.paused` | {action} · paused | {action} · pausiert | {action} · 已暂停 | {action} · en pausa | {action} · suspendu | {action} · in pausa | {action} · пауза |
| `pill.queued` | {action} · waiting | {action} · wartet | {action} · 正在等待 | {action} · en espera | {action} · en attente | {action} · in attesa | {action} · в очереди |
| `pill.waiting_network` | {action} · waiting for connection | {action} · wartet auf Verbindung | {action} · 正在等待连接 | {action} · esperando conexión | {action} · en attente de connexion | {action} · in attesa di connessione | {action} · ожидание подключения |
| `pill.done` | Done | Fertig | 完成 | Terminado | Terminé | Completato | Готово |
| `pill.stopped` | Stopped | Gestoppt | 已停止 | Detenido | Arrêté | Interrotto | Остановлено |
| `pill.failed` | Transfer failed | Übertragung fehlgeschlagen | 传输失败 | Error en la transferencia | Échec du transfert | Trasferimento non riuscito | Сбой передачи |
| `pill.uploading` | {provider} is uploading {what} | {provider} lädt {what} hoch | {provider} 正在上传 {what} | {provider} está subiendo {what} | {provider} téléverse {what} | {provider} sta caricando {what} | {provider} выгружает {what} |
| `resume.files.one` | {count} file | {count} Datei | {count} 个文件 | {count} archivo | {count} fichier | {count} file | {count} файл |
| `resume.files.other` | {count} files | {count} Dateien | {count} 个文件 | {count} archivos | {count} fichiers | {count} file | {count} файла |
| `resume.needs` | needs {server} | benötigt {server} | 需要 {server} | requiere {server} | nécessite {server} | richiede {server} | нужен {server} |
| `resume.more.one` | and {count} more | und {count} weitere | 及另外 {count} 个 | y {count} más | et {count} autre | e {count} in più | и еще {count} |
| `resume.more.other` | and {count} more | und {count} weitere | 及另外 {count} 个 | y {count} más | et {count} autres | e altri {count} | и еще {count} |
| `resume.continue` | Resume | Fortsetzen | 继续 | Reanudar | Reprendre | Riprendi | Продолжить |
| `resume.discard` | Discard | Verwerfen | 放弃 | Descartar | Abandonner | Scarta | Отбросить |

## App: Settings

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `settings.title` | Settings | Einstellungen | 设置 | Ajustes | Réglages | Impostazioni | Настройки |
| `settings.general` | General | Allgemein | 通用 | General | Général | Generali | Основные |
| `settings.plugins` | Plugins | Plugins | 插件 | Plugins | Plug-ins | Plug-in | Плагины |
| `settings.section.appearance` | Appearance | Erscheinungsbild | 外观 | Apariencia | Apparence | Aspetto | Оформление |
| `settings.appearance` | Appearance | Erscheinungsbild | 外观 | Apariencia | Apparence | Aspetto | Оформление |
| `settings.accent` | Accent colour | Akzentfarbe | 强调色 | Color de acentuación | Couleur d’accentuation | Colore di accento | Цвет акцента |
| `settings.section.language` | Language | Sprache | 语言 | Idioma | Langue | Lingua | Язык |
| `settings.language` | Language | Sprache | 语言 | Idioma | Langue | Lingua | Язык |
| `settings.section.files` | Files | Dateien | 文件 | Archivos | Fichiers | File | Файлы |
| `settings.show_hidden` | Show hidden files | Unsichtbare Dateien einblenden | 显示隐藏文件 | Mostrar archivos ocultos | Afficher les fichiers masqués | Mostra file nascosti | Показывать скрытые файлы |
| `settings.open_archives` | Open archives as folders | Archive als Ordner öffnen | 将归档作为文件夹打开 | Abrir archivos comprimidos como carpetas | Ouvrir les archives comme des dossiers | Apri gli archivi come cartelle | Открывать архивы как папки |
| `settings.section.preview` | Preview | Vorschau | 预览 | Vista previa | Aperçu | Anteprima | Просмотр |
| `settings.space_opens` | Space opens | Leertaste öffnet | 空格键打开 | La barra espaciadora abre | La barre d’espace ouvre | Barra spaziatrice | Пробел открывает |
| `settings.space.quick_look` | Quick Look (macOS) | Schnellansicht (macOS) | 快速查看（macOS） | Vista Rápida (macOS) | Coup d’œil (macOS) | Visualizzazione rapida (macOS) | Быстрый просмотр (macOS) |
| `settings.space.built_in` | Built-in preview | Integrierte Vorschau | 内建预览 | Vista previa integrada | Aperçu intégré | Anteprima integrata | Встроенный просмотр |
| `settings.mouse_nav` | Mouse side buttons | Maus-Seitentasten | 鼠标侧键 | Botones laterales del ratón | Boutons latéraux de la souris | Tasti laterali del mouse | Боковые кнопки мыши |
| `settings.mouse_nav.back_forward` | Back / Forward | Zurück / Vorwärts | 后退 / 前进 | Atrás / Adelante | Précédent / Suivant | Indietro / Avanti | Назад / Вперёд |
| `settings.mouse_nav.swapped` | Forward / Back (swapped) | Vorwärts / Zurück (vertauscht) | 前进 / 后退（互换） | Adelante / Atrás (invertido) | Suivant / Précédent (inversés) | Avanti / Indietro (invertiti) | Вперёд / Назад (поменять) |
| `settings.mouse_nav.off` | Off | Aus | 关 | Desactivado | Désactivés | Disattivati | Выкл. |
| `settings.new_tabs` | New tabs open in | Neue Tabs öffnen in | 新标签页打开位置 | Las pestañas nuevas se abren en | Les nouveaux onglets s’ouvrent dans | Le nuove schede si aprono in | Новые вкладки открываются в |
| `settings.new_tabs.same` | Same folder | Gleicher Ordner | 同一文件夹 | La misma carpeta | Le même dossier | La stessa cartella | Той же папке |
| `settings.new_tabs.custom` | Other… | Anderer … | 其他… | Otra… | Autre… | Altra… | Другой… |
| `settings.new_tabs.choose` | Folder for new tabs | Ordner für neue Tabs | 新标签页的文件夹 | Carpeta para las pestañas nuevas | Dossier des nouveaux onglets | Cartella per le nuove schede | Папка для новых вкладок |
| `settings.section.copy_move` | Copy and move | Kopieren und Bewegen | 拷贝和移动 | Copiar y mover | Copie et déplacement | Copia e spostamento | Копирование и перемещение |
| `settings.drop` | When you drop files | Beim Ablegen von Dateien | 拖放文件时 | Al soltar archivos | Lorsque vous déposez des fichiers | Quando rilasci dei file | При перетаскивании файлов |
| `settings.drop.ask` | Ask (Copy preselected) | Nachfragen (Kopieren vorausgewählt) | 询问（预选“拷贝”） | Preguntar (Copiar preseleccionado) | Demander (Copier présélectionné) | Chiedi (Copia preselezionato) | Спрашивать (по умолчанию копировать) |
| `settings.drop.copy` | Always copy | Immer kopieren | 始终拷贝 | Copiar siempre | Toujours copier | Copia sempre | Всегда копировать |
| `settings.drop.move` | Always move | Immer bewegen | 始终移动 | Mover siempre | Toujours déplacer | Sposta sempre | Всегда перемещать |
| `settings.conflict` | If an item already exists | Wenn ein Objekt schon existiert | 如果项目已存在 | Si un ítem ya existe | Si un élément existe déjà | Se un elemento esiste già | Если объект уже существует |
| `settings.section.cloud` | Cloud | Cloud | 云端 | Nube | Cloud | Cloud | Облако |
| `settings.section.about` | About Manta | Über Manta | 关于 Manta | Acerca de Manta | À propos de Manta | Informazioni su Manta | О программе Manta |
| `settings.about.website` | Website & Support | Website & Support | 网站与支持 | Sitio web y soporte | Site web et assistance | Sito web e supporto | Сайт и поддержка |
| `settings.about.source` | Source Code | Quellcode | 源代码 | Código fuente | Code source | Codice sorgente | Исходный код |
| `settings.about.feedback` | Feedback | Rückmeldung | 反馈 | Comentarios | Commentaires | Segnalazioni | Обратная связь |
| `settings.about.feedback.report` | Report an Issue | Fehler melden | 报告问题 | Informar de un problema | Signaler un problème | Segnala un problema | Сообщить о проблеме |
| `settings.about.license` | MIT Licence | MIT-Lizenz | MIT 许可证 | Licencia MIT | Licence MIT | Licenza MIT | Лицензия MIT |
| `settings.about.license.read` | Read | Lesen | 阅读 | Leer | Lire | Leggi | Читать |
| `settings.about.licenses.show` | Show | Anzeigen | 显示 | Mostrar | Afficher | Mostra | Показать |
| `settings.cloud.gdrive` | Google Drive | Google Drive | Google Drive | Google Drive | Google Drive | Google Drive | Google Drive |
| `settings.cloud.expert` | Expert: Your Own Google OAuth Client | Experte: eigener Google-OAuth-Client | 专家：你自己的 Google OAuth 客户端 | Experto: tu propio cliente OAuth de Google | Expert : votre propre client OAuth Google | Esperti: client OAuth Google personale | Для экспертов: свой клиент Google OAuth |
| `settings.cloud.expert.inactive` | Not set up | Nicht eingerichtet | 未设置 | Sin configurar | Non configuré | Non configurato | Не настроено |
| `settings.cloud.expert.save` | Save | Speichern | 存储 | Guardar | Enregistrer | Salva | Сохранить |
| `settings.cloud.expert.remove` | Remove | Entfernen | 移除 | Eliminar | Supprimer | Rimuovi | Удалить |

## App: Viewer

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `viewer.prompt.changes` | Opens in Manta | Öffnet sich in Manta | 在 Manta 中打开 | Se abre en Manta | S’ouvre dans Manta | Si apre in Manta | Откроется в Manta |
| `viewer.prompt.stays` | Stays with Finder | Bleibt beim Finder | 仍由访达处理 | Sigue en Finder | Reste dans le Finder | Resta nel Finder | Останется в Finder |
| `viewer.effect.reveal` | “Show in Finder” in other apps | „Im Finder anzeigen“ in anderen Apps | 其他 App 中的“在访达中显示” | “Mostrar en Finder” en otras apps | « Afficher dans le Finder » dans les autres apps | “Mostra nel Finder” nelle altre app | «Показать в Finder» в других программах |
| `viewer.effect.finder_windows` | Double-clicks inside Finder windows | Doppelklicks in Finder-Fenstern | 在访达窗口中的连按操作 | Dobles clics dentro de ventanas de Finder | Les doubles-clics dans les fenêtres du Finder | Doppi clic nelle finestre del Finder | Двойные нажатия в окнах Finder |
| `viewer.currently` | Currently: {name} | Derzeit: {name} | 当前：{name} | Actualmente: {name} | Actuellement : {name} | Attuale: {name} | Сейчас: {name} |
| `viewer.prompt.set` | Set as Default | Als Standard festlegen | 设为默认 | Establecer por omisión | Définir par défaut | Imposta come predefinito | Сделать основным |
| `viewer.prompt.not_now` | Not Now | Nicht jetzt | 以后 | Ahora no | Plus tard | Non ora | Не сейчас |
| `viewer.prompt.never` | Don’t Ask Again | Nicht mehr fragen | 不再询问 | No volver a preguntar | Ne plus demander | Non chiedere più | Больше не спрашивать |
| `viewer.section` | Default file manager | Standard-Dateimanager | 默认文件管理器 | Gestor de archivos predeterminado | Gestionnaire de fichiers par défaut | Gestore file predefinito | Файловый менеджер по умолчанию |
| `viewer.settings.title` | Use Manta as default file manager | Manta als Standard-Dateimanager | 将 Manta 用作默认文件管理器 | Usar Manta como gestor de archivos predeterminado | Utiliser Manta comme gestionnaire de fichiers par défaut | Usa Manta come gestore file predefinito | Manta как файловый менеджер по умолчанию |
| `viewer.palette` | Default File Manager… | Standard-Dateimanager … | 默认文件管理器… | Gestor de archivos predeterminado… | Gestionnaire de fichiers par défaut… | Gestore file predefinito… | Файловый менеджер по умолчанию… |
| `viewer.toggle.on` | Use Manta as Default File Manager… | Manta als Standard-Dateimanager verwenden … | 将 Manta 用作默认文件管理器… | Usar Manta como gestor de archivos predeterminado… | Utiliser Manta comme gestionnaire de fichiers par défaut… | Usa Manta come gestore file predefinito… | Сделать Manta файловым менеджером по умолчанию… |
| `viewer.menu` | Manta as Default File Manager | Manta als Standard-Dateimanager | 将 Manta 用作默认文件管理器 | Manta como gestor de archivos predeterminado | Manta comme gestionnaire de fichiers par défaut | Manta come gestore file predefinito | Manta как файловый менеджер по умолчанию |

## App: Updates

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `menu.check_updates` | Check for Updates… | Nach Updates suchen … | 检查更新… | Buscar actualizaciones… | Rechercher les mises à jour… | Verifica aggiornamenti… | Проверить обновления… |
| `menu.updates_off` | Updates Not Set Up | Updates nicht eingerichtet | 未设置更新 | Actualizaciones no configuradas | Mises à jour non configurées | Aggiornamenti non configurati | Обновления не настроены |
| `update.pill` | Update {version} | Update {version} | 更新 {version} | Actualización {version} | Mise à jour {version} | Aggiornamento {version} | Обновление {version} |
| `update.pill.ready` | Update {version} ready | Update {version} bereit | 更新 {version} 已就绪 | Actualización {version} lista | Mise à jour {version} prête | Aggiornamento {version} pronto | Обновление {version} готово |
| `update.window_title` | Software Update | Softwareupdate | 软件更新 | Actualización de software | Mise à jour de logiciels | Aggiornamento software | Обновление ПО |
| `update.title.downloading` | Downloading Manta {version} … | Manta {version} wird geladen … | 正在下载 Manta {version}… | Descargando Manta {version}… | Téléchargement de Manta {version}… | Download di Manta {version}… | Загрузка Manta {version}… |
| `update.title.ready` | Manta {version} is ready to install | Manta {version} ist bereit zur Installation | Manta {version} 已可安装 | Manta {version} está listo para instalarse | Manta {version} est prêt à être installé | Manta {version} è pronto per l’installazione | Manta {version} готова к установке |
| `update.title.installing` | Installing Manta {version} … | Manta {version} wird installiert … | 正在安装 Manta {version}… | Instalando Manta {version}… | Installation de Manta {version}… | Installazione di Manta {version}… | Установка Manta {version}… |
| `update.meta` | {size} · released {date} | {size} · veröffentlicht am {date} | {size} · 发布于 {date} | {size} · publicada el {date} | {size} · publiée le {date} | {size} · pubblicato il {date} | {size} · выпущено {date} |
| `update.beta` | Beta | Beta | Beta | Beta | Bêta | Beta | Бета |
| `update.whats_new` | What’s new | Neu in dieser Version | 新功能 | Novedades | Nouveautés | Novità | Что нового |
| `update.install` | Install and Relaunch | Installieren und neu starten | 安装并重新启动 | Instalar y reiniciar | Installer et relancer | Installa e riavvia | Установить и перезапустить |
| `update.later` | Later | Später | 稍后 | Más tarde | Plus tard | Più tardi | Позже |
| `update.skip` | Skip This Version | Diese Version überspringen | 跳过此版本 | Omitir esta versión | Ignorer cette version | Salta questa versione | Пропустить эту версию |
| `update.relaunching` | Relaunching Manta … | Manta wird neu gestartet … | 正在重新启动 Manta… | Reiniciando Manta… | Relance de Manta… | Riavvio di Manta… | Перезапуск Manta… |
| `update.retry` | Try Again | Erneut versuchen | 重试 | Reintentar | Réessayer | Riprova | Повторить |
| `update.close` | Close | Schließen | 关闭 | Cerrar | Fermer | Chiudi | Закрыть |
| `update.checking` | Checking for updates … | Suche nach Updates … | 正在检查更新… | Buscando actualizaciones… | Recherche de mises à jour… | Verifica degli aggiornamenti… | Проверка обновлений… |
| `update.up_to_date` | Manta is up to date | Manta ist auf dem neuesten Stand | Manta 已是最新版本 | Manta está actualizado | Manta est à jour | Manta è aggiornato | Установлена последняя версия Manta |
| `update.blocker.dmg` | Manta runs from a disk image | Manta läuft von einem Disk-Image | Manta 正从磁盘映像运行 | Manta se ejecuta desde una imagen de disco | Manta est exécuté depuis une image disque | Manta è in esecuzione da un’immagine disco | Manta запущена с образа диска |
| `update.blocker.translocated` | Manta runs from a temporary location | Manta läuft von einem temporären Ort | Manta 正从临时位置运行 | Manta se ejecuta desde una ubicación temporal | Manta est exécuté depuis un emplacement temporaire | Manta è in esecuzione da una posizione temporanea | Manta запущена из временного места |
| `update.blocker.dev` | This is a development build | Dies ist ein Entwickler-Build | 这是开发版本 | Esta es una versión de desarrollo | Ceci est une version de développement | Questa è una build di sviluppo | Это сборка для разработки |
| `update.failed.check` | Couldn't check for updates | Suche nach Updates fehlgeschlagen | 无法检查更新 | No se han podido buscar actualizaciones | Impossible de rechercher les mises à jour | Impossibile verificare gli aggiornamenti | Не удалось проверить обновления |
| `update.failed.download` | The download failed | Der Download ist fehlgeschlagen | 下载失败 | Error en la descarga | Le téléchargement a échoué | Download non riuscito | Не удалось загрузить обновление |
| `update.failed.install` | The update couldn't be installed | Das Update konnte nicht installiert werden | 无法安装更新 | No se ha podido instalar la actualización | Impossible d’installer la mise à jour | Impossibile installare l’aggiornamento | Не удалось установить обновление |
| `update.settings.section` | Updates | Updates | 更新 | Actualizaciones | Mises à jour | Aggiornamenti | Обновления |
| `update.settings.auto` | Check for updates at launch | Beim Start nach Updates suchen | 启动时检查更新 | Buscar actualizaciones al abrir | Rechercher les mises à jour au lancement | Verifica aggiornamenti all’avvio | Проверять обновления при запуске |
| `update.settings.channel` | Channel | Kanal | 通道 | Canal | Canal | Canale | Канал |
| `update.channel.stable` | Stable | Stabil | 稳定版 | Estable | Stable | Stabile | Стабильный |
| `update.channel.beta` | Beta | Beta | Beta | Beta | Bêta | Beta | Бета |
| `update.settings.version` | Manta {version} | Manta {version} | Manta {version} | Manta {version} | Manta {version} | Manta {version} | Manta {version} |
| `update.settings.last_check` | Last checked: {time} | Zuletzt gesucht: {time} | 上次检查：{time} | Última búsqueda: {time} | Dernière recherche : {time} | Ultima verifica: {time} | Последняя проверка: {time} |
| `update.settings.never` | never | noch nie | 从未 | nunca | jamais | mai | никогда |
| `update.settings.check_now` | Check Now | Jetzt suchen | 立即检查 | Buscar ahora | Rechercher | Verifica ora | Проверить |
| `update.settings.skipped` | Skipped: Manta {version} | Übersprungen: Manta {version} | 已跳过：Manta {version} | Omitida: Manta {version} | Ignorée : Manta {version} | Saltata: Manta {version} | Пропущено: Manta {version} |
| `update.settings.reset` | Reset | Zurücksetzen | 还原 | Restablecer | Réinitialiser | Ripristina | Сбросить |
| `update.settings.disabled` | Updates aren’t set up in this build | Updates sind in diesem Build nicht eingerichtet | 此版本未设置更新 | Las actualizaciones no están configuradas en esta versión | Les mises à jour ne sont pas configurées dans cette version | Gli aggiornamenti non sono configurati in questa build | В этой сборке обновления не настроены |
| `update.disabled.invalid` | Its update settings are invalid: {error} | Seine Update-Einstellungen sind ungültig: {error} | 其更新设置无效：{error} | Sus ajustes de actualización no son válidos: {error} | Ses réglages de mise à jour ne sont pas valides : {error} | Le sue impostazioni di aggiornamento non sono valide: {error} | Ее настройки обновлений недействительны: {error} |

## App: Help

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `help.window_title` | Manta Help | Manta-Hilfe | Manta 帮助 | Ayuda de Manta | Aide Manta | Aiuto di Manta | Справка Manta |
| `help.search` | Search the Manual | Handbuch durchsuchen | 搜索手册 | Buscar en el manual | Rechercher dans le manuel | Cerca nel manuale | Поиск в руководстве |
| `help.back` | Back | Zurück | 返回 | Atrás | Précédent | Indietro | Назад |
| `help.forward` | Forward | Vorwärts | 前进 | Adelante | Suivant | Avanti | Вперед |
| `help.chapter` | Chapter | Kapitel | 章节 | Capítulo | Chapitre | Capitolo | Глава |
| `help.button` | Help | Hilfe | 帮助 | Ayuda | Aide | Aiuto | Справка |
| `menu.help` | Help | Hilfe | 帮助 | Ayuda | Aide | Aiuto | Справка |
| `menu.file` | File | Ablage | 文件 | Archivo | Fichier | Archivio | Файл |
| `menu.edit` | Edit | Bearbeiten | 编辑 | Edición | Édition | Modifica | Правка |
| `menu.view` | View | Darstellung | 显示 | Visualización | Présentation | Vista | Вид |
| `menu.go` | Go | Gehe zu | 前往 | Ir | Aller | Vai | Переход |
| `menu.window` | Window | Fenster | 窗口 | Ventana | Fenêtre | Finestra | Окно |
| `menu.manual` | Manta Manual | Manta-Handbuch | Manta 手册 | Manual de Manta | Manuel de Manta | Manuale di Manta | Руководство Manta |

## App: Tags, search and Get Info

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `tags.menu` | Tags | Tags | 标签 | Etiquetas | Tags | Tag | Теги |
| `tags.color.red` | Red | Rot | 红色 | Rojo | Rouge | Rosso | Красный |
| `tags.color.orange` | Orange | Orange | 橙色 | Naranja | Orange | Arancione | Оранжевый |
| `tags.color.yellow` | Yellow | Gelb | 黄色 | Amarillo | Jaune | Giallo | Желтый |
| `tags.color.green` | Green | Grün | 绿色 | Verde | Vert | Verde | Зеленый |
| `tags.color.blue` | Blue | Blau | 蓝色 | Azul | Bleu | Blu | Синий |
| `tags.color.purple` | Purple | Lila | 紫色 | Morado | Violet | Viola | Фиолетовый |
| `tags.color.gray` | Gray | Grau | 灰色 | Gris | Gris | Grigio | Серый |
| `tags.color.none` | No Color | Keine Farbe | 无颜色 | Sin color | Aucune couleur | Nessun colore | Без цвета |
| `tags.add_named` | Add Tag “{name}” | Tag „{name}“ hinzufügen | 添加标签“{name}” | Añadir la etiqueta “{name}” | Ajouter le tag « {name} » | Aggiungi il tag “{name}” | Добавить тег «{name}» |
| `tags.remove_named` | Remove Tag “{name}” | Tag „{name}“ entfernen | 移除标签“{name}” | Eliminar la etiqueta “{name}” | Supprimer le tag « {name} » | Rimuovi il tag “{name}” | Удалить тег «{name}» |
| `tags.spoken` | Tags: {names} | Tags: {names} | 标签：{names} | Etiquetas: {names} | Tags : {names} | Tag: {names} | Теги: {names} |
| `tags.failed` | Couldn't change the tags | Die Tags konnten nicht geändert werden | 无法更改标签 | No se han podido cambiar las etiquetas | Impossible de modifier les tags | Impossibile modificare i tag | Не удалось изменить теги |
| `sidebar.tags` | Tags | Tags | 标签 | Etiquetas | Tags | Tag | Теги |
| `search.scope.folder` | This Folder | Dieser Ordner | 此文件夹 | Esta carpeta | Ce dossier | Questa cartella | Эта папка |
| `search.scope.subfolders` | Subfolders | Unterordner | 子文件夹 | Subcarpetas | Sous-dossiers | Sottocartelle | Вложенные папки |
| `search.scope.mac` | This Mac | Dieser Mac | 此 Mac | Este Mac | Ce Mac | Questo Mac | Этот Mac |
| `search.title` | Search: “{query}” | Suche: „{query}“ | 搜索：“{query}” | Búsqueda: “{query}” | Recherche : « {query} » | Ricerca: “{query}” | Поиск: «{query}» |
| `search.title_tag` | Tag “{name}” | Tag „{name}“ | 标签“{name}” | Etiqueta “{name}” | Tag « {name} » | Elementi con tag “{name}” | Тег «{name}» |
| `search.where_folder` | in “{folder}” and below | in „{folder}“ und darunter | 位于“{folder}”及其子文件夹 | en “{folder}” y sus subcarpetas | dans « {folder} » et ses sous-dossiers | in “{folder}” e nelle sottocartelle | в «{folder}» и вложенных папках |
| `search.where_mac` | on This Mac | auf diesem Mac | 在此 Mac 上 | en este Mac | sur ce Mac | su questo Mac | на этом Mac |
| `search.col.path` | Path | Pfad | 路径 | Ruta | Chemin | Percorso | Путь |
| `search.stop` | Stop Search | Suche stoppen | 停止搜索 | Detener la búsqueda | Arrêter la recherche | Interrompi ricerca | Остановить поиск |
| `search.searching.one` | Searching… {count} item found | Suche läuft … {count} Objekt gefunden | 正在搜索… 已找到 {count} 项 | Buscando… {count} ítem encontrado | Recherche… {count} élément trouvé | Ricerca in corso… {count} elemento trovato | Поиск… найден {count} объект |
| `search.searching.other` | Searching… {count} items found | Suche läuft … {count} Objekte gefunden | 正在搜索… 已找到 {count} 项 | Buscando… {count} ítems encontrados | Recherche… {count} éléments trouvés | Ricerca in corso… {count} elementi trovati | Поиск… найдено {count} объекта |
| `search.found.one` | {count} item found | {count} Objekt gefunden | 找到 {count} 项 | {count} ítem encontrado | {count} élément trouvé | {count} elemento trovato | Найден {count} объект |
| `search.found.other` | {count} items found | {count} Objekte gefunden | 找到 {count} 项 | {count} ítems encontrados | {count} éléments trouvés | {count} elementi trovati | Найдено {count} объекта |
| `search.stopped.one` | Stopped – {count} item found | Gestoppt – {count} Objekt gefunden | 已停止 – 找到 {count} 项 | Detenida – {count} ítem encontrado | Arrêtée – {count} élément trouvé | Interrotta – {count} elemento trovato | Остановлено – найден {count} объект |
| `search.stopped.other` | Stopped – {count} items found | Gestoppt – {count} Objekte gefunden | 已停止 – 找到 {count} 项 | Detenida – {count} ítems encontrados | Arrêtée – {count} éléments trouvés | Interrotta – {count} elementi trovati | Остановлено – найдено {count} объекта |
| `search.truncated.one` | Showing the first {count} item | Nur das erste Objekt ({count}) wird angezeigt | 仅显示前 {count} 项 | Se muestra solo el primer ítem ({count}) | Seul le premier élément ({count}) est affiché | Viene mostrato solo il primo elemento ({count}) | Показан первый {count} объект |
| `search.truncated.other` | Showing the first {count} items | Die ersten {count} Objekte werden angezeigt | 仅显示前 {count} 项 | Se muestran los primeros {count} ítems | Les {count} premiers éléments sont affichés | Vengono mostrati i primi {count} elementi | Показаны первые {count} объекта |
| `search.failed` | The search failed: {error} | Die Suche ist fehlgeschlagen: {error} | 搜索失败：{error} | La búsqueda ha fallado: {error} | La recherche a échoué : {error} | La ricerca non è riuscita: {error} | Поиск не удался: {error} |
| `search.searching_title` | Searching… | Suche läuft … | 正在搜索… | Buscando… | Recherche en cours… | Ricerca in corso… | Поиск… |
| `search.try_mac` | Search This Mac | Diesen Mac durchsuchen | 搜索此 Mac | Buscar en este Mac | Rechercher sur ce Mac | Cerca su questo Mac | Искать на этом Mac |
| `search.reveal` | Show in Enclosing Folder | Im umgebenden Ordner anzeigen | 在所在文件夹中显示 | Mostrar en la carpeta contenedora | Afficher dans le dossier parent | Mostra nella cartella superiore | Показать в папке |
| `search.not_here` | Not available in search results | In Suchergebnissen nicht verfügbar | 在搜索结果中不可用 | No disponible en los resultados de búsqueda | Non disponible dans les résultats de recherche | Non disponibile nei risultati della ricerca | Недоступно в результатах поиска |
| `info.title` | {name} Info | Informationen zu „{name}“ | “{name}”简介 | Información de “{name}” | Informations sur « {name} » | Informazioni su “{name}” | Свойства: «{name}» |
| `info.title_multi.one` | Info: {count} Item | Informationen zu {count} Objekt | {count} 个项目的简介 | Información de {count} ítem | Informations sur {count} élément | Informazioni su {count} elemento | Свойства: {count} объект |
| `info.title_multi.other` | Info: {count} Items | Informationen zu {count} Objekten | {count} 个项目的简介 | Información de {count} ítems | Informations sur {count} éléments | Informazioni su {count} elementi | Свойства: {count} объекта |
| `info.name` | Name | Name | 名称 | Nombre | Nom | Nome | Имя |
| `info.items.one` | {count} item | {count} Objekt | {count} 个项目 | {count} ítem | {count} élément | {count} elemento | {count} объект |
| `info.items.other` | {count} items | {count} Objekte | {count} 个项目 | {count} ítems | {count} éléments | {count} elementi | {count} объекта |
| `info.kind_multi` | Multiple items | Mehrere Objekte | 多个项目 | Varios ítems | Plusieurs éléments | Più elementi | Несколько объектов |
| `info.general` | General | Allgemein | 通用 | General | Général | Generali | Общие |
| `info.add_tag` | Add Tag | Tag hinzufügen | 添加标签 | Añadir etiqueta | Ajouter un tag | Aggiungi tag | Добавить тег |
| `info.size_bytes` | {size} ({bytes} bytes) | {size} ({bytes} Byte) | {size}（{bytes} 字节） | {size} ({bytes} bytes) | {size} ({bytes} octets) | {size} ({bytes} byte) | {size} ({bytes} Б) |
| `info.size_items.one` | {size} for {count} item | {size} für {count} Objekt | {size}，共 {count} 项 | {size} en {count} ítem | {size} pour {count} élément | {size} per {count} elemento | {size} на {count} объект |
| `info.size_items.other` | {size} for {count} items | {size} für {count} Objekte | {size}，共 {count} 项 | {size} en {count} ítems | {size} pour {count} éléments | {size} per {count} elementi | {size} на {count} объекта |
| `info.counting` | {size} – counting… | {size} – wird gezählt … | {size} – 正在计算… | {size} – calculando… | {size} – calcul en cours… | {size} – calcolo in corso… | {size} – подсчет… |
| `info.locked` | Locked | Geschützt | 已锁定 | Bloqueado | Verrouillé | Bloccato | Защита |
| `info.hidden` | Hidden | Unsichtbar | 隐藏 | Oculto | Masqué | Nascosto | Скрытый |
| `info.comments` | Comments | Kommentare | 注释 | Comentarios | Commentaires | Commenti | Комментарии |
| `info.no_comment` | No comment | Kein Kommentar | 无注释 | Sin comentario | Aucun commentaire | Nessun commento | Нет комментария |
| `info.open_with` | Open with | Öffnen mit | 打开方式 | Abrir con | Ouvrir avec | Apri con | Открывать в программе |
| `info.no_app` | None | Keines | 无 | Ninguna | Aucune | Nessuna | Нет |
| `info.open_with_menu` | Open With | Öffnen mit | 打开方式 | Abrir con | Ouvrir avec | Apri con | Открыть в программе |
| `info.app_default` | {app} (default) | {app} (Standard) | {app}（默认） | {app} (por omisión) | {app} (par défaut) | {app} (predefinita) | {app} (по умолчанию) |
| `info.permissions` | Sharing & Permissions | Freigabe & Zugriffsrechte | 共享与权限 | Compartir y permisos | Partage et permissions | Condivisione e permessi | Общий доступ и права доступа |
| `info.owner` | Owner | Eigentümer | 所有者 | Propietario | Propriétaire | Proprietario | Владелец |
| `info.group` | Group | Gruppe | 群组 | Grupo | Groupe | Gruppo | Группа |
| `info.everyone` | Everyone | Jeder | 所有人 | Todos | Tout le monde | Tutti | Все |
| `info.read` | Read | Lesen | 读 | Lectura | Lecture | Lettura | Чтение |
| `info.write` | Write | Schreiben | 写 | Escritura | Écriture | Scrittura | Запись |
| `info.execute` | Execute | Ausführen | 执行 | Ejecución | Exécution | Esecuzione | Выполнение |

## Plugin: Duplicates

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | Duplicates | Duplikate | 重复项 | Duplicados | Doublons | Duplicati | Дубликаты |
| `cmd.scan` | Find Duplicates in This Folder | Duplikate in diesem Ordner suchen | 在此文件夹中查找重复项 | Buscar duplicados en esta carpeta | Rechercher les doublons dans ce dossier | Trova duplicati in questa cartella | Найти дубликаты в этой папке |
| `cmd.panel` | Show Duplicates Panel | Duplikate-Panel einblenden | 显示重复项面板 | Mostrar el panel Duplicados | Afficher le panneau Doublons | Mostra pannello Duplicati | Показать панель «Дубликаты» |
| `cmd.settings` | Edit Duplicates Settings… | Duplikate-Einstellungen bearbeiten … | 编辑重复项设置… | Editar ajustes de Duplicados… | Modifier les réglages de Doublons… | Modifica impostazioni Duplicati… | Изменить настройки дубликатов… |
| `cmd.scan_here` | Find Duplicates Here | Duplikate hier suchen | 在此处查找重复项 | Buscar duplicados aquí | Rechercher les doublons ici | Trova duplicati qui | Найти дубликаты здесь |
| `cmd.keep_here` | Keep Duplicates in This Folder | Duplikate in diesem Ordner behalten | 保留此文件夹中的重复项 | Conservar duplicados en esta carpeta | Conserver les doublons dans ce dossier | Mantieni duplicati in questa cartella | Оставлять дубликаты в этой папке |
| `panel.duplicates` | Duplicates | Duplikate | 重复项 | Duplicados | Doublons | Duplicati | Дубликаты |
| `panel.confirm` | Move to Trash | In den Papierkorb legen | 移到废纸篓 | Trasladar a la papelera | Placer dans la corbeille | Sposta nel Cestino | Переместить в Корзину |
| `rule.oldest` | Oldest | Älteste | 最旧 | Más antiguo | Plus ancien | Più vecchio | Самый старый |
| `rule.newest` | Newest | Neueste | 最新 | Más reciente | Plus récent | Più recente | Самый новый |
| `rule.shortest` | Shortest Path | Kürzester Pfad | 最短路径 | Ruta más corta | Chemin le plus court | Percorso più breve | Самый короткий путь |
| `rule.folder` | In Folder | Im Ordner | 位于文件夹 | En carpeta | Dans le dossier | Nella cartella | В папке |
| `rule.in` | In “{name}” | In „{name}“ | 位于“{name}” | En “{name}” | Dans « {name} » | In “{name}” | В папке «{name}» |
| `rule.a_folder` | a folder | einem Ordner | 某个文件夹 | una carpeta | un dossier | una cartella | без имени |
| `rule.unset` | In Folder (not set) | Im Ordner (nicht festgelegt) | 位于文件夹（未设定） | En carpeta (sin definir) | Dans le dossier (non défini) | Nella cartella (non impostata) | В папке (не задана) |
| `opt.section` | Options | Optionen | 选项 | Opciones | Options | Opzioni | Параметры |
| `opt.keep` | Keep | Behalten | 保留 | Conservar | Conserver | Mantieni | Оставлять |
| `opt.min_size` | Minimum size | Mindestgröße | 最小大小 | Tamaño mínimo | Taille minimale | Dimensione minima | Мин. размер |
| `opt.hidden` | Hidden files | Unsichtbare Dateien | 隐藏文件 | Archivos ocultos | Fichiers masqués | File nascosti | Скрытые файлы |
| `opt.included` | Included | Einbezogen | 包括 | Incluidos | Inclus | Inclusi | Учитываются |
| `opt.skipped` | Skipped | Übersprungen | 跳过 | Omitidos | Ignorés | Saltati | Пропускаются |
| `opt.skipped_key` | Skipped | Ausgelassen | 已跳过 | Omitido | Ignorés | Esclusi | Пропускаются |
| `opt.packages` | packages | Pakete | 包 | paquetes | paquets | pacchetti | пакеты |
| `opt.nothing` | nothing | nichts | 无 | nada | rien | niente | ничего |
| `btn.keep_rule` | Keep Rule | Behalten-Regel | 保留规则 | Regla de conservación | Règle de conservation | Regola | Что оставлять |
| `btn.min_size` | Min Size | Mindestgröße | 最小大小 | Tamaño mín. | Taille min. | Dim. minima | Мин. размер |
| `btn.hidden` | Hidden | Unsichtbare | 隐藏文件 | Ocultos | Masqués | Nascosti | Скрытые |
| `btn.settings` | Settings… | Einstellungen … | 设置… | Ajustes… | Réglages… | Impostazioni… | Настройки… |
| `group.kept` | Kept ({why}) · modified {date} | Bleibt ({why}) · geändert {date} | 保留（{why}）· 修改于 {date} | Se conserva ({why}) · modificado {date} | Conservé ({why}) · modifié le {date} | Mantenuto ({why}) · modificato {date} | Остается ({why}) · изменен {date} |
| `group.chosen` | chosen by you | von dir gewählt | 由你选择 | elegido por ti | votre choix | scelto da te | выбран вами |
| `group.each` | {size} each · {total} | je {size} · {total} | 每个 {size} · {total} | {size} cada uno · {total} | {size} chacun · {total} | {size} ciascuno · {total} | по {size} · {total} |
| `btn.trash_extras` | Trash Extras | Kopien entfernen | 移除多余副本 | Eliminar copias | Retirer les copies | Elimina copie | Удалить копии |
| `btn.reveal` | Reveal | Im Finder zeigen | 在访达中显示 | Mostrar en Finder | Afficher dans le Finder | Mostra nel Finder | Показать в Finder |
| `btn.select` | Select in Pane | Im Bereich auswählen | 在面板中选择 | Seleccionar en el panel | Sélectionner dans la sous-fenêtre | Seleziona nel pannello | Выбрать в панели |
| `scan.title` | Scanning “{name}” | „{name}“ wird durchsucht | 正在扫描“{name}” | Analizando “{name}” | Analyse de « {name} » | Analisi di “{name}” | Поиск в «{name}» |
| `scan.listing` | Listing files … {n} found | Dateien werden aufgelistet … {n} gefunden | 正在列出文件… 已找到 {n} 个 | Listando archivos… {n} encontrados | Inventaire des fichiers… {n} trouvés | Elenco dei file… {n} trovati | Составление списка файлов… найдено: {n} |
| `scan.hashing` | Hashed {done} / {total} files | {done} / {total} Dateien geprüft | 已计算哈希值 {done} / {total} 个文件 | {done} / {total} archivos comprobados | {done} / {total} fichiers hachés | Hash calcolato per {done} / {total} file | Хешировано файлов: {done} из {total} |
| `scan.verifying` | Verifying {done} / {total} files | {done} / {total} Dateien werden verifiziert | 正在验证 {done} / {total} 个文件 | Verificando {done} / {total} archivos | Vérification de {done} / {total} fichiers | Verifica di {done} / {total} file | Проверка файлов: {done} из {total} |
| `btn.cancel` | Cancel | Abbrechen | 取消 | Cancelar | Annuler | Annulla | Отменить |
| `btn.find` | Find Duplicates | Duplikate suchen | 查找重复项 | Buscar duplicados | Rechercher les doublons | Trova duplicati | Найти дубликаты |
| `key.groups` | Groups | Gruppen | 组 | Grupos | Groupes | Gruppi | Группы |
| `key.files` | Duplicate files | Doppelte Dateien | 重复文件 | Archivos duplicados | Fichiers en double | File duplicati | Дубликаты |
| `key.reclaimable` | Reclaimable | Freizugeben | 可释放 | Recuperable | Récupérable | Recuperabile | Можно освободить |
| `btn.trash_all` | Move Extras to Trash | Kopien in den Papierkorb | 将多余副本移到废纸篓 | Trasladar copias a la papelera | Placer les copies dans la corbeille | Sposta copie nel Cestino | Переместить копии в Корзину |
| `btn.scan_folder` | Scan “{name}” | „{name}“ durchsuchen | 扫描“{name}” | Analizar “{name}” | Analyser « {name} » | Analizza “{name}” | Искать в «{name}» |
| `btn.scan_again` | Scan Again | Erneut suchen | 重新扫描 | Volver a analizar | Relancer l’analyse | Analizza di nuovo | Искать снова |
| `btn.clear` | Clear | Leeren | 清除 | Borrar | Effacer | Cancella | Очистить |
| `more` | {groups} more | {groups} mehr | 还有 {groups} | {groups} más | {groups} de plus | altri {groups} | еще {groups} |
| `btn.more` | Show More | Mehr zeigen | 显示更多 | Mostrar más | Afficher plus | Mostra altro | Показать еще |
| `remote.server` | this server | diesen Server | 此服务器 | este servidor | ce serveur | questo server | этот сервер |
| `confirm.more` | and {n} more | und {n} weitere | 以及其他 {n} 个 | y {n} más | et {n} de plus | e altri {n} | и еще {n} |
| `btn.move` | Move to Trash | In den Papierkorb | 移到废纸篓 | Trasladar a la papelera | Placer dans la corbeille | Sposta nel Cestino | Переместить в Корзину |
| `files.one` | {count} file | {count} Datei | {count} 个文件 | {count} archivo | {count} fichier | {count} file | {count} файл |
| `files.other` | {count} files | {count} Dateien | {count} 个文件 | {count} archivos | {count} fichiers | {count} file | {count} файла |
| `folders.one` | {count} folder | {count} Ordner | {count} 个文件夹 | {count} carpeta | {count} dossier | {count} cartella | {count} папка |
| `folders.other` | {count} folders | {count} Ordner | {count} 个文件夹 | {count} carpetas | {count} dossiers | {count} cartelle | {count} папки |
| `links.one` | {count} hard link | {count} fester Link | {count} 个硬链接 | {count} enlace físico | {count} lien physique | {count} hard link | {count} жесткая ссылка |
| `links.other` | {count} hard links | {count} feste Links | {count} 个硬链接 | {count} enlaces físicos | {count} liens physiques | {count} hard link | {count} жесткие ссылки |
| `groups.one` | {count} group | {count} Gruppe | {count} 组 | {count} grupo | {count} groupe | {count} gruppo | {count} группа |
| `groups.other` | {count} groups | {count} Gruppen | {count} 组 | {count} grupos | {count} groupes | {count} gruppi | {count} группы |
| `bytes.one` | {count} byte | {count} Byte | {count} 字节 | {count} byte | {count} octet | {count} byte | {count} байт |
| `bytes.other` | {count} bytes | {count} Byte | {count} 字节 | {count} bytes | {count} octets | {count} byte | {count} байта |
| `settings.broken` | settings.json: {error} | settings.json: {error} | settings.json：{error} | settings.json: {error} | settings.json : {error} | settings.json: {error} | settings.json: {error} |
| `settings.not_object` | settings.json must hold an object | settings.json muss ein Objekt enthalten | settings.json 必须包含一个对象 | settings.json debe contener un objeto | settings.json doit contenir un objet | settings.json deve contenere un oggetto | settings.json должен содержать объект |
| `settings.skip_packages` | Skip packages (.app, .photoslibrary, …) | Pakete überspringen (.app, .photoslibrary, …) | 跳过包（.app、.photoslibrary 等） | Omitir paquetes (.app, .photoslibrary, …) | Ignorer les paquets (.app, .photoslibrary, …) | Salta i pacchetti (.app, .photoslibrary, …) | Пропускать пакеты (.app, .photoslibrary, …) |
| `settings.include_hidden` | Include hidden files | Unsichtbare Dateien einbeziehen | 包括隐藏文件 | Incluir archivos ocultos | Inclure les fichiers masqués | Includi file nascosti | Учитывать скрытые файлы |
| `settings.edit_json` | Edit settings.json … | settings.json bearbeiten … | 编辑 settings.json… | Editar settings.json… | Modifier settings.json… | Modifica settings.json… | Изменить settings.json… |

## Plugin: Folder Sizes

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | Folder Sizes | Ordnergrößen | 文件夹大小 | Tamaños de carpeta | Tailles des dossiers | Dimensioni cartelle | Размеры папок |
| `cmd.show` | Show Disk Usage | Speicherbelegung einblenden | 显示磁盘使用情况 | Mostrar uso del disco | Afficher l’utilisation du disque | Mostra utilizzo disco | Показать использование диска |
| `cmd.calculate` | Calculate Size | Größe berechnen | 计算大小 | Calcular tamaño | Calculer la taille | Calcola dimensione | Рассчитать размер |
| `panel.title` | Disk Usage | Speicherbelegung | 磁盘使用情况 | Uso del disco | Utilisation du disque | Utilizzo disco | Использование диска |
| `settings.auto` | Calculate folder sizes automatically | Ordnergrößen automatisch berechnen | 自动计算文件夹大小 | Calcular automáticamente el tamaño de las carpetas | Calculer automatiquement la taille des dossiers | Calcola automaticamente le dimensioni delle cartelle | Рассчитывать размеры папок автоматически |
| `settings.manual` | Only on demand | Nur bei Bedarf | 仅在需要时 | Solo cuando se solicite | Uniquement sur demande | Solo su richiesta | Только по запросу |
| `empty.local.title` | Local folders only | Nur lokale Ordner | 仅限本地文件夹 | Solo carpetas locales | Dossiers locaux uniquement | Solo cartelle locali | Только локальные папки |
| `reading` | Reading folder… | Ordner wird gelesen … | 正在读取文件夹… | Leyendo carpeta… | Lecture du dossier… | Lettura della cartella… | Чтение папки… |
| `measuring` | Measuring… | Wird gemessen … | 正在测量… | Midiendo… | Mesure en cours… | Misurazione… | Измерение… |
| `largest` | Largest Items | Größte Objekte | 最大的项目 | Ítems más grandes | Éléments les plus volumineux | Elementi più grandi | Самые большие объекты |
| `counting` | Counting… | Wird gezählt … | 正在计数… | Contando… | Comptage… | Conteggio… | Подсчет… |
| `files.one` | {count} file | {count} Datei | {count} 个文件 | {count} archivo | {count} fichier | {count} file | {count} файл |
| `files.other` | {count} files | {count} Dateien | {count} 个文件 | {count} archivos | {count} fichiers | {count} file | {count} файла |
| `smaller.one` | {count} smaller item | {count} kleineres Objekt | {count} 个较小的项目 | {count} ítem más pequeño | {count} élément plus petit | {count} elemento più piccolo | {count} объект меньшего размера |
| `smaller.other` | {count} smaller items | {count} kleinere Objekte | {count} 个较小的项目 | {count} ítems más pequeños | {count} éléments plus petits | {count} elementi più piccoli | {count} объекта меньшего размера |
| `share_of` | {size} · {pct} of “{parent}” | {size} · {pct} von „{parent}“ | {size} · 占“{parent}”的 {pct} | {size} · {pct} de “{parent}” | {size} · {pct} de « {parent} » | {size} · {pct} di “{parent}” | {size} · {pct} от «{parent}» |
| `on_disk` | {size} on disk | {size} auf dem Volume | 磁盘上 {size} | {size} en el disco | {size} sur le disque | {size} su disco | {size} на диске |
| `legend.folders` | Folders (shaded) | Ordner (schattiert) | 文件夹（阴影） | Carpetas (sombreadas) | Dossiers (ombrés) | Cartelle (ombreggiate) | Папки (заштрихованы) |
| `bytes.one` | {count} byte | {count} Byte | {count} 字节 | {count} byte | {count} octet | {count} byte | {count} байт |
| `bytes.other` | {count} bytes | {count} Byte | {count} 字节 | {count} bytes | {count} octets | {count} byte | {count} байта |
| `cat.images` | Images | Bilder | 图像 | Imágenes | Images | Immagini | Изображения |
| `cat.video` | Video | Videos | 视频 | Vídeo | Vidéo | Video | Видео |
| `cat.audio` | Audio | Audio | 音频 | Audio | Audio | Audio | Аудио |
| `cat.documents` | Documents | Dokumente | 文稿 | Documentos | Documents | Documenti | Документы |
| `cat.code` | Code | Code | 代码 | Código | Code | Codice | Код |
| `cat.archives` | Archives | Archive | 归档 | Archivos comprimidos | Archives | Archivi | Архивы |
| `cat.apps` | Apps | Apps | 应用程序 | Apps | Apps | App | Программы |
| `cat.misc` | Other | Andere | 其他 | Otros | Autre | Altro | Другое |

## Plugin: Git

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `git.cmd.toggle_panel` | Toggle Git Panel | Git-Panel ein-/ausblenden | 显示/隐藏 Git 面板 | Mostrar/ocultar panel Git | Afficher/masquer le panneau Git | Mostra/nascondi pannello Git | Показать/скрыть панель Git |
| `git.cmd.refresh` | Refresh Git Status | Git-Status aktualisieren | 刷新 Git 状态 | Actualizar estado de Git | Actualiser l’état Git | Aggiorna stato Git | Обновить статус Git |
| `git.panel.title` | Git | Git | Git | Git | Git | Git | Git |
| `git.label.merge` | Merging | Merge | 合并中 | Fusionando | Fusion | Merge | Слияние |
| `git.label.rebase` | Rebasing | Rebase | 变基中 | Rebase | Rebasage | Rebase | Перебазирование |
| `git.label.cherry_pick` | Cherry-picking | Cherry-Pick | 拣选中 | Cherry-pick | Cherry-pick | Cherry-pick | Cherry-pick |
| `git.label.revert` | Reverting | Revert | 还原中 | Revirtiendo | Revert | Revert | Отмена коммита |
| `git.label.bisect` | Bisecting | Bisect | 二分查找中 | Bisect | Bissection | Bisect | Bisect |
| `git.label.am` | Applying patches | Patches anwenden | 正在应用补丁 | Aplicando parches | Application des patchs | Applicazione patch | Применение патчей |
| `git.state.conflict` | Merge conflict | Merge-Konflikt | 合并冲突 | Conflicto de fusión | Conflit de fusion | Conflitto di merge | Конфликт слияния |
| `git.state.untracked` | Untracked | Nicht verfolgt | 未跟踪 | Sin seguimiento | Non suivi | Non tracciato | Не отслеживается |
| `git.state.ignored` | Ignored by .gitignore | Durch .gitignore ignoriert | 已被 .gitignore 忽略 | Ignorado por .gitignore | Ignoré par .gitignore | Ignorato da .gitignore | Игнорируется в .gitignore |
| `git.state.renamed_staged` | Renamed from {orig}, staged for commit | Umbenannt von {orig}, zum Commit vorgemerkt | 从 {orig} 重新命名，已暂存待提交 | Renombrado desde {orig}, preparado para el commit | Renommé depuis {orig}, indexé pour le commit | Rinominato da {orig}, preparato per il commit | Переименован из {orig}, проиндексирован для коммита |
| `git.state.copied_staged` | Copied from {orig}, staged for commit | Kopiert von {orig}, zum Commit vorgemerkt | 从 {orig} 拷贝，已暂存待提交 | Copiado desde {orig}, preparado para el commit | Copié depuis {orig}, indexé pour le commit | Copiato da {orig}, preparato per il commit | Скопирован из {orig}, проиндексирован для коммита |
| `git.state.deleted_unstaged` | Deleted, not staged | Gelöscht, nicht vorgemerkt | 已删除，未暂存 | Eliminado, no preparado | Supprimé, non indexé | Eliminato, non preparato | Удален, не проиндексирован |
| `git.state.submodule` | Submodule has changes | Submodul hat Änderungen | 子模块有更改 | El submódulo tiene cambios | Le sous-module contient des modifications | Il sottomodulo contiene modifiche | В подмодуле есть изменения |
| `git.state.modified_unstaged` | Modified, not staged | Geändert, nicht vorgemerkt | 已修改，未暂存 | Modificado, no preparado | Modifié, non indexé | Modificato, non preparato | Изменен, не проиндексирован |
| `git.state.added_staged` | Added, staged for commit | Hinzugefügt, zum Commit vorgemerkt | 已添加，已暂存待提交 | Añadido, preparado para el commit | Ajouté, indexé pour le commit | Aggiunto, preparato per il commit | Добавлен, проиндексирован для коммита |
| `git.state.deletion_staged` | Deletion staged for commit | Löschung zum Commit vorgemerkt | 删除已暂存待提交 | Eliminación preparada para el commit | Suppression indexée pour le commit | Eliminazione preparata per il commit | Удаление проиндексировано для коммита |
| `git.state.staged` | Staged for commit | Zum Commit vorgemerkt | 已暂存待提交 | Preparado para el commit | Indexé pour le commit | Preparato per il commit | Проиндексирован для коммита |
| `git.state.renamed_short` | Renamed from {orig}, staged | Umbenannt von {orig}, vorgemerkt | 从 {orig} 重新命名，已暂存 | Renombrado desde {orig}, preparado | Renommé depuis {orig}, indexé | Rinominato da {orig}, preparato | Переименован из {orig}, проиндексирован |
| `git.folder.changed.one` | {count} changed file inside | {count} geänderte Datei darin | 其中有 {count} 个已更改的文件 | {count} archivo modificado dentro | {count} fichier modifié à l’intérieur | {count} file modificato all’interno | Внутри {count} измененный файл |
| `git.folder.changed.other` | {count} changed files inside | {count} geänderte Dateien darin | 其中有 {count} 个已更改的文件 | {count} archivos modificados dentro | {count} fichiers modifiés à l’intérieur | {count} file modificati all’interno | Внутри {count} измененных файла |
| `git.folder.conflicts.one` | {count} conflict | {count} Konflikt | {count} 个冲突 | {count} conflicto | {count} conflit | {count} conflitto | {count} конфликт |
| `git.folder.conflicts.other` | {count} conflicts | {count} Konflikte | {count} 个冲突 | {count} conflictos | {count} conflits | {count} conflitti | {count} конфликта |
| `git.chip.detached` | Detached HEAD at {name} | Losgelöster HEAD bei {name} | 分离的 HEAD，位于 {name} | HEAD desacoplado en {name} | HEAD détachée sur {name} | HEAD scollegato su {name} | Отсоединенный HEAD на {name} |
| `git.chip.branch` | Branch {name} | Branch {name} | 分支 {name} | Rama {name} | Branche {name} | Branch {name} | Ветка {name} |
| `git.chip.up_to_date` | up to date | aktuell | 已是最新 | actualizada | à jour | aggiornato | актуально |
| `git.chip.ahead` | {a} ahead | {a} voraus | 领先 {a} | {a} por delante | {a} en avance | {a} avanti | впереди на {a} |
| `git.chip.behind` | {d} behind | {d} zurück | 落后 {d} | {d} por detrás | {d} en retard | {d} indietro | позади на {d} |
| `git.chip.ahead_behind` | {a} ahead, {d} behind | {a} voraus, {d} zurück | 领先 {a}，落后 {d} | {a} por delante, {d} por detrás | {a} en avance, {d} en retard | {a} avanti, {d} indietro | впереди на {a}, позади на {d} |
| `git.chip.tracking` | Tracking {up} ({rel}) | Folgt {up} ({rel}) | 跟踪 {up}（{rel}） | Sigue a {up} ({rel}) | Suit {up} ({rel}) | Traccia {up} ({rel}) | Отслеживает {up} ({rel}) |
| `git.chip.tracking_gone` | Tracking {up} (gone) | Folgt {up} (entfernt) | 跟踪 {up}（已不存在） | Sigue a {up} (eliminada) | Suit {up} (supprimée) | Traccia {up} (rimosso) | Отслеживает {up} (удалена) |
| `git.chip.no_upstream` | No upstream branch | Kein Upstream-Branch | 无上游分支 | Sin rama upstream | Aucune branche amont | Nessun branch upstream | Нет вышестоящей ветки |
| `git.chip.last_commit` | Last commit: {subject} | Letzter Commit: {subject} | 上次提交：{subject} | Último commit: {subject} | Dernier commit : {subject} | Ultimo commit: {subject} | Последний коммит: {subject} |
| `git.chip.no_commits` | No commits yet | Noch keine Commits | 尚无提交 | Aún no hay commits | Aucun commit pour l’instant | Ancora nessun commit | Коммитов пока нет |
| `git.chip.click` | Click to show the Git panel | Klicke, um das Git-Panel zu zeigen | 点按以显示 Git 面板 | Haz clic para mostrar el panel Git | Cliquez pour afficher le panneau Git | Fai clic per mostrare il pannello Git | Нажмите, чтобы показать панель Git |
| `git.empty.not_local` | Not a local folder | Kein lokaler Ordner | 不是本地文件夹 | No es una carpeta local | Pas un dossier local | Non è una cartella locale | Не локальная папка |
| `git.empty.not_repo` | Not a git repository | Kein Git-Repository | 不是 Git 仓库 | No es un repositorio Git | Pas un dépôt Git | Non è un repository git | Не репозиторий Git |
| `git.loading.repo` | Looking for a repository… | Repository wird gesucht … | 正在查找仓库… | Buscando un repositorio… | Recherche d’un dépôt… | Ricerca di un repository… | Поиск репозитория… |
| `git.loading.status` | Loading status… | Status wird geladen … | 正在载入状态… | Cargando estado… | Chargement de l’état… | Caricamento dello stato… | Загрузка статуса… |
| `git.header.refresh` | Refresh Git status | Git-Status aktualisieren | 刷新 Git 状态 | Actualizar estado de Git | Actualiser l’état Git | Aggiorna stato Git | Обновить статус Git |
| `git.header.detached` | detached | losgelöst | 已分离 | desacoplado | détachée | scollegato | отсоединен |
| `git.header.to_push.one` | {count} commit to push | {count} Commit zum Pushen | {count} 个提交待推送 | {count} commit para hacer push | {count} commit à pousser | {count} commit da inviare | {count} коммит для отправки |
| `git.header.to_push.other` | {count} commits to push | {count} Commits zum Pushen | {count} 个提交待推送 | {count} commits para hacer push | {count} commits à pousser | {count} commit da inviare | {count} коммита для отправки |
| `git.header.to_pull.one` | {count} commit to pull | {count} Commit zum Pullen | {count} 个提交待拉取 | {count} commit para hacer pull | {count} commit à tirer | {count} commit da ricevere | {count} коммит для получения |
| `git.header.to_pull.other` | {count} commits to pull | {count} Commits zum Pullen | {count} 个提交待拉取 | {count} commits para hacer pull | {count} commits à tirer | {count} commit da ricevere | {count} коммита для получения |
| `git.header.tracking_gone` | Tracking {up} (gone) | Folgt {up} (entfernt) | 跟踪 {up}（已不存在） | Sigue a {up} (eliminada) | Suit {up} (supprimée) | Traccia {up} (rimosso) | Отслеживает {up} (удалена) |
| `git.header.tracking_current` | Tracking {up} · up to date | Folgt {up} · aktuell | 跟踪 {up} · 已是最新 | Sigue a {up} · actualizada | Suit {up} · à jour | Traccia {up} · aggiornato | Отслеживает {up} · актуально |
| `git.header.tracking_ahead` | Tracking {up} · {a} ahead | Folgt {up} · {a} voraus | 跟踪 {up} · 领先 {a} | Sigue a {up} · {a} por delante | Suit {up} · {a} en avance | Traccia {up} · {a} avanti | Отслеживает {up} · впереди на {a} |
| `git.header.tracking_behind` | Tracking {up} · {d} behind | Folgt {up} · {d} zurück | 跟踪 {up} · 落后 {d} | Sigue a {up} · {d} por detrás | Suit {up} · {d} en retard | Traccia {up} · {d} indietro | Отслеживает {up} · позади на {d} |
| `git.header.tracking_both` | Tracking {up} · {a} ahead, {d} behind | Folgt {up} · {a} voraus, {d} zurück | 跟踪 {up} · 领先 {a}，落后 {d} | Sigue a {up} · {a} por delante, {d} por detrás | Suit {up} · {a} en avance, {d} en retard | Traccia {up} · {a} avanti, {d} indietro | Отслеживает {up} · впереди на {a}, позади на {d} |
| `git.header.detached_head` | Detached HEAD | Losgelöster HEAD | 分离的 HEAD | HEAD desacoplado | HEAD détachée | HEAD scollegato | Отсоединенный HEAD |
| `git.commit.working` | Working… | Wird ausgeführt … | 正在处理… | Procesando… | En cours… | Esecuzione… | Выполняется… |
| `git.commit.nothing_staged` | Nothing staged | Nichts vorgemerkt | 未暂存任何内容 | Nada preparado | Rien d’indexé | Niente di preparato | Ничего не проиндексировано |
| `git.commit.staged.one` | {count} file staged | {count} Datei vorgemerkt | 已暂存 {count} 个文件 | {count} archivo preparado | {count} fichier indexé | {count} file preparato | Проиндексирован {count} файл |
| `git.commit.staged.other` | {count} files staged | {count} Dateien vorgemerkt | 已暂存 {count} 个文件 | {count} archivos preparados | {count} fichiers indexés | {count} file preparati | Проиндексировано {count} файла |
| `git.commit.button` | Commit | Commit | 提交 | Commit | Commit | Commit | Коммит |
| `git.group.conflicts` | Conflicts | Konflikte | 冲突 | Conflictos | Conflits | Conflitti | Конфликты |
| `git.group.staged` | Staged | Vorgemerkt | 已暂存 | Preparados | Indexé | Preparate | Проиндексированные |
| `git.group.changes` | Changes | Änderungen | 更改 | Cambios | Modifications | Modifiche | Изменения |
| `git.group.untracked` | Untracked | Nicht verfolgt | 未跟踪 | Sin seguimiento | Non suivi | Non tracciati | Неотслеживаемые |
| `git.action.stage_all` | Stage All | Alle vormerken | 全部暂存 | Preparar todo | Tout indexer | Prepara tutto | Индексировать все |
| `git.action.unstage_all` | Unstage All | Alle zurücknehmen | 全部取消暂存 | Quitar todo | Tout désindexer | Annulla tutto | Убрать все из индекса |
| `git.action.stage` | Stage | Vormerken | 暂存 | Preparar | Indexer | Prepara | Индексировать |
| `git.action.unstage` | Unstage | Vormerkung aufheben | 取消暂存 | Quitar | Désindexer | Annulla preparazione | Убрать из индекса |
| `git.action.resolve` | Mark as resolved (stage) | Als gelöst markieren (vormerken) | 标记为已解决（暂存） | Marcar como resuelto (preparar) | Marquer comme résolu (indexer) | Segna come risolto (prepara) | Отметить как разрешенный (индексировать) |
| `git.diff.staged` | Staged | Vorgemerkt | 已暂存 | Preparado | Indexé | Preparato | Проиндексировано |
| `git.diff.not_staged` | Not staged | Nicht vorgemerkt | 未暂存 | No preparado | Non indexé | Non preparato | Не проиндексировано |
| `git.diff.new_file` | New file | Neue Datei | 新文件 | Archivo nuevo | Nouveau fichier | Nuovo file | Новый файл |
| `git.diff.conflict` | Conflict | Konflikt | 冲突 | Conflicto | Conflit | Conflitto | Конфликт |
| `git.diff.close` | Close preview | Vorschau schließen | 关闭预览 | Cerrar vista previa | Fermer l’aperçu | Chiudi anteprima | Закрыть просмотр |
| `git.diff.loading` | Loading diff… | Diff wird geladen … | 正在载入差异… | Cargando diff… | Chargement du diff… | Caricamento del diff… | Загрузка изменений… |
| `git.diff.binary` | Binary file, no preview | Binärdatei, keine Vorschau | 二进制文件，无法预览 | Archivo binario, sin vista previa | Fichier binaire, pas d’aperçu | File binario, nessuna anteprima | Двоичный файл, просмотр недоступен |
| `git.diff.empty` | No changes to show | Keine Änderungen zum Anzeigen | 没有可显示的更改 | No hay cambios que mostrar | Aucune modification à afficher | Nessuna modifica da mostrare | Нет изменений для показа |
| `git.diff.truncated` | … showing the first {n} lines | … nur die ersten {n} Zeilen | … 仅显示前 {n} 行 | … se muestran solo las primeras {n} líneas | … affichage des {n} premières lignes | … solo le prime {n} righe | … показаны первые {n} строк |
| `git.commits.title` | Recent Commits | Letzte Commits | 最近的提交 | Commits recientes | Commits récents | Commit recenti | Недавние коммиты |
| `git.commits.none` | No commits yet | Noch keine Commits | 尚无提交 | Aún no hay commits | Aucun commit pour l’instant | Ancora nessun commit | Коммитов пока нет |
| `git.time.now` | just now | gerade eben | 刚刚 | ahora mismo | à l’instant | adesso | только что |
| `git.time.min` | {n} min ago | vor {n} Min. | {n} 分钟前 | hace {n} min | il y a {n} min | {n} min fa | {n} мин назад |
| `git.time.hours` | {n} h ago | vor {n} Std. | {n} 小时前 | hace {n} h | il y a {n} h | {n} h fa | {n} ч назад |
| `git.time.yesterday` | yesterday | gestern | 昨天 | ayer | hier | ieri | вчера |
| `git.time.days` | {n} days ago | vor {n} Tagen | {n} 天前 | hace {n} días | il y a {n} jours | {n} giorni fa | {n} дн. назад |
| `git.time.weeks` | {n} weeks ago | vor {n} Wochen | {n} 周前 | hace {n} semanas | il y a {n} semaines | {n} settimane fa | {n} нед. назад |
| `git.time.months` | {n} months ago | vor {n} Monaten | {n} 个月前 | hace {n} meses | il y a {n} mois | {n} mesi fa | {n} мес. назад |
| `git.time.year` | 1 year ago | vor 1 Jahr | 1 年前 | hace 1 año | il y a 1 an | 1 anno fa | 1 год назад |
| `git.time.years` | {n} years ago | vor {n} Jahren | {n} 年前 | hace {n} años | il y a {n} ans | {n} anni fa | {n} г. назад |

## Plugin: Image Tools

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | Image Tools | Bildwerkzeuge | 图像工具 | Herramientas de imagen | Outils d’image | Strumenti immagine | Инструменты изображений |
| `retina.exists` | “{name}” already exists | „{name}“ gibt es schon | “{name}”已存在 | “{name}” ya existe | « {name} » existe déjà | “{name}” esiste già | «{name}» уже существует |
| `preview.failed` | Can't preview this image | Für dieses Bild gibt es keine Vorschau | 无法预览此图像 | No se puede previsualizar esta imagen | Impossible d’afficher l’aperçu de cette image | Impossibile mostrare l’anteprima di questa immagine | Не удается показать просмотр этого изображения |
| `cmd.panel` | Show Image Panel | Bild-Panel einblenden | 显示图像面板 | Mostrar panel de imagen | Afficher le panneau Image | Mostra pannello Immagine | Показать панель изображений |
| `cmd.resize` | Resize… | Größe ändern … | 调整大小… | Ajustar tamaño… | Redimensionner… | Ridimensiona… | Изменить размер… |
| `cmd.convert` | Convert to… | Konvertieren in … | 转换为… | Convertir a… | Convertir en… | Converti in… | Конвертировать в… |
| `cmd.rotate_left` | Rotate 90° Left | 90° nach links drehen | 向左旋转 90° | Girar 90° a la izquierda | Faire pivoter de 90° à gauche | Ruota di 90° a sinistra | Повернуть на 90° влево |
| `cmd.rotate_right` | Rotate 90° Right | 90° nach rechts drehen | 向右旋转 90° | Girar 90° a la derecha | Faire pivoter de 90° à droite | Ruota di 90° a destra | Повернуть на 90° вправо |
| `cmd.rotate_180` | Rotate 180° | 180° drehen | 旋转 180° | Girar 180° | Faire pivoter de 180° | Ruota di 180° | Повернуть на 180° |
| `cmd.flip_h` | Flip Horizontal | Horizontal spiegeln | 水平翻转 | Voltear horizontalmente | Retourner horizontalement | Rifletti orizzontalmente | Отразить по горизонтали |
| `cmd.flip_v` | Flip Vertical | Vertikal spiegeln | 垂直翻转 | Voltear verticalmente | Retourner verticalement | Rifletti verticalmente | Отразить по вертикали |
| `cmd.strip` | Strip Metadata… | Metadaten entfernen … | 移除元数据… | Eliminar metadatos… | Supprimer les métadonnées… | Rimuovi metadati… | Удалить метаданные… |
| `cmd.optimize` | Optimize for Web… | Fürs Web optimieren … | 针对网页优化… | Optimizar para la web… | Optimiser pour le Web… | Ottimizza per il web… | Оптимизировать для веба… |
| `cmd.retina` | Make @2x/@1x Set… | @2x/@1x-Set erstellen … | 生成 @2x/@1x 图像集… | Crear conjunto @2x/@1x… | Créer un jeu @2x/@1x… | Crea set @2x/@1x… | Создать набор @2x/@1x… |
| `cmd.datauri` | Copy as Data URI | Als Data-URI kopieren | 拷贝为 Data URI | Copiar como URI de datos | Copier comme URI de données | Copia come data URI | Скопировать как data URI |
| `panel.image` | Image | Bild | 图像 | Imagen | Image | Immagine | Изображение |
| `panel.tools` | Image Tools | Bildwerkzeuge | 图像工具 | Herramientas de imagen | Outils d’image | Strumenti immagine | Инструменты изображений |
| `op.resize` | Resize | Größe ändern | 调整大小 | Ajustar tamaño | Redimensionner | Ridimensiona | Изменить размер |
| `op.convert` | Convert | Konvertieren | 转换 | Convertir | Convertir | Converti | Конвертировать |
| `op.rotate` | Rotate & Flip | Drehen & Spiegeln | 旋转与翻转 | Girar y voltear | Rotation et retournement | Ruota e rifletti | Повернуть и отразить |
| `op.strip` | Strip Metadata | Metadaten entfernen | 移除元数据 | Eliminar metadatos | Supprimer les métadonnées | Rimuovi metadati | Удалить метаданные |
| `op.optimize` | Optimize for Web | Fürs Web optimieren | 针对网页优化 | Optimizar para la web | Optimiser pour le Web | Ottimizza per il web | Оптимизировать для веба |
| `op.retina` | Make @2x/@1x Set | @2x/@1x-Set erstellen | 生成 @2x/@1x 图像集 | Crear conjunto @2x/@1x | Créer un jeu @2x/@1x | Crea set @2x/@1x | Создать набор @2x/@1x |
| `ing.resize` | Resizing | Größe ändern: | 正在调整大小 | Ajustando el tamaño de | Redimensionnement : | Ridimensionamento: | Изменение размера: |
| `ing.convert` | Converting | Konvertieren: | 正在转换 | Convirtiendo | Conversion : | Conversione: | Конвертация: |
| `ing.rotate` | Rotating | Drehen: | 正在旋转 | Girando | Rotation : | Rotazione: | Поворот: |
| `ing.strip` | Stripping | Metadaten entfernen: | 正在移除元数据 | Eliminando metadatos de | Suppression des métadonnées : | Rimozione metadati: | Удаление метаданных: |
| `ing.optimize` | Optimizing | Optimieren: | 正在优化 | Optimizando | Optimisation : | Ottimizzazione: | Оптимизация: |
| `ing.retina` | Making | Erstellen: | 正在生成 | Creando | Création : | Creazione: | Создание: |
| `done.resize` | Resized {what} | Größe von {what} geändert | 已调整 {what}的大小 | Tamaño ajustado: {what} | Redimensionnement terminé : {what} | Ridimensionamento completato: {what} | Размер изменен: {what} |
| `done.convert` | Converted {what} | {what} konvertiert | 已转换 {what} | Conversión completada: {what} | Conversion terminée : {what} | Conversione completata: {what} | Конвертировано: {what} |
| `done.rotate` | Rotated {what} | {what} gedreht | 已旋转 {what} | Giro completado: {what} | Rotation terminée : {what} | Rotazione completata: {what} | Повернуто: {what} |
| `done.strip` | Stripped {what} | Metadaten von {what} entfernt | 已移除 {what}的元数据 | Metadatos eliminados: {what} | Métadonnées supprimées : {what} | Metadati rimossi: {what} | Метаданные удалены: {what} |
| `done.optimize` | Optimized {what} | {what} optimiert | 已优化 {what} | Optimización completada: {what} | Optimisation terminée : {what} | Ottimizzazione completata: {what} | Оптимизировано: {what} |
| `done.retina` | Made {what} | {what} erstellt | 已生成 {what} | Creación completada: {what} | Création terminée : {what} | Creazione completata: {what} | Создано: {what} |
| `quality.60` | Smaller | Kleiner | 较小 | Menor tamaño | Plus petite | Più piccola | Меньше |
| `quality.82` | Balanced | Ausgewogen | 均衡 | Equilibrada | Équilibrée | Bilanciata | Сбалансированное |
| `quality.90` | High | Hoch | 高 | Alta | Élevée | Alta | Высокое |
| `quality.100` | Maximum | Maximal | 最高 | Máxima | Maximale | Massima | Максимальное |
| `images.one` | {count} image | {count} Bild | {count} 个图像 | {count} imagen | {count} image | {count} immagine | {count} изображение |
| `images.other` | {count} images | {count} Bilder | {count} 个图像 | {count} imágenes | {count} images | {count} immagini | {count} изображения |
| `sets.one` | {count} @2x/@1x set | {count} @2x/@1x-Set | {count} 组 @2x/@1x 图像集 | {count} conjunto @2x/@1x | {count} jeu @2x/@1x | {count} set @2x/@1x | {count} набор @2x/@1x |
| `sets.other` | {count} @2x/@1x sets | {count} @2x/@1x-Sets | {count} 组 @2x/@1x 图像集 | {count} conjuntos @2x/@1x | {count} jeux @2x/@1x | {count} set @2x/@1x | {count} набора @2x/@1x |
| `originals.one` | {count} original | {count} Original | {count} 个原件 | {count} original | {count} original | {count} originale | {count} оригинал |
| `originals.other` | {count} originals | {count} Originale | {count} 个原件 | {count} originales | {count} originaux | {count} originali | {count} оригинала |
| `frames.one` | {count} frame | {count} Frame | {count} 帧 | {count} fotograma | {count} image clé | {count} fotogramma | {count} кадр |
| `frames.other` | {count} frames | {count} Frames | {count} 帧 | {count} fotogramas | {count} images clés | {count} fotogrammi | {count} кадра |
| `bytes.one` | {count} byte | {count} Byte | {count} 字节 | {count} byte | {count} octet | {count} byte | {count} байт |
| `bytes.other` | {count} bytes | {count} Byte | {count} 字节 | {count} bytes | {count} octets | {count} byte | {count} байта |
| `make_sets.one` | Make {count} Set | {count} Set erstellen | 生成 {count} 组图像集 | Crear {count} conjunto | Créer {count} jeu | Crea {count} set | Создать {count} набор |
| `make_sets.other` | Make {count} Sets | {count} Sets erstellen | 生成 {count} 组图像集 | Crear {count} conjuntos | Créer {count} jeux | Crea {count} set | Создать {count} набора |
| `tag.large` | Large for the web | Groß fürs Web | 对网页来说过大 | Grande para la web | Volumineuse pour le Web | Grande per il web | Большое для веба |
| `tag.low` | Low resolution | Niedrige Auflösung | 低分辨率 | Baja resolución | Basse résolution | Bassa risoluzione | Низкое разрешение |
| `tag.animated` | Animated | Animiert | 动画 | Animada | Animée | Animata | Анимированное |
| `tag.animated_frames` | Animated · {frames} | Animiert · {frames} | 动画 · {frames} | Animada · {frames} | Animée · {frames} | Animata · {frames} | Анимированное · {frames} |
| `tag.alpha` | Transparency | Transparenz | 透明度 | Transparencia | Transparence | Trasparenza | Прозрачность |
| `tag.interlaced` | Interlaced | Interlaced (Zeilensprung) | 隔行扫描 | Entrelazada | Entrelacé | Interlacciata | Чересстрочное |
| `tag.gps` | GPS location | GPS-Standort | GPS 位置 | Ubicación GPS | Position GPS | Posizione GPS | Геопозиция GPS |
| `key.dimensions` | Dimensions | Abmessungen | 尺寸 | Dimensiones | Dimensions | Dimensioni | Размеры |
| `key.aspect` | Aspect ratio | Seitenverhältnis | 长宽比 | Proporción | Proportions | Proporzioni | Соотношение сторон |
| `key.file_size` | File size | Dateigröße | 文件大小 | Tamaño del archivo | Taille du fichier | Dimensione file | Размер файла |
| `key.format` | Format | Format | 格式 | Formato | Format | Formato | Формат |
| `key.profile` | Colour profile | Farbprofil | 颜色描述文件 | Perfil de color | Profil colorimétrique | Profilo colore | Цветовой профиль |
| `key.dpi` | DPI | DPI | DPI | PPP | PPP | DPI | DPI |
| `key.alpha` | Alpha | Alphakanal | Alpha 通道 | Canal alfa | Couche alpha | Canale alfa | Альфа-канал |
| `key.animated` | Animated | Animiert | 动画 | Animada | Animée | Animata | Анимация |
| `key.total_size` | Total size | Gesamtgröße | 总大小 | Tamaño total | Taille totale | Dimensione totale | Общий размер |
| `key.file_sizes` | File sizes | Dateigrößen | 文件大小 | Tamaños de archivo | Tailles des fichiers | Dimensioni file | Размеры файлов |
| `key.formats` | Formats | Formate | 格式 | Formatos | Formats | Formati | Форматы |
| `key.over_2mb` | Over 2 MB | Über 2 MB | 超过 2 MB | Más de 2 MB | Plus de 2 Mo | Oltre 2 MB | Более 2 МБ |
| `key.coordinates` | Coordinates | Koordinaten | 坐标 | Coordenadas | Coordonnées | Coordinate | Координаты |
| `key.width` | Width | Breite | 宽度 | Anchura | Largeur | Larghezza | Ширина |
| `key.height` | Height | Höhe | 高度 | Altura | Hauteur | Altezza | Высота |
| `key.size` | Size | Größe | 大小 | Tamaño | Taille | Dimensione | Размер |
| `key.colours` | Colours | Farben | 颜色 | Colores | Couleurs | Colori | Цвета |
| `val.rotated` | {dims} (rotated by EXIF) | {dims} (per EXIF gedreht) | {dims}（已按 EXIF 旋转） | {dims} (girada según EXIF) | {dims} (pivotée selon EXIF) | {dims} (ruotata da EXIF) | {dims} (повернуто по EXIF) |
| `val.bytes` | {size} ({bytes} bytes) | {size} ({bytes} Byte) | {size}（{bytes} 字节） | {size} ({bytes} bytes) | {size} ({bytes} octets) | {size} ({bytes} byte) | {size} ({bytes} байт) |
| `val.no_profile` | None (browsers assume sRGB) | Keins (Browser nehmen sRGB an) | 无（浏览器默认使用 sRGB） | Ninguno (los navegadores asumen sRGB) | Aucun (les navigateurs supposent sRGB) | Nessuno (i browser presumono sRGB) | Нет (браузеры предполагают sRGB) |
| `val.yes` | Yes | Ja | 是 | Sí | Oui | Sì | Да |
| `val.no` | No | Nein | 否 | No | Non | No | Нет |
| `val.yes_frames` | Yes · {frames} | Ja · {frames} | 是 · {frames} | Sí · {frames} | Oui · {frames} | Sì · {frames} | Да · {frames} |
| `val.unreadable` | {format} (size unreadable) | {format} (Größe nicht lesbar) | {format}（无法读取大小） | {format} (tamaño ilegible) | {format} (taille illisible) | {format} (dimensione non leggibile) | {format} (размер не читается) |
| `section.camera` | Camera | Kamera | 相机 | Cámara | Appareil photo | Fotocamera | Камера |
| `section.location` | Location | Ort | 位置 | Ubicación | Lieu | Posizione | Место съемки |
| `section.actions` | Actions | Aktionen | 操作 | Acciones | Actions | Azioni | Действия |
| `section.largest` | Largest | Die größten | 最大的图像 | Las más grandes | Les plus volumineuses | Le più grandi | Самые большие |
| `section.working` | Working | In Arbeit | 处理中 | En curso | En cours | In corso | Выполняется |
| `btn.open_maps` | Open in Maps | In Karten öffnen | 在“地图”中打开 | Abrir en Mapas | Ouvrir dans Plans | Apri in Mappe | Открыть в Картах |
| `btn.cancel` | Cancel | Abbrechen | 取消 | Cancelar | Annuler | Annulla | Отменить |
| `btn.back` | Back | Zurück | 返回 | Atrás | Retour | Indietro | Назад |
| `exif.camera` | Camera | Kamera | 相机 | Cámara | Appareil photo | Fotocamera | Камера |
| `exif.lens` | Lens | Objektiv | 镜头 | Objetivo | Objectif | Obiettivo | Объектив |
| `exif.exposure` | Exposure | Belichtung | 曝光 | Exposición | Exposition | Esposizione | Экспозиция |
| `exif.focal` | Focal length | Brennweite | 焦距 | Distancia focal | Focale | Lunghezza focale | Фокусное расстояние |
| `exif.equiv` | {fl} ({mm} mm equiv.) | {fl} (entspr. {mm} mm) | {fl}（等效 {mm} 毫米） | {fl} (equiv. a {mm} mm) | {fl} (équiv. {mm} mm) | {fl} (equiv. {mm} mm) | {fl} (экв. {mm} мм) |
| `exif.taken` | Taken | Aufgenommen | 拍摄时间 | Tomada | Prise le | Scattata | Снято |
| `exif.software` | Software | Software | 软件 | Software | Logiciel | Software | ПО |
| `state.waiting` | Waiting for Manta… | Warte auf Manta … | 正在等待 Manta… | Esperando a Manta… | En attente de Manta… | In attesa di Manta… | Ожидание Manta… |
| `state.remote` | Remote folder | Entfernter Ordner | 远程文件夹 | Carpeta remota | Dossier distant | Cartella remota | Удаленная папка |
| `state.none` | No image selected | Kein Bild ausgewählt | 未选择图像 | Ninguna imagen seleccionada | Aucune image sélectionnée | Nessuna immagine selezionata | Изображение не выбрано |
| `state.cant_read` | Can't read {name} | {name} kann nicht gelesen werden | 无法读取 {name} | No se puede leer {name} | Impossible de lire {name} | Impossibile leggere {name} | Не удается прочитать {name} |
| `confirm.btn_replace` | Replace {what} | {what} ersetzen | 替换 {what} | Reemplazar {what} | Remplacer {what} | Sostituisci {what} | Заменить {what} |
| `confirm.btn_overwrite` | Overwrite {what} | {what} überschreiben | 覆盖 {what} | Sobrescribir {what} | Écraser {what} | Sovrascrivi {what} | Перезаписать {what} |
| `resize.size` | Size | Größe | 大小 | Tamaño | Taille | Dimensione | Размер |
| `resize.keeps` | keeps the aspect ratio | behält das Seitenverhältnis | 保持长宽比 | mantiene la proporción | conserve les proportions | mantiene le proporzioni | сохраняет соотношение сторон |
| `resize.smaller` | already smaller | schon kleiner | 已经更小 | ya es más pequeña | déjà plus petite | già più piccola | уже меньше |
| `resize.edge` | longest edge | längste Kante | 最长边 | lado más largo | plus grand côté | lato più lungo | длинная сторона |
| `resize.custom` | Custom… | Eigene … | 自定… | Personalizado… | Personnalisée… | Personalizzata… | Другой… |
| `resize.width_px` | width {px} px | Breite {px} px | 宽度 {px} 像素 | anchura {px} px | largeur {px} px | larghezza {px} px | ширина {px} пикс. |
| `resize.height_px` | height {px} px | Höhe {px} px | 高度 {px} 像素 | altura {px} px | hauteur {px} px | altezza {px} px | высота {px} пикс. |
| `resize.set_height` | Set height instead | Stattdessen Höhe festlegen | 改为设定高度 | Definir la altura en su lugar | Définir plutôt la hauteur | Imposta invece l’altezza | Задать высоту |
| `resize.set_width` | Set width instead | Stattdessen Breite festlegen | 改为设定宽度 | Definir la anchura en su lugar | Définir plutôt la largeur | Imposta invece la larghezza | Задать ширину |
| `convert.format` | Format | Format | 格式 | Formato | Format | Formato | Формат |
| `fmt.jpeg` | photos, no transparency | Fotos, keine Transparenz | 照片，不支持透明度 | fotos, sin transparencia | photos, sans transparence | foto, senza trasparenza | фото, без прозрачности |
| `fmt.png` | lossless, transparency | verlustfrei, Transparenz | 无损，支持透明度 | sin pérdida, transparencia | sans perte, transparence | senza perdita, trasparenza | без потерь, прозрачность |
| `fmt.heic` | small, Apple devices | klein, Apple-Geräte | 体积小，适用于 Apple 设备 | pequeño, dispositivos Apple | compact, appareils Apple | compatto, dispositivi Apple | компактный, устройства Apple |
| `fmt.tiff` | print, lossless | Druck, verlustfrei | 打印，无损 | impresión, sin pérdida | impression, sans perte | stampa, senza perdita | печать, без потерь |
| `fmt.webp` | web, via cwebp | Web, mit cwebp | 网页，通过 cwebp | web, mediante cwebp | Web, via cwebp | web, tramite cwebp | веб, через cwebp |
| `convert.webp_needs` | WebP needs cwebp: brew install webp | WebP braucht cwebp: brew install webp | WebP 需要 cwebp：brew install webp | WebP requiere cwebp: brew install webp | WebP nécessite cwebp : brew install webp | WebP richiede cwebp: brew install webp | Для WebP нужен cwebp: brew install webp |
| `convert.quality` | Quality | Qualität | 品质 | Calidad | Qualité | Qualità | Качество |
| `rotate.transform` | Transform | Transformation | 变换 | Transformación | Transformation | Trasformazione | Преобразование |
| `strip.what` | What it does | Was passiert | 作用 | Qué hace | Effet | Cosa fa | Что произойдет |
| `strip.builtin` | built in, lossless (JPEG, PNG, WebP) | integriert, verlustfrei (JPEG, PNG, WebP) | 内建，无损（JPEG、PNG、WebP） | integrado, sin pérdida (JPEG, PNG, WebP) | intégré, sans perte (JPEG, PNG, WebP) | integrato, senza perdita (JPEG, PNG, WebP) | встроенное, без потерь (JPEG, PNG, WebP) |
| `opt.colours` | converted to sRGB | in sRGB umgewandelt | 转换为 sRGB | convertido a sRGB | converti en sRGB | convertita in sRGB | преобразование в sRGB |
| `retina.files` | Files | Dateien | 文件 | Archivos | Fichiers | File | Файлы |
| `retina.original` | original | Original | 原件 | original | original | originale | оригинал |
| `retina.original_renamed` | original, renamed | Original, umbenannt | 原件，已重新命名 | original, renombrado | original, renommé | originale, rinominato | оригинал, переименован |
| `retina.half` | half size | halbe Größe | 一半大小 | mitad de tamaño | demi-taille | metà dimensione | половинный размер |
| `retina.more` | and {images} more | und {images} mehr | 以及其他 {images} | y {images} más | et {images} de plus | e altre {images} | и еще {images} |
| `save.as` | Save as | Sichern als | 存储为 | Guardar como | Enregistrer sous | Salva come | Сохранить как |
| `save.next_to` | Next to the original | Neben dem Original | 原件旁边 | Junto al original | À côté de l’original | Accanto all’originale | Рядом с оригиналом |
| `save.subfolder` | In a subfolder | In einem Unterordner | 子文件夹中 | En una subcarpeta | Dans un sous-dossier | In una sottocartella | Во вложенную папку |
| `save.replace` | Replace originals | Originale ersetzen | 替换原件 | Reemplazar originales | Remplacer les originaux | Sostituisci originali | Заменить оригиналы |
| `save.overwrite` | Overwrite originals | Originale überschreiben | 覆盖原件 | Sobrescribir originales | Écraser les originaux | Sovrascrivi originali | Перезаписать оригиналы |
| `save.no_undo` | can't be undone | nicht widerrufbar | 无法撤销 | no se puede deshacer | irréversible | non annullabile | нельзя отменить |
| `save.trash` | originals go to the Trash | Originale kommen in den Papierkorb | 原件将移到废纸篓 | los originales van a la papelera | les originaux vont dans la corbeille | gli originali vanno nel Cestino | оригиналы перемещаются в Корзину |
| `save.asks` | Asks before it overwrites anything | Fragt nach, bevor etwas überschrieben wird | 覆盖任何内容前都会询问 | Pregunta antes de sobrescribir nada | Demande confirmation avant d’écraser quoi que ce soit | Chiede prima di sovrascrivere qualcosa | Спрашивает перед перезаписью |
| `run.resize` | Resize | Größe ändern | 调整大小 | Ajustar tamaño | Redimensionner | Ridimensiona | Изменить размер |
| `run.convert` | Convert | Konvertieren | 转换 | Convertir | Convertir | Converti | Конвертировать |
| `run.rotate` | Apply | Anwenden | 应用 | Aplicar | Appliquer | Applica | Применить |
| `run.strip` | Strip Metadata | Metadaten entfernen | 移除元数据 | Eliminar metadatos | Supprimer les métadonnées | Rimuovi metadati | Удалить метаданные |
| `run.optimize` | Optimize | Optimieren | 优化 | Optimizar | Optimiser | Ottimizza | Оптимизировать |
| `job.cancelled` | Cancelled after {what} | Abgebrochen nach {what} | 处理 {what}后已取消 | Cancelado tras {what} | Annulé après {what} | Annullato dopo {what} | Отменено, обработано: {what} |
| `job.failed` | {op} failed: {name}: {error} | {op} fehlgeschlagen: {name}: {error} | {op}失败：{name}：{error} | Error en {op}: {name}: {error} | Échec de l’opération {op} : {name} : {error} | {op} non riuscito: {name}: {error} | Ошибка операции «{op}»: {name}: {error} |
| `job.some_failed` | {images} failed ({name}: {error}) | {images} fehlgeschlagen ({name}: {error}) | {images}失败（{name}：{error}） | Error en {images} ({name}: {error}) | Échec : {images} ({name} : {error}) | Non riuscito: {images} ({name}: {error}) | Ошибка: {images} ({name}: {error}) |
| `job.skipped` | {images} skipped ({why}) | {images} übersprungen ({why}) | 已跳过 {images}（{why}） | {images} omitidas ({why}) | Ignorées : {images} ({why}) | Saltato: {images} ({why}) | Пропущено: {images} ({why}) |
| `job.trashed` | {originals} moved to the Trash | {originals} in den Papierkorb gelegt | 已将 {originals}移到废纸篓 | {originals} trasladados a la papelera | Placés dans la corbeille : {originals} | Spostato nel Cestino: {originals} | В Корзину перемещено: {originals} |
| `skip.unreadable` | unreadable size | Größe nicht lesbar | 无法读取大小 | tamaño ilegible | taille illisible | dimensione non leggibile | размер не читается |
| `skip.small` | already small enough | schon klein genug | 已足够小 | ya es lo bastante pequeña | déjà assez petite | già abbastanza piccola | уже достаточно маленькие |
| `uri.unreadable` | can't read the file | Die Datei kann nicht gelesen werden | 无法读取该文件 | no se puede leer el archivo | impossible de lire le fichier | impossibile leggere il file | не удается прочитать файл |
| `uri.format` | browsers can't show this format | Browser können dieses Format nicht anzeigen | 浏览器无法显示此格式 | los navegadores no pueden mostrar este formato | les navigateurs ne peuvent pas afficher ce format | i browser non possono mostrare questo formato | браузеры не могут показать этот формат |
| `webp.needs_cwebp` | WebP needs cwebp (brew install webp) | WebP braucht cwebp (brew install webp) | WebP 需要 cwebp（brew install webp） | WebP requiere cwebp (brew install webp) | WebP nécessite cwebp (brew install webp) | WebP richiede cwebp (brew install webp) | Для WebP нужен cwebp (brew install webp) |

## Plugin: Markdown Preview

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | Markdown Preview | Markdown-Vorschau | Markdown 预览 | Vista previa de Markdown | Aperçu Markdown | Anteprima Markdown | Просмотр Markdown |
| `cmd.toggle` | Toggle Markdown Preview | Markdown-Vorschau ein-/ausblenden | 显示/隐藏 Markdown 预览 | Mostrar/ocultar vista previa de Markdown | Afficher/masquer l’aperçu Markdown | Mostra/nascondi anteprima Markdown | Показать/скрыть просмотр Markdown |
| `panel.title` | Markdown | Markdown | Markdown | Markdown | Markdown | Markdown | Markdown |
| `settings.web_images` | Load images from the web (http/https) | Bilder aus dem Internet laden (http/https) | 从网络载入图像（http/https） | Cargar imágenes de internet (http/https) | Charger les images depuis le Web (http/https) | Carica immagini dal web (http/https) | Загружать изображения из интернета (http/https) |
| `back` | Back | Zurück | 返回 | Atrás | Retour | Indietro | Назад |
| `reveal` | Reveal in Pane | Im Bereich zeigen | 在面板中显示 | Mostrar en el panel | Afficher dans la sous-fenêtre | Mostra nel pannello | Показать в панели |
| `edit` | Open in Editor | Im Editor öffnen | 在编辑器中打开 | Abrir en el editor | Ouvrir dans l’éditeur | Apri nell’editor | Открыть в редакторе |
| `edit.local_only` | Open in Editor — local files only | Im Editor öffnen – nur lokale Dateien | 在编辑器中打开——仅限本地文件 | Abrir en el editor: solo archivos locales | Ouvrir dans l’éditeur — fichiers locaux uniquement | Apri nell’editor – solo file locali | Открыть в редакторе — только локальные файлы |
| `contents` | Contents | Inhalt | 目录 | Contenido | Sommaire | Indice | Содержание |
| `contents.caption` | CONTENTS | INHALT | 目录 | CONTENIDO | SOMMAIRE | INDICE | СОДЕРЖАНИЕ |
| `empty.title` | Select a Markdown file | Wähle eine Markdown-Datei aus | 选择一个 Markdown 文件 | Selecciona un archivo Markdown | Sélectionnez un fichier Markdown | Seleziona un file Markdown | Выберите файл Markdown |
| `copy` | Copy | Kopieren | 拷贝 | Copiar | Copier | Copia | Скопировать |
| `copied` | Copied | Kopiert | 已拷贝 | Copiado | Copié | Copiato | Скопировано |
| `loading` | Loading… | Wird geladen … | 正在载入… | Cargando… | Chargement… | Caricamento… | Загрузка… |
| `image.unsupported` | This kind of image isn't shown | Diese Art von Bild wird nicht angezeigt | 不显示此类图像 | Este tipo de imagen no se muestra | Ce type d’image n’est pas affiché | Questo tipo di immagine non viene mostrato | Изображения этого типа не показываются |
| `image.svg` | SVG images aren't shown | SVG-Bilder werden nicht angezeigt | 不显示 SVG 图像 | Las imágenes SVG no se muestran | Les images SVG ne sont pas affichées | Le immagini SVG non vengono mostrate | Изображения SVG не показываются |
| `image.failed` | Couldn't load the image: {error} | Das Bild konnte nicht geladen werden: {error} | 无法载入图像：{error} | No se ha podido cargar la imagen: {error} | Impossible de charger l’image : {error} | Impossibile caricare l’immagine: {error} | Не удалось загрузить изображение: {error} |
| `image.too_large` | too large | zu groß | 太大 | demasiado grande | trop volumineuse | troppo grande | слишком большое |
| `alert.important` | Important | Wichtig | 重要 | Importante | Important | Importante | Важно |
| `alert.warning` | Warning | Warnung | 警告 | Advertencia | Avertissement | Avviso | Предупреждение |
| `alert.caution` | Caution | Vorsicht | 注意 | Precaución | Attention | Attenzione | Осторожно |
| `bytes.one` | {count} byte | {count} Byte | {count} 字节 | {count} byte | {count} octet | {count} byte | {count} байт |
| `bytes.other` | {count} bytes | {count} Byte | {count} 字节 | {count} bytes | {count} octets | {count} byte | {count} байта |

## Plugin: Project Info

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | Project Info | Projektinfo | 项目信息 | Info del proyecto | Infos projet | Info progetto | О проекте |
| `cmd.run` | Run Project Script… | Projektskript ausführen … | 运行项目脚本… | Ejecutar script del proyecto… | Exécuter un script du projet… | Esegui script del progetto… | Запустить скрипт проекта… |
| `cmd.panel` | Show Project Panel | Projekt-Panel einblenden | 显示项目面板 | Mostrar panel del proyecto | Afficher le panneau Projet | Mostra pannello Progetto | Показать панель проекта |
| `cmd.rerun` | Re-run Last Script | Letztes Skript erneut ausführen | 重新运行上一个脚本 | Volver a ejecutar el último script | Réexécuter le dernier script | Esegui di nuovo l’ultimo script | Повторить последний скрипт |
| `cmd.show_scripts` | Show Scripts | Skripte zeigen | 显示脚本 | Mostrar scripts | Afficher les scripts | Mostra script | Показать скрипты |
| `panel.project` | Project | Projekt | 项目 | Proyecto | Projet | Progetto | Проект |
| `panel.picker` | Run Project Script | Projektskript ausführen | 运行项目脚本 | Ejecutar script del proyecto | Exécuter un script du projet | Esegui script del progetto | Запустить скрипт проекта |
| `panel.confirm` | Run Script | Skript ausführen | 运行脚本 | Ejecutar script | Exécuter le script | Esegui script | Запустить скрипт |
| `chip.scripts.one` | {label} · {count} script | {label} · {count} Skript | {label} · {count} 个脚本 | {label} · {count} script | {label} · {count} script | {label} · {count} script | {label} · {count} скрипт |
| `chip.scripts.other` | {label} · {count} scripts | {label} · {count} Skripte | {label} · {count} 个脚本 | {label} · {count} scripts | {label} · {count} scripts | {label} · {count} script | {label} · {count} скрипта |
| `chip.inside` | Inside {names} | Innerhalb von {names} | 位于 {names} 内 | Dentro de {names} | Dans {names} | All’interno di {names} | Внутри {names} |
| `chip.click` | Click to show or hide the Project panel | Klicken, um das Projekt-Panel ein- oder auszublenden | 点按以显示或隐藏项目面板 | Haz clic para mostrar u ocultar el panel del proyecto | Cliquez pour afficher ou masquer le panneau Projet | Fai clic per mostrare o nascondere il pannello Progetto | Нажмите, чтобы показать или скрыть панель проекта |
| `kind.project` | Project | Projekt | 项目 | Proyecto | Projet | Progetto | Проект |
| `section.workspace` | Workspace | Workspace | 工作区 | Espacio de trabajo | Espace de travail | Workspace | Рабочее пространство |
| `ws.packages.one` | {count} package | {count} Paket | {count} 个包 | {count} paquete | {count} paquet | {count} pacchetto | {count} пакет |
| `ws.packages.other` | {count} packages | {count} Pakete | {count} 个包 | {count} paquetes | {count} paquets | {count} pacchetti | {count} пакета |
| `section.recent` | Recent | Zuletzt ausgeführt | 最近运行 | Recientes | Récents | Eseguiti di recente | Недавние |
| `section.inside` | Inside {name} | Innerhalb von {name} | 位于 {name} 内 | Dentro de {name} | Dans {name} | All’interno di {name} | Внутри {name} |
| `ago.now` | just now | gerade eben | 刚刚 | ahora mismo | à l’instant | adesso | только что |
| `ago.min` | {n} min ago | vor {n} Min. | {n} 分钟前 | hace {n} min | il y a {n} min | {n} min fa | {n} мин назад |
| `ago.hours` | {n} h ago | vor {n} Std. | {n} 小时前 | hace {n} h | il y a {n} h | {n} h fa | {n} ч назад |
| `ago.days` | {n} d ago | vor {n} T. | {n} 天前 | hace {n} d | il y a {n} j | {n} g fa | {n} дн. назад |
| `header.on_branch` | on {branch} | auf {branch} | 位于 {branch} | en {branch} | sur {branch} | su {branch} | в ветке {branch} |
| `header.detached` | {commit} (detached) | {commit} (losgelöst) | {commit}（已分离） | {commit} (desacoplado) | {commit} (détachée) | {commit} (scollegato) | {commit} (отсоединен) |
| `btn.vscode` | Open in VS Code | In VS Code öffnen | 在 VS Code 中打开 | Abrir en VS Code | Ouvrir dans VS Code | Apri in VS Code | Открыть в VS Code |
| `btn.finder` | Finder | Finder | 访达 | Finder | Finder | Finder | Finder |
| `btn.go_root` | Go to Root | Zum Stammordner | 前往根目录 | Ir a la raíz | Aller à la racine | Vai alla radice | К корневой папке |
| `btn.close` | Close | Schließen | 关闭 | Cerrar | Fermer | Chiudi | Закрыть |
| `btn.cancel` | Cancel | Abbrechen | 取消 | Cancelar | Annuler | Annulla | Отменить |
| `btn.run` | Run | Ausführen | 运行 | Ejecutar | Exécuter | Esegui | Запустить |
| `panel.waiting` | Waiting for Manta… | Warte auf Manta … | 正在等待 Manta… | Esperando a Manta… | En attente de Manta… | In attesa di Manta… | Ожидание Manta… |
| `panel.archive` | Inside an archive | In einem Archiv | 位于归档内 | Dentro de un archivo comprimido | Dans une archive | All’interno di un archivio | Внутри архива |
| `panel.remote` | Remote folder | Entfernter Ordner | 远程文件夹 | Carpeta remota | Dossier distant | Cartella remota | Удаленная папка |
| `panel.no_project` | No project here | Kein Projekt hier | 此处没有项目 | No hay ningún proyecto aquí | Aucun projet ici | Nessun progetto qui | Здесь нет проекта |
| `picker.heading` | Run a script in {name} | Skript in {name} ausführen | 在 {name} 中运行脚本 | Ejecutar un script en {name} | Exécuter un script dans {name} | Esegui uno script in {name} | Запустить скрипт в {name} |
| `picker.runs_in` | Runs in the terminal, in {path} | Läuft im Terminal, in {path} | 在终端中运行，位置：{path} | Se ejecuta en el terminal, en {path} | S’exécute dans le terminal, dans {path} | Viene eseguito nel Terminale, in {path} | Запускается в Терминале, в {path} |
| `confirm.in` | in {path} | in {path} | 位于 {path} | en {path} | dans {path} | in {path} | в {path} |
| `what.vscode_missing` | VS Code (is it installed?) | VS Code (ist es installiert?) | VS Code（是否已安装？） | VS Code (¿está instalado?) | VS Code (est-il installé ?) | VS Code (è installato?) | VS Code (он установлен?) |
| `what.folder` | the folder | Der Ordner | 该文件夹 | la carpeta | le dossier | la cartella | папку |
| `problem.unreadable` | can't be read | kann nicht gelesen werden | 无法读取 | no se puede leer | illisible | non può essere letto | не удается прочитать |
| `tag.compose_services.one` | Compose · {count} service | Compose · {count} Dienst | Compose · {count} 个服务 | Compose · {count} servicio | Compose · {count} service | Compose · {count} servizio | Compose · {count} служба |
| `tag.compose_services.other` | Compose · {count} services | Compose · {count} Dienste | Compose · {count} 个服务 | Compose · {count} servicios | Compose · {count} services | Compose · {count} servizi | Compose · {count} службы |
| `tag.cargo_workspace` | Cargo workspace | Cargo-Workspace | Cargo 工作区 | Espacio de trabajo de Cargo | Espace de travail Cargo | Workspace Cargo | Рабочее пространство Cargo |
| `info.members.one` | {count} member | {count} Mitglied | {count} 个成员 | {count} miembro | {count} membre | {count} membro | {count} участник |
| `info.members.other` | {count} members | {count} Mitglieder | {count} 个成员 | {count} miembros | {count} membres | {count} membri | {count} участника |

## Plugin: Quick Actions

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | Quick Actions | Schnellaktionen | 快速操作 | Acciones rápidas | Actions rapides | Azioni rapide | Быстрые действия |
| `cmd.edit` | Edit Quick Actions… | Schnellaktionen bearbeiten … | 编辑快速操作… | Editar acciones rápidas… | Modifier les actions rapides… | Modifica azioni rapide… | Изменить быстрые действия… |
| `cmd.reload` | Reload Quick Actions | Schnellaktionen neu laden | 重新载入快速操作 | Volver a cargar acciones rápidas | Recharger les actions rapides | Ricarica azioni rapide | Перезагрузить быстрые действия |
| `cmd.panel` | Show Quick Actions Panel | Schnellaktionen-Panel einblenden | 显示快速操作面板 | Mostrar panel de acciones rápidas | Afficher le panneau Actions rapides | Mostra pannello Azioni rapide | Показать панель быстрых действий |
| `panel.actions` | Quick Actions | Schnellaktionen | 快速操作 | Acciones rápidas | Actions rapides | Azioni rapide | Быстрые действия |
| `panel.confirm` | Quick Action | Schnellaktion | 快速操作 | Acción rápida | Action rapide | Azione rapida | Быстрое действие |
| `load.ok.one` | Loaded {count} quick action | {count} Schnellaktion geladen | 已载入 {count} 个快速操作 | {count} acción rápida cargada | {count} action rapide chargée | {count} azione rapida caricata | Загружено {count} быстрое действие |
| `load.ok.other` | Loaded {count} quick actions | {count} Schnellaktionen geladen | 已载入 {count} 个快速操作 | {count} acciones rápidas cargadas | {count} actions rapides chargées | {count} azioni rapide caricate | Загружено {count} быстрых действия |
| `panel.waiting` | Waiting for Manta… | Warte auf Manta … | 正在等待 Manta… | Esperando a Manta… | En attente de Manta… | In attesa di Manta… | Ожидание Manta… |
| `panel.in_archive` | Inside an archive | In einem Archiv | 位于归档内 | Dentro de un archivo comprimido | Dans une archive | All’interno di un archivio | Внутри архива |
| `panel.remote` | Remote folder | Entfernter Ordner | 远程文件夹 | Carpeta remota | Dossier distant | Cartella remota | Удаленная папка |
| `panel.nothing_selected` | Nothing selected | Keine Auswahl | 未选择任何内容 | Ninguna selección | Aucune sélection | Nessuna selezione | Ничего не выбрано |
| `panel.in_folder` | in {path} | in {path} | 位于 {path} | en {path} | dans {path} | in {path} | в {path} |
| `panel.last_runs` | Last runs | Zuletzt ausgeführt | 最近运行 | Últimas ejecuciones | Dernières exécutions | Ultime esecuzioni | Последние запуски |
| `panel.cancel_named` | Cancel “{title}” | „{title}“ abbrechen | 取消“{title}” | Cancelar “{title}” | Annuler « {title} » | Annulla “{title}” | Отменить «{title}» |
| `panel.edit` | Edit Actions… | Aktionen bearbeiten … | 编辑操作… | Editar acciones… | Modifier les actions… | Modifica azioni… | Изменить действия… |
| `panel.reload` | Reload | Neu laden | 重新载入 | Volver a cargar | Recharger | Ricarica | Перезагрузить |
| `items.one` | {count} item | {count} Objekt | {count} 项 | {count} ítem | {count} élément | {count} elemento | {count} объект |
| `items.other` | {count} items | {count} Objekte | {count} 项 | {count} ítems | {count} éléments | {count} elementi | {count} объекта |
| `confirm.and_more` | {names} and {count} more | {names} und {count} weitere | {names} 以及其他 {count} 个 | {names} y {count} más | {names} et {count} autres | {names} e altri {count} | {names} и еще {count} |
| `confirm.run` | Run | Ausführen | 运行 | Ejecutar | Exécuter | Esegui | Запустить |
| `cancel` | Cancel | Abbrechen | 取消 | Cancelar | Annuler | Annulla | Отменить |
| `default.resize-50` | Resize Images to 50% | Bilder auf 50 % verkleinern | 将图像缩小到 50% | Reducir imágenes al 50 % | Réduire les images à 50 % | Ridimensiona immagini al 50% | Уменьшить изображения до 50 % |
| `default.resize-50.message.one` | Resized {count} image | {count} Bild verkleinert | 已调整 {count} 个图像的大小 | {count} imagen reducida | {count} image redimensionnée | {count} immagine ridimensionata | Уменьшено {count} изображение |
| `default.resize-50.message.other` | Resized {count} images | {count} Bilder verkleinert | 已调整 {count} 个图像的大小 | {count} imágenes reducidas | {count} images redimensionnées | {count} immagini ridimensionate | Уменьшено {count} изображения |
| `default.to-jpeg` | Convert to JPEG | In JPEG umwandeln | 转换为 JPEG | Convertir a JPEG | Convertir en JPEG | Converti in JPEG | Конвертировать в JPEG |
| `default.to-jpeg.message.one` | Converted {count} image to JPEG | {count} Bild in JPEG umgewandelt | 已将 {count} 个图像转换为 JPEG | {count} imagen convertida a JPEG | {count} image convertie en JPEG | {count} immagine convertita in JPEG | В JPEG конвертировано {count} изображение |
| `default.to-jpeg.message.other` | Converted {count} images to JPEG | {count} Bilder in JPEG umgewandelt | 已将 {count} 个图像转换为 JPEG | {count} imágenes convertidas a JPEG | {count} images converties en JPEG | {count} immagini convertite in JPEG | В JPEG конвертировано {count} изображения |
| `default.to-png` | Convert to PNG | In PNG umwandeln | 转换为 PNG | Convertir a PNG | Convertir en PNG | Converti in PNG | Конвертировать в PNG |
| `default.to-png.message.one` | Converted {count} image to PNG | {count} Bild in PNG umgewandelt | 已将 {count} 个图像转换为 PNG | {count} imagen convertida a PNG | {count} image convertie en PNG | {count} immagine convertita in PNG | В PNG конвертировано {count} изображение |
| `default.to-png.message.other` | Converted {count} images to PNG | {count} Bilder in PNG umgewandelt | 已将 {count} 个图像转换为 PNG | {count} imágenes convertidas a PNG | {count} images converties en PNG | {count} immagini convertite in PNG | В PNG конвертировано {count} изображения |
| `default.zip` | Compress to ZIP | Als ZIP komprimieren | 压缩为 ZIP | Comprimir en ZIP | Compresser en ZIP | Comprimi in ZIP | Сжать в ZIP |
| `default.zip-many` | Compress to ZIP | Als ZIP komprimieren | 压缩为 ZIP | Comprimir en ZIP | Compresser en ZIP | Comprimi in ZIP | Сжать в ZIP |
| `default.extract` | Extract Archive | Archiv entpacken | 解压缩归档 | Extraer archivo comprimido | Extraire l’archive | Estrai archivio | Извлечь архив |
| `default.extract.message.one` | Extracted {count} archive | {count} Archiv entpackt | 已解压缩 {count} 个归档 | {count} archivo comprimido extraído | {count} archive extraite | {count} archivio estratto | Извлечен {count} архив |
| `default.extract.message.other` | Extracted {count} archives | {count} Archive entpackt | 已解压缩 {count} 个归档 | {count} archivos comprimidos extraídos | {count} archives extraites | {count} archivi estratti | Извлечено {count} архива |
| `default.vscode` | Open in VS Code | In VS Code öffnen | 在 VS Code 中打开 | Abrir en VS Code | Ouvrir dans VS Code | Apri in VS Code | Открыть в VS Code |
| `default.terminal-app` | Open in Terminal.app | In Terminal.app öffnen | 在 Terminal.app 中打开 | Abrir en Terminal.app | Ouvrir dans Terminal.app | Apri in Terminal.app | Открыть в Terminal.app |
| `default.copy-path` | Copy Path | Pfad kopieren | 拷贝路径 | Copiar ruta | Copier le chemin | Copia percorso | Скопировать путь |
| `default.copy-path.message.one` | Copied {count} path | {count} Pfad kopiert | 已拷贝 {count} 个路径 | {count} ruta copiada | {count} chemin copié | {count} percorso copiato | Скопирован {count} путь |
| `default.copy-path.message.other` | Copied {count} paths | {count} Pfade kopiert | 已拷贝 {count} 个路径 | {count} rutas copiadas | {count} chemins copiés | {count} percorsi copiati | Скопировано {count} пути |
| `default.reveal` | Reveal in Finder | Im Finder zeigen | 在访达中显示 | Mostrar en Finder | Afficher dans le Finder | Mostra nel Finder | Показать в Finder |
| `default.chmod-x` | Make Executable | Ausführbar machen | 设为可执行 | Hacer ejecutable | Rendre exécutable | Rendi eseguibile | Сделать исполняемым |
| `default.chmod-x.message.one` | Made {count} file executable | {count} Datei ausführbar gemacht | 已将 {count} 个文件设为可执行 | {count} archivo convertido en ejecutable | {count} fichier rendu exécutable | {count} file reso eseguibile | {count} файл сделан исполняемым |
| `default.chmod-x.message.other` | Made {count} files executable | {count} Dateien ausführbar gemacht | 已将 {count} 个文件设为可执行 | {count} archivos convertidos en ejecutables | {count} fichiers rendus exécutables | {count} file resi eseguibili | {count} файла сделаны исполняемыми |
| `default.count-lines` | Count Lines | Zeilen zählen | 统计行数 | Contar líneas | Compter les lignes | Conta righe | Подсчитать строки |
| `default.sha256` | Calculate SHA-256 | SHA-256 berechnen | 计算 SHA-256 | Calcular SHA-256 | Calculer le SHA-256 | Calcola SHA-256 | Рассчитать SHA-256 |
| `default.rsync-other` | Copy to Other Pane (rsync) | Per rsync in anderen Bereich | 拷贝到另一个面板（rsync） | Copiar al otro panel (rsync) | Copier dans l’autre sous-fenêtre (rsync) | Copia nell’altro pannello (rsync) | Копировать в другую панель (rsync) |

## Plugin: Batch Rename

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `info.name` | Batch Rename | Stapelweise umbenennen | 批量重新命名 | Renombrar en lote | Renommage par lot | Rinomina in blocco | Пакетное переименование |
| `cmd.batch` | Batch Rename… | Stapelweise umbenennen … | 批量重新命名… | Renombrar en lote… | Renommer par lot… | Rinomina in blocco… | Пакетное переименование… |
| `cmd.undo` | Undo Batch Rename | Stapelweises Umbenennen widerrufen | 撤销批量重新命名 | Deshacer renombrado en lote | Annuler le renommage par lot | Annulla rinomina in blocco | Отменить пакетное переименование |
| `panel.title` | Batch Rename | Stapelweise umbenennen | 批量重新命名 | Renombrar en lote | Renommage par lot | Rinomina in blocco | Пакетное переименование |
| `banner.nothing_renamed` | Nothing was renamed | Es wurde nichts umbenannt | 未重新命名任何项目 | No se ha renombrado nada | Aucun élément n’a été renommé | Non è stato rinominato nulla | Ничего не переименовано |
| `rule.replace` | Find & Replace | Suchen & Ersetzen | 查找与替换 | Buscar y reemplazar | Rechercher et remplacer | Trova e sostituisci | Найти и заменить |
| `rule.insert` | Insert Text | Text einfügen | 插入文本 | Insertar texto | Insérer du texte | Inserisci testo | Вставить текст |
| `rule.remove` | Remove Characters | Zeichen entfernen | 移除字符 | Eliminar caracteres | Supprimer des caractères | Rimuovi caratteri | Удалить символы |
| `rule.case` | Change Case | Groß-/Kleinschreibung | 更改大小写 | Mayúsculas/minúsculas | Modifier la casse | Maiuscole/minuscole | Изменить регистр |
| `rule.number` | Numbering | Nummerierung | 编号 | Numeración | Numérotation | Numerazione | Нумерация |
| `rule.date` | Date | Datum | 日期 | Fecha | Date | Data | Дата |
| `rule.extension` | Extension | Suffix | 扩展名 | Extensión | Extension | Estensione | Расширение |
| `rule.clean` | Clean Up | Aufräumen | 整理 | Limpiar | Nettoyer | Pulizia | Очистка |
| `issue.empty` | Empty name | Leerer Name | 名称为空 | Nombre vacío | Nom vide | Nome vuoto | Пустое имя |
| `issue.contains` | Contains “{char}” | Enthält „{char}“ | 包含“{char}” | Contiene “{char}” | Contient « {char} » | Contiene “{char}” | Содержит «{char}» |
| `issue.invalid_char` | Invalid character | Ungültiges Zeichen | 无效字符 | Carácter no válido | Caractère non valide | Carattere non valido | Недопустимый символ |
| `issue.too_long` | Name too long | Name zu lang | 名称过长 | Nombre demasiado largo | Nom trop long | Nome troppo lungo | Слишком длинное имя |
| `issue.duplicate` | Duplicate name | Doppelter Name | 名称重复 | Nombre duplicado | Nom en double | Nome duplicato | Повторяющееся имя |
| `issue.exists` | Already exists | Existiert bereits | 已存在 | Ya existe | Existe déjà | Esiste già | Уже существует |
| `scope.selected.one` | {count} selected item | {count} ausgewähltes Objekt | {count} 个所选项目 | {count} ítem seleccionado | {count} élément sélectionné | {count} elemento selezionato | {count} выбранный объект |
| `scope.selected.other` | {count} selected items | {count} ausgewählte Objekte | {count} 个所选项目 | {count} ítems seleccionados | {count} éléments sélectionnés | {count} elementi selezionati | {count} выбранных объекта |
| `scope.in` | in “{name}” | in „{name}“ | 位于“{name}” | en “{name}” | dans « {name} » | in “{name}” | в «{name}» |
| `scope.nothing_selected` | Nothing selected | Keine Auswahl | 未选择任何内容 | Ninguna selección | Aucune sélection | Nessuna selezione | Ничего не выбрано |
| `scope.all_files.one` | renaming the only file in “{name}” | die einzige Datei in „{name}“ wird umbenannt | 将重新命名“{name}”中唯一的文件 | se renombrará el único archivo de “{name}” | renommage de {count} fichier dans « {name} » | verrà rinominato l’unico file in “{name}” | переименовывается {count} файл в «{name}» |
| `scope.all_files.other` | renaming all {count} files in “{name}” | alle {count} Dateien in „{name}“ werden umbenannt | 将重新命名“{name}”中的全部 {count} 个文件 | se renombrarán los {count} archivos de “{name}” | renommage des {count} fichiers dans « {name} » | verranno rinominati tutti i {count} file in “{name}” | переименовываются все {count} файла в «{name}» |
| `scope.remote` | on {remote} | auf {remote} | 位于 {remote} | en {remote} | sur {remote} | su {remote} | на {remote} |
| `presets.button` | Presets | Vorlagen | 预设 | Preajustes | Préréglages | Preimpostazioni | Шаблоны |
| `presets.saved` | SAVED RULE SETS | GESICHERTE REGELSÄTZE | 已存储的规则集 | CONJUNTOS DE REGLAS GUARDADOS | ENSEMBLES DE RÈGLES ENREGISTRÉS | SET DI REGOLE SALVATI | СОХРАНЕННЫЕ НАБОРЫ ПРАВИЛ |
| `presets.none` | No presets yet | Noch keine Vorlagen | 尚无预设 | Aún no hay preajustes | Aucun préréglage pour l’instant | Ancora nessuna preimpostazione | Шаблонов пока нет |
| `presets.rules.one` | {count} rule | {count} Regel | {count} 条规则 | {count} regla | {count} règle | {count} regola | {count} правило |
| `presets.rules.other` | {count} rules | {count} Regeln | {count} 条规则 | {count} reglas | {count} règles | {count} regole | {count} правила |
| `presets.delete` | Delete preset | Vorlage löschen | 删除预设 | Eliminar preajuste | Supprimer le préréglage | Elimina preimpostazione | Удалить шаблон |
| `presets.load` | Load these rules | Diese Regeln laden | 载入这些规则 | Cargar estas reglas | Charger ces règles | Carica queste regole | Загрузить эти правила |
| `presets.save_current` | SAVE CURRENT RULES | AKTUELLE REGELN SICHERN | 存储当前规则 | GUARDAR REGLAS ACTUALES | ENREGISTRER LES RÈGLES ACTUELLES | SALVA LE REGOLE ATTUALI | СОХРАНИТЬ ТЕКУЩИЕ ПРАВИЛА |
| `presets.save` | Save | Sichern | 存储 | Guardar | Enregistrer | Salva | Сохранить |
| `presets.replaces` | Replaces the preset with this name | Ersetzt die Vorlage mit diesem Namen | 将替换同名预设 | Reemplaza el preajuste con este nombre | Remplace le préréglage portant ce nom | Sostituisce la preimpostazione con questo nome | Заменит шаблон с таким именем |
| `rules.title` | Rules | Regeln | 规则 | Reglas | Règles | Regole | Правила |
| `rules.order` | applied top to bottom · {n} | von oben nach unten · {n} | 从上到下应用 · {n} | se aplican de arriba abajo · {n} | appliquées de haut en bas · {n} | applicate dall’alto in basso · {n} | применяются сверху вниз · {n} |
| `rules.add` | Add Rule | Regel hinzufügen | 添加规则 | Añadir regla | Ajouter une règle | Aggiungi regola | Добавить правило |
| `rules.empty` | No rules yet | Noch keine Regeln | 尚无规则 | Aún no hay reglas | Aucune règle pour l’instant | Ancora nessuna regola | Правил пока нет |
| `card.remove` | Remove rule | Regel entfernen | 移除规则 | Eliminar regla | Supprimer la règle | Rimuovi regola | Удалить правило |
| `card.move_down` | Move down | Nach unten | 下移 | Bajar | Descendre | Sposta giù | Переместить вниз |
| `card.move_up` | Move up | Nach oben | 上移 | Subir | Monter | Sposta su | Переместить вверх |
| `card.on` | Rule is on | Regel ist an | 规则已打开 | La regla está activada | La règle est activée | Regola attiva | Правило включено |
| `card.off` | Rule is off | Regel ist aus | 规则已关闭 | La regla está desactivada | La règle est désactivée | Regola disattivata | Правило выключено |
| `field.apply_to` | Apply to | Gilt für | 应用于 | Aplicar a | Appliquer à | Applica a | Применить к |
| `target.name` | Name | Name | 名称 | Nombre | Nom | Nome | Имя |
| `target.extension` | Extension | Suffix | 扩展名 | Extensión | Extension | Estensione | Расширение |
| `target.both` | Both | Beides | 两者 | Ambos | Les deux | Entrambi | Оба |
| `field.find` | Find | Suchen | 查找 | Buscar | Rechercher | Trova | Найти |
| `field.replace` | Replace | Ersetzen | 替换 | Reemplazar | Remplacer | Sostituisci | Заменить |
| `opt.regex` | Regex | Regex | 正则表达式 | Expr. regular | Regex | Regex | Рег. выражение |
| `opt.match_case` | Match case | Groß/klein beachten | 区分大小写 | Distinguir mayúsculas | Respecter la casse | Maiuscole/minuscole | С учетом регистра |
| `occurrence.first` | First | Erstes | 第一个 | Primera | Première | Prima | Первое |
| `occurrence.all` | All | Alle | 全部 | Todas | Toutes | Tutte | Все |
| `field.text` | Text | Text | 文本 | Texto | Texte | Testo | Текст |
| `field.at` | At | Stelle | 位置 | Posición | Emplacement | Dove | Место |
| `at.start` | Start | Anfang | 开头 | Principio | Début | Inizio | Начало |
| `at.end` | End | Ende | 结尾 | Final | Fin | Fine | Конец |
| `at.position` | Position | Position | 指定位置 | Posición | Position | Posizione | Позиция |
| `field.remove` | Remove | Entfernen | 移除 | Eliminar | Supprimer | Rimuovi | Удалить |
| `remove.first` | First | Erste | 开头 | Primeros | Premiers | Primi | Первые |
| `remove.last` | Last | Letzte | 末尾 | Últimos | Derniers | Ultimi | Последние |
| `remove.range` | Range | Bereich | 范围 | Rango | Plage | Intervallo | Диапазон |
| `field.count` | Count | Anzahl | 数量 | Cantidad | Nombre | Quantità | Количество |
| `caption.characters` | characters | Zeichen | 个字符 | caracteres | caractères | caratteri | символов |
| `caption.from` | from | ab | 起始于 | desde | à partir de | da | с |
| `field.case` | Case | Schreibung | 大小写 | Formato | Casse | Maiuscole | Регистр |
| `case.lower` | lower | klein | 小写 | minúsculas | minuscules | minuscole | строчные |
| `case.upper` | UPPER | GROSS | 大写 | MAYÚSCULAS | MAJUSCULES | MAIUSCOLE | ПРОПИСНЫЕ |
| `case.title` | Title | Titel | 标题 | Título | Titre | Titolo | Каждое Слово |
| `case.sentence` | Sentence | Satz | 句子 | Frase | Phrase | Frase | Как в предложении |
| `field.start` | Start | Beginn | 起始 | Inicio | Début | Inizio | Начать с |
| `caption.step` | Step | Schritt | 步长 | Incremento | Pas | Passo | Шаг |
| `caption.digits` | Digits | Stellen | 位数 | Dígitos | Chiffres | Cifre | Цифр |
| `field.place` | Place | Position | 位置 | Ubicación | Position | Posizione | Место |
| `place.prefix` | Prefix | Davor | 前缀 | Prefijo | Préfixe | Prefisso | Перед |
| `place.suffix` | Suffix | Dahinter | 后缀 | Sufijo | Suffixe | Suffisso | После |
| `place.replace` | Replace name | Name ersetzen | 替换名称 | Reemplazar nombre | Remplacer le nom | Sostituisci nome | Заменить имя |
| `field.separator` | Separator | Trenner | 分隔符 | Separador | Séparateur | Separatore | Разделитель |
| `field.date` | Date | Datum | 日期 | Fecha | Date | Data | Дата |
| `date.modified` | Modified | Geändert | 修改日期 | Modificación | Modification | Modifica | Изменения |
| `date.today` | Today | Heute | 今天 | Hoy | Aujourd’hui | Oggi | Сегодня |
| `field.format` | Format | Format | 格式 | Formato | Format | Formato | Формат |
| `date.presets` | Presets | Vorlagen | 预设 | Preajustes | Préréglages | Preimpostazioni | Шаблоны |
| `field.extension` | Extension | Suffix | 扩展名 | Extensión | Extension | Estensione | Расширение |
| `ext.keep` | Keep | Behalten | 保留 | Conservar | Conserver | Mantieni | Оставить |
| `ext.change` | Change | Ändern | 更改 | Cambiar | Modifier | Modifica | Изменить |
| `ext.remove` | Remove | Entfernen | 移除 | Eliminar | Supprimer | Rimuovi | Удалить |
| `field.new` | New | Neu | 新扩展名 | Nueva | Nouvelle | Nuova | Новое |
| `clean.trim` | Trim | Trimmen | 去除首尾空格 | Recortar espacios | Supprimer les espaces aux extrémités | Rimuovi spazi ai bordi | Обрезать пробелы |
| `clean.collapse` | Collapse spaces | Leerzeichen zusammenfassen | 合并连续空格 | Unificar espacios | Fusionner les espaces | Comprimi spazi | Объединять пробелы |
| `field.spaces` | Spaces | Leerzeichen | 空格 | Espacios | Espaces | Spazi | Пробелы |
| `spaces.keep` | Keep | Behalten | 保留 | Conservar | Conserver | Mantieni | Оставить |
| `clean.umlauts` | Transliterate umlauts (ä → ae, é → e) | Umlaute umschreiben (ä → ae, é → e) | 转写变音字母（ä → ae，é → e） | Transliterar diéresis y acentos (ä → ae, é → e) | Translittérer les accents (ä → ae, é → e) | Translittera caratteri accentati (ä → ae, é → e) | Транслитерировать умлауты (ä → ae, é → e) |
| `preview.title` | Preview | Vorschau | 预览 | Vista previa | Aperçu | Anteprima | Просмотр |
| `preview.count.one` | {count} item in pane order | {count} Objekt in Listenreihenfolge | {count} 项，按面板顺序 | {count} ítem en el orden del panel | {count} élément dans l’ordre de la sous-fenêtre | {count} elemento nell’ordine del pannello | {count} объект в порядке панели |
| `preview.count.other` | {count} items in pane order | {count} Objekte in Listenreihenfolge | {count} 项，按面板顺序 | {count} ítems en el orden del panel | {count} éléments dans l’ordre de la sous-fenêtre | {count} elementi nell’ordine del pannello | {count} объекта в порядке панели |
| `preview.original` | Original name | Bisheriger Name | 原名称 | Nombre original | Nom d’origine | Nome originale | Исходное имя |
| `preview.new` | New name | Neuer Name | 新名称 | Nombre nuevo | Nouveau nom | Nuovo nome | Новое имя |
| `preview.reading` | Reading the folder… | Ordner wird gelesen … | 正在读取文件夹… | Leyendo la carpeta… | Lecture du dossier… | Lettura della cartella… | Чтение папки… |
| `preview.no_files` | No files to rename | Keine Dateien zum Umbenennen | 没有要重新命名的文件 | No hay archivos que renombrar | Aucun fichier à renommer | Nessun file da rinominare | Нет файлов для переименования |
| `preview.empty_name` | (empty) | (leer) | （空） | (vacío) | (vide) | (vuoto) | (пусто) |
| `footer.no_change` | No names change yet | Noch ändert sich kein Name | 尚无名称更改 | Aún no cambia ningún nombre | Aucun nom ne change pour l’instant | Ancora nessun nome cambia | Имена пока не меняются |
| `footer.files.one` | {count} file will be renamed | {count} Datei wird umbenannt | 将重新命名 {count} 个文件 | Se renombrará {count} archivo | {count} fichier sera renommé | Verrà rinominato {count} file | Будет переименован {count} файл |
| `footer.files.other` | {count} files will be renamed | {count} Dateien werden umbenannt | 将重新命名 {count} 个文件 | Se renombrarán {count} archivos | {count} fichiers seront renommés | Verranno rinominati {count} file | Будут переименованы {count} файла |
| `footer.items.one` | {count} item will be renamed | {count} Objekt wird umbenannt | 将重新命名 {count} 项 | Se renombrará {count} ítem | {count} élément sera renommé | Verrà rinominato {count} elemento | Будет переименован {count} объект |
| `footer.items.other` | {count} items will be renamed | {count} Objekte werden umbenannt | 将重新命名 {count} 项 | Se renombrarán {count} ítems | {count} éléments seront renommés | Verranno rinominati {count} elementi | Будут переименованы {count} объекта |
| `footer.conflicts.one` | {count} conflict | {count} Konflikt | {count} 个冲突 | {count} conflicto | {count} conflit | {count} conflitto | {count} конфликт |
| `footer.conflicts.other` | {count} conflicts | {count} Konflikte | {count} 个冲突 | {count} conflictos | {count} conflits | {count} conflitti | {count} конфликта |
| `footer.unchanged` | {n} unchanged | {n} unverändert | {n} 个未更改 | {n} sin cambios | {n} inchangé(s) | {n} invariati | без изменений: {n} |
| `btn.rename` | Rename | Umbenennen | 重新命名 | Renombrar | Renommer | Rinomina | Переименовать |
| `btn.renaming` | Renaming… | Wird umbenannt … | 正在重新命名… | Renombrando… | Renommage… | Rinomina in corso… | Переименование… |
| `btn.resolve_first` | Resolve the conflicts first | Löse zuerst die Konflikte | 请先解决冲突 | Resuelve primero los conflictos | Résolvez d’abord les conflits | Risolvi prima i conflitti | Сначала разрешите конфликты |
| `btn.cancel` | Cancel | Abbrechen | 取消 | Cancelar | Annuler | Annulla | Отменить |

## Plugin: System Load

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `plugin.name` | System Load | Systemauslastung | 系统负载 | Carga del sistema | Charge du système | Carico di sistema | Загрузка системы |
| `settings.show_gpu` | Show GPU load | GPU-Last anzeigen | 显示 GPU 负载 | Mostrar la carga de la GPU | Afficher la charge de la carte graphique | Mostra il carico della GPU | Показывать загрузку ГП |
| `label.cpu` | CPU | CPU | CPU | CPU | CPU | CPU | ЦП |
| `label.gpu` | GPU | GPU | GPU | GPU | GPU | GPU | ГП |
| `label.ram` | RAM | RAM | 内存 | RAM | RAM | RAM | Память |
| `pressure.normal` | normal | normal | 正常 | normal | normale | normale | нормальная |
| `pressure.warning` | elevated | erhöht | 偏高 | elevada | élevée | elevata | повышенная |
| `pressure.critical` | critical | kritisch | 严重 | crítica | critique | critica | критическая |

## Plugin: Terminal

| key | en | de | zh-Hans | es | fr | it | ru |
|---|---|---|---|---|---|---|---|
| `term.info.name` | Terminal | Terminal | 终端 | Terminal | Terminal | Terminale | Терминал |
| `term.panel.title` | Terminal | Terminal | 终端 | Terminal | Terminal | Terminale | Терминал |
| `term.cmd.toggle` | Toggle Terminal | Terminal ein-/ausblenden | 显示/隐藏终端 | Mostrar/ocultar terminal | Afficher/masquer le Terminal | Mostra/nascondi Terminale | Показать/скрыть Терминал |
| `term.cmd.new_session` | New Terminal Session | Neue Terminal-Sitzung | 新建终端会话 | Nueva sesión de terminal | Nouvelle session Terminal | Nuova sessione Terminale | Новый сеанс Терминала |
| `term.cmd.clear` | Clear Terminal | Terminal leeren | 清除终端 | Borrar terminal | Effacer le Terminal | Pulisci Terminale | Очистить Терминал |
| `term.remote.connecting` | Opening a shell in {target} … | Öffne eine Shell in {target} … | 正在 {target} 中打开 Shell… | Abriendo una shell en {target}… | Ouverture d’un shell dans {target}… | Apertura di una shell in {target}… | Открытие оболочки в {target}… |
| `term.remote.stopped` | Container “{name}” is stopped | Container „{name}“ ist gestoppt | 容器“{name}”已停止 | El contenedor “{name}” está detenido | Le conteneur « {name} » est arrêté | Il container “{name}” è arrestato | Контейнер «{name}» остановлен |
| `term.remote.no_cli` | {program} is not installed | {program} ist nicht installiert | 未安装 {program} | {program} no está instalado | {program} n’est pas installé | {program} non è installato | {program} не установлен |
| `term.remote.failed` | Could not open a shell in {target} | In {target} konnte keine Shell geöffnet werden | 无法在 {target} 中打开 Shell | No se ha podido abrir una shell en {target} | Impossible d’ouvrir un shell dans {target} | Impossibile aprire una shell in {target} | Не удалось открыть оболочку в {target} |
| `term.pill.run` | Will run “{command}” when idle | Führt „{command}“ aus, sobald die Shell bereit ist | Shell 空闲时将运行“{command}” | Ejecutará “{command}” cuando la shell esté lista | Exécutera « {command} » dès que le shell sera prêt | Eseguirà “{command}” quando la shell è pronta | Выполнит «{command}», когда оболочка освободится |
| `term.pill.cd` | Will cd to “{name}” when idle | Wechselt nach „{name}“, sobald die Shell bereit ist | Shell 空闲时将 cd 到“{name}” | Cambiará a “{name}” cuando la shell esté lista | Ira dans « {name} » dès que le shell sera prêt | Passerà a “{name}” quando la shell è pronta | Перейдет в «{name}», когда оболочка освободится |
| `term.pill.show` | Will show {actions} when idle | Zeigt {actions}, sobald die Shell bereit ist | Shell 空闲时将显示 {actions} | Mostrará {actions} cuando la shell esté lista | Affichera {actions} dès que le shell sera prêt | Mostrerà {actions} quando la shell è pronta | Покажет {actions}, когда оболочка освободится |
| `term.pill.actions.one` | {count} file action | {count} Dateiaktion | {count} 个文件操作 | {count} acción de archivo | {count} action sur fichier | {count} azione sui file | {count} действие с файлами |
| `term.pill.actions.other` | {count} file actions | {count} Dateiaktionen | {count} 个文件操作 | {count} acciones de archivo | {count} actions sur fichier | {count} azioni sui file | {count} действия с файлами |
| `term.settings.shell` | Shell | Shell | Shell | Shell | Shell | Shell | Оболочка |
| `term.settings.font_size` | Font size | Schriftgröße | 字体大小 | Tamaño de letra | Taille de police | Dimensione font | Размер шрифта |
| `term.settings.scrollback` | Scrollback | Verlauf | 回滚 | Historial | Historique | Cronologia | Буфер прокрутки |
| `term.settings.lines_suffix` |  lines |  Zeilen |  行 |  líneas |  lignes |  righe |  строк |
| `term.settings.option_meta` | Use Option as Meta key | Wahltaste als Meta-Taste verwenden | 将 Option 键用作 Meta 键 | Usar Opción como tecla Meta | Utiliser Option comme touche Méta | Usa Opzione come tasto Meta | Использовать Option как клавишу Meta |
| `term.settings.follow` | Pane follows terminal cd | Bereich folgt „cd“ im Terminal | 面板跟随终端 cd | El panel sigue a cd del terminal | La sous-fenêtre suit « cd » du terminal | Il pannello segue “cd” nel terminale | Панель следует за cd в Терминале |
| `term.settings.show_actions` | Show file actions in terminal | Dateiaktionen im Terminal zeigen | 在终端中显示文件操作 | Mostrar acciones de archivo en el terminal | Afficher les actions sur fichier dans le terminal | Mostra azioni sui file nel terminale | Показывать действия с файлами в Терминале |
| `term.more` | … (+{n} more) | … (+{n} weitere) | …（另有 {n} 个） | … (+{n} más) | … (+{n} autres) | … (+{n} altri) | … (еще +{n}) |
| `term.verb.copy` | copy | kopieren | 拷贝 | copiar | copier | copia | копировать |
| `term.verb.move` | move | bewegen | 移动 | mover | déplacer | sposta | переместить |
| `term.verb.delete` | delete | löschen | 删除 | eliminar | supprimer | elimina | удалить |
| `term.verb.trash` | move to Trash | in den Papierkorb legen | 移到废纸篓 | trasladar a la papelera | placer dans la corbeille | sposta nel Cestino | переместить в Корзину |
| `term.verb.rename` | rename | umbenennen | 重新命名 | renombrar | renommer | rinomina | переименовать |
| `term.verb.mkdir` | mkdir | neuer Ordner | 新建文件夹 | nueva carpeta | nouveau dossier | nuova cartella | новая папка |
| `term.verb.touch` | touch | neue Datei | 新建文件 | nuevo archivo | nouveau fichier | nuovo file | новый файл |
| `term.verb.open` | open | öffnen | 打开 | abrir | ouvrir | apri | открыть |
| `term.verb.connect` | connect | verbinden | 连接 | conectar | se connecter | connetti | подключить |
| `term.verb.disconnect` | disconnect | trennen | 断开连接 | desconectar | se déconnecter | disconnetti | отключить |
| `term.verb.sync` | sync | synchronisieren | 同步 | sincronizar | synchroniser | sincronizza | синхронизировать |
| `term.verb.compress` | compress | komprimieren | 压缩 | comprimir | compresser | comprimi | сжать |
| `term.verb.extract` | extract | entpacken | 解压缩 | extraer | extraire | estrai | извлечь |
