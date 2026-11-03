//! Manta plugin system.
//!
//! There are two kinds of plugins, managed by one [`host::PluginHost`] and
//! shown together in Settings → Plugins:
//!
//! - **Built-in plugins** are Rust types implementing [`Plugin`], compiled
//!   into the app (e.g. Git, Terminal). They may draw their own egui UI.
//! - **External plugins** are user-installed folders with a `plugin.toml`
//!   manifest and any executable. They talk to the host via JSON lines over
//!   stdin/stdout ([`protocol`]); [`external::ExternalPlugin`] adapts them to
//!   the same [`Plugin`] trait.
//!
//! # Threading contract
//! All [`Plugin`] methods are called on the UI thread and must return quickly
//! (no blocking IO, no process spawning that waits). Do slow work on your own
//! threads and push results back through the [`PluginHandle`] you receive in
//! [`Plugin::enable`]; the handle is `Clone + Send + Sync` and wakes the UI.

use std::path::PathBuf;
use std::sync::mpsc::Sender;

use manta_core::{Protocol, VfsPath};
use serde::{Deserialize, Serialize};

pub mod bundled;
pub mod external;
pub mod host;
pub mod i18n;
pub mod install;
pub mod manifest;
pub mod protocol;
pub mod registry;
pub mod sandbox;
pub mod shortcut;

/// A file pane. The window shows any number of panes (at least one) side by
/// side; each has a stable id for as long as it exists.
///
/// Ids are identities, not positions: closing a pane never renumbers the
/// others ([`PluginEvent::PaneClosed`] tells plugins to drop what they keep
/// for it), and a new pane gets the smallest free id
/// ([`PluginEvent::PaneOpened`]). [`PaneId::LEFT`] (0) and
/// [`PaneId::RIGHT`] (1) are the two panes of the classic two-pane window.
///
/// JSON: `0` ↔ `"left"`, `1` ↔ `"right"`, every other pane as a number.
/// Both spellings are accepted on input (`"left"`, `0`, `"0"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PaneId(pub u16);

impl PaneId {
    /// The first pane (the left one in a two-pane window).
    pub const LEFT: PaneId = PaneId(0);
    /// The second pane (the right one in a two-pane window).
    pub const RIGHT: PaneId = PaneId(1);
    /// Source-compatible alias of [`PaneId::LEFT`] (the enum variant before
    /// panes became numbered).
    #[allow(non_upper_case_globals)]
    #[doc(hidden)]
    pub const Left: PaneId = PaneId::LEFT;
    /// Source-compatible alias of [`PaneId::RIGHT`].
    #[allow(non_upper_case_globals)]
    #[doc(hidden)]
    pub const Right: PaneId = PaneId::RIGHT;

    /// The id as an index (0, 1, 2 …).
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// `"left"`, `"right"` or the number (as in JSON and saved state).
    pub fn key(self) -> String {
        match self.0 {
            0 => "left".into(),
            1 => "right".into(),
            n => n.to_string(),
        }
    }

    /// Parse [`PaneId::key`] (also accepts `"0"` / `"1"`).
    pub fn from_key(s: &str) -> Option<PaneId> {
        match s {
            "left" => Some(PaneId::LEFT),
            "right" => Some(PaneId::RIGHT),
            n => n.parse().ok().map(PaneId),
        }
    }
}

impl std::fmt::Display for PaneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key())
    }
}

impl Serialize for PaneId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            0 => s.serialize_str("left"),
            1 => s.serialize_str("right"),
            n => s.serialize_u16(n),
        }
    }
}

impl<'de> Deserialize<'de> for PaneId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = PaneId;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("\"left\", \"right\" or a pane number")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<PaneId, E> {
                PaneId::from_key(v).ok_or_else(|| E::custom(format!("unknown pane {v:?}")))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<PaneId, E> {
                u16::try_from(v)
                    .map(PaneId)
                    .map_err(|_| E::custom(format!("pane number {v} out of range")))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<PaneId, E> {
                u16::try_from(v)
                    .map(PaneId)
                    .map_err(|_| E::custom(format!("pane number {v} out of range")))
            }
        }
        d.deserialize_any(V)
    }
}

/// What a plugin knows about a pane: its current directory and selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaneState {
    pub pane: PaneId,
    /// Current directory inside the pane's backend.
    pub path: VfsPath,
    /// The real OS path if the pane shows the local file system, else `None`.
    /// Git and Terminal only work when this is `Some`.
    pub local_path: Option<PathBuf>,
    pub protocol: Protocol,
    /// e.g. "Local" or "anna@files.example.org (SFTP)".
    pub backend_label: String,
    /// Selected entries (absolute paths), in display order.
    pub selection: Vec<VfsPath>,
    /// True if this is the focused pane.
    pub active: bool,
    /// The server, container or bucket the pane shows (`None` for local
    /// panes and archives). The Terminal opens its shell there (Docker,
    /// Kubernetes, SFTP) and sends [`HostRequest::NavigateIn`] with it.
    #[serde(default)]
    pub remote: Option<RemoteEndpoint>,
}

/// Events delivered to enabled plugins via [`Plugin::on_event`].
#[derive(Debug, Clone, PartialEq)]
pub enum PluginEvent {
    /// The pane now shows a different directory (navigation, tab switch,
    /// connect/disconnect). Also sent once per pane right after `enable`.
    DirectoryChanged(PaneState),
    /// The same directory was re-listed (refresh, after a file operation).
    ListingRefreshed(PaneState),
    SelectionChanged(PaneState),
    ActivePaneChanged(PaneId),
    /// The user started a file action in the GUI (copy, delete, rename, …),
    /// or one finished. Every action is reported twice with the same `id`:
    /// once with [`FileActionPhase::Started`], once with `Finished`.
    /// Purely informational: the app performs the action itself.
    FileAction(FileAction),
    /// Result of this plugin's [`HostRequest::RenameMany`] (sent only to
    /// the plugin that asked).
    RenameResult(RenameResult),
    /// Answer to this plugin's [`HostRequest::ListDir`] (sent only to the
    /// plugin that asked).
    DirListing(DirListing),
    /// Type `command` into the terminal below `pane` and press Enter, after
    /// `cd`-ing to `cwd` if set (see [`HostRequest::RunInTerminal`]). Sent
    /// only to the Terminal plugin (`manta.terminal`); the host has
    /// already opened its panel for `pane`.
    RunInTerminal {
        pane: PaneId,
        command: String,
        cwd: Option<VfsPath>,
    },
    /// Answer to this plugin's [`HostRequest::ReadFile`] (sent only to the
    /// plugin that asked).
    FileContent(FileContent),
    /// The app's UI language changed (Settings → General → Language). The
    /// value is a BCP 47 language code such as `"en"` or `"de"`; the current
    /// one is always available from [`PluginHandle::locale`] / [`locale`].
    LocaleChanged(String),
    /// A pane appeared (the user split a pane, or the window restored
    /// several panes at launch). Sent before its first `DirectoryChanged`,
    /// and once per existing pane right after `enable`.
    PaneOpened(PaneId),
    /// A pane was closed. Drop everything kept for it (terminal sessions are
    /// hung up, caches cleared); the host has already dropped its
    /// decorations, chips, sizes and `PaneBottom` panel instances. The id
    /// may be given to a new pane later (after a new `PaneOpened`).
    PaneClosed(PaneId),
}

static LOCALE: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

/// The app's UI language as a BCP 47 code (`"en"`, `"de"`). `"en"` until
/// the host sets it.
pub fn locale() -> String {
    if let Some(l) = i18n::thread_lang() {
        return l.code().to_string();
    }
    let l = LOCALE.read().map(|l| l.clone()).unwrap_or_default();
    if l.is_empty() {
        "en".to_string()
    } else {
        l
    }
}

/// Set the UI language without notifying plugins (the host calls this
/// before plugins start; afterwards use [`host::PluginHost::set_locale`]).
pub fn set_global_locale(code: &str) {
    if let Ok(mut l) = LOCALE.write() {
        *l = code.to_string();
    }
    i18n::set_global(code);
}

/// The bytes of a file read with [`HostRequest::ReadFile`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileContent {
    pub pane: PaneId,
    pub path: VfsPath,
    /// The first `max_bytes` bytes of the file (empty on error).
    #[serde(default)]
    pub bytes: Vec<u8>,
    /// The file's full size, if known.
    #[serde(default)]
    pub size: Option<u64>,
    /// The file is larger than `max_bytes`; `bytes` holds only its start.
    #[serde(default)]
    pub truncated: bool,
    /// Last modification, seconds since the Unix epoch (UTC), if known.
    #[serde(default)]
    pub modified: Option<i64>,
    /// Set if the file could not be read.
    #[serde(default)]
    pub error: Option<String>,
}

/// The outcome of a [`HostRequest::RenameMany`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameResult {
    pub pane: PaneId,
    /// The folder the names are in.
    pub dir: VfsPath,
    /// Renames that were done: (old name, new name).
    #[serde(default)]
    pub ok: Vec<(String, String)>,
    /// Renames that failed or were refused: (old name, error). If the whole
    /// batch was refused (e.g. a target collides with an existing entry),
    /// every pair is listed here and nothing was renamed.
    #[serde(default)]
    pub failed: Vec<(String, String)>,
}

/// One entry of a [`DirListing`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListedEntry {
    pub name: String,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default)]
    pub size: u64,
    /// Last modification, seconds since the Unix epoch (UTC), if known.
    #[serde(default)]
    pub modified: Option<i64>,
    /// The entry is visible in the pane (not hidden, not filtered out).
    #[serde(default)]
    pub shown: bool,
}

/// The entries of a folder, as the pane lists them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirListing {
    pub pane: PaneId,
    pub dir: VfsPath,
    /// Shown entries first, in the pane's display order; then the entries
    /// the pane hides (hidden files, filtered out).
    #[serde(default)]
    pub entries: Vec<ListedEntry>,
    /// Set if the folder could not be listed (`entries` is then empty).
    #[serde(default)]
    pub error: Option<String>,
}

/// What kind of file action the user started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileActionKind {
    /// `sources` → into the folder `destination`.
    Copy,
    /// `sources` → into the folder `destination`.
    Move,
    /// `sources` are deleted permanently (not moved to the Trash).
    Delete,
    /// `sources` (local items) were moved to the macOS Trash, where Finder's
    /// "Put Back" can restore them.
    Trash,
    /// `sources[0]` (old path) → `destination` (new path). A batch rename
    /// lists every pair in [`FileAction::renames`] (`sources` holds all old
    /// paths, `destination` the first new path).
    Rename,
    /// `sources[0]` is the new folder.
    NewFolder,
    /// `sources[0]` is the new (empty) file.
    NewFile,
    /// `sources` are opened with their default app.
    Open,
    /// Connected to `src_remote`; `sources[0]` is the folder opened.
    Connect,
    /// Disconnected from `src_remote`.
    Disconnect,
    /// The folder `sources[0]` is synced to the folder `destination`
    /// (see [`FileAction::sync`]).
    Sync,
    /// `sources` are packed into a new archive: [`FileAction::archive`]
    /// holds its path and format, `destination` the folder it is created in.
    Compress,
    /// `sources` (paths *inside* the archive [`FileAction::archive`], from
    /// its root `/`) are extracted into the folder `destination`.
    Extract,
}

/// Start or end of a [`FileAction`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileActionPhase {
    Started,
    Finished {
        /// Everything worked (no per-item errors, not cancelled).
        ok: bool,
        /// Files processed (for Delete: entries incl. folders).
        files: u64,
        bytes: u64,
        /// Per-item errors (or the one fatal error), first ones first.
        #[serde(default)]
        errors: Vec<String>,
        #[serde(default)]
        cancelled: bool,
        elapsed_ms: u64,
    },
}

/// A remote server taking part in a file action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteEndpoint {
    pub protocol: Protocol,
    /// May be empty (anonymous FTP). Kubernetes: the pod's container ("" =
    /// the pod's default).
    pub user: String,
    /// Docker: the container name. Kubernetes: `namespace/pod` (or `pod`).
    pub host: String,
    pub port: u16,
    /// Kubernetes: the kubeconfig context ("" = the current context).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub context: String,
}

/// Extra details of a [`FileActionKind::Sync`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SyncSpec {
    /// Files only on the target are deleted (one-way sync).
    #[serde(default)]
    pub delete_extra: bool,
    /// "Newer file wins" in both directions.
    #[serde(default)]
    pub both_ways: bool,
}

/// Details of a [`FileActionKind::Compress`] or [`FileActionKind::Extract`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveSpec {
    /// The archive file (created by `Compress`, read by `Extract`), as an
    /// absolute path in its backend.
    pub path: VfsPath,
    /// "zip", "tar.gz", "tar.xz", "tar.zst", "tar.bz2", "tar", "gzip", …
    pub format: String,
    /// The archive file is on the local file system (`path` is an OS path).
    #[serde(default)]
    pub local: bool,
    /// The server the archive file is on, if it is remote.
    #[serde(default)]
    pub remote: Option<RemoteEndpoint>,
    /// `Compress`: the compression level (1 = fastest … 9 = smallest).
    #[serde(default)]
    pub level: Option<u32>,
    /// `Compress`: `.DS_Store` files and `__MACOSX` folders were left out.
    #[serde(default)]
    pub exclude_junk: bool,
}

/// A file action done in the GUI, as reported to plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAction {
    /// Same for `Started` and `Finished` of one action; unique per app run.
    pub id: u64,
    /// The pane the action started in (for drag & drop: the pane dragged
    /// from).
    pub pane: PaneId,
    pub kind: FileActionKind,
    /// Absolute paths in the source backend (see [`FileActionKind`]).
    #[serde(default)]
    pub sources: Vec<VfsPath>,
    /// Absolute path in the destination backend (see [`FileActionKind`]).
    #[serde(default)]
    pub destination: Option<VfsPath>,
    /// The source backend is the local file system (paths are OS paths).
    pub src_local: bool,
    /// The destination backend is the local file system.
    pub dst_local: bool,
    /// e.g. "Local" or "anna@files.example.org (SFTP)".
    pub src_label: String,
    #[serde(default)]
    pub dst_label: Option<String>,
    pub phase: FileActionPhase,
    /// The source server, if the source is remote.
    #[serde(default)]
    pub src_remote: Option<RemoteEndpoint>,
    /// The destination server, if the destination is remote.
    #[serde(default)]
    pub dst_remote: Option<RemoteEndpoint>,
    /// Set for [`FileActionKind::Sync`].
    #[serde(default)]
    pub sync: Option<SyncSpec>,
    /// For a batch [`FileActionKind::Rename`]: every (old path, new path)
    /// pair, in the order they were requested. Empty otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub renames: Vec<(VfsPath, VfsPath)>,
    /// Set for [`FileActionKind::Compress`] and [`FileActionKind::Extract`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive: Option<Box<ArchiveSpec>>,
}

impl FileAction {
    /// A `Started` action on the local file system with no destination;
    /// set the other fields as needed.
    pub fn new(id: u64, pane: PaneId, kind: FileActionKind, sources: Vec<VfsPath>) -> Self {
        FileAction {
            id,
            pane,
            kind,
            sources,
            destination: None,
            src_local: true,
            dst_local: true,
            src_label: "Local".into(),
            dst_label: None,
            phase: FileActionPhase::Started,
            src_remote: None,
            dst_remote: None,
            sync: None,
            renames: Vec::new(),
            archive: None,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.phase, FileActionPhase::Finished { .. })
    }
}

/// Semantic colour; the host maps it to the current theme (light/dark).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
    Muted,
}

/// A small badge drawn next to a file name, e.g. "M" for modified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Badge {
    /// 1–3 characters.
    pub text: String,
    pub tone: Tone,
}

/// How a plugin decorates one entry in the file list.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Decoration {
    #[serde(default)]
    pub badge: Option<Badge>,
    /// Tint for the file name.
    #[serde(default)]
    pub name_tone: Option<Tone>,
    /// Draw the row dimmed (e.g. git-ignored files).
    #[serde(default)]
    pub dimmed: bool,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// The size of a folder (or file) in the file list, sent by a plugin with
/// [`HostRequest::SetSizes`]. The host shows it in the Size column, sorts
/// folders by it and adds it to the selection size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SizeInfo {
    /// Logical size in bytes (the sum of file lengths, like the sizes of
    /// files in the list). While `complete` is false: what was counted so far.
    pub bytes: u64,
    /// The count is finished. `false` = still counting; the host shows the
    /// value as "≥ …" with a progress hint.
    pub complete: bool,
    /// Files counted (hard links once).
    #[serde(default)]
    pub files: u64,
    /// Allocated size on disk (`st_blocks × 512`), if known. Shown in the
    /// tooltip.
    #[serde(default)]
    pub on_disk: Option<u64>,
    /// Sub-folders that could not be read (e.g. permission denied). The
    /// value is then a lower bound even when `complete`.
    #[serde(default)]
    pub unreadable: u64,
}

/// A status chip shown in the pane's path bar, e.g. the git branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chip {
    pub text: String,
    #[serde(default)]
    pub tone: Tone,
    /// Optional icon name: one of "branch", "terminal", "info", "warning",
    /// "check", "cloud", "plug". Unknown names are ignored by the host.
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub tooltip: Option<String>,
    /// If set, clicking the chip runs this command of the same plugin.
    #[serde(default)]
    pub command: Option<String>,
}

/// How urgent a [`StatusItem`] looks: `warn` is amber, `error` red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusLevel {
    #[default]
    Normal,
    Warn,
    Error,
}

/// What a click on a [`StatusItem`] does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StatusAction {
    /// Run this command of the same plugin (as [`Plugin::run_command`] /
    /// the `run_command` message).
    Command { command: String },
    /// Open a URL with its default app (`https:`, `mailto:`,
    /// `x-apple.systempreferences:` …). Needs a scheme.
    OpenUrl { url: String },
    /// Launch or bring up the app with this bundle identifier, e.g.
    /// `com.apple.ActivityMonitor` (`open -b`).
    OpenApp { bundle_id: String },
}

impl StatusAction {
    /// The action is well-formed: a known kind with a plausible target
    /// (a URL with a scheme, a bundle id of letters, digits, `.` and `-`).
    /// The host ignores clicks on items whose action isn't valid.
    pub fn is_valid(&self) -> bool {
        match self {
            StatusAction::Command { command } => !command.is_empty(),
            StatusAction::OpenUrl { url } => {
                let scheme = url.split_once(':').map(|(s, _)| s).unwrap_or("");
                !scheme.is_empty()
                    && scheme
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic())
                    && scheme
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
            }
            StatusAction::OpenApp { bundle_id } => {
                !bundle_id.is_empty()
                    && !bundle_id.starts_with(['-', '.'])
                    && bundle_id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            }
        }
    }
}

/// An item a plugin shows in the window's status bar (right side, before
/// the command palette hint), sent with [`HostRequest::SetStatusItems`].
///
/// When the window is narrow the host shrinks the status bar step by step:
/// first the palette hint goes, then items without a `compact` form
/// (lowest `priority` first), then the remaining items switch to their
/// `compact` form, then those are dropped, lowest `priority` first. The
/// tooltip shows the full text whenever an item is shortened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusItem {
    /// Unique within the plugin (for tests and accessibility).
    pub id: String,
    /// e.g. "CPU 12 %". Up to 40 characters.
    pub text: String,
    /// A shorter text for narrow windows, e.g. "12 %"; `""` = icon and
    /// meter only. `None`: the item is hidden instead of shortened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Optional icon name: the chip icons plus "cpu", "gpu" and "memory".
    /// Unknown names are ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub level: StatusLevel,
    /// A small bar meter after the text, 0.0–1.0 (clamped).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meter: Option<f32>,
    /// Higher stays longer when space is short. Default 0.
    #[serde(default)]
    pub priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<StatusAction>,
}

impl StatusItem {
    /// A plain item with `id` and `text`; set the other fields as needed.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        StatusItem {
            id: id.into(),
            text: text.into(),
            compact: None,
            tooltip: None,
            icon: None,
            level: StatusLevel::Normal,
            meter: None,
            priority: 0,
            action: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandContext {
    /// Only in the command palette.
    Palette,
    /// In the palette and in the file list's context menu.
    FileContextMenu,
}

/// An action a plugin contributes to the palette / context menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSpec {
    /// Unique within the plugin.
    pub id: String,
    pub title: String,
    /// Keyboard shortcut, e.g. "⌃`", "⇧⌘G", "⌥⌘T": modifier symbols
    /// (⌃ ⌥ ⇧ ⌘) followed by one key. The host binds it globally (it fires
    /// even while a plugin panel has focus) and shows it in the palette.
    #[serde(default)]
    pub shortcut: Option<String>,
    pub context: CommandContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelPlacement {
    /// Inside each file pane, below the file list (one instance per pane).
    PaneBottom,
    /// A panel on the right side of the window (one instance).
    RightSidebar,
    /// A centred sheet over a dimmed window, like the app's dialogs (one
    /// instance). The host draws the frame, title bar and close button;
    /// Esc or the close button closes it. Its size comes from
    /// [`Plugin::panel_size`] (externals: `size` in the panel declaration).
    /// Modal panels never open by themselves at startup.
    Modal,
}

/// A UI area a plugin wants to own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelSpec {
    /// Unique within the plugin.
    pub id: String,
    pub title: String,
    pub placement: PanelPlacement,
    /// Open when the plugin is enabled for the first time.
    pub default_open: bool,
}

/// Static description, shown in Settings → Plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Stable id, e.g. "manta.git". Used as the key for the enable state.
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    /// Compiled into the app (cannot be uninstalled).
    pub builtin: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Requests from a plugin to the host, sent through a [`PluginHandle`].
#[derive(Debug, Clone, PartialEq)]
pub enum HostRequest {
    /// Decorations for the entries of `dir` (by file name) in `pane`. Replaces
    /// any earlier decorations of this plugin for that pane. The host ignores
    /// it if the pane no longer shows `dir`.
    SetDecorations {
        pane: PaneId,
        dir: VfsPath,
        entries: Vec<(String, Decoration)>,
    },
    /// Path-bar chips for `pane` while it shows `dir` (replaces earlier ones).
    SetChips {
        pane: PaneId,
        dir: VfsPath,
        chips: Vec<Chip>,
    },
    /// Show a toast.
    Toast { level: ToastLevel, text: String },
    /// Navigate `pane` to `path` in its current backend (e.g. the terminal
    /// followed a `cd`). The host ignores it if the path is unchanged.
    Navigate { pane: PaneId, path: VfsPath },
    /// Like [`HostRequest::Navigate`], for a pane that may show a server or
    /// container: the host navigates only while the pane still shows exactly
    /// `remote` (`None`: a local folder), so a request that raced with the
    /// user switching servers is dropped. Used by the Terminal when the shell
    /// inside a container or on a server changes directory.
    NavigateIn {
        pane: PaneId,
        remote: Option<RemoteEndpoint>,
        path: VfsPath,
    },
    /// Open or close one of this plugin's panels (`pane` for PaneBottom).
    SetPanelOpen {
        panel: String,
        pane: Option<PaneId>,
        open: bool,
    },
    /// The command / panel list changed; the host calls `commands()` /
    /// `panels()` again.
    ContributionsChanged,
    /// Open the panel if it is closed, close it if it is open (`pane`
    /// selects the instance of a `PaneBottom` panel; `None` = the active
    /// pane. Ignored for `RightSidebar` and `Modal` panels).
    TogglePanel { panel: String, pane: Option<PaneId> },
    /// Sizes for entries of `dir` (by file name) in `pane`, usually folders.
    /// Merged into this plugin's earlier sizes for that folder (send only
    /// what changed); dropped when the pane shows another folder. The host
    /// ignores it if the pane no longer shows `dir`.
    SetSizes {
        pane: PaneId,
        dir: VfsPath,
        sizes: Vec<(String, SizeInfo)>,
    },
    /// Rename entries of `dir` (old name, new name) in `pane`'s backend
    /// (local or remote). The host does it in the background: it plans a
    /// safe order (swaps and cycles such as a→b→a go through temporary
    /// names, as do case-only renames), refuses the whole batch if a target
    /// collides with an entry that isn't renamed away, reports one
    /// [`FileActionKind::Rename`] action, refreshes the pane and answers
    /// with [`PluginEvent::RenameResult`]. Undo = another `RenameMany` with
    /// the `ok` pairs reversed.
    RenameMany {
        pane: PaneId,
        dir: VfsPath,
        renames: Vec<(String, String)>,
    },
    /// List `dir` as `pane` shows it (names, folders, sizes, dates, display
    /// order). Answered with [`PluginEvent::DirListing`]; instant when the
    /// pane shows `dir`, else listed in the background.
    ListDir { pane: PaneId, dir: VfsPath },
    /// A toast with a button that runs one of this plugin's commands, e.g.
    /// "Renamed 12 items" with "Undo".
    ActionToast {
        level: ToastLevel,
        text: String,
        button: String,
        command: String,
    },
    /// Run `command` visibly in the built-in Terminal below `pane` (`None` =
    /// the active pane): the host opens the terminal panel if it is closed
    /// and hands it to the Terminal plugin, which `cd`s to `cwd` (if set)
    /// once the shell is idle, then types the command and presses Enter, so
    /// it lands in the shell history. While a program runs in the shell the
    /// command waits; a half-typed command line is kept. If the Terminal
    /// plugin is switched off, the host shows a toast instead. `command` is
    /// shell text: quote paths yourself.
    RunInTerminal {
        pane: Option<PaneId>,
        command: String,
        cwd: Option<VfsPath>,
    },
    /// List the folder of `pane` again (`None` = both panes), e.g. after a
    /// command changed files there.
    RefreshPane { pane: Option<PaneId> },
    /// Show the SFTP server `user@host:port` in `pane`, e.g. the one an
    /// `ssh` in the Terminal logged in to: an open connection to it, else a
    /// matching favorite, else the Connect dialog filled in. Built-in
    /// plugins only (not part of the external protocol).
    OpenSftp {
        pane: PaneId,
        user: String,
        host: String,
        port: u16,
    },
    /// The shell in the Terminal below `pane` was left (`exit`) while the
    /// pane showed `remote`: the pane goes back to this Mac, and a
    /// connection no other tab shows is closed (an SMB share Manta mounted
    /// is ejected). Ignored once the pane shows something else. Built-in
    /// plugins only (not part of the external protocol).
    LeaveConnection {
        pane: PaneId,
        remote: RemoteEndpoint,
    },
    /// Read up to `max_bytes` of the file `path` in `pane`'s backend (local
    /// or remote), in the background. Answered with
    /// [`PluginEvent::FileContent`]. The host caps `max_bytes` at
    /// [`MAX_READ_BYTES`].
    ReadFile {
        pane: PaneId,
        path: VfsPath,
        max_bytes: u64,
    },
    /// Select (and scroll to) `paths` in `pane` (`None` = the active pane),
    /// navigating there first if the pane shows another folder. Local panes
    /// only. The folder is the parent of the first path; paths in other
    /// folders are ignored.
    Select {
        pane: Option<PaneId>,
        paths: Vec<VfsPath>,
    },
    /// This plugin's items in the window's status bar; replaces the ones
    /// sent before (an empty list removes them). They stay until replaced
    /// or the plugin is switched off. At most
    /// [`protocol::MAX_STATUS_ITEMS`] are kept.
    SetStatusItems { items: Vec<StatusItem> },
}

/// The most a [`HostRequest::ReadFile`] returns (16 MiB).
pub const MAX_READ_BYTES: u64 = 16 * 1024 * 1024;

/// The plugin's channel to the host. Cheap to clone; usable from any thread.
/// Every send wakes the UI.
#[derive(Clone)]
pub struct PluginHandle {
    plugin_id: String,
    tx: Sender<(String, HostRequest)>,
    ctx: egui::Context,
    data_dir: PathBuf,
    cache_dir: Option<PathBuf>,
}

impl PluginHandle {
    pub fn new(
        plugin_id: String,
        tx: Sender<(String, HostRequest)>,
        ctx: egui::Context,
        data_dir: PathBuf,
    ) -> Self {
        PluginHandle {
            plugin_id,
            tx,
            ctx,
            data_dir,
            cache_dir: None,
        }
    }

    /// Keep caches in `dir` (else in [`PluginHandle::data_dir`]).
    pub fn with_cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = Some(dir);
        self
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Send a request to the host. Silently ignored after the host is gone.
    pub fn send(&self, req: HostRequest) {
        let _ = self.tx.send((self.plugin_id.clone(), req));
        self.ctx.request_repaint();
    }

    pub fn request_repaint(&self) {
        self.ctx.request_repaint();
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.ctx
    }

    /// Per-plugin directory for settings/state (created by the host).
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Per-plugin directory for caches: in `~/Library/Caches` (not backed
    /// up, may be deleted at any time); not created until written to.
    pub fn cache_dir(&self) -> &PathBuf {
        self.cache_dir.as_ref().unwrap_or(&self.data_dir)
    }

    /// The app's UI language (`"en"`, `"de"`). Changes arrive as
    /// [`PluginEvent::LocaleChanged`].
    pub fn locale(&self) -> String {
        locale()
    }
}

/// A plugin. See the module docs for the threading contract.
pub trait Plugin: Send {
    fn info(&self) -> PluginInfo;

    /// Called when the plugin is switched on (at startup if enabled). The
    /// host then sends a `DirectoryChanged` for each pane.
    fn enable(&mut self, handle: PluginHandle);

    /// Called when switched off or at app exit. Stop threads, kill child
    /// processes. The host drops this plugin's decorations, chips and panels.
    fn disable(&mut self);

    fn on_event(&mut self, _event: &PluginEvent) {}

    fn commands(&self) -> Vec<CommandSpec> {
        Vec::new()
    }

    /// Run a command; `pane` is the active pane's state.
    fn run_command(&mut self, _id: &str, _pane: &PaneState) {}

    fn panels(&self) -> Vec<PanelSpec> {
        Vec::new()
    }

    /// Preferred size (width, height in points) of a [`PanelPlacement::Modal`]
    /// panel; the host clamps it to the window. `None` = a default size.
    fn panel_size(&self, _panel: &str) -> Option<[f32; 2]> {
        None
    }

    /// Draw a panel's contents. `pane` is set for `PaneBottom` panels (the
    /// pane the panel instance belongs to) and is the active pane for
    /// `RightSidebar` panels. The host provides the frame, title bar and
    /// close button; draw only the body.
    fn panel_ui(&mut self, _panel: &str, _pane: &PaneState, _ui: &mut egui::Ui) {}

    /// Declarative content for a panel instead of [`Plugin::panel_ui`]: the
    /// host draws these blocks exactly like an external plugin's
    /// `set_panel_content` (same renderer, light and dark) and reports
    /// clicks on rows and buttons with an `action` to
    /// [`Plugin::panel_action`]. `None` (the default) = the plugin draws the
    /// panel itself in `panel_ui`. Called every frame the panel is visible
    /// (`pane` as for `panel_ui`: set for `PaneBottom` panels), so build it
    /// from cached state and do no IO.
    fn panel_blocks(
        &self,
        _panel: &str,
        _pane: Option<PaneId>,
    ) -> Option<std::sync::Arc<Vec<protocol::Block>>> {
        None
    }

    /// A click on a row or button with an `action` in blocks returned by
    /// [`Plugin::panel_blocks`]. `pane` is the state of the panel's pane
    /// (`PaneBottom`) or the active pane.
    fn panel_action(&mut self, _panel: &str, _action: &str, _pane: &PaneState) {}

    /// Optional plugin settings, drawn below the plugin in Settings → Plugins.
    /// Return `false` if the plugin has no settings (the host hides the area).
    fn settings_ui(&mut self, _ui: &mut egui::Ui) -> bool {
        false
    }
}

/// Run state of a plugin, shown in Settings → Plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// Switched off.
    Stopped,
    /// External plugin: the process is being started.
    Starting,
    Running,
    /// Crashed, failed to start, or panicked (built-ins). The message says why.
    Error(String),
}
