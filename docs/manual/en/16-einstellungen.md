# Settings {#einstellungen}

Every one of Manta's settings, section by section. All changes apply immediately; you never need to restart.

## Open Settings {#oeffnen}

- **⌘,**
- **Manta** → **Settings…** in the menu bar
- the gear in the toolbar
- **Settings…** in the command palette

Settings is a window of its own with the sections **General** and **Plugins**. Close it with the red window button or **⌘W**. **⌘,** brings it back to the front.

While Settings is open, the main window waits: it is slightly dimmed, and clicks and shortcuts there do nothing – a click brings Settings back to the front with the alert sound. Close Settings to continue. The Help window, by contrast, can stay open while you work.

![Settings → General](images/settings-general.png)

## General {#allgemein}

### Appearance {#erscheinungsbild}

| Setting | Options |
|---|---|
| **Appearance** | **Auto** (follows the system's light and dark mode), **Light**, **Dark**. Also in the command palette: "Appearance: Light/Dark/Auto (System)". |
| **Accent colour** | **Blue**, **Purple**, **Pink**, **Red**, **Orange**, **Yellow**, **Green**, **Graphite** – for selection, buttons and focus. Every colour stays readable in light and dark. |

### Language {#sprache}

**Language**: **System** (the default) or one of the app's languages (**Deutsch**, **English**, **简体中文**, **Español**, **Français**, **Italiano**, **Русский**), each shown in its own name. The switch applies immediately. "System" follows your preferred languages in macOS (or English if Manta speaks none of them). Dates ("Today 14:02"), sizes ("1.2 MB") and numbers follow the language. The built-in plugins switch along; file names, shell output and your own Quick Actions stay as they are.

### Default file manager {#standard-dateimanager}

**Use Manta as default file manager**: when it's on, "Show in Finder" and folders from other apps open in Manta. Below it you see which app has the role now ("Currently: …") and which app gets it back when you turn it off. What macOS allows is explained in [Getting Started](01-erste-schritte.md#standard-dateimanager).

The same switch is in the **Manta** menu (**Manta as Default File Manager**, with a checkmark when it's on) and in the command palette (**Use Manta as Default File Manager…**, or **Give the Default File Manager Back to …** when Manta has it). Switching on there asks the same question as at the first launch, with **Cancel**; switching off gives the role back right away.

Only available when Manta runs from the **Applications** folder ("Move Manta to the Applications folder first").

### Files {#dateien}

| Setting | Meaning |
|---|---|
| **Show hidden files** | For new tabs and at launch. **⇧⌘.** shows or hides them in the current tab. |
| **Open archives as folders** | Return or double-click browses ZIP and TAR archives in the pane (read-only). **⌥↩** opens them with the default app. See [Archives](10-archive.md). |
| **New tabs open in** | Where **⌘T**, **New Tab** in the Dock menu and a start without a saved session open: **Same folder** (default; your home folder at launch), **Home**, **Desktop**, **Documents**, **Downloads** or **Other…** (a folder of your choice). Like Finder's "New Finder windows show". |
| **Remember view per folder** | Each folder opens in the view it was last shown in (Icons, List, Columns, Gallery). Off (default): the view belongs to the pane. See [Views](03-ansichten.md#pro-ordner). |
| **Mouse side buttons** | The thumb buttons go back and forward in the pane under the pointer, like **⌘[** and **⌘]**, in Help too. **Back / Forward** (default), **Forward / Back (swapped)** or **Off**. |
| **Load thumbnails for server files** | Downloads small images (under 5 MB) in the background. Off (default): server files show their type icon. See [Thumbnails](03-ansichten.md#server-vorschaubilder). |
| **File icons from macOS** | Files show the icon macOS has for their type, apps their own – like in Finder, on servers too. Off: Manta's symbols. Default: on. |

### Preview {#vorschau}

**Space opens**: **Quick Look (macOS)** or **Built-in preview**. Files on servers and in archives always use the built-in preview. See [Quick Look & Preview](06-schnellansicht.md).

### Copy and move {#kopieren-bewegen}

**When you drop files**: **Ask (Copy preselected)** (default), **Always copy** or **Always move**. Applies to dragging between panes and from other apps. "Ask" shows the dialog with Copy / Move; the two "Always" settings start right away. **⌘** while dropping moves, **⌥** copies, whatever is set. Dropping onto the Trash and Favorites is not affected. See [Drag and drop](05-dateien.md#drag-drop).

**If an item already exists**: **Ask** (the default), **Skip**, **Replace if newer**, **Replace** or **Keep both**. This is preselected when you copy or move (also when dropping); you can still change it each time. When a drop starts without a dialog ("Always copy" / "Always move"), this setting applies. With **Ask**, a job stops at each item that already exists and asks you. New installs ask; if you used Manta before, your stored setting stays (including the former default **Skip**) – switch it here. See [Name conflicts](05-dateien.md#konflikte).

### Cloud {#cloud}

**Google Drive**: Shows whether Google Drive for desktop is set up, otherwise the download link. **Hint in the Sidebar** turns the "Connect Google Drive…" row on or off. **Expert: Your Own Google OAuth Client** is for advanced users only, see [Google Drive for experts](09-cloud.md#experte).

### Updates {#updates}

| Setting | Meaning |
|---|---|
| **Check for updates at launch** | Quietly, in the background. Nothing is installed without asking you. Default: on. |
| **Channel** | **Stable** or **Beta**. Beta brings new features earlier but may contain bugs. |
| **Check Now** | Checks right away. Next to it: "Last checked: …". |
| **Skipped: Manta …** / **Reset** | Appears when you've skipped a version. This version and older ones aren't offered until you click **Reset**. |

If your build lacks the update signing key, this section says "Updates aren't set up in this build". See [Updates](17-updates.md).

### About Manta {#ueber}

Manta is free and open source. **Website & Support** → **manta-filemanager.com** opens [the website](https://manta-filemanager.com/) with news, help and contact. **Third-Party Licences** → **Show** opens the licence notices of the open-source components Manta is built on. **Source Code** → **GitHub** opens [the repository](https://github.com/web-workers-com/manta); contributions are welcome. **MIT Licence** → **Read** shows the licence: you may use, change and share Manta freely. The website, the source code and the third-party licences are also in the **Help** menu. **Feedback** → **Report an Issue** opens GitHub's bug report form with the Manta and macOS versions filled in, see [Report a bug](19-fehlerbehebung.md#fehler-melden).

## Plugins {#plugins}

At the top: **Install Plugin…**, **Open Plugins Folder** and **Reload**. Below are three sections:

- **Built-in** – the built-in plugins (Git, Terminal, Folder Sizes, Batch Rename, Markdown Preview, Quick Actions, Project Info, Image Tools, Duplicates, System Load).
- **Installed** – external plugins you've installed.
- **Available** – plugins that come with Manta, ready to install.

Each card shows the name, description, status (**Running**, **Starting…**, **Off**, **Error**) and the buttons **Turn On**/**Turn Off**, **Settings** (where available) and **Show Log**. More in [Your Own Plugins](15-eigene-plugins.md).

### Plugin settings {#plugin-einstellungen}

| Plugin | Settings |
|---|---|
| **Terminal** | Shell, Font size, Scrollback, Use Option as Meta key, Pane follows terminal cd, Show file actions in terminal – see [Terminal](11-terminal.md#einstellungen). |
| **Folder Sizes** | **Calculate folder sizes automatically** or **Only on demand** – see [Folder Sizes](14-plugins.md#ordnergroessen). |
| **Markdown Preview** | **Load images from the web (http/https)** – off by default. |
| **Duplicates** | **Skip packages (.app, .photoslibrary, …)**, **Include hidden files**, **Edit settings.json …** for exclusions and a custom trash command. |
| **System Load** | **Show GPU load** – see [System Load](14-plugins.md#systemauslastung). |

## Accessibility {#bedienungshilfen}

Manta follows **System Settings › Accessibility** – no switches of its own:

- **VoiceOver**: every button, field, list and dialog has a name in your language. File rows are read in full, e.g. "README.md, Text, 214 bytes, modified Today 09:41, selected" – including cloud and Git status. As you move with the arrow keys, VoiceOver reads the current row. Messages such as "3 items copied" and errors are announced.
- **Reduce motion** (Display): no animations – the flash of a favourite, the pulsing upload badge and fades are off.
- **Increase contrast** (Display): stronger borders and separators, darker (in Dark Mode lighter) secondary text, a bolder focus ring.
- **Reduce transparency** (Display): no translucent dimming behind dialogs.

Manta picks up changes right away, without a restart. Even without these settings, text has a contrast ratio of at least 4.5:1. For keyboard-only use see [Keyboard Shortcuts](18-tastenkuerzel.md#tastatur).

## Where Manta keeps its settings {#dateien-speicherort}

Everything is in `~/Library/Application Support/Manta/`:

| File | Contents |
|---|---|
| `settings.json` | General settings |
| `favorites.json` | Favorites and saved connections (no passwords – those are in the Keychain) |
| `session.json` | Panes, tabs and widths from the last quit |
| `plugins.json` | Which plugins are on, panel states |
| `plugin-data/` | Plugin data, such as `actions.json` |
| `update-state.json` | Update channel, last check, skipped version |
| `default-viewer.json` | Default file manager and the previous app |
| `folder-views.json` | Remembered view per folder (with **Remember view per folder**) |
| `transfers.json` | Interrupted transfers to resume (no passwords), see [Transfers](05-dateien.md#neustart) |

Thumbnails are kept separately in `~/Library/Caches/Manta/thumbs` and can be deleted at any time.

> [!TIP]
> To move Manta to another Mac, copy this folder while Manta isn't running.
