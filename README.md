<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="packaging/icon/manta/wordmark/manta-wordmark-a-dark@2x.png">
    <img src="packaging/icon/manta/wordmark/manta-wordmark-a@2x.png" alt="Manta" width="300">
  </picture>
</p>

<p align="center">
  <strong>A fast multi-pane file manager and remote file client for macOS, written in Rust.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licence-MIT-blue.svg" alt="Licence: MIT"></a>
  <img src="https://img.shields.io/badge/platform-macOS%2011%2B%20%7C%20Apple%20silicon-lightgrey.svg" alt="Platform: macOS 11+ | Apple silicon">
  <img src="https://img.shields.io/badge/Rust-stable-orange.svg?logo=rust" alt="Rust">
  <a href="https://github.com/web-workers-com/manta/releases"><img src="https://img.shields.io/github/v/release/web-workers-com/manta" alt="Latest release"></a>
</p>

<p align="center">
  <a href="https://manta-filemanager.com/download/Manta.dmg">Download</a> ·
  <a href="https://manta-filemanager.com/">Website</a> ·
  <a href="https://manta-filemanager.com/manual/en/">Manual</a> ·
  <a href="docs/plugins.md">Plugins</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

![Manta in dark mode with three panes: a Google Drive folder with sync badges, a Docker container with a terminal below it, and a Git repository with the Git panel open; the status bar shows a running copy and CPU, GPU and RAM load](docs/images/readme-full.png)

Manta brings your Mac, servers, containers and cloud folders into one window, with as many
panes as you need. It is free and open source: no ads, no tracking, no account. Manta collects
no usage data; it only goes online for the connections you open yourself and to check for
updates.

## Features

**Panes and views**
- Any number of panes side by side, each with tabs, history, a breadcrumb path bar, filter,
  sorting and hidden files; panes, tabs and widths come back at the next launch; the
  mouse's thumb buttons go back and forward
- Four views like Finder, per pane: Icons (with thumbnails), List, Columns and Gallery;
  files and apps show their macOS icons, like Finder
- Command palette (⇧⌘P) with every action and its shortcut, Go to Path (⇧⌘G), and a full
  menu bar generated from the same commands
- Local folders refresh by themselves when other apps change them (FSEvents)
- Disk images, USB sticks and external drives under Ejectable in the sidebar, with their
  own icon and an eject button, like Finder

![Manta in light mode with two local panes side by side and the sidebar with Favorites, Locations, Devices and Servers](docs/images/readme-overview.png)

**Servers, containers and cloud**
- SFTP (password, key or ssh-agent; host keys checked against `~/.ssh/known_hosts`), FTP and
  FTPS (explicit TLS)
- WebDAV over HTTPS or HTTP (Basic and Digest sign-in, Nextcloud / ownCloud presets,
  resumable downloads, server-side copy)
- SMB shares mounted through macOS (NetFS: Kerberos, Keychain, SMB 2/3), with a share
  picker; network volumes mounted in Finder show up under Devices with an eject button
- Amazon S3 and S3-compatible storage (Cloudflare R2, MinIO, Backblaze B2, Wasabi,
  DigitalOcean Spaces, Hetzner)
- Docker containers and Kubernetes pods as folders, through the `docker` / `kubectl` CLIs;
  start, restart and stop Docker containers from the sidebar
- iCloud Drive, Google Drive, OneDrive, Dropbox, Box and other cloud folders synced by
  macOS, with download state and "Download now" / "Remove download"; iCloud Drive shows
  the apps' folders by name and with their app's icon, like Finder
- Free space in the status bar: the Mac's disk, the iCloud or Google Drive quota, a
  server's disk (SFTP, SMB) or WebDAV quota
- Passwords, key passphrases and S3 secret keys are kept in the macOS Keychain

**Files**
- Background transfer queue for copy, move and delete across all backends, with progress,
  conflict policies, pause and resume; transfers survive network drops and restarts and
  continue each file where it stopped
- Drag and drop between panes and to and from other apps (copy by default)
- Archives as folders: zip, tar and compressed tars open right in the pane (read-only);
  Compress… creates zip, tar.gz, tar.xz or tar.zst
- Move to Trash like Finder (Put Back works), Quick Look with Space
- New Folder (⇧⌘N) and New File (⌥⌘N), also in the context menu (right-click a folder to
  create inside it); click a selected item again to rename it in place, like Finder
- Undo (⌘Z) and Redo (⇧⌘Z) for Move to Trash, rename and new items, like Finder
- Folder compare and sync, folder sizes (kept across restarts, updated live), a Disk Usage
  treemap, batch rename with a live preview and Undo, duplicate finder

**Built-in tools**
- A terminal below each pane that follows the pane in both directions, locally, in
  containers and over SSH
- Git status badges, Markdown preview, Quick Actions (your own shell commands for the
  selection), Project Info (scripts of Laravel, npm, Cargo, Make … projects), Image Tools
  and System Load in the status bar
- A plugin system: write your own plugins in any language ([Plugins](#plugins))

**Mac integration**
- Can become the default file manager ("Show in Finder" in other apps opens Manta)
- Light and dark appearance with accent colours, and a UI in seven languages: English,
  German, Simplified Chinese, Spanish, French, Italian and Russian
- Signed, verified in-app updates ([docs/updates.md](docs/updates.md))

The [user manual](https://manta-filemanager.com/manual/en/) describes every feature
(source: [docs/manual/en/](docs/manual/en/)); keyboard shortcuts are in
[its shortcuts chapter](docs/manual/en/18-tastenkuerzel.md).

## Download

- **Website:** [Manta.dmg](https://manta-filemanager.com/download/Manta.dmg) (latest stable
  release)
- **GitHub:** [Releases](https://github.com/web-workers-com/manta/releases)

Requirements: macOS 11 (Big Sur) or later on a Mac with Apple silicon (M1 or later).

Open the disk image, drag **Manta** onto **Applications** and start it from there (only then
can Manta update itself and become your default file manager).

### First launch

Manta is not notarized by Apple, so macOS blocks the first launch with a message such as
"“Manta” Not Opened". To open it anyway:

1. Click **Done** in the message (not "Move to Trash").
2. Open **System Settings** → **Privacy & Security**.
3. Scroll down to the **Security** section, where there is a note about Manta.
4. Click **Open Anyway** and confirm with your password or Touch ID.

On macOS versions before macOS 15, Control-click Manta in Finder and choose **Open** instead.
You only need to do this once. See
[Getting Started](docs/manual/en/01-erste-schritte.md#gatekeeper) in the manual.

## Building from source

You need macOS with the Xcode Command Line Tools (`xcode-select --install`) and Rust via
[rustup](https://rustup.rs/).

```sh
git clone https://github.com/web-workers-com/manta.git
cd manta
cargo run -p manta              # debug build
cargo run -p manta --release    # optimised build
```

To build `Manta.app` and a drag-to-install disk image:

```sh
scripts/build-release.sh        # → dist/Manta.app and dist/Manta-<version>.dmg
```

It builds for Apple silicon by default (`MANTA_ARCHS=arm64`; `universal`, `x86_64` and
`native` are also accepted). Without a Developer ID the app is self-signed or ad-hoc signed
and not notarized. Signing, notarization, all build variables, the app icon and releasing
updates are described in [docs/development.md](docs/development.md) and
[docs/updates.md](docs/updates.md).

## Project structure

A Cargo workspace; the app is the `manta` package in `crates/app`.

| Crate | Purpose |
|---|---|
| [`crates/app`](crates/app) | The egui/eframe GUI (binary `manta`) |
| [`crates/core`](crates/core) | Shared types and the virtual file system abstraction (`Vfs` trait, `VfsPath`) |
| [`crates/vfs`](crates/vfs) | Backends: local, SFTP, FTP/FTPS, WebDAV, SMB (macOS mounts), S3, Google Drive, Docker / Kubernetes, read-only archives, cloud folders synced by macOS |
| [`crates/ops`](crates/ops) | File operations across backends: transfer queue, compare and sync |
| [`crates/plugin`](crates/plugin) | Plugin system: contract, plugin host, external plugin processes, localisation helpers |
| [`crates/plugin-protocol`](crates/plugin-protocol) | Message types and a small stdio loop for writing Manta plugins in Rust |
| [`crates/plugin-git`](crates/plugin-git) | Built-in Git plugin |
| [`crates/plugin-terminal`](crates/plugin-terminal) | Built-in Terminal plugin |
| [`crates/plugin-foldersize`](crates/plugin-foldersize) | Built-in Folder Sizes plugin (size walker, Disk Usage treemap) |
| [`crates/plugin-rename`](crates/plugin-rename) | Built-in Batch Rename plugin |
| [`crates/plugin-markdown`](crates/plugin-markdown) | Built-in Markdown Preview plugin |
| [`crates/plugin-quickactions`](crates/plugin-quickactions) | Built-in Quick Actions plugin |
| [`crates/plugin-projectinfo`](crates/plugin-projectinfo) | Built-in Project Info plugin |
| [`crates/plugin-duplicates`](crates/plugin-duplicates) | Built-in Duplicates plugin |
| [`crates/plugin-imagetools`](crates/plugin-imagetools) | Built-in Image Tools plugin |
| [`crates/plugin-sysstats`](crates/plugin-sysstats) | Built-in System Load plugin |
| [`crates/updater`](crates/updater) | Self-update engine: signed feed, download, verification, bundle swap (no UI) |
| [`crates/update-upload`](crates/update-upload) | `manta-update-upload`: uploads release files over FTPS with Manta's own FTP backend |

Other folders: `docs/` (developer docs and the [user manual](docs/manual/) in seven
languages), `examples/` (example plugins), `packaging/` (Info.plist, icons, DMG layout),
`plugins/` (plugin registry), `release-notes/` and `scripts/` (build, test, publish,
translation tools).
More detail: [docs/development.md](docs/development.md).

## Plugins

Plugins add badges to file names, chips to the path bar, commands to the palette and context
menu, items to the status bar and panels below a pane or in a right sidebar. External plugins
are a folder with a `plugin.toml` and a program in any language that speaks JSON lines over
stdin/stdout. They run in a macOS sandbox that enforces the permissions they declare, start
switched off and Manta asks before the first start. Settings › Plugins › Discover installs
community plugins (download checked against the directory's SHA-256); double-clicking a
`.mantaplugin` file installs it too.

- Tutorial and protocol reference: [docs/plugins.md](docs/plugins.md)
- Examples: [examples/plugins/](examples/plugins/)

## Translations

The UI and the manual are available in English, German, Simplified Chinese, Spanish, French,
Italian and Russian. Every UI string lives in a string catalog: source tables
`(key, English, German)` in `crates/app/src/i18n/catalog/` and `crates/plugin-*/src/i18n.rs`,
plus one translation table per further language (`zh`, `es`, `fr`, `it`, `ru`).

```sh
python3 scripts/i18n/skeleton.py        # add new keys to every translation table (marked // TODO)
python3 scripts/i18n/check.py           # check all languages (must report 0 problems)
python3 scripts/i18n/check.py fr        # one language
python3 scripts/i18n/check.py --stats   # coverage per language
```

[docs/i18n/glossary.md](docs/i18n/glossary.md) lists the short UI terms in all seven
languages; it is generated from the catalogs. The rules for adding strings are in
[CONTRIBUTING.md](CONTRIBUTING.md#translations).

## Testing

```sh
scripts/test.sh          # unit and UI tests, no external services
scripts/test.sh full     # everything, including Docker, S3, SSH, Git and terminal integration tests
MANTA_RENDER_DIR=target/renders scripts/test.sh full    # also writes UI screenshots (needs a GPU)
```

Integration tests skip themselves when the service they need (Docker, `minio`, `sshd`, a
local Kubernetes context) is not available. Details: [docs/development.md](docs/development.md#tests).

## Contributing

Contributions are welcome: bug reports, translations, plugins and code. Please read
[CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md) first.

## Security

Please don't report security vulnerabilities in public issues. See [SECURITY.md](SECURITY.md).

## Licence

Manta is released under the [MIT licence](LICENSE).

## Acknowledgements

Manta is built on many open-source projects, among them [egui](https://github.com/emilk/egui)
and eframe, the [Phosphor](https://phosphoricons.com/) icons, `ssh2`, `suppaftp`, rustls,
`pulldown-cmark` and `syntect`. The licence notices of every linked crate are in
[docs/third-party-licenses.md](docs/third-party-licenses.md) (in the app: Help → Third-Party
Licences).
