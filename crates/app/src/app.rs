//! Main application state, layout, keyboard shortcuts and action handling.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{self, CornerRadius, Key, Modifiers, RichText, Sense, Stroke};
use manta_core::{ConnectionConfig, EntryKind, SharedVfs, VfsPath};
use manta_ops::{ConflictPolicy, Job, JobId, JobKind, JobStatus, TransferManager};
use manta_vfs::container::discover::DockerOp;

pub(crate) mod archive_ops;
mod conflicts;
mod finder_ops;
mod native_dnd;
mod transfer_resume;
pub(crate) mod trash_place;
pub(crate) use trash_place::shows_trash;

use archive_ops::ReadOnlyOp;

use crate::commands::Cmd;
use crate::dialogs::{self, ConnectForm, DeleteForm, Dialog, DialogResult, NameForm, TransferForm};
use crate::favorites::{self, Favorite, Favorites};
use crate::file_actions::ActionTracker;
use crate::goto::{GotoPopup, GotoRequest, GotoTarget};
use crate::host_key::{Checked, HostKeyForm, HostKeyRetry};
use crate::icons;
use crate::palette::{CommandPalette, PaletteEntry, PaletteId};
use crate::pane::{self, Location, Pane, PaneInfo, Tab};
use crate::plugin_ui;
use crate::prefs::Prefs;
use crate::preview::Preview;
use crate::settings::{Confirm, SettingsView};
use crate::sidebar::{self, ConnectionInfo, Place};
use crate::sync_view::{SyncRequest, SyncView};
use crate::theme::{self, pal, radius, space, text};
use crate::toasts::{ToastKind, Toasts};
use crate::transfers::{self, TransfersView};
use crate::undo::Op;
use crate::util::{self, same_fs};
use crate::widgets;
use crate::worker::{self, ConnectRequest, WorkResult, Worker};
use manta_plugin::host::{HostEvent, HostPaths, PanelContent, PanelLayout, PanelRef, PluginHost};
use manta_plugin::{
    DirListing, FileAction, FileActionKind, ListedEntry, PaneId, PaneState, PanelPlacement, Plugin,
    PluginEvent, RemoteEndpoint, RenameResult, SyncSpec, ToastLevel,
};

/// A file pane, by its stable id (the same id plugins see). There can be
/// any number of panes, at least one; see [`crate::pane_set`].
pub type Side = PaneId;

/// Space between two panes (the divider's grab area).
pub(crate) const PANE_GAP: f32 = space::SM;

/// The plugins compiled into Manta.
fn builtin_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(manta_plugin_git::GitPlugin::new()),
        Box::new(manta_plugin_terminal::TerminalPlugin::new()),
        Box::new(manta_plugin_foldersize::FolderSizePlugin::new()),
        Box::new(manta_plugin_rename::BatchRenamePlugin::new()),
        Box::new(manta_plugin_markdown::MarkdownPlugin::new()),
        Box::new(manta_plugin_quickactions::QuickActionsPlugin::new()),
        Box::new(manta_plugin_projectinfo::ProjectInfoPlugin::new()),
        Box::new(manta_plugin_duplicates::DuplicatesPlugin::new()),
        Box::new(manta_plugin_imagetools::ImageToolsPlugin::new()),
        Box::new(manta_plugin_sysstats::SysStatsPlugin::new()),
    ]
}

/// The built-in batch rename command (F2 / Rename with several items).
const BATCH_RENAME_CMD: &str = "manta.rename:batch-rename";

fn toast_kind(level: ToastLevel) -> ToastKind {
    match level {
        ToastLevel::Info => ToastKind::Info,
        ToastLevel::Success => ToastKind::Success,
        ToastLevel::Warning => ToastKind::Warning,
        ToastLevel::Error => ToastKind::Error,
    }
}

fn listed(e: &manta_core::Entry, shown: bool) -> ListedEntry {
    ListedEntry {
        name: e.name.clone(),
        is_dir: e.is_dir(),
        size: e.size,
        modified: e.modified.map(|t| t.timestamp()),
        shown,
    }
}

/// A tab's entries for a plugin: shown ones in display order, then the
/// hidden / filtered ones.
fn listed_entries(tab: &Tab) -> Vec<ListedEntry> {
    let mut v: Vec<ListedEntry> = tab
        .view
        .iter()
        .filter_map(|&i| tab.entries.get(i))
        .map(|e| listed(e, true))
        .collect();
    let shown: std::collections::HashSet<usize> = tab.view.iter().copied().collect();
    v.extend(
        tab.entries
            .iter()
            .enumerate()
            .filter(|(i, _)| !shown.contains(i))
            .map(|(_, e)| listed(e, false)),
    );
    v
}

/// Everything the UI can ask the app to do. Collected during a frame and
/// processed after drawing to keep borrows simple.
pub enum Action {
    Activate(Side),
    Navigate(Side, VfsPath),
    OpenSelected(Side),
    Up(Side),
    Back(Side),
    Forward(Side),
    Refresh(Side),
    NewTab(Side),
    CloseTab(Side, usize),
    SwitchTab(Side, usize),
    Preview(Side),
    /// `true` = move.
    CopyToOther(Side, bool),
    Rename(Side),
    /// Click-to-rename fired: edit the selected item's name in place
    /// (List view; the rename dialog in the other views).
    RenameInPlace(Side),
    /// The in-place name editor lost focus: rename unless Esc (`commit`
    /// false); `enter`: an invalid name keeps the editor open.
    InlineRenameEnd {
        side: Side,
        commit: bool,
        enter: bool,
    },
    NewFolder(Side),
    /// An empty file in the pane's folder (asks for the name).
    NewFile(Side),
    /// New folder (`file`: new file) inside the folder `dir` shown in the
    /// pane (its context menu).
    NewIn {
        side: Side,
        dir: VfsPath,
        file: bool,
    },
    /// ⌘⌫: local items to the Trash; elsewhere a permanent delete (asks).
    Delete(Side),
    /// ⌥⌘⌫: delete permanently (asks).
    DeletePermanently(Side),
    /// Items dragged from pane `from` onto the sidebar's Trash: local ones
    /// go to the Trash, server items are deleted permanently after asking.
    DropOnTrash {
        from: Side,
        fs: SharedVfs,
        paths: Vec<VfsPath>,
    },
    /// Files from another app dropped onto the sidebar's Trash.
    TrashExternal(Vec<PathBuf>),
    /// "Empty Trash…" (asks first).
    EmptyTrash,
    /// ⌘Z / ⇧⌘Z: the next step of the file operation history.
    Undo,
    Redo,
    /// Show the Trash in Finder (explicitly Finder: Manta may be the
    /// default file viewer).
    ShowTrashInFinder,
    /// System Settings › Privacy & Security › Full Disk Access.
    OpenFullDiskAccess,
    DropFiles {
        from: Side,
        to: Side,
        fs: SharedVfs,
        paths: Vec<VfsPath>,
        mv: bool,
    },
    /// Files dropped onto a pane by another app (or Manta's own native drag
    /// coming back): copy (`mv`: move) them into `dir` of pane `to`.
    DropExternal {
        to: Side,
        dir: VfsPath,
        paths: Vec<PathBuf>,
        mv: bool,
        /// Exactly the files of Manta's last drag out of the window.
        from_self: bool,
    },
    ToggleHidden(Side),
    OpenFavorite(usize),
    EditFavorite(usize),
    RemoveFavorite(usize),
    AddCurrentToFavorites,
    /// Folders dropped onto the favorites, inserted before index `at`
    /// (`conn`: the server / container they are on, `None` = local).
    AddFavoriteFolders {
        at: usize,
        conn: Option<u64>,
        folders: Vec<VfsPath>,
    },
    /// Re-sort: the favorite at `.0` goes before the one at `.1`.
    MoveFavorite(usize, usize),
    OpenLocal(PathBuf),
    /// Leave a server / container / bucket: back to the last local folder
    /// of that tab (or the home folder).
    GoLocal(Side),
    /// Read `/Volumes` and look for cloud folders again.
    RescanVolumes,
    /// Eject a volume (Devices, Ejectable).
    EjectVolume(PathBuf),
    /// Open a local folder (a volume) in a new tab of the active pane.
    OpenLocalInNewTab(PathBuf),
    /// "Show in Enclosing Folder" of a volume: a disk image's file, else
    /// the volume in `/Volumes`.
    RevealVolume(PathBuf),
    /// "Download Now" for the selection (cloud folders).
    CloudDownload(Side),
    /// "Remove Download" for the selection (cloud folders).
    CloudEvict(Side),
    /// Open a web page in the default browser (only `https://`).
    OpenWebLink(String),
    /// Hide the sidebar's "Connect Google Drive …" hint for good.
    DismissDriveHint,
    OpenConnection(u64),
    Disconnect(u64),
    ShowConnect,
    ToggleTransfers,
    ShowSync,
    SelectAll(Side),
    EditPath(Side),
    /// Open the "Go to Path" popup for the active tab of this pane.
    GoToPath(Side),
    FocusFilter(Side),
    /// Next pane, wrapping (Tab).
    SwitchPane,
    /// Add a pane right of this one, showing the same folder (⌘D).
    SplitPane(Side),
    /// Close this pane (⇧⌘D); refused for the last one.
    ClosePane(Side),
    /// Focus the neighbour pane: -1 left, 1 right (⌥⌘← / ⌥⌘→).
    FocusPane(isize),
    /// Compare & Sync with three or more panes: compare this pane as A or B.
    SetSyncPane(Side, crate::pane::SyncSlot),
    ShowPalette,
    ShowSettings,
    /// Restart external plugins and rescan the plugins folder.
    ReloadPlugins,
    /// The manual, at a chapter / section (`"05-dateien.md#pausieren"`,
    /// `""` for where it was).
    ShowHelp(String),
    /// Settings → General (the default-file-manager switch is there).
    ShowSettingsGeneral,
    /// Settings → Plugins → Discover (the community directory).
    BrowsePlugins,
    /// Manta as default file manager on (asks first, like at launch) or
    /// off (gives the role back) — palette and menu bar.
    ToggleDefaultViewer,
    /// Settings → General → Appearance, from the command palette.
    SetAppearance(crate::theme::Appearance),
    /// Run `plugin_id:command_id` on the active pane.
    RunPluginCommand(String),
    /// A click on a plugin's status bar item (plugin id, its action).
    StatusItem(String, manta_plugin::StatusAction),
    /// Open/close a plugin panel (plugin, panel, pane for PaneBottom).
    TogglePanel(String, String, Option<Side>),
    /// Reveal the diagnostics log in Finder.
    ShowLogFile,
    /// Go to another location (e.g. a breadcrumb outside the archive) and
    /// select the named entry there.
    NavigateTo(Side, Location, Option<String>),
    /// Open the selected archive as a folder (also `.docx`, `.jar` …).
    BrowseArchive(Side),
    /// Open the selection with the default app (⌥↩), archives included.
    OpenWithApp(Side),
    ExtractHere(Side),
    ExtractToOther(Side),
    /// Show the "Compress…" sheet for the selection.
    Compress(Side),
    /// Stop downloading / indexing an archive.
    CancelArchiveOpen(Side),
    /// Open a Docker container / pod in the active pane (sidebar).
    OpenContainer(Box<ConnectionConfig>),
    /// Show the pods of another namespace in the sidebar.
    SetKubeNamespace(String),
    /// Start, restart or stop a Docker container (sidebar context menu).
    DockerControl {
        id: String,
        name: String,
        op: DockerOp,
    },
    /// "Start Container" on the pane's "not running" page.
    StartContainer(Side),
    /// Open a shell inside the pane's container in the terminal.
    OpenContainerShell(Side),
    /// Show the pane as Icons, List, Columns or Gallery (⌘1 – ⌘4).
    SetView(Side, crate::views::ViewMode),
    /// Columns view: make `.1` the active column and select `.2` in it.
    ColumnsSelect(Side, VfsPath, Option<String>),
    /// Columns view →: open the folder under the cursor as the next column,
    /// its first item selected.
    ColumnsEnter(Side),
    /// Manta → Check for Updates… (Settings → Updates when updates are off).
    CheckForUpdates,
    /// Open the update window for the update on offer (toast, status bar).
    ShowUpdate,
    /// ⌘I: the Info window of the selection.
    GetInfo(Side),
    /// Add or remove a Finder tag on the selection (context menu).
    SetTag(Side, crate::tags::TagEdit),
    /// Sidebar: everything with this tag (a Spotlight search in the active
    /// pane).
    ShowTag(String),
    /// The scope bar: This Folder / Subfolders / This Mac.
    SetSearchScope(Side, crate::search::Scope),
    /// Stop the running search (the results stay).
    StopSearch(Side),
    /// Leave the search results: the folder again.
    EndSearch(Side),
    /// Search results: show the selected item in its folder.
    RevealResult(Side),
}

struct Connection {
    id: u64,
    label: String,
    config: ConnectionConfig,
    fs: SharedVfs,
    /// The backend's notice (`Vfs::notice`) has been shown as a toast.
    notice_shown: bool,
}

pub struct MantaApp {
    local: SharedVfs,
    /// The file panes, left to right (at least one).
    panes: Vec<Pane>,
    active: Side,
    /// Panes, most recently active first: the first one that isn't the
    /// active pane is the default copy / compare target.
    mru: Vec<Side>,
    /// Width of the pane area in the last frame (refuses a new pane that
    /// wouldn't fit).
    panes_width: f32,
    /// Where each pane was drawn in the last frame.
    pub(crate) pane_rects: Vec<(Side, egui::Rect)>,
    /// Where the toolbar's command buttons were drawn in the last frame.
    pub(crate) toolbar_rects: Vec<(Cmd, egui::Rect)>,
    /// Where the panes and their tabs are saved (`None` in tests).
    session_path: Option<PathBuf>,
    /// What was saved last (saved again only when it changes).
    saved_session: Option<crate::pane_set::Session>,
    /// When the session was last compared with what is saved.
    session_checked: Instant,
    next_id: u64,
    worker: Worker,
    transfers: TransferManager,
    /// Interrupted transfers: launch prompt and reconnects.
    resume: transfer_resume::ResumeState,
    /// Conflict questions of transfers (`ConflictPolicy::Ask`).
    conflicts: conflicts::ConflictState,
    transfers_view: TransfersView,
    show_transfers: bool,
    /// The status bar's progress pill while the panel is hidden.
    pub(crate) transfer_pill: crate::transfer_pill::TransferPill,
    jobs: Vec<JobStatus>,
    /// Directories to refresh when a job we enqueued finishes.
    job_refresh: HashMap<JobId, Vec<(SharedVfs, VfsPath)>>,
    /// File actions reported to plugins (start/finish, jobs behind them).
    actions: ActionTracker,
    connections: Vec<Connection>,
    favorites: Favorites,
    favorites_path: Option<PathBuf>,
    /// A favorite that was dropped again flashes (index, since when).
    /// Flashing favorite and when it started, on egui's clock (`None` until
    /// the next frame picks it up), so tests step it deterministically.
    fav_flash: Option<(usize, Option<f64>)>,
    /// Where the favorite rows were drawn in the last frame.
    pub(crate) fav_rects: Vec<egui::Rect>,
    locations: Vec<Place>,
    devices: Vec<Place>,
    dialog: Option<Dialog>,
    sync: SyncView,
    pub(crate) preview: Option<Preview>,
    /// The preview's native window (kept across previews for its position).
    pub(crate) preview_window: crate::aux_window::AuxWindow,
    toasts: Toasts,
    palette: CommandPalette,
    /// "Go to Path" popup (⇧⌘G), while open.
    goto: Option<GotoPopup>,
    /// Folders opened with "Go to Path", most recent first, per connection.
    goto_recent: Vec<(Option<u64>, VfsPath)>,
    /// Tab key presses intercepted in `raw_input_hook` (switch panes).
    /// Tab (+1) / ⇧Tab (-1) pressed while the file list had the keyboard.
    pending_tab_switch: isize,
    /// Keyboard focus to give back when the modal that is open now closes
    /// (`Some(None)`: the file list had it).
    focus_before_modal: Option<Option<egui::Id>>,
    /// The widget that had keyboard focus last frame (Esc then only leaves
    /// it; a dialog opening takes it away).
    last_focus: Option<egui::Id>,
    /// Focus to give back next frame (a dialog just closed).
    focus_restore: Option<Option<egui::Id>>,
    /// The first toolbar button that can take focus (⌃F5 goes there).
    toolbar_first: Option<egui::Id>,
    /// A press outside an open popup was swallowed: drop its release too.
    popup_swallow_release: bool,
    /// `modal_open()` when the menu bar was last updated.
    menu_modal: bool,
    plugins: PluginHost,
    settings: SettingsView,
    prefs: Prefs,
    prefs_path: Option<PathBuf>,
    /// Apple's Quick Look panel (Space on local files).
    pub(crate) quicklook: crate::quicklook::QuickLook,
    /// Hash of each pane's plugin-visible state when last sent.
    pane_sigs: HashMap<Side, u64>,
    /// Panel sizes (mirrors the host; saved when a drag ends).
    pub(crate) layout: PanelLayout,
    /// Plugin panel areas drawn in the last frame.
    plugin_panel_rects: Vec<egui::Rect>,
    /// A widget inside a plugin panel has keyboard focus.
    plugin_focus: bool,
    /// Selected tab of the right sidebar and of each pane's bottom area.
    sidebar_tab: Option<(String, String)>,
    bottom_tab: HashMap<Side, (String, String)>,
    /// The sidebar's Docker / Kubernetes section.
    pub(crate) containers: crate::containers::Containers,
    /// "Default file manager" (macOS): prefs, live state, first-run prompt.
    pub(crate) viewer: crate::default_viewer::DefaultViewer,
    /// When the app started (a request this early replaces the start folder).
    pub(crate) started: Instant,
    /// The first open / reveal request was handled (cold start is over).
    cold_open_used: bool,
    /// Tests: show the macOS type icons (off so renders don't change).
    pub(crate) test_type_icons: bool,
    /// Free space in the status bar, per folder.
    space: crate::space::SpaceCache,
    /// Tests: ask for free space (off so renders don't change).
    pub(crate) test_space: bool,
    /// Settings was open last frame (re-check the default state on open).
    settings_was_open: bool,
    /// Software updates: launch check, update window, Settings section.
    pub(crate) updates: crate::updates::Updates,
    /// The Help window (user manual).
    pub(crate) help: crate::help::Help,
    /// Thumbnails for the Icons, Columns and Gallery views.
    pub(crate) thumbs: crate::thumbs::Thumbs,
    /// "Remember view per folder" (Settings → General).
    pub(crate) folder_views: crate::views::FolderViews,
    folder_views_path: Option<PathBuf>,
    /// Moves local items to the Trash (⌘⌫); a fake in tests.
    pub(crate) trasher: crate::trash::SharedTrasher,
    /// The sidebar's Trash: its folder and full / empty state.
    pub(crate) trash: trash_place::TrashPlace,
    /// Undo and Redo (⌘Z, ⇧⌘Z) of Manta's own file operations.
    pub(crate) history: crate::undo::History,
    /// Where the Trash row was drawn in the last frame.
    pub(crate) trash_rect: Option<egui::Rect>,
    /// Secrets of favorites: the login Keychain (tests: memory).
    pub(crate) secrets: crate::secrets::SharedSecrets,
    /// Watches the visible local folders and reloads them on changes.
    pub(crate) live: crate::live_refresh::LiveRefresh,
    /// iCloud Drive / File Provider folders: sidebar section, sync badges.
    pub(crate) cloud: crate::cloud::Cloud,
    /// Finder tags of the visible folders and the user's tag list.
    pub(crate) tags: crate::tags::TagStore,
    /// The tag store version the menus last got (see `tags::set_known`).
    tags_known_version: u64,
    /// Spotlight, for searches on This Mac and the sidebar's tags.
    pub(crate) spotlight: Arc<dyn crate::search::Spotlight>,
    /// The Get Info windows.
    pub(crate) info: crate::info::InfoWindows,
    /// Web links opened (tests: instead of a browser).
    #[cfg(test)]
    pub(crate) opened_links: Vec<String>,
    /// When the menu bar's enabled commands were last updated.
    menu_checked: Option<Instant>,
    /// `open` commands run (tests: instead of running them).
    #[cfg(test)]
    pub(crate) launched: Vec<Vec<String>>,
    /// Test seam: job list shown instead of the transfer manager's snapshot.
    #[cfg(test)]
    fake_jobs: Option<Vec<JobStatus>>,
    /// "Manta quit unexpectedly last time" (a crash report from an earlier run).
    pub(crate) crash_notice: Option<crate::crash::Notice>,
}

/// Optional overrides for [`MantaApp::with_init`].
#[derive(Default)]
pub(crate) struct AppInit {
    /// Directory both panes start in (default: home).
    pub start_dir: Option<PathBuf>,
    /// Favorites to use instead of loading (and saving) favorites.json.
    pub favorites: Option<Favorites>,
    /// Sidebar (locations, devices) instead of scanning the system.
    pub places: Option<(Vec<Place>, Vec<Place>)>,
    /// Build the plugin host instead of the default one (built-ins plus
    /// the user's installed plugins).
    #[allow(clippy::type_complexity)]
    pub plugins: Option<Box<dyn FnOnce(&egui::Context) -> PluginHost>>,
    /// Preferences to use instead of loading (and saving) settings.json.
    pub prefs: Option<Prefs>,
    /// Quick Look backend instead of the native panel (tests).
    pub quicklook: Option<crate::quicklook::QuickLook>,
    /// Containers section (tests: fixed data). Default: live discovery,
    /// except in unit tests.
    pub containers: Option<crate::containers::Containers>,
    /// Update controller (tests: a fake engine). Default: the real one,
    /// except in unit tests (updates off there).
    pub updates: Option<crate::updates::Updates>,
    /// Where the panes are saved and restored from. Default: the user's
    /// `session.json`, except in unit tests (none: two fresh panes).
    pub session_path: Option<PathBuf>,
    /// Trash backend instead of the system Trash (tests).
    pub trasher: Option<crate::trash::SharedTrasher>,
    /// Where favorites' secrets are kept. Default: the login Keychain,
    /// except in unit tests (memory).
    pub secrets: Option<crate::secrets::SharedSecrets>,
    /// Live folder refresh. Default: on, except in unit tests.
    pub live_refresh: Option<bool>,
    /// Cloud folders and their sync states. Default: the user's (unit
    /// tests: none).
    pub cloud: Option<crate::cloud::Cloud>,
    /// Where the user's own Google OAuth client is saved (Settings → Cloud
    /// → Expert). Default: `gdrive-client.json` in the config folder;
    /// unit tests: none.
    pub gdrive_client_file: Option<PathBuf>,
    /// Whether Manta ran here before (decides the one-time "what's new"
    /// notice). Default: settings.json or session.json existed at launch;
    /// unit tests: `false` (no notice).
    pub existing_install: Option<bool>,
    /// Finder tags (tests: a fixed tag list). Default: Finder's.
    pub tags: Option<crate::tags::TagStore>,
    /// Spotlight (tests: a fake). Default: `mdfind`, except in unit tests.
    pub spotlight: Option<Arc<dyn crate::search::Spotlight>>,
    /// Get Info lookups (tests: a fake). Default: the system's, except in
    /// unit tests.
    pub info: Option<crate::info::InfoWindows>,
    /// Where crash reports are looked for at launch ("Manta quit
    /// unexpectedly last time"). Default: `~/Library/Logs/Manta`, except
    /// in unit tests (none).
    pub crash_dir: Option<PathBuf>,
}

/// The current one-time notice. Raise it for a new notice;
/// `Prefs::whats_new_seen` remembers the last one shown.
/// 1: Finder-like delete and shortcuts (0.2.0).
/// 2: copy by default, ask on conflicts, 7 languages, location chip, menu
///    bar (for installs coming from 0.2.x or earlier).
pub(crate) const WHATS_NEW: u32 = 2;

/// The user's cloud folders; none in unit tests.
fn default_cloud() -> crate::cloud::Cloud {
    #[cfg(test)]
    {
        crate::cloud::Cloud::fixed(
            Vec::new(),
            std::sync::Arc::new(crate::cloud::FakeProbe::default()),
        )
    }
    #[cfg(not(test))]
    {
        crate::cloud::Cloud::system()
    }
}

/// Finder's tag list and background reads; in unit tests no tag list and
/// inline reads (temp files only).
fn default_tags() -> crate::tags::TagStore {
    #[cfg(test)]
    {
        crate::tags::TagStore::new(Box::new(Vec::new), true)
    }
    #[cfg(not(test))]
    {
        crate::tags::TagStore::system()
    }
}

/// Spotlight (`mdfind`); a fake without results in unit tests.
fn default_spotlight() -> Arc<dyn crate::search::Spotlight> {
    #[cfg(test)]
    {
        Arc::new(crate::search::FakeSpotlight::default())
    }
    #[cfg(not(test))]
    {
        Arc::new(crate::search::MdFind)
    }
}

/// Info windows with the system's lookups; fixed answers in unit tests.
fn default_info() -> crate::info::InfoWindows {
    #[cfg(test)]
    {
        crate::info::InfoWindows::new(Arc::new(crate::info::FakeInfo::default()), true)
    }
    #[cfg(not(test))]
    {
        crate::info::InfoWindows::system()
    }
}

/// Items of `fs` can go to the macOS Trash: a writable local folder.
/// Servers, containers, cloud APIs and archives have no Trash.
pub(crate) fn can_trash(fs: &SharedVfs) -> bool {
    fs.protocol() == manta_core::Protocol::Local
        && !fs.is_read_only()
        && fs.local_path(&VfsPath::root()).is_some()
}

/// "New File": an empty file at `path`. `open_write` truncates, so a file
/// that appeared since the folder was listed is left alone.
pub(crate) fn create_empty_file(
    fs: &dyn manta_core::Vfs,
    path: &VfsPath,
) -> manta_core::VfsResult<()> {
    if fs.stat(path).is_ok() {
        return Err(manta_core::VfsError::AlreadyExists(path.to_string()));
    }
    let mut w = fs.open_write(path)?;
    // Uploading backends (S3, Drive, WebDAV) commit on flush.
    std::io::Write::flush(&mut w)?;
    Ok(())
}

fn local_vfs_path(p: &std::path::Path) -> VfsPath {
    VfsPath::new(&p.to_string_lossy())
}

/// The local folders `tab` shows (its folder; in the Columns view also the
/// columns left of it and the folder right of it). Empty for servers,
/// containers, cloud APIs and archives.
fn live_folders(tab: &Tab, view: crate::views::ViewMode) -> Vec<PathBuf> {
    let loc = &tab.loc;
    // Search results are no folder (they are pruned after file operations).
    if tab.in_search() {
        return Vec::new();
    }
    if loc.conn.is_some()
        || loc.archive.is_some()
        || loc.fs.protocol() != manta_core::Protocol::Local
    {
        return Vec::new();
    }
    let mut dirs = vec![loc.path.clone()];
    if view == crate::views::ViewMode::Columns {
        if let Some(root) = &tab.view_state.cols.root {
            dirs = crate::views::column_chain(root, &loc.path);
        }
        if tab.selected.len() <= 1 {
            if let Some(e) = tab.cursor_entry().filter(|e| e.is_dir()) {
                dirs.push(e.path.clone());
            }
        }
    }
    dirs.iter().filter_map(|d| loc.fs.local_path(d)).collect()
}

/// Start (re)loading the listing of a tab in the background.
/// `path: None` lists the backend's home directory.
fn start_listing(worker: &Worker, tab: &mut Tab, path: Option<VfsPath>) {
    tab.begin_loading();
    spawn_listing(worker, tab, path);
}

/// Reload a tab's folder without the loading indicators (live refresh: the
/// old listing stays until the new one replaces it, so nothing flickers).
fn start_quiet_listing(worker: &Worker, tab: &mut Tab) {
    let p = tab.loc.path.clone();
    spawn_listing(worker, tab, Some(p));
}

/// A folder being listed fresh shows what was read so far once it has
/// this many entries, or after [`PART_FIRST_AFTER`] (once; the complete
/// listing follows when it is read: more parts would cost the listing
/// thread a copy and a sort each, for little).
const PART_MIN: usize = 5_000;
const PART_FIRST_AFTER: Duration = Duration::from_millis(40);

fn spawn_listing(worker: &Worker, tab: &mut Tab, path: Option<VfsPath>) {
    tab.gen += 1;
    tab.listed_at = Some(Instant::now());
    let gen = tab.gen;
    let tab_id = tab.id;
    let fs = tab.loc.fs.clone();
    // A folder shown fresh shows its first part at once (very large
    // folders); a reload keeps the old listing until the new one is whole.
    let stream = tab.entries.is_empty() && path.is_some();
    let sort = tab.sort;
    let send = worker.messenger();
    worker.spawn(
        "list",
        move || {
            let dir = match path {
                Some(p) => p,
                None => worker::with_reconnect(&fs, || fs.home()).map_err(|e| e.to_string())?,
            };
            let start = Instant::now();
            let mut entries: Vec<manta_core::Entry> = Vec::new();
            let mut shown = !stream;
            let no_sizes = HashMap::new();
            // In the tab's order: sorting here keeps the UI thread's own
            // sort (of already sorted entries) cheap.
            let sorted = |mut v: Vec<manta_core::Entry>| {
                v.sort_unstable_by(|a, b| util::compare_entries(a, b, sort, &no_sizes));
                v
            };
            worker::with_reconnect(&fs, || {
                entries.clear();
                fs.list_parts(&dir, &mut |part| {
                    entries.extend(part);
                    if !shown && (entries.len() >= PART_MIN || start.elapsed() >= PART_FIRST_AFTER)
                    {
                        shown = true;
                        // Sorted in place: the final sort then finds most of
                        // the listing in order already.
                        entries = sorted(std::mem::take(&mut entries));
                        send(WorkResult::ListingPart {
                            tab_id,
                            gen,
                            entries: entries.clone(),
                        });
                    }
                })
            })
            .map_err(|e| e.to_string())?;
            Ok((dir, sorted(entries)))
        },
        move |result| WorkResult::Listing {
            tab_id,
            gen,
            result,
        },
    );
}

/// Change a tab's location (optionally recording history) and load it.
fn navigate_tab(worker: &Worker, tab: &mut Tab, loc: Location, home: bool, push_history: bool) {
    // A name being edited (or about to be) belongs to the folder left.
    tab.rename_click = None;
    tab.inline_rename = None;
    // Leaving the folder stops an archive that is still being opened.
    if let Some(o) = tab.opening.take() {
        o.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    // Going somewhere ends a search (and its scope, like Finder).
    if tab.in_search() {
        tab.end_search();
        tab.filter.clear();
    }
    tab.scope = crate::search::Scope::Folder;
    tab.search_due = None;
    if push_history {
        tab.back.push(tab.loc.clone());
        tab.forward.clear();
    }
    let same_dir = same_fs(&tab.loc.fs, &loc.fs) && tab.loc.path == loc.path;
    let path = loc.path.clone();
    tab.loc = loc;
    tab.path_edit = None;
    if !same_dir {
        tab.filter.clear();
        tab.entries.clear();
        tab.view.clear();
        tab.selected.clear();
        tab.cursor = None;
        tab.anchor = None;
    }
    tab.error = None;
    start_listing(worker, tab, if home { None } else { Some(path) });
}

impl MantaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Temp files of archives from earlier runs that didn't exit cleanly.
        let _ = std::thread::Builder::new()
            .name("manta-temp-cleanup".into())
            .spawn(manta_vfs::archive::clean_stale_temps);
        Self::with_init(&cc.egui_ctx, AppInit::default())
    }

    /// Build the app with optional overrides (used by tests to get a
    /// deterministic start directory, favorites and sidebar places).
    pub(crate) fn with_init(ctx: &egui::Context, init: AppInit) -> Self {
        let ctx = ctx.clone();
        egui_extras::install_image_loaders(&ctx);
        theme::install(&ctx);
        ctx.add_plugin(crate::a11y::SpeechCleanup);

        let local: SharedVfs = std::sync::Arc::new(manta_vfs::LocalFs::new());
        let secrets = init
            .secrets
            .clone()
            .unwrap_or_else(crate::secrets::default_store);
        let (favorites_path, mut favorites, fav_error) = match init.favorites {
            Some(f) => (None, f, None),
            None => {
                let favorites_path = Favorites::default_path();
                let (favorites, fav_error) = match favorites_path.as_deref().map(Favorites::load) {
                    Some(Ok(f)) => (f, None),
                    Some(Err(e)) => (
                        Favorites::default(),
                        Some(t!("error.load_favorites", error = e)),
                    ),
                    None => (Favorites::default(), None),
                };
                (favorites_path, favorites, fav_error)
            }
        };
        if let (Some(path), None) = (&favorites_path, &fav_error) {
            favorites::migrate_secrets(&mut favorites, path, &*secrets);
        }
        let (locations, devices) = init.places.unwrap_or_else(sidebar::discover_places);
        let mut prefs_existed = false;
        let (prefs, prefs_path) = match init.prefs {
            Some(p) => (p, None),
            None => {
                let path = Prefs::default_path();
                prefs_existed = path.as_deref().is_some_and(|p| p.exists());
                (path.as_deref().map(Prefs::load).unwrap_or_default(), path)
            }
        };
        // System is egui's default; leave the preference alone then (tests pick a theme).
        theme::apply_accent(&ctx, prefs.accent);
        if prefs.appearance != theme::Appearance::System {
            ctx.set_theme(prefs.appearance.preference());
        }
        crate::i18n::set_locale(prefs.language.resolve());
        let locale = prefs.language.resolve();
        if locale == crate::i18n::Locale::Zh {
            // The fonts were set up before the language was known.
            ctx.set_fonts(theme::font_definitions());
        }
        if !cfg!(test) {
            manta_plugin::set_global_locale(locale.code());
        }
        // Tests and renders never run docker / kubectl: they would list
        // (and screenshots would show) the developer's own containers.
        #[cfg(test)]
        {
            manta_vfs::container::block_cli_for_tests();
            manta_plugin_terminal::remote::block_cli_for_tests();
        }
        #[allow(unused_mut)]
        let mut plugins = match init.plugins {
            Some(make) => make(&ctx),
            None => {
                let mut host =
                    PluginHost::new(ctx.clone(), builtin_plugins(), HostPaths::default_paths());
                host.set_bundled(manta_plugin::bundled::embedded());
                host
            }
        };
        // Tests never contact the plugin directory on GitHub: a closed
        // local port (each test that needs a directory sets its own).
        #[cfg(test)]
        plugins.set_registry_url(TEST_REGISTRY_URL);
        let layout = plugins.layout();
        let ctx_for_viewer = ctx.clone();

        // Without a saved session the panes open in Settings → "New tabs
        // open in" (the home folder for "Same folder").
        let home = init
            .start_dir
            .clone()
            .or_else(|| prefs.new_tabs.dir())
            .or_else(dirs::home_dir)
            .map(|p| local_vfs_path(&p))
            .unwrap_or_else(VfsPath::root);
        let loc = Location {
            fs: local.clone(),
            conn: None,
            path: home,
            archive: None,
        };
        let session_path = match init.session_path {
            Some(p) => Some(p),
            None if cfg!(test) => None,
            None => crate::pane_set::Session::default_path(),
        };
        let session = session_path
            .as_deref()
            .and_then(crate::pane_set::Session::load);
        let mut next_id = 1;
        let (panes, active, mru) =
            Self::restore_panes(session.as_ref(), &loc, &local, &mut next_id);
        let mut app = MantaApp {
            local,
            panes,
            active,
            mru,
            panes_width: 0.0,
            pane_rects: Vec::new(),
            toolbar_rects: Vec::new(),
            session_path,
            saved_session: session,
            session_checked: Instant::now(),
            next_id,
            worker: Worker::new(ctx),
            transfers: transfer_resume::new_manager(),
            conflicts: Default::default(),
            resume: Default::default(),
            transfers_view: TransfersView::default(),
            show_transfers: false,
            transfer_pill: Default::default(),
            jobs: Vec::new(),
            job_refresh: HashMap::new(),
            connections: Vec::new(),
            favorites,
            favorites_path,
            fav_flash: None,
            fav_rects: Vec::new(),
            locations,
            devices,
            dialog: None,
            sync: SyncView::default(),
            preview: None,
            preview_window: Default::default(),
            toasts: Toasts::default(),
            palette: CommandPalette::default(),
            goto: None,
            goto_recent: Vec::new(),
            pending_tab_switch: 0,
            focus_before_modal: None,
            last_focus: None,
            focus_restore: None,
            toolbar_first: None,
            popup_swallow_release: false,
            menu_modal: false,
            plugins,
            settings: SettingsView::with_client_file(init.gdrive_client_file.clone().or_else(
                || {
                    if cfg!(test) {
                        None
                    } else {
                        manta_vfs::gdrive::OAuthClient::config_file()
                    }
                },
            )),
            prefs,
            prefs_path,
            quicklook: init
                .quicklook
                .unwrap_or_else(crate::quicklook::QuickLook::platform_default),
            pane_sigs: HashMap::new(),
            layout,
            plugin_panel_rects: Vec::new(),
            plugin_focus: false,
            sidebar_tab: None,
            bottom_tab: HashMap::new(),
            containers: init.containers.unwrap_or_else(|| {
                if cfg!(test) {
                    crate::containers::Containers::with_data(Default::default())
                } else {
                    crate::containers::Containers::default()
                }
            }),
            actions: ActionTracker::new(),
            viewer: crate::default_viewer::DefaultViewer::platform(),
            started: Instant::now(),
            cold_open_used: false,
            test_type_icons: false,
            space: Default::default(),
            test_space: false,
            settings_was_open: false,
            help: crate::help::Help::default(),
            updates: init.updates.unwrap_or_else(|| {
                if cfg!(test) {
                    crate::updates::Updates::new(
                        None,
                        Some(crate::updates::Disabled::NoKey),
                        Default::default(),
                        manta_updater::Version::new(0, 1, 0),
                    )
                } else {
                    crate::updates::Updates::for_app(&ctx_for_viewer)
                }
            }),
            thumbs: crate::thumbs::Thumbs::new(
                &ctx_for_viewer,
                if cfg!(test) {
                    None
                } else {
                    crate::thumbs::Thumbs::default_disk_dir()
                },
            ),
            folder_views: Default::default(),
            folder_views_path: None,
            trasher: init.trasher.clone().unwrap_or_else(crate::trash::system),
            trash: trash_place::TrashPlace::default(),
            history: Default::default(),
            trash_rect: None,
            secrets,
            live: crate::live_refresh::LiveRefresh::new(
                &ctx_for_viewer,
                init.live_refresh.unwrap_or(!cfg!(test)),
            ),
            cloud: init.cloud.unwrap_or_else(default_cloud),
            tags: init.tags.unwrap_or_else(default_tags),
            tags_known_version: 0,
            spotlight: init.spotlight.unwrap_or_else(default_spotlight),
            info: init.info.unwrap_or_else(default_info),
            #[cfg(test)]
            opened_links: Vec::new(),
            menu_checked: None,
            #[cfg(test)]
            launched: Vec::new(),
            #[cfg(test)]
            fake_jobs: None,
            crash_notice: init
                .crash_dir
                .clone()
                .or_else(|| {
                    if cfg!(test) {
                        None
                    } else {
                        crate::crash::dir()
                    }
                })
                .and_then(|d| crate::crash::Notice::take(&d)),
        };
        if !cfg!(test) {
            app.folder_views_path = crate::views::FolderViews::default_path();
            if let Some(p) = &app.folder_views_path {
                app.folder_views = crate::views::FolderViews::load(p);
            }
        }
        if !cfg!(test) {
            crate::default_viewer::set_repaint_context(&ctx_for_viewer);
            if !crate::default_viewer::selftest::active() {
                app.viewer.maybe_prompt();
            }
        }
        if let Some(e) = fav_error {
            app.set_error(e);
        }
        // `manta --plugin-dev <dir>` (or MANTA_PLUGIN_DEV=<dir>:<dir>):
        // run plugins straight from their folders for this session.
        if !cfg!(test) {
            let env = std::env::var("MANTA_PLUGIN_DEV").ok();
            for dir in crate::plugin_ui::plugin_dev_dirs(std::env::args().skip(1), env.as_deref()) {
                match app.plugins.add_dev_plugin(&dir) {
                    Ok(id) => {
                        log::info!("plugin dev mode: {id} from {}", dir.display());
                        app.toasts.push(
                            toast_kind(ToastLevel::Info),
                            t!("plugins.toast.dev_loaded", name = id),
                            None,
                        );
                    }
                    Err(e) => app.set_error(t!("plugins.toast.dev_failed", error = e)),
                }
            }
        }
        let existing = init
            .existing_install
            .unwrap_or(!cfg!(test) && (prefs_existed || app.saved_session.is_some()));
        app.check_whats_new(existing);
        let fresh = app.saved_session.is_none();
        for pane in &mut app.panes {
            for tab in &mut pane.tabs {
                if fresh {
                    tab.show_hidden = app.prefs.show_hidden;
                }
            }
            // Only the visible tab lists now; the others when shown.
            let p = pane.tab().loc.path.clone();
            start_listing(&app.worker, pane.tab_mut(), Some(p));
        }
        log::info!(
            "panes: {} at launch ({})",
            app.panes.len(),
            if fresh { "new" } else { "restored" }
        );
        app
    }

    /// The panes of `session` (local folders only), or the classic two
    /// panes at `home` when there is none (first launch, or a version with
    /// two fixed panes).
    fn restore_panes(
        session: Option<&crate::pane_set::Session>,
        home: &Location,
        local: &SharedVfs,
        next_id: &mut u64,
    ) -> (Vec<Pane>, Side, Vec<Side>) {
        let mut new_tab = |loc: Location| {
            let t = Tab::new(*next_id, loc);
            *next_id += 1;
            t
        };
        let Some(s) = session else {
            let panes = vec![
                Pane::new(PaneId::LEFT, new_tab(home.clone())),
                Pane::new(PaneId::RIGHT, new_tab(home.clone())),
            ];
            let mut panes = panes;
            for p in &mut panes {
                p.weight = 0.5;
            }
            return (panes, PaneId::LEFT, vec![PaneId::LEFT, PaneId::RIGHT]);
        };
        let panes: Vec<Pane> = s
            .panes
            .iter()
            .map(|sp| {
                let mut tabs: Vec<Tab> = sp
                    .tabs
                    .iter()
                    .map(|st| {
                        let mut t =
                            new_tab(Location::new(local.clone(), None, VfsPath::new(&st.path)));
                        t.show_hidden = st.show_hidden;
                        t
                    })
                    .collect();
                let first = tabs.remove(0);
                let mut pane = Pane::new(sp.id, first);
                pane.tabs.extend(tabs);
                pane.active = sp.active_tab.min(pane.tabs.len() - 1);
                pane.weight = sp.width;
                pane.view = sp.view;
                if let Some(s) = sp.icon_size {
                    pane.icon_size = crate::views::clamp_icon_size(s);
                }
                pane.column_widths = sp.column_widths.clone();
                pane
            })
            .collect();
        let active = s.active.unwrap_or(panes[0].id);
        let mut mru = s.mru.clone();
        for p in &panes {
            if !mru.contains(&p.id) {
                mru.push(p.id);
            }
        }
        crate::pane_set::touch_mru(&mut mru, active);
        (panes, active, mru)
    }

    /// The panes as they would be saved now.
    pub(crate) fn session(&self) -> crate::pane_set::Session {
        use crate::pane_set::{SavedPane, SavedTab, Session};
        let local_tabs = |p: &Pane| -> (Vec<SavedTab>, usize) {
            let mut v = Vec::new();
            let mut active = 0;
            for (i, t) in p.tabs.iter().enumerate() {
                // Servers, containers and archives need a connection.
                let loc = if t.loc.conn.is_none() && t.loc.archive.is_none() {
                    Some(&t.loc)
                } else {
                    t.back
                        .iter()
                        .rev()
                        .find(|l| l.conn.is_none() && l.archive.is_none())
                };
                let Some(loc) = loc else { continue };
                if i == p.active {
                    active = v.len();
                }
                v.push(SavedTab {
                    path: loc.path.to_string(),
                    show_hidden: t.show_hidden,
                });
            }
            (v, active)
        };
        let mut panes = Vec::new();
        for p in &self.panes {
            let (mut tabs, active_tab) = local_tabs(p);
            if tabs.is_empty() {
                let home = dirs::home_dir()
                    .map(|h| h.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".into());
                tabs.push(SavedTab {
                    path: home,
                    show_hidden: p.tab().show_hidden,
                });
            }
            panes.push(SavedPane {
                id: p.id,
                width: p.weight,
                tabs,
                active_tab,
                view: p.view,
                icon_size: Some(p.icon_size),
                column_widths: p.column_widths.clone(),
            });
        }
        Session {
            version: 1,
            panes,
            active: Some(self.active),
            mru: self.mru.clone(),
        }
    }

    /// Save the panes if they changed (cheap: compares with the last save).
    fn save_session(&mut self) {
        let Some(path) = self.session_path.clone() else {
            return;
        };
        let s = self.session();
        // Widths change on every drag frame: compare with some slack.
        let same = self.saved_session.as_ref().is_some_and(|old| {
            old.active == s.active
                && old.mru == s.mru
                && old.panes.len() == s.panes.len()
                && old.panes.iter().zip(&s.panes).all(|(a, b)| {
                    a.id == b.id
                        && a.tabs == b.tabs
                        && a.active_tab == b.active_tab
                        && a.view == b.view
                        && a.icon_size == b.icon_size
                        && a.column_widths == b.column_widths
                        && (a.width - b.width).abs() < 0.002
                })
        });
        if same {
            return;
        }
        if let Err(e) = s.save(&path) {
            log::warn!("session: couldn't save {}: {e}", path.display());
        }
        self.saved_session = Some(s);
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Position of a pane (0 = leftmost). An id that is gone (a late
    /// plugin request) maps to the active pane, so nothing panics.
    pub(crate) fn pos(&self, side: Side) -> usize {
        self.panes
            .iter()
            .position(|p| p.id == side)
            .or_else(|| self.panes.iter().position(|p| p.id == self.active))
            .unwrap_or(0)
    }

    pub(crate) fn has_pane(&self, side: Side) -> bool {
        self.panes.iter().any(|p| p.id == side)
    }

    /// The pane ids, left to right.
    pub(crate) fn pane_order(&self) -> Vec<Side> {
        self.panes.iter().map(|p| p.id).collect()
    }

    #[cfg(test)]
    pub(crate) fn active_pane(&self) -> Side {
        self.active
    }

    fn pane(&self, side: Side) -> &Pane {
        &self.panes[self.pos(side)]
    }

    fn pane_mut(&mut self, side: Side) -> &mut Pane {
        let i = self.pos(side);
        &mut self.panes[i]
    }

    /// Focus a pane (and remember it for the copy / compare target).
    pub(crate) fn activate(&mut self, side: Side) {
        if self.has_pane(side) {
            self.active = side;
            crate::pane_set::touch_mru(&mut self.mru, side);
        }
    }

    /// Where copy / move / compare go from `side` by default: the most
    /// recently active other pane (`None` with a single pane).
    pub(crate) fn target_of(&self, side: Side) -> Option<Side> {
        crate::pane_set::target_of(side, &self.mru, &self.pane_order())
    }

    /// "1", "2" … for the pane's position (as shown in pickers).
    pub(crate) fn pane_number(&self, side: Side) -> usize {
        self.pos(side) + 1
    }

    /// Tell the user once per connection when a backend reports something
    /// about it (e.g. FTPS continuing with clear data connections). The
    /// pane's location chip keeps showing it.
    fn poll_connection_notices(&mut self) {
        let mut toasts = Vec::new();
        for c in self.connections.iter_mut().filter(|c| !c.notice_shown) {
            if let Some(notice) = c.fs.notice() {
                c.notice_shown = true;
                match notice {
                    manta_core::ConnectionNotice::ClearDataChannel => toasts.push((
                        t!("status.ftps_clear_data").to_string(),
                        t!("status.ftps_clear_data.detail", label = c.label),
                    )),
                }
            }
        }
        for (title, detail) in toasts {
            self.toasts.push(ToastKind::Warning, title, Some(detail));
        }
    }

    fn set_info(&mut self, text: impl Into<String>) {
        self.toasts.push(ToastKind::Info, text, None);
    }

    fn set_success(&mut self, text: impl Into<String>) {
        self.toasts.push(ToastKind::Success, text, None);
    }

    fn set_error(&mut self, text: impl Into<String>) {
        let text = text.into();
        log::warn!("{text}");
        self.toasts
            .push(ToastKind::Error, crate::i18n::localize_error(&text), None);
    }

    /// Error toast with a short title and the technical detail underneath.
    fn set_error_detail(&mut self, title: impl Into<String>, detail: impl Into<String>) {
        let (title, detail) = (title.into(), detail.into());
        log::warn!("{title}: {detail}");
        self.toasts.push(
            ToastKind::Error,
            title,
            Some(crate::i18n::localize_error(&detail)),
        );
    }

    /// Where `loc` is, as the user sees it in the path bar: „Lokal“,
    /// „Google Drive“, „SFTP · deploy@web-01“ … (the chip's short label).
    pub(crate) fn place_label(&self, loc: &Location) -> String {
        self.place_chip(loc).label
    }

    /// The chip label plus the folder, for Compare & Sync: cloud folders show
    /// their path inside the provider folder instead of ~/Library/CloudStorage/….
    fn place_and_path(&self, loc: &Location) -> String {
        let chip = self.place_chip(loc);
        let path = match (&chip.cloud_root, loc.fs.local_path(&loc.path)) {
            (Some(root), Some(p)) => match p.strip_prefix(root) {
                Ok(rel) if rel.as_os_str().is_empty() => "/".to_string(),
                Ok(rel) => format!("/{}", rel.display()),
                Err(_) => loc.path.to_string(),
            },
            _ => loc.path.to_string(),
        };
        format!("{} {}", chip.label, path)
    }

    /// The path bar's location chip for `loc`.
    pub(crate) fn place_chip(&self, loc: &Location) -> crate::place::PlaceChip {
        let conn = loc
            .conn
            .and_then(|id| self.connections.iter().find(|c| c.id == id))
            .map(|c| (&c.config, c.label.as_str()));
        let local = if loc.conn.is_none() && loc.archive.is_none() {
            loc.fs.local_path(&loc.path)
        } else {
            None
        };
        let cloud = local.as_deref().and_then(|p| self.cloud.location_of(p));
        crate::place::place_chip(loc, conn, cloud)
    }

    /// A folder on the Mac outside archives and cloud folders ("Local").
    fn is_plain_local(&self, l: &Location) -> bool {
        l.conn.is_none()
            && l.archive.is_none()
            && l.fs
                .local_path(&l.path)
                .is_none_or(|p| self.cloud.location_of(&p).is_none())
    }

    fn conn_label(&self, conn: Option<u64>) -> String {
        match conn {
            None => t!("app.local").to_string(),
            Some(id) => self
                .connections
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.label.clone())
                .unwrap_or_else(|| t!("app.disconnected").into()),
        }
    }

    fn navigate(&mut self, side: Side, loc: Location, home: bool) {
        let worker = &self.worker;
        let pi = self.pos(side);
        let tab = self.panes[pi].tab_mut();
        navigate_tab(worker, tab, loc, home, true);
    }

    fn navigate_path(&mut self, side: Side, path: VfsPath) {
        let loc = self.pane(side).tab().loc.with_path(path);
        self.navigate(side, loc, false);
    }

    fn refresh(&mut self, side: Side) {
        if self.pane(side).tab().in_search() {
            self.restart_search(side);
            return;
        }
        let worker = &self.worker;
        let pi = self.pos(side);
        let tab = self.panes[pi].tab_mut();
        let p = tab.loc.path.clone();
        start_listing(worker, tab, Some(p));
    }

    /// Refresh every tab (in both panes) showing `dir` (or something below it) on `fs`.
    fn refresh_matching(&mut self, fs: &SharedVfs, dir: &VfsPath) {
        // Something changed in the Trash: the sidebar looks again.
        if fs.protocol() == manta_core::Protocol::Local {
            if let Some(d) = fs.local_path(dir) {
                self.trash.note_change(&d);
            }
        }
        self.prune_search_results(fs);
        let worker = &self.worker;
        for pane in &mut self.panes {
            for tab in &mut pane.tabs {
                if tab.in_search() {
                    continue;
                }
                if same_fs(&tab.loc.fs, fs) && tab.loc.path.starts_with(dir) {
                    let p = tab.loc.path.clone();
                    start_listing(worker, tab, Some(p));
                }
            }
        }
    }

    /// Live folder refresh, once per frame: watch the visible local
    /// folders, reload those whose changes have settled (see
    /// [`crate::live_refresh`]).
    fn live_refresh_frame(&mut self) {
        if !self.live.enabled() {
            return;
        }
        let now = Instant::now();
        // Every open tab is watched (FSEvents streams are cheap); background
        // tabs only remember that they changed (`Tab::stale`) and reload
        // once they are shown.
        let mut wanted: std::collections::BTreeSet<PathBuf> = self
            .panes
            .iter()
            .flat_map(|p| p.tabs.iter().flat_map(move |t| live_folders(t, p.view)))
            .collect();
        // New or removed cloud providers (an app was installed or signed out).
        let storage = manta_vfs::cloud_local::cloud_storage_dir().filter(|d| d.is_dir());
        wanted.extend(storage.clone());
        // The Trash (sidebar full / empty), when Manta may read it.
        let trash = self.trash.watch_dir();
        wanted.extend(trash.clone());
        // Mounted volumes ("Devices"): a disk image was opened, a drive
        // plugged in or ejected elsewhere.
        let volumes = std::path::Path::new(sidebar::VOLUMES_DIR);
        wanted.insert(volumes.to_path_buf());
        self.live.sync(wanted);
        self.live.poll(now);
        let (due, next) = self.live.due(now);
        if let Some(wait) = next {
            self.worker.ctx().request_repaint_after(wait);
        }
        for (dir, last) in due {
            if storage.as_ref() == Some(&dir) && self.cloud.rediscover() {
                log::debug!("live refresh: cloud folders changed");
            }
            self.cloud.invalidate(&dir);
            if trash.as_ref() == Some(&dir) {
                self.trash.note_change(&dir);
            }
            if dir.as_path() == volumes {
                log::debug!("live refresh: volumes changed");
                // Off the UI thread: asking a slow drive for its volume
                // keys or icon must not stall the window.
                self.worker.spawn(
                    "discover-places",
                    || Ok::<_, String>(sidebar::discover_places()),
                    |result| WorkResult::Places(result.ok()),
                );
            }
            for pane in &mut self.panes {
                let view = pane.view;
                let active = pane.active;
                for (i, tab) in pane.tabs.iter_mut().enumerate() {
                    if i != active
                        && !tab.listed_at.is_some_and(|t| t > last)
                        && live_folders(tab, view).contains(&dir)
                    {
                        tab.stale = Some(tab.stale.map_or(last, |s| s.max(last)));
                    }
                }
                let tab = pane.tab_mut();
                if !live_folders(tab, view).contains(&dir) {
                    continue;
                }
                // Not under a rename / new folder name being typed, nor over
                // a listing that is loading anyway: try again shortly.
                let naming = tab.inline_rename.is_some()
                    || matches!(&self.dialog, Some(Dialog::Name(f)) if f.tab_id == tab.id);
                if naming || tab.loading {
                    self.live.defer(&dir, last, now);
                    continue;
                }
                // A listing started after the last event already shows it.
                if tab.listed_at.is_some_and(|t| t > last) {
                    continue;
                }
                log::debug!("live refresh: {}", tab.loc.path);
                tab.stale = None;
                start_quiet_listing(&self.worker, tab);
            }
        }
        // A tab that changed in the background is shown again: catch up
        // (not under a name being typed, nor over a listing in progress).
        for pane in &mut self.panes {
            let tab = pane.tab_mut();
            let Some(last) = tab.stale else { continue };
            let naming = tab.inline_rename.is_some()
                || matches!(&self.dialog, Some(Dialog::Name(f)) if f.tab_id == tab.id);
            if naming || tab.loading {
                continue;
            }
            tab.stale = None;
            // A listing started after the last change already shows it.
            if tab.listed_at.is_none_or(|t| t > last) {
                continue;
            }
            log::debug!("live refresh: {} (was in the background)", tab.loc.path);
            start_quiet_listing(&self.worker, tab);
        }
    }

    /// Cloud folders, once per frame: read the sync states of the visible
    /// cloud folders in the background, hand them to their tabs, report
    /// finished downloads / evictions.
    fn cloud_frame(&mut self, ctx: &egui::Context) {
        let visible: Vec<PathBuf> = self
            .panes
            .iter()
            .filter_map(|p| {
                let loc = &p.tab().loc;
                (loc.conn.is_none() && loc.archive.is_none())
                    .then(|| loc.fs.local_path(&loc.path))
                    .flatten()
            })
            .collect();
        for done in self.cloud.frame(ctx, &visible) {
            use crate::cloud::CloudOp;
            let ok = done.count - done.errors.len();
            if !done.errors.is_empty() {
                let title = match done.op {
                    CloudOp::Download => t!("cloud.download_failed"),
                    CloudOp::Evict => t!("cloud.evict_failed"),
                };
                self.set_error_detail(title, done.errors.join("\n"));
            }
            if ok > 0 {
                match done.op {
                    CloudOp::Download => self.set_info(t!("cloud.download_started", count = ok)),
                    CloudOp::Evict => self.set_success(t!("cloud.evicted", count = ok)),
                }
            }
        }
        for pane in &mut self.panes {
            let tab = pane.tab_mut();
            let dir = (tab.loc.conn.is_none() && tab.loc.archive.is_none())
                .then(|| tab.loc.fs.local_path(&tab.loc.path))
                .flatten();
            tab.cloud = dir.and_then(|d| self.cloud.states(&d));
        }
    }

    /// Open an `https://` page in the default browser (tests: recorded).
    /// A button of the "Manta quit unexpectedly" notice.
    fn crash_choice(&mut self, choice: crate::crash::Choice) {
        use crate::crash::Choice;
        let Some(notice) = &self.crash_notice else {
            return;
        };
        match choice {
            Choice::Report => {
                let url = notice.issue_url();
                self.crash_notice = None;
                self.open_web_link(url);
            }
            // TextEdit (the default for plain text), not Manta itself.
            Choice::Show => {
                let path = notice.path.clone();
                self.launch(vec!["-t".into(), path.into_os_string()]);
            }
            Choice::Ignore => self.crash_notice = None,
        }
    }

    fn open_web_link(&mut self, url: String) {
        if !url.starts_with("https://") {
            return;
        }
        #[cfg(test)]
        self.opened_links.push(url);
        #[cfg(not(test))]
        if let Err(e) = open::that_detached(&url) {
            log::warn!("can't open {url}: {e}");
        }
    }

    /// "Download Now" / "Remove Download" for the selection of pane `side`.
    fn cloud_op(&mut self, side: Side, op: crate::cloud::CloudOp) {
        let tab = self.pane(side).tab();
        let fs = tab.loc.fs.clone();
        let paths: Vec<PathBuf> = tab
            .selected_entries()
            .iter()
            .filter_map(|e| fs.local_path(&e.path))
            .collect();
        if paths.is_empty() {
            return;
        }
        let ctx = self.worker.ctx().clone();
        self.cloud.run(&ctx, op, paths);
    }

    fn save_favorites(&mut self) {
        if let Some(path) = &self.favorites_path {
            if let Err(e) = self.favorites.save(path) {
                self.set_error_detail(t!("error.save_favorites"), e);
            }
        }
    }

    // ------------------------------------------------------------ workers

    fn poll_worker(&mut self) {
        for msg in self.worker.poll() {
            match msg {
                WorkResult::Listing {
                    tab_id,
                    gen,
                    result,
                } => {
                    let Some(tab) = self
                        .panes
                        .iter_mut()
                        .flat_map(|p| p.tabs.iter_mut())
                        .find(|t| t.id == tab_id)
                    else {
                        continue;
                    };
                    if tab.gen != gen {
                        continue; // stale
                    }
                    match result {
                        Ok((path, entries)) => tab.apply_listing(path, entries),
                        Err(e) => {
                            log::warn!(
                                "Listing failed: {} (backend {}): {e}",
                                tab.loc.path,
                                if tab.loc.conn.is_some() {
                                    "remote"
                                } else if tab.loc.archive.is_some() {
                                    "archive"
                                } else {
                                    "local"
                                }
                            );
                            tab.loading = false;
                            tab.error = Some(e);
                        }
                    }
                }
                WorkResult::ListingPart {
                    tab_id,
                    gen,
                    entries,
                } => {
                    if let Some(tab) = self
                        .panes
                        .iter_mut()
                        .flat_map(|p| p.tabs.iter_mut())
                        .find(|t| t.id == tab_id && t.gen == gen)
                    {
                        tab.apply_partial(entries);
                    }
                }
                WorkResult::Connected { request, result } => self.on_connected(*request, result),
                WorkResult::ArchiveOpened(msg) => self.on_archive_opened(*msg),
                WorkResult::Compare { gen, result } => {
                    if gen == self.sync.gen {
                        self.sync.loading = false;
                        match result {
                            Ok(items) => {
                                self.sync.items = Some(items);
                                self.sync.error = None;
                            }
                            Err(e) => self.sync.error = Some(e),
                        }
                    }
                }
                WorkResult::Preview { id, result } => {
                    if let Some(p) = self.preview.as_mut().filter(|p| p.id == id) {
                        p.loading = false;
                        match result {
                            Ok(c) => p.content = Some(c),
                            Err(e) => {
                                log::warn!("Preview failed: {}: {e}", p.entry.path);
                                p.error = Some(e);
                            }
                        }
                    }
                }
                WorkResult::HostKey {
                    info,
                    changed,
                    retry,
                } => self.on_host_key(*info, changed, retry),
                WorkResult::TestConnection { result } => {
                    if let Some(Dialog::Connect(f)) = &mut self.dialog {
                        f.testing = false;
                        f.test_result = Some(result);
                    }
                }
                WorkResult::GotoChecked {
                    gen,
                    new_tab,
                    result,
                } => {
                    let target = self.goto.as_mut().and_then(|g| g.on_checked(gen, result));
                    // Only close the popup on success; errors stay inline.
                    if let Some(t) = target {
                        if let Some(g) = self.goto.take() {
                            self.open_goto_target(g.side, g.conn, g.fs, t, new_tab);
                        }
                    }
                }
                WorkResult::GotoFolders {
                    gen,
                    dir_part,
                    result,
                } => {
                    if let Some(g) = &mut self.goto {
                        g.on_folders(gen, dir_part, result);
                    }
                }
                WorkResult::RenameMany {
                    plugin,
                    pane,
                    dir,
                    fs,
                    action,
                    outcome,
                } => {
                    let errors: Vec<String> = outcome
                        .failed
                        .iter()
                        .map(|(n, e)| format!("{n}: {e}"))
                        .collect();
                    if let Some(a) =
                        self.actions
                            .finish_with(action, outcome.ok.len() as u64, errors)
                    {
                        self.plugins.file_action(a);
                    }
                    // Keep the renamed items selected.
                    for tab in self.panes.iter_mut().flat_map(|p| p.tabs.iter_mut()) {
                        if same_fs(&tab.loc.fs, &fs) && tab.loc.path == dir {
                            // All at once: a swap renames onto selected names.
                            let moved: Vec<&String> = outcome
                                .ok
                                .iter()
                                .filter(|(o, _)| tab.selected.contains(o))
                                .map(|(_, n)| n)
                                .collect();
                            for (old, _) in &outcome.ok {
                                tab.selected.remove(old);
                            }
                            tab.selected.extend(moved.into_iter().cloned());
                        }
                    }
                    self.refresh_matching(&fs, &dir);
                    let result = RenameResult {
                        pane,
                        dir,
                        ok: outcome.ok,
                        failed: outcome.failed,
                    };
                    self.plugins
                        .deliver(&plugin, PluginEvent::RenameResult(result));
                }
                WorkResult::PluginFileRead { plugin, content } => {
                    self.plugins
                        .deliver(&plugin, PluginEvent::FileContent(content));
                }
                WorkResult::PluginListing {
                    plugin,
                    pane,
                    dir,
                    result,
                } => {
                    let (entries, error) = match result {
                        Ok(v) => {
                            let show_hidden = self.pane(pane).tab().show_hidden;
                            let mut v: Vec<ListedEntry> = v
                                .iter()
                                .map(|e| listed(e, show_hidden || !e.hidden))
                                .collect();
                            v.sort_by(|a, b| {
                                b.shown
                                    .cmp(&a.shown)
                                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                            });
                            (v, None)
                        }
                        Err(e) => (Vec::new(), Some(e)),
                    };
                    let l = DirListing {
                        pane,
                        dir,
                        entries,
                        error,
                    };
                    self.plugins.deliver(&plugin, PluginEvent::DirListing(l));
                }
                WorkResult::Trashed {
                    action,
                    refresh,
                    outcome,
                } => self.on_trashed(action, refresh, outcome),
                WorkResult::Undone {
                    redo,
                    refresh,
                    outcome,
                } => self.on_undone(redo, refresh, outcome),
                WorkResult::TrashMeasured(summary) => self.on_trash_measured(summary),
                WorkResult::TrashEmptied(result) => self.on_trash_emptied(result),
                WorkResult::Ejected { path, name, result } => self.on_ejected(path, name, result),
                WorkResult::Places(Some((l, d))) => {
                    self.locations = l;
                    self.devices = d;
                }
                WorkResult::Places(None) => {}
                WorkResult::VolumeRevealed { volume, image } => {
                    self.on_volume_revealed(volume, image)
                }
                WorkResult::Space { key, result } => {
                    if let Err(e) = &result {
                        log::debug!("free space of {}: {e}", key.1);
                    }
                    self.space
                        .answer(key, result.ok().flatten(), Instant::now());
                }
                WorkResult::DockerControlled {
                    id,
                    name,
                    op,
                    result,
                } => self.on_docker_controlled(id, name, op, result),
                WorkResult::FileOp {
                    description,
                    failure,
                    refresh,
                    select,
                    action,
                    result,
                    undo,
                } => {
                    if let Some(id) = action {
                        let outcome = result.clone().map(|()| 0);
                        self.finish_action(id, outcome);
                    }
                    if let (Ok(()), Some(op)) = (&result, undo) {
                        self.history.record(op);
                    }
                    match result {
                        Ok(()) => self.set_success(description),
                        Err(e) => self.set_error_detail(failure, e),
                    }
                    if let Some((tab_id, name)) = select {
                        if let Some(tab) = self
                            .panes
                            .iter_mut()
                            .flat_map(|p| p.tabs.iter_mut())
                            .find(|t| t.id == tab_id)
                        {
                            tab.pending_select = Some(name);
                        }
                    }
                    for (fs, dir) in refresh {
                        self.refresh_matching(&fs, &dir);
                    }
                }
            }
        }
    }

    /// Move `paths` (local items of `fs`) to the Trash in the background.
    fn trash_items(&mut self, side: Side, fs: SharedVfs, paths: Vec<VfsPath>) {
        let mut refresh: Vec<(SharedVfs, VfsPath)> = Vec::new();
        for p in &paths {
            let parent = p.parent().unwrap_or_else(VfsPath::root);
            if !refresh.iter().any(|(_, d)| *d == parent) {
                refresh.push((fs.clone(), parent));
            }
        }
        let local: Vec<PathBuf> = paths.iter().filter_map(|p| fs.local_path(p)).collect();
        let a = self.new_action(side, FileActionKind::Trash, &fs, paths, None);
        let action = self.start_action(a);
        let trasher = self.trasher.clone();
        self.worker.spawn(
            "trash",
            move || Ok(crate::trash::trash_all(trasher.as_ref(), &local)),
            move |result: Result<crate::trash::TrashOutcome, String>| {
                let outcome = result.unwrap_or_else(|e| crate::trash::TrashOutcome {
                    failed: vec![(PathBuf::new(), e)],
                    ..Default::default()
                });
                WorkResult::Trashed {
                    action,
                    refresh,
                    outcome,
                }
            },
        );
    }

    fn on_trashed(
        &mut self,
        action: u64,
        refresh: Vec<(SharedVfs, VfsPath)>,
        outcome: crate::trash::TrashOutcome,
    ) {
        self.trash.stale();
        if !outcome.put_back.is_empty() {
            self.history.record(Op::PutBack(outcome.put_back.clone()));
        }
        let name_of = |p: &std::path::Path| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        match outcome.failed.first() {
            None => {
                self.finish_action(action, Ok(outcome.moved.len() as u64));
                let n = outcome.moved.len();
                self.set_success(match outcome.moved.first() {
                    Some(p) if n == 1 => t!("status.trashed.single", name = name_of(p)),
                    _ => t!("status.trashed", count = n),
                });
            }
            Some((path, err)) => {
                self.finish_action(action, Err(err.clone()));
                let n = outcome.failed.len();
                let title = if n == 1 {
                    t!("error.trash.single", name = name_of(path))
                } else {
                    t!("error.trash", count = n)
                };
                self.set_error_detail(title, err.clone());
            }
        }
        for (fs, dir) in refresh {
            self.refresh_matching(&fs, &dir);
        }
    }

    /// ⌘Z (⇧⌘Z: `redo`): run the next step of the history in the
    /// background. Nothing happens while a step still runs.
    fn undo_redo(&mut self, redo: bool) {
        if self.history.busy {
            return;
        }
        let Some(op) = self.history.take(redo) else {
            self.set_info(if redo {
                t!("status.nothing_to_redo")
            } else {
                t!("status.nothing_to_undo")
            });
            return;
        };
        let refresh = op.refresh(&self.local);
        let trasher = self.trasher.clone();
        self.worker.spawn(
            if redo { "redo" } else { "undo" },
            move || Ok(op.run(trasher.as_ref())),
            move |result: Result<crate::undo::Outcome, String>| WorkResult::Undone {
                redo,
                refresh,
                outcome: result.unwrap_or_else(|e| crate::undo::Outcome {
                    error: Some((t!("error.undo_failed").to_string(), e)),
                    ..Default::default()
                }),
            },
        );
    }

    fn on_undone(
        &mut self,
        redo: bool,
        refresh: Vec<(SharedVfs, VfsPath)>,
        mut outcome: crate::undo::Outcome,
    ) {
        self.history.finish(redo, &mut outcome);
        self.trash.stale();
        match (outcome.error, outcome.done) {
            (Some((title, detail)), _) => self.set_error_detail(title, detail),
            (None, Some(done)) => self.set_success(done),
            (None, None) => {}
        }
        for (fs, dir) in refresh {
            self.refresh_matching(&fs, &dir);
        }
    }

    /// Show the one-time "what's new" notice after an update, or mark it
    /// seen right away on a new install.
    fn check_whats_new(&mut self, existing_install: bool) {
        if self.prefs.whats_new_seen >= WHATS_NEW {
            return;
        }
        if existing_install {
            if self.dialog.is_none() {
                self.dialog = Some(Dialog::WhatsNew(crate::dialogs::WhatsNewForm));
            }
        } else {
            self.mark_whats_new_seen();
        }
    }

    fn mark_whats_new_seen(&mut self) {
        self.prefs.whats_new_seen = WHATS_NEW;
        self.save_prefs();
    }

    fn save_prefs(&mut self) {
        if let Some(path) = &self.prefs_path {
            if let Err(e) = self.prefs.save(path) {
                self.set_error_detail(t!("error.save_settings"), e);
            }
        }
    }

    fn on_connected(
        &mut self,
        request: ConnectRequest,
        result: Result<(SharedVfs, VfsPath), String>,
    ) {
        let dialog_waiting = matches!(&self.dialog, Some(Dialog::Connect(f)) if f.connecting);
        self.finish_action(
            request.action,
            result.as_ref().map(|_| 0).map_err(Clone::clone),
        );
        match result {
            Err(e) => {
                if dialog_waiting {
                    if let Some(Dialog::Connect(f)) = &mut self.dialog {
                        f.connecting = false;
                        f.error = Some(e);
                    }
                } else {
                    let mut form = self.connect_form(ConnectForm::from_config(
                        &request.config,
                        request.side,
                        None,
                    ));
                    form.error = Some(e.clone());
                    self.dialog = Some(Dialog::Connect(form));
                    let (title, _) = dialogs::friendly_connect_error(
                        &e,
                        &request.config.host,
                        request.config.port,
                    );
                    self.set_error_detail(title, e);
                }
            }
            Ok((fs, home)) => {
                let editing = match &self.dialog {
                    Some(Dialog::Connect(f)) if dialog_waiting => f.editing_favorite,
                    _ => None,
                };
                if dialog_waiting {
                    self.dialog = None;
                }
                if let Some(fav_cfg) = request.favorite {
                    let fav = self.favorite_with_secret_in_keychain(fav_cfg, request.keychain);
                    match editing {
                        Some(i) if i < self.favorites.items.len() => {
                            self.forget_replaced_secret(i, &fav);
                            self.favorites.items[i] = fav;
                        }
                        _ => self.favorites.add(fav),
                    }
                    self.save_favorites();
                }
                let id = self.next_id();
                let label = fs.label();
                // Never in a crash report.
                for s in [&label, &request.config.host, &request.config.username] {
                    crate::crash::note_sensitive(s);
                }
                self.set_success(t!("status.connected", label = label));
                // Transfers waiting for this server continue.
                self.transfers
                    .provide_connection(&manta_ops::Endpoint::remote(&request.config), fs.clone());
                self.connections.push(Connection {
                    id,
                    label,
                    config: request.config,
                    fs: fs.clone(),
                    notice_shown: false,
                });
                self.activate(request.side);
                self.navigate(
                    request.side,
                    Location {
                        fs,
                        conn: Some(id),
                        path: home,
                        archive: None,
                    },
                    false,
                );
            }
        }
    }

    /// The favorite to save for `cfg`: a secret it carries goes to the
    /// Keychain (the favorite only keeps the marker). `in_keychain`: the
    /// secret is already there (connected with it). If the Keychain refuses,
    /// the secret is not saved at all (never in plain text).
    fn favorite_with_secret_in_keychain(
        &mut self,
        cfg: ConnectionConfig,
        in_keychain: bool,
    ) -> Favorite {
        let account = crate::secrets::account(&cfg);
        match favorites::secret_of(&cfg) {
            Some(secret) => match self.secrets.set(&account, secret) {
                Ok(()) => Favorite::Connection {
                    config: favorites::strip_secrets(&cfg),
                    keychain: true,
                },
                Err(e) => {
                    log::warn!(
                        "favorites: cannot save the secret of {account} in the Keychain: {e}"
                    );
                    self.set_error_detail(t!("error.keychain_save"), e);
                    Favorite::Connection {
                        config: favorites::strip_secrets(&cfg),
                        keychain: false,
                    }
                }
            },
            None => Favorite::Connection {
                config: cfg,
                keychain: in_keychain,
            },
        }
    }

    /// Favorite `i` is replaced by `new` (edited) or removed: delete its
    /// Keychain item unless `new` or another favorite still uses it.
    fn forget_replaced_secret(&mut self, i: usize, new: &Favorite) {
        let Some(Favorite::Connection {
            config,
            keychain: true,
        }) = self.favorites.items.get(i)
        else {
            return;
        };
        let account = crate::secrets::account(config);
        let kept = matches!(new, Favorite::Connection { config: n, keychain: true } if crate::secrets::account(n) == account);
        if !kept && !self.favorites.keychain_in_use(&account, Some(i)) {
            if let Err(e) = self.secrets.delete(&account) {
                log::warn!("favorites: cannot delete the Keychain item of {account}: {e}");
            }
        }
    }

    /// A connection attempt hit an unknown or changed SFTP host key: ask
    /// (unknown) or warn (changed). The Connect dialog, if open, comes back
    /// afterwards.
    fn on_host_key(&mut self, info: manta_core::HostKeyInfo, changed: bool, retry: HostKeyRetry) {
        let refused: String = if changed {
            t!("hostkey.refused").into()
        } else {
            t!("hostkey.cancelled").into()
        };
        if let HostKeyRetry::Connect(r) = &retry {
            self.finish_action(r.action, Err(refused));
        }
        log::info!(
            "SFTP host key {} for {}: {} {}",
            if changed { "CHANGED" } else { "unknown" },
            info.host_label(),
            info.key_type,
            info.fingerprint
        );
        let back = match self.dialog.take() {
            Some(Dialog::Connect(mut f)) => {
                f.connecting = false;
                f.testing = false;
                Some(Box::new(f))
            }
            Some(Dialog::HostKey(old)) => old.back,
            other => {
                // Another question is open: keep it, don't stack dialogs.
                if other.is_some() {
                    self.dialog = other;
                    self.set_error(if changed {
                        t!("hostkey.refused")
                    } else {
                        t!("hostkey.cancelled")
                    });
                    return;
                }
                None
            }
        };
        self.dialog = Some(Dialog::HostKey(Box::new(HostKeyForm {
            info,
            changed,
            retry,
            back,
        })));
    }

    /// The user answered the host key dialog (`accept`: "Connect and
    /// Remember"; never for a changed key).
    fn host_key_answered(&mut self, form: HostKeyForm, accept: bool) {
        let HostKeyForm {
            info,
            changed,
            retry,
            back,
        } = form;
        let accept = accept && !changed;
        if accept {
            if let Err(e) = manta_vfs::known_hosts::trust_host_key(&info) {
                log::warn!("known_hosts: {e}");
                if matches!(e, manta_core::VfsError::HostKeyMismatch(_)) {
                    // Changed in the meantime: never connect.
                    self.on_host_key(info, true, retry);
                    return;
                }
                self.set_error_detail(t!("hostkey.not_saved"), e.to_string());
            }
        }
        let mut back = back.map(|b| *b);
        if !accept {
            let msg = if changed {
                t!("hostkey.refused")
            } else {
                t!("hostkey.cancelled")
            };
            match back.as_mut() {
                Some(f) => f.error = Some(msg.into()),
                None => self.set_error(msg),
            }
            self.dialog = back.map(Dialog::Connect);
            return;
        }
        match retry {
            HostKeyRetry::Connect(r) => {
                if let Some(f) = back.as_mut() {
                    f.connecting = true;
                    f.error = None;
                }
                self.dialog = back.map(Dialog::Connect);
                self.start_connect(r.config, r.favorite, r.side, r.keychain);
            }
            HostKeyRetry::Test => {
                self.dialog = back.map(Dialog::Connect);
                self.test_connection();
            }
            HostKeyRetry::Reconnect(ep) => {
                self.dialog = back.map(Dialog::Connect);
                let ctx = self.worker.ctx().clone();
                self.reconnect_endpoint(&ep, true, &ctx);
            }
        }
    }

    /// Connect in the background and open the server in pane `side`.
    /// `keychain`: take the secret from the Keychain if `config` has none
    /// (a favorite saved with "Remember password").
    fn start_connect(
        &mut self,
        config: ConnectionConfig,
        favorite: Option<ConnectionConfig>,
        side: Side,
        keychain: bool,
    ) {
        let ep = RemoteEndpoint {
            protocol: config.protocol,
            user: config.username.clone(),
            host: config.host.clone(),
            port: config.port,
            context: config
                .container
                .as_ref()
                .map(|c| c.context.clone())
                .unwrap_or_default(),
        };
        let id = self.actions.next_id();
        let mut a = FileAction::new(id, side, FileActionKind::Connect, Vec::new());
        if let Some(p) = config
            .initial_path
            .as_deref()
            .filter(|p| !p.trim().is_empty())
        {
            a.sources.push(VfsPath::new(p));
        }
        a.src_local = false;
        // S3: the "user" is an access key ID, not worth showing.
        a.src_label = if config.username.is_empty() || config.protocol == manta_core::Protocol::S3 {
            format!("{} ({})", config.host, config.protocol)
        } else {
            format!("{}@{} ({})", config.username, config.host, config.protocol)
        };
        a.src_remote = Some(ep);
        self.start_action(a);
        let request = ConnectRequest {
            config: config.clone(),
            favorite,
            side,
            action: id,
            keychain,
        };
        let secrets = keychain.then(|| self.secrets.clone());
        self.set_info(t!("status.connecting", host = config.host));
        // Google Drive (Expert, own OAuth client) without a stored sign-in:
        // sign in with the browser first; the account it gives replaces
        // the one in the request.
        let signed_in: std::sync::Arc<std::sync::Mutex<Option<ConnectionConfig>>> =
            Default::default();
        let signed_in_tx = signed_in.clone();
        self.worker.spawn(
            "connect",
            move || {
                let mut config = match secrets {
                    Some(store) => crate::secrets::fill_from(&*store, config)?,
                    None => config,
                };
                if config.protocol == manta_core::Protocol::GoogleDrive
                    && (config.username.is_empty()
                        || !manta_vfs::gdrive::has_sign_in(&config.username))
                {
                    let open_url = |url: &str| {
                        if let Err(e) = open::that_detached(url) {
                            log::warn!("Google Drive sign-in: can't open the browser: {e}");
                        }
                    };
                    let signed =
                        manta_vfs::gdrive::sign_in(open_url, &manta_vfs::gdrive::AuthCancel::new())
                            .map_err(|e| e.to_string())?;
                    config.username = signed.username.clone();
                    if config.name.trim().is_empty() {
                        config.name = signed.name.clone();
                    }
                    *signed_in_tx.lock().unwrap() = Some(config.clone());
                }
                let fs = match Checked::from_result(manta_vfs::connect(&config))? {
                    Checked::Done(fs) => fs,
                    Checked::Ask { info, changed } => return Ok(Checked::Ask { info, changed }),
                };
                let home = match config.initial_path.as_deref() {
                    Some(p) if !p.trim().is_empty() => VfsPath::new(p),
                    _ => fs.home().map_err(|e| e.to_string())?,
                };
                Ok(Checked::Done((fs, home)))
            },
            move |result| {
                let mut request = request;
                if let Some(cfg) = signed_in.lock().unwrap().take() {
                    request.favorite = request.favorite.map(|f| ConnectionConfig {
                        username: cfg.username.clone(),
                        name: cfg.name.clone(),
                        ..f
                    });
                    request.config = cfg;
                }
                match result {
                    Ok(Checked::Ask { info, changed }) => WorkResult::HostKey {
                        info,
                        changed,
                        retry: HostKeyRetry::Connect(Box::new(request)),
                    },
                    Ok(Checked::Done(v)) => WorkResult::Connected {
                        request: Box::new(request),
                        result: Ok(v),
                    },
                    Err(e) => WorkResult::Connected {
                        request: Box::new(request),
                        result: Err(e),
                    },
                }
            },
        );
    }

    /// Enqueue on the transfer manager and remember which dirs to refresh.
    fn enqueue(&mut self, job: Job, refresh: Vec<(SharedVfs, VfsPath)>) -> JobId {
        let src = self.transfer_endpoint(&job.src_fs);
        let dst = job.dst_fs.as_ref().map(|f| self.transfer_endpoint(f));
        let id = self.transfers.enqueue_with_endpoints(job, src, dst);
        self.job_refresh.insert(id, refresh);
        self.show_transfers = true;
        id
    }

    // ------------------------------------------------------------ file actions

    /// Report a started file action to plugins and remember it.
    fn start_action(&mut self, a: FileAction) -> u64 {
        let id = a.id;
        let a = self.actions.start(a);
        self.plugins.file_action(a);
        id
    }

    /// Report the end of a file action (`Ok(files)` or `Err(error)`).
    fn finish_action(&mut self, id: u64, outcome: Result<u64, String>) {
        if let Some(a) = self.actions.finish(id, Some(outcome)) {
            self.plugins.file_action(a);
        }
    }

    /// The server behind `fs`, if it is one of our connections.
    fn endpoint(&self, fs: &SharedVfs) -> Option<RemoteEndpoint> {
        self.connections
            .iter()
            .find(|c| same_fs(&c.fs, fs))
            .map(|c| RemoteEndpoint {
                protocol: c.config.protocol,
                user: c.config.username.clone(),
                host: c.config.host.clone(),
                port: c.config.port,
                context: c
                    .config
                    .container
                    .as_ref()
                    .map(|o| o.context.clone())
                    .unwrap_or_default(),
            })
    }

    /// A new `Started` action on `src_fs` (and `dst`, if any).
    fn new_action(
        &mut self,
        side: Side,
        kind: FileActionKind,
        src_fs: &SharedVfs,
        sources: Vec<VfsPath>,
        dst: Option<(&SharedVfs, VfsPath)>,
    ) -> FileAction {
        let id = self.actions.next_id();
        let mut a = FileAction::new(id, side, kind, sources);
        // An archive is read through the app, not as OS paths.
        a.src_local = src_fs.protocol() == manta_core::Protocol::Local && !src_fs.is_read_only();
        a.src_label = src_fs.label();
        a.src_remote = if a.src_local {
            None
        } else {
            self.endpoint(src_fs)
        };
        match dst {
            Some((fs, path)) => {
                a.destination = Some(path);
                a.dst_local = fs.protocol() == manta_core::Protocol::Local && !fs.is_read_only();
                a.dst_label = Some(fs.label());
                a.dst_remote = if a.dst_local { None } else { self.endpoint(fs) };
            }
            None => a.dst_local = a.src_local,
        }
        a
    }

    /// Which pane a tab belongs to.
    /// Any number of panes (one after closing the other, three or more);
    /// a tab that is gone (its pane closed meanwhile): the active pane.
    fn side_of_tab(&self, tab_id: u64) -> Side {
        self.panes
            .iter()
            .find(|p| p.tabs.iter().any(|t| t.id == tab_id))
            .map_or(self.active, |p| p.id)
    }

    /// Report finished transfer jobs of tracked actions.
    fn finish_job_actions(&mut self) {
        let tracked = self.actions.tracked_jobs();
        if tracked.is_empty() {
            return;
        }
        let mut done = Vec::new();
        for id in tracked {
            match self.jobs.iter().find(|j| j.id == id) {
                Some(j) if transfers::is_finished(&j.state) => {
                    if let Some(a) = self.actions.job_finished(j) {
                        done.push(a);
                    }
                }
                Some(_) => {}
                None => {
                    if let Some(a) = self.actions.forget_job(id) {
                        done.push(a);
                    }
                }
            }
        }
        for a in done {
            self.plugins.file_action(a);
        }
    }

    fn process_finished_jobs(&mut self) {
        self.finish_job_actions();
        let finished: Vec<JobId> = self
            .jobs
            .iter()
            .filter(|j| transfers::is_finished(&j.state) && self.job_refresh.contains_key(&j.id))
            .map(|j| j.id)
            .collect();
        for id in finished {
            if let Some(dirs) = self.job_refresh.remove(&id) {
                for (fs, dir) in dirs {
                    self.refresh_matching(&fs, &dir);
                }
            }
            self.space.stale();
            if let Some(j) = self.jobs.iter().find(|j| j.id == id) {
                match &j.state {
                    manta_ops::JobState::Failed(e) => {
                        let (title, e) = (crate::i18n::localize_job_title(&j.title), e.clone());
                        self.set_error_detail(t!("error.couldnt_finish", title = title), e);
                    }
                    manta_ops::JobState::Done if j.errors.is_empty() => {
                        let msg = crate::i18n::localize_job_title(&j.title);
                        self.set_success(msg);
                    }
                    manta_ops::JobState::Done => {
                        for e in &j.errors {
                            log::warn!("Job “{}”: {e}", j.title);
                        }
                        let n = j.errors.len();
                        let detail = t!("toast.items_failed", count = n);
                        let title = crate::i18n::localize_job_title(&j.title);
                        self.toasts.push(
                            ToastKind::Error,
                            t!("toast.finished_with_issues", title = title),
                            Some(detail),
                        );
                    }
                    _ => {}
                }
            }
        }
        // Forget jobs that disappeared (e.g. cleared) without being seen as finished.
        let ids: Vec<JobId> = self.jobs.iter().map(|j| j.id).collect();
        self.job_refresh.retain(|id, _| ids.contains(id));
    }

    // ------------------------------------------------------------ actions

    fn handle_action(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::Activate(side) => self.activate(side),
            Action::Navigate(side, path) => {
                self.activate(side);
                self.navigate_path(side, path);
            }
            Action::OpenSelected(side) => self.open_selected(side),
            Action::NavigateTo(side, loc, select) => self.navigate_to(side, loc, select),
            Action::BrowseArchive(side) => {
                if let [e] = self.pane(side).tab().selected_entries().as_slice() {
                    let e = e.clone();
                    if !self.maybe_browse_archive(side, &e, true) {
                        self.set_info(t!("status.select_archive"));
                    }
                }
            }
            Action::OpenWithApp(side) => self.open_selected_with(side, false),
            Action::ExtractHere(side) => self.extract_here(side),
            Action::ExtractToOther(side) => self.extract_to_other(side),
            Action::Compress(side) => self.show_compress(side),
            Action::CancelArchiveOpen(side) => self.cancel_archive_open(side),
            Action::Up(side) if self.pane(side).tab().in_search() => self.reveal_result(side),
            Action::Back(side) if self.pane(side).tab().in_search() => self.end_search(side),
            Action::NewFolder(side) | Action::NewFile(side)
                if self.pane(side).tab().in_search() =>
            {
                self.set_info(t!("search.not_here"));
            }
            Action::Up(side) if self.leave_archive(side) => {}
            Action::Up(side) => {
                let tab = self.pane(side).tab();
                if let Some(parent) = tab.loc.path.parent() {
                    let child = tab.loc.path.file_name().to_string();
                    if self.pane(side).view == crate::views::ViewMode::Columns {
                        self.columns_go(side, parent, Some(child), false);
                    } else {
                        self.navigate_path(side, parent);
                        self.pane_mut(side).tab_mut().pending_select = Some(child);
                    }
                }
            }
            Action::SetView(side, mode) => self.set_view(side, mode),
            Action::ColumnsSelect(side, dir, name) => {
                self.activate(side);
                let tab = self.pane_mut(side).tab_mut();
                if tab.loc.path == dir {
                    match name.and_then(|n| tab.view_index_of(&n)) {
                        Some(i) => tab.select_only(i),
                        None => tab.selected.clear(),
                    }
                } else {
                    self.columns_go(side, dir, name, false);
                }
            }
            Action::ColumnsEnter(side) => {
                let tab = self.pane(side).tab();
                if let Some(e) = tab.cursor_entry().filter(|e| e.is_dir()) {
                    let path = e.path.clone();
                    self.columns_go(side, path, None, true);
                }
            }
            Action::Back(side) => {
                let worker = &self.worker;
                let pi = self.pos(side);
                let tab = self.panes[pi].tab_mut();
                if let Some(loc) = tab.back.pop() {
                    let current = tab.loc.clone();
                    let child = current.path.clone();
                    tab.forward.push(current.clone());
                    navigate_tab(worker, tab, loc, false, false);
                    // Re-select the folder we came from if it is a child
                    // (or the archive we were in).
                    let from_archive = current.archive.as_ref().filter(|m| {
                        same_fs(&m.outer.fs, &tab.loc.fs) && m.outer.path == tab.loc.path
                    });
                    if let Some(m) = from_archive {
                        tab.pending_select = Some(m.file_name.clone());
                    } else if child.parent().as_ref() == Some(&tab.loc.path) {
                        tab.pending_select = Some(child.file_name().to_string());
                    }
                }
            }
            Action::Forward(side) => {
                let worker = &self.worker;
                let pi = self.pos(side);
                let tab = self.panes[pi].tab_mut();
                if let Some(loc) = tab.forward.pop() {
                    tab.back.push(tab.loc.clone());
                    navigate_tab(worker, tab, loc, false, false);
                }
            }
            Action::Refresh(side) => self.refresh(side),
            Action::NewTab(side) => match self.prefs.new_tabs.dir() {
                // Settings → "New tabs open in" (like Finder's "New Finder
                // windows show"); "Same folder" copies the tab.
                Some(dir) => {
                    self.activate(side);
                    self.open_local_in_new_tab(dir);
                }
                None => {
                    let id = self.next_id();
                    let pane = self.pane_mut(side);
                    let tab = pane.tab().duplicate(id);
                    pane.tabs.push(tab);
                    pane.active = pane.tabs.len() - 1;
                    self.activate(side);
                    self.refresh(side);
                }
            },
            Action::CloseTab(side, i) => {
                let pane = self.pane_mut(side);
                if pane.tabs.len() > 1 && i < pane.tabs.len() {
                    pane.tabs.remove(i);
                    if pane.active >= pane.tabs.len() || pane.active > i {
                        pane.active = pane.active.saturating_sub(1);
                    }
                }
            }
            Action::SwitchTab(side, i) => {
                let pane = self.pane_mut(side);
                if i < pane.tabs.len() {
                    pane.active = i;
                }
                self.activate(side);
            }
            Action::Preview(side) => self.open_preview(ctx, side),
            Action::CopyToOther(side, true) if self.refuse_app_folders(side, true) => {}
            Action::CopyToOther(side, mv) => {
                let Some(to) = self.target_of(side) else {
                    self.set_info(t!("panes.needs_two"));
                    return;
                };
                let tab = self.pane(side).tab();
                let paths: Vec<VfsPath> =
                    tab.selected_entries().into_iter().map(|e| e.path).collect();
                let fs = tab.loc.fs.clone();
                self.confirm_transfer(side, fs, paths, to, mv, false);
            }
            Action::DropFiles {
                ref fs,
                ref paths,
                mv: true,
                ..
            } if fs.protocol() == manta_core::Protocol::Local
                && self.refuse_app_folder_paths(paths, true) => {}
            Action::DropFiles {
                from,
                to,
                fs,
                paths,
                mv,
            } => {
                self.activate(to);
                self.confirm_transfer(from, fs, paths, to, mv, true);
            }
            Action::DropExternal {
                to,
                dir,
                paths,
                mv,
                from_self,
            } => self.drop_external(to, dir, paths, mv, from_self),
            Action::Rename(side) if self.refuse_read_only(side, ReadOnlyOp::Rename) => {}
            Action::NewFolder(side) if self.refuse_read_only(side, ReadOnlyOp::NewFolder) => {}
            Action::NewFile(side) if self.refuse_read_only(side, ReadOnlyOp::NewFile) => {}
            Action::NewIn { side, file, .. }
                if self.refuse_read_only(
                    side,
                    if file {
                        ReadOnlyOp::NewFile
                    } else {
                        ReadOnlyOp::NewFolder
                    },
                ) => {}
            Action::Delete(side) | Action::DeletePermanently(side)
                if self.refuse_read_only(side, ReadOnlyOp::Delete)
                    || self.refuse_app_folders(side, true) => {}
            Action::Rename(side) | Action::RenameInPlace(side)
                if self.refuse_app_folders(side, false) => {}
            Action::DropOnTrash {
                ref fs, ref paths, ..
            } if fs.protocol() == manta_core::Protocol::Local
                && self.refuse_app_folder_paths(paths, true) => {}
            Action::Rename(side) => {
                let tab = self.pane(side).tab();
                let sel = tab.selected_entries();
                if sel.len() > 1 && self.batch_rename_available() {
                    self.activate(side);
                    let state = self.pane_state(side);
                    self.plugins.run_command(BATCH_RENAME_CMD, &state);
                    return;
                }
                match sel.len() {
                    0 => self.set_info(t!("status.select_rename")),
                    1 => {
                        if let Some(form) = self.rename_form(side) {
                            self.dialog = Some(Dialog::Name(form));
                        }
                    }
                    _ => self.set_info(t!("status.select_single_rename")),
                }
            }
            Action::RenameInPlace(side) => self.rename_in_place(side),
            Action::InlineRenameEnd {
                side,
                commit,
                enter,
            } => self.inline_rename_end(side, commit, enter),
            Action::NewFolder(side) => {
                let tab = self.pane(side).tab();
                let existing = tab.entries.iter().map(|e| e.name.clone()).collect();
                let form = NameForm::new(
                    tab.id,
                    tab.loc.fs.clone(),
                    tab.loc.path.clone(),
                    None,
                    existing,
                );
                self.dialog = Some(Dialog::Name(form));
            }
            Action::NewFile(side) => {
                let tab = self.pane(side).tab();
                let existing = tab.entries.iter().map(|e| e.name.clone()).collect();
                let form =
                    NameForm::new_file(tab.id, tab.loc.fs.clone(), tab.loc.path.clone(), existing);
                self.dialog = Some(Dialog::Name(form));
            }
            Action::NewIn { side, dir, file } => {
                let tab = self.pane(side).tab();
                let fs = tab.loc.fs.clone();
                // Names in that folder for the default name and the clash
                // check: on this Mac only (a server listing would block);
                // elsewhere creating an existing name fails with a message.
                let existing: Vec<String> = match tab.loc.conn {
                    None => fs
                        .list(&dir)
                        .map(|l| l.into_iter().map(|e| e.name).collect())
                        .unwrap_or_default(),
                    Some(_) => Vec::new(),
                };
                let mut form = if file {
                    NameForm::new_file(tab.id, fs, dir, existing)
                } else {
                    NameForm::new(tab.id, fs, dir, None, existing)
                };
                form.inside = true;
                self.dialog = Some(Dialog::Name(form));
            }
            Action::Delete(side) | Action::DeletePermanently(side) => {
                let permanent = matches!(action, Action::DeletePermanently(_));
                let tab = self.pane(side).tab();
                let paths: Vec<VfsPath> =
                    tab.selected_entries().into_iter().map(|e| e.path).collect();
                let fs = tab.loc.fs.clone();
                // In the Trash, ⌘⌫ deletes immediately (asks), like Finder.
                let in_trash = self.all_in_trash(&fs, &paths);
                if paths.is_empty() {
                    self.set_info(t!("status.select_delete"));
                } else if !permanent && can_trash(&fs) && !in_trash {
                    self.trash_items(side, fs, paths);
                } else {
                    let folders = self.folder_paths(&fs, &paths);
                    self.dialog = Some(Dialog::Delete(DeleteForm {
                        side,
                        no_trash: !can_trash(&fs),
                        in_trash,
                        fs,
                        paths,
                        folders,
                    }));
                }
            }
            Action::DropOnTrash { from, fs, paths } => self.drop_on_trash(from, fs, paths),
            Action::TrashExternal(paths) => self.trash_external(paths),
            Action::EmptyTrash => self.ask_empty_trash(),
            Action::Undo => self.undo_redo(false),
            Action::Redo => self.undo_redo(true),
            Action::ShowTrashInFinder => self.show_trash_in_finder(),
            Action::OpenFullDiskAccess => self.open_full_disk_access(),
            Action::ToggleHidden(side) => {
                let tab = self.pane_mut(side).tab_mut();
                tab.show_hidden = !tab.show_hidden;
                tab.rebuild_view();
            }
            Action::OpenFavorite(i) => {
                let Some(fav) = self.favorites.items.get(i).cloned() else {
                    return;
                };
                match fav {
                    Favorite::Local { path, .. } => {
                        self.handle_action(ctx, Action::OpenLocal(PathBuf::from(path)))
                    }
                    Favorite::Connection { config, keychain } => {
                        self.open_connection_favorite(self.active, i, config, keychain);
                    }
                }
            }
            Action::EditFavorite(i) => {
                if let Some(Favorite::Connection { config, keychain }) = self.favorites.items.get(i)
                {
                    let mut form =
                        self.connect_form(ConnectForm::from_config(config, self.active, Some(i)));
                    form.set_stored_secret(*keychain);
                    self.dialog = Some(Dialog::Connect(form));
                }
            }
            Action::RemoveFavorite(i) => {
                if i < self.favorites.items.len() {
                    let gone = Favorite::Local {
                        name: String::new(),
                        path: String::new(),
                    };
                    self.forget_replaced_secret(i, &gone);
                    self.favorites.items.remove(i);
                    self.save_favorites();
                }
            }
            Action::AddCurrentToFavorites if self.pane(self.active).tab().loc.archive.is_some() => {
                self.set_info(t!("status.fav_archive"));
            }
            Action::AddCurrentToFavorites => {
                let tab = self.pane(self.active).tab();
                let fav = match tab
                    .loc
                    .conn
                    .and_then(|id| self.connections.iter().find(|c| c.id == id))
                {
                    None => Favorite::Local {
                        name: tab.title(),
                        path: tab.loc.path.to_string(),
                    },
                    Some(c) => {
                        let mut cfg = favorites::strip_secrets(&c.config);
                        cfg.initial_path = Some(tab.loc.path.to_string());
                        if cfg.name.is_empty() {
                            cfg.name = format!("{} – {}", c.label, tab.title());
                        }
                        let keychain = self
                            .favorites
                            .keychain_in_use(&crate::secrets::account(&cfg), None);
                        Favorite::Connection {
                            config: cfg,
                            keychain,
                        }
                    }
                };
                let name = fav.display_name();
                self.favorites.add(fav);
                self.save_favorites();
                self.set_success(t!("status.fav_added", name = name));
            }
            Action::AddFavoriteFolders { at, conn, folders } => {
                self.add_favorite_folders(at, conn, folders)
            }
            Action::MoveFavorite(from, to) => {
                if self.favorites.move_item(from, to).is_some() {
                    self.save_favorites();
                }
            }
            Action::GoLocal(side) => {
                // The last folder on the Mac (outside archives and cloud
                // folders) in the history, else home.
                let tab = self.pane(side).tab();
                let last_local = tab
                    .back
                    .iter()
                    .rev()
                    .find(|l| self.is_plain_local(l))
                    .map(|l| l.path.clone());
                let path = last_local.unwrap_or_else(|| {
                    local_vfs_path(&dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")))
                });
                let loc = Location {
                    fs: self.local.clone(),
                    conn: None,
                    path,
                    archive: None,
                };
                self.navigate(side, loc, false);
            }
            Action::OpenLocal(path) => {
                let loc = Location {
                    fs: self.local.clone(),
                    conn: None,
                    path: local_vfs_path(&path),
                    archive: None,
                };
                self.navigate(self.active, loc, false);
            }
            Action::OpenContainer(config) => {
                let config = *config;
                let side = self.active;
                let existing = self.connections.iter().find(|c| {
                    c.config.protocol == config.protocol
                        && c.config.host == config.host
                        && c.config.username == config.username
                        && c.config.container == config.container
                });
                match existing {
                    Some(c) => {
                        let loc = Location {
                            fs: c.fs.clone(),
                            conn: Some(c.id),
                            path: VfsPath::root(),
                            archive: None,
                        };
                        self.navigate(side, loc, true);
                    }
                    None => self.start_connect(config, None, side, false),
                }
            }
            Action::SetKubeNamespace(ns) => self.containers.set_namespace(ns),
            Action::DockerControl { id, name, op } => self.docker_control(id, name, op),
            Action::StartContainer(side) => self.start_pane_container(side),
            Action::OpenContainerShell(side) => self.open_container_shell(side),
            Action::CloudDownload(side) => self.cloud_op(side, crate::cloud::CloudOp::Download),
            Action::CloudEvict(side) => self.cloud_op(side, crate::cloud::CloudOp::Evict),
            Action::OpenWebLink(url) => self.open_web_link(url),
            Action::DismissDriveHint => {
                self.prefs.gdrive_hint_dismissed = true;
                self.save_prefs();
            }
            Action::RescanVolumes => {
                let (l, d) = sidebar::discover_places();
                self.locations = l;
                self.devices = d;
                self.cloud.rediscover();
                self.tags.reload_finder();
            }
            Action::EjectVolume(path) => self.eject_volume(path),
            Action::OpenLocalInNewTab(path) => self.open_local_in_new_tab(path),
            Action::RevealVolume(path) => self.reveal_volume(path),
            Action::OpenConnection(id) => {
                if let Some(c) = self.connections.iter().find(|c| c.id == id) {
                    let loc = Location {
                        fs: c.fs.clone(),
                        conn: Some(id),
                        path: VfsPath::root(),
                        archive: None,
                    };
                    self.navigate(self.active, loc, true);
                }
            }
            Action::Disconnect(id) => self.disconnect(id),
            Action::ShowConnect => {
                self.dialog = Some(Dialog::Connect(
                    self.connect_form(ConnectForm::new(self.active)),
                ))
            }
            Action::ToggleTransfers => self.show_transfers = !self.show_transfers,
            Action::ShowSync if self.panes.len() < 2 => self.set_info(t!("panes.needs_two")),
            Action::ShowSync => {
                if !self.sync.open {
                    // Fresh window: compare the active pane with its target.
                    self.sync.pick = None;
                }
                self.sync.open = true;
                self.sync.focus();
                self.start_compare();
            }
            Action::SplitPane(side) => self.split_pane(side),
            Action::ClosePane(side) => self.close_pane(side),
            Action::FocusPane(step) => {
                if let Some(p) = crate::pane_set::neighbour(&self.pane_order(), self.active, step) {
                    self.activate(p);
                }
            }
            Action::SetSyncPane(side, slot) => {
                if self.sync.open && self.has_pane(side) {
                    let (a, b) = self.sync_pair().unwrap_or((side, side));
                    let pick = match slot {
                        crate::pane::SyncSlot::A if side == b => (b, a),
                        crate::pane::SyncSlot::A => (side, b),
                        crate::pane::SyncSlot::B if side == a => (b, a),
                        crate::pane::SyncSlot::B => (a, side),
                    };
                    self.sync.pick = Some(pick);
                    self.start_compare();
                }
            }
            Action::SelectAll(side) => self.pane_mut(side).tab_mut().select_all(),
            Action::EditPath(side) => {
                self.activate(side);
                let tab = self.pane_mut(side).tab_mut();
                tab.path_edit = Some(tab.loc.path.to_string());
            }
            Action::GoToPath(side) => self.show_goto(side),
            Action::FocusFilter(side) => {
                self.activate(side);
                // A narrow pane opens its folded filter field first.
                self.pane_mut(side).tab_mut().filter_focus = true;
                ctx.memory_mut(|m| m.request_focus(pane::filter_field_id(side)));
            }
            Action::SwitchPane => {
                self.activate(crate::pane_set::cycle(&self.pane_order(), self.active, 1))
            }
            Action::ShowPalette => self.palette.show(),
            Action::ShowSettings => self.settings.show(None),
            Action::ReloadPlugins => {
                self.plugins.reload_all();
                self.toasts.push(
                    toast_kind(ToastLevel::Info),
                    t!("plugins.toast.reloaded"),
                    None,
                );
            }
            Action::ShowHelp(target) => self.help.show(&target),
            Action::ToggleDefaultViewer => {
                self.viewer.refresh();
                self.viewer.toggle();
            }
            Action::ShowSettingsGeneral => self
                .settings
                .show(Some(crate::settings::SettingsTab::General)),
            Action::BrowsePlugins => self.settings.show_discover(),
            Action::CheckForUpdates => {
                if !self.updates.check_now() {
                    // Not set up in this build: Settings → General → Updates says why.
                    self.settings
                        .show(Some(crate::settings::SettingsTab::General));
                    self.settings.reveal_updates = true;
                }
            }
            Action::ShowUpdate => self.updates.show(),
            Action::GetInfo(side) => self.get_info(side),
            Action::SetTag(side, edit) => self.set_tag(side, edit),
            Action::ShowTag(name) => self.show_tag(name),
            Action::SetSearchScope(side, scope) => self.set_search_scope(side, scope),
            Action::StopSearch(side) => {
                if let Some(s) = self.pane_mut(side).tab_mut().search.as_mut() {
                    s.stop();
                }
            }
            Action::EndSearch(side) => self.end_search(side),
            Action::RevealResult(side) => self.reveal_result(side),
            Action::SetAppearance(a) => {
                let before = self.prefs.clone();
                self.prefs.appearance = a;
                self.prefs_changed(ctx, before);
            }
            Action::ShowLogFile => match crate::logging::reveal_log_file() {
                Ok(path) => log::info!("Revealed log file {}", path.display()),
                Err(e) => self.set_error_detail(t!("error.show_log"), e),
            },
            Action::RunPluginCommand(id) => {
                let state = self.pane_state(self.active);
                self.plugins.run_command(&id, &state);
            }
            Action::StatusItem(plugin, action) => match action {
                manta_plugin::StatusAction::Command { command } => {
                    let state = self.pane_state(self.active);
                    self.plugins
                        .run_command(&format!("{plugin}:{command}"), &state);
                }
                // Checked by the host (`StatusAction::is_valid`): a scheme
                // or a bundle id, never an option for `open`.
                manta_plugin::StatusAction::OpenUrl { url } => self.launch(vec![url.into()]),
                manta_plugin::StatusAction::OpenApp { bundle_id } => {
                    self.launch(vec!["-b".into(), bundle_id.into()])
                }
            },
            Action::TogglePanel(plugin, panel, side) => {
                let pane = side;
                let open = self.plugins.is_panel_open(&plugin, &panel, pane);
                self.plugins.set_panel_open(&plugin, &panel, pane, !open);
                if !open {
                    let sel = (plugin, panel);
                    match side {
                        Some(s) => {
                            self.bottom_tab.insert(s, sel);
                        }
                        None => self.sidebar_tab = Some(sel),
                    }
                }
            }
        }
    }

    /// Add folders dropped onto the sidebar as favorites, before index `at`.
    /// Local folders become local favorites; folders on a server or in a
    /// container become favorites of that connection, starting in the
    /// folder. A folder that is already a favorite isn't added again: its
    /// row flashes instead.
    pub(crate) fn add_favorite_folders(
        &mut self,
        at: usize,
        conn: Option<u64>,
        folders: Vec<VfsPath>,
    ) {
        let mut at = at;
        let mut added: Vec<String> = Vec::new();
        let mut existing: Option<Favorite> = None;
        for path in folders {
            let folder = path.file_name().to_string();
            let fav = match conn {
                None => Favorite::Local {
                    name: if folder.is_empty() {
                        "/".into()
                    } else {
                        folder
                    },
                    path: path.to_string(),
                },
                Some(id) => {
                    let Some(c) = self.connections.iter().find(|c| c.id == id) else {
                        continue;
                    };
                    let mut cfg = favorites::strip_secrets(&c.config);
                    let base = if cfg.name.trim().is_empty() {
                        c.label.clone()
                    } else {
                        cfg.name.clone()
                    };
                    cfg.name = if folder.is_empty() {
                        base
                    } else {
                        format!("{base} – {folder}")
                    };
                    cfg.initial_path = Some(path.to_string());
                    let keychain = self
                        .favorites
                        .keychain_in_use(&crate::secrets::account(&cfg), None);
                    Favorite::Connection {
                        config: cfg,
                        keychain,
                    }
                }
            };
            let name = fav.display_name();
            match self.favorites.insert_at(at, fav.clone()) {
                Ok(i) => {
                    at = i + 1;
                    added.push(name);
                }
                Err(_) => existing = Some(fav),
            }
        }
        if let Some(i) = existing.and_then(|f| self.favorites.position_of(&f)) {
            self.fav_flash = Some((i, None));
            if added.is_empty() {
                let name = self.favorites.items[i].display_name();
                self.set_info(t!("status.fav_exists", name = name));
            }
        }
        match added.as_slice() {
            [] => {}
            [one] => {
                let msg = t!("status.fav_added", name = one);
                self.save_favorites();
                self.set_success(msg);
            }
            many => {
                let msg = t!("status.fav_added_n", count = many.len());
                self.save_favorites();
                self.set_success(msg);
            }
        }
    }

    /// The flashing favorite and how strong (two soft pulses, ~0.9 s).
    fn fav_flash_now(&mut self, ctx: &egui::Context) -> Option<(usize, f32)> {
        const DUR: f32 = 0.9;
        let now = ctx.input(|inp| inp.time);
        let (i, start) = self.fav_flash.as_mut()?;
        let i = *i;
        let t = (now - *start.get_or_insert(now)) as f32;
        if t >= DUR {
            self.fav_flash = None;
            return None;
        }
        ctx.request_repaint();
        // Reduce motion: a steady highlight instead of the pulses.
        if crate::a11y::reduce_motion() {
            return Some((i, 1.0));
        }
        let pulse = (t / DUR * 2.0 * std::f32::consts::PI).cos() * -0.5 + 0.5;
        Some((i, pulse.max(0.15) * (1.0 - t / DUR).sqrt()))
    }

    /// Show `side` as `mode`. With "Remember view per folder" the folder
    /// keeps it.
    pub(crate) fn set_view(&mut self, side: Side, mode: crate::views::ViewMode) {
        self.activate(side);
        let remember = self.prefs.remember_view_per_folder;
        let key = self.folder_view_key(side);
        let pane = self.pane_mut(side);
        if pane.view == mode {
            return;
        }
        log::info!("view: pane {side} → {mode:?}");
        pane.view = mode;
        pane.view_folder = Some(key.clone());
        let tab = pane.tab_mut();
        tab.scroll_to_cursor = true;
        tab.view_state.band = None;
        // The columns start at the folder shown now.
        tab.view_state.cols.root = None;
        if remember {
            self.folder_views.set(key, mode);
            if let Some(p) = &self.folder_views_path {
                if let Err(e) = self.folder_views.save(p) {
                    log::warn!("view: couldn't save folder views: {e}");
                }
            }
        }
    }

    /// The "Remember view per folder" key of the folder `side` shows.
    fn folder_view_key(&self, side: Side) -> String {
        let loc = &self.pane(side).tab().loc;
        let origin = if loc.conn.is_some() || loc.archive.is_some() {
            loc.fs.label()
        } else {
            String::new()
        };
        crate::views::FolderViews::key(&origin, &loc.path)
    }

    /// "Remember view per folder": a pane that shows another folder now
    /// switches to the view that folder had.
    fn apply_folder_views(&mut self) {
        if !self.prefs.remember_view_per_folder {
            return;
        }
        for i in 0..self.panes.len() {
            let side = self.panes[i].id;
            let key = self.folder_view_key(side);
            if self.panes[i].view_folder.as_deref() == Some(key.as_str()) {
                continue;
            }
            if let Some(mode) = self.folder_views.get(&key) {
                self.panes[i].view = mode;
                self.panes[i].tab_mut().scroll_to_cursor = true;
            }
            self.panes[i].view_folder = Some(key);
        }
    }

    /// Columns view: make `dir` the active column, selecting `select` (or
    /// the first item). A folder listed for a column already shows at once
    /// while it loads again.
    fn columns_go(&mut self, side: Side, dir: VfsPath, select: Option<String>, first: bool) {
        self.activate(side);
        self.navigate_path(side, dir.clone());
        let tab = self.pane_mut(side).tab_mut();
        tab.pending_select = select;
        tab.pending_select_first = first;
        if let Some(entries) = tab.view_state.cols.cached(&tab.loc.fs, &dir) {
            tab.apply_listing(dir, entries);
            // The real listing is still on its way.
            tab.begin_loading();
            tab.scroll_to_cursor = true;
        }
    }

    // ------------------------------------------------------------ panes

    /// Add a pane right of `side`, showing the same folder (and the same
    /// bottom panels). Refused with a toast when the window is too narrow
    /// for one more pane of [`crate::pane_set::MIN_PANE_W`].
    pub(crate) fn split_pane(&mut self, side: Side) {
        let n = self.panes.len();
        if self.panes_width > 0.0 && !crate::pane_set::can_add(self.panes_width, n, PANE_GAP) {
            log::info!(
                "panes: no room for pane {} ({} pt wide)",
                n + 1,
                self.panes_width
            );
            self.toasts.push(
                ToastKind::Warning,
                t!("panes.no_room"),
                Some(t!(
                    "panes.no_room.detail",
                    min = crate::pane_set::MIN_PANE_W as u32
                )),
            );
            return;
        }
        let at = self.pos(side);
        let id = crate::pane_set::next_free_id(&self.pane_order());
        let tab_id = self.next_id();
        let src = &self.panes[at];
        let mut pane = Pane::new(id, src.tab().duplicate(tab_id));
        pane.col_widths = src.col_widths;
        pane.view = src.view;
        pane.icon_size = src.icon_size;
        pane.column_widths = src.column_widths.clone();
        let mut weights: Vec<f32> = self.panes.iter().map(|p| p.weight).collect();
        crate::pane_set::split_weights(&mut weights, at);
        self.panes.insert(at + 1, pane);
        for (p, w) in self.panes.iter_mut().zip(weights) {
            p.weight = w;
        }
        self.plugins.copy_pane_panels(self.panes[at].id, id);
        if let Some(t) = self.bottom_tab.get(&self.panes[at].id).cloned() {
            self.bottom_tab.insert(id, t);
        }
        self.activate(id);
        self.refresh(id);
        log::info!(
            "panes: split pane {} → {} panes (new id {id})",
            at + 1,
            self.panes.len()
        );
        self.save_session();
    }

    /// Close `side` (never the last pane): its plugin state goes (the
    /// terminal hangs up), focus moves to the most recently active pane.
    pub(crate) fn close_pane(&mut self, side: Side) {
        if self.panes.len() < 2 {
            self.set_info(t!("panes.last_pane"));
            return;
        }
        let Some(at) = self.panes.iter().position(|p| p.id == side) else {
            return;
        };
        let pane = self.panes.remove(at);
        for t in &pane.tabs {
            // An archive still being opened there stops.
            if let Some(o) = &t.opening {
                o.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        let mut weights: Vec<f32> = self.panes.iter().map(|p| p.weight).collect();
        weights.insert(at, pane.weight);
        crate::pane_set::remove_weight(&mut weights, at);
        for (p, w) in self.panes.iter_mut().zip(weights) {
            p.weight = w;
        }
        self.mru.retain(|p| *p != side);
        self.pane_sigs.remove(&side);
        self.bottom_tab.remove(&side);
        if self.active == side {
            let next = self
                .mru
                .first()
                .copied()
                .filter(|p| self.has_pane(*p))
                .unwrap_or(self.panes[at.min(self.panes.len() - 1)].id);
            self.activate(next);
        }
        if self.sync.pick.is_some_and(|(a, b)| a == side || b == side) {
            self.sync.pick = None;
        }
        if self.goto.as_ref().is_some_and(|g| g.side == side) {
            self.goto = None;
        }
        // A dialog about the pane's items is moot now.
        let stale = match &self.dialog {
            Some(Dialog::Transfer(f)) => f.src_side == side || f.dst_side == side,
            Some(Dialog::Delete(f)) => f.side == side,
            Some(Dialog::Name(f)) => pane.tabs.iter().any(|t| t.id == f.tab_id),
            _ => false,
        };
        if stale {
            self.dialog = None;
        }
        if let Some(Dialog::Connect(f)) = &mut self.dialog {
            if f.side == side {
                f.side = self.active;
            }
        }
        self.plugins.close_pane(side);
        log::info!(
            "panes: closed pane {} (id {side}) → {} panes",
            at + 1,
            self.panes.len()
        );
        if self.sync.open {
            if self.panes.len() < 2 {
                self.sync.open = false;
            } else {
                self.start_compare();
            }
        }
        self.save_session();
    }

    /// The pane to mark as the copy target: with three or more panes, while
    /// the active pane has a selection (with two the target is obvious).
    pub(crate) fn target_highlight(&self) -> Option<Side> {
        if self.panes.len() < 3 || self.pane(self.active).tab().selected.is_empty() {
            return None;
        }
        self.target_of(self.active)
    }

    /// The two panes Compare & Sync compares: left and right with two
    /// panes; with more, the user's choice (default: the active pane and
    /// its target). `None` with a single pane.
    pub(crate) fn sync_pair(&self) -> Option<(Side, Side)> {
        let order = self.pane_order();
        match order.len() {
            0 | 1 => None,
            2 => Some((order[0], order[1])),
            _ => match self.sync.pick {
                Some((a, b)) if a != b && self.has_pane(a) && self.has_pane(b) => Some((a, b)),
                _ => Some((self.active, self.target_of(self.active)?)),
            },
        }
    }

    /// The actions a command stands for, applied to the active pane.
    fn cmd_actions(&self, cmd: Cmd) -> Vec<Action> {
        let side = self.active;
        vec![match cmd {
            Cmd::Open => Action::OpenSelected(side),
            Cmd::QuickLook => Action::Preview(side),
            Cmd::GetInfo => Action::GetInfo(side),
            Cmd::EnclosingFolder => Action::Up(side),
            Cmd::Back => Action::Back(side),
            Cmd::Forward => Action::Forward(side),
            Cmd::Refresh => Action::Refresh(side),
            Cmd::CopyToOther => Action::CopyToOther(side, false),
            Cmd::MoveToOther => Action::CopyToOther(side, true),
            Cmd::Rename => Action::Rename(side),
            Cmd::NewFolder => Action::NewFolder(side),
            Cmd::NewFile => Action::NewFile(side),
            Cmd::MoveToTrash => Action::Delete(side),
            Cmd::DeletePermanently => Action::DeletePermanently(side),
            Cmd::EmptyTrash => Action::EmptyTrash,
            Cmd::Undo => Action::Undo,
            Cmd::Redo => Action::Redo,
            Cmd::SelectAll => Action::SelectAll(side),
            Cmd::ToggleHidden => Action::ToggleHidden(side),
            Cmd::NewTab => Action::NewTab(side),
            Cmd::CloseTab => Action::CloseTab(side, self.pane(side).active),
            Cmd::SwitchPane => Action::SwitchPane,
            Cmd::SplitPane => Action::SplitPane(side),
            Cmd::ClosePane => Action::ClosePane(side),
            Cmd::FocusPaneLeft => Action::FocusPane(-1),
            Cmd::FocusPaneRight => Action::FocusPane(1),
            Cmd::EditPath => Action::EditPath(side),
            Cmd::GoToPath => Action::GoToPath(side),
            Cmd::Filter => Action::FocusFilter(side),
            Cmd::AddToFavorites => Action::AddCurrentToFavorites,
            Cmd::Connect => Action::ShowConnect,
            Cmd::Transfers => Action::ToggleTransfers,
            Cmd::CompareSync => Action::ShowSync,
            Cmd::CommandPalette => Action::ShowPalette,
            Cmd::Settings => Action::ShowSettings,
            Cmd::Help => Action::ShowHelp(String::new()),
            Cmd::Support => Action::OpenWebLink(crate::help::WEBSITE_URL.to_string()),
            Cmd::SourceCode => Action::OpenWebLink(crate::help::SOURCE_URL.to_string()),
            Cmd::ReportIssue => Action::OpenWebLink(crate::feedback::current_issue_url(
                crate::feedback::Kind::Bug,
            )),
            Cmd::SuggestIdea => Action::OpenWebLink(crate::feedback::current_issue_url(
                crate::feedback::Kind::Idea,
            )),
            Cmd::ThirdPartyLicenses => {
                Action::ShowHelp(crate::help::THIRD_PARTY_LICENSES.to_string())
            }
            Cmd::ReloadPlugins => Action::ReloadPlugins,
            Cmd::BrowsePlugins => Action::BrowsePlugins,
            Cmd::AppearanceLight => Action::SetAppearance(crate::theme::Appearance::Light),
            Cmd::AppearanceDark => Action::SetAppearance(crate::theme::Appearance::Dark),
            Cmd::AppearanceSystem => Action::SetAppearance(crate::theme::Appearance::System),
            Cmd::Compress => Action::Compress(side),
            Cmd::ExtractHere => Action::ExtractHere(side),
            Cmd::OpenWithApp => Action::OpenWithApp(side),
            Cmd::OpenShellInContainer => Action::OpenContainerShell(side),
            Cmd::ViewIcons => Action::SetView(side, crate::views::ViewMode::Icons),
            Cmd::ViewList => Action::SetView(side, crate::views::ViewMode::List),
            Cmd::ViewColumns => Action::SetView(side, crate::views::ViewMode::Columns),
            Cmd::ViewGallery => Action::SetView(side, crate::views::ViewMode::Gallery),
        }]
    }

    /// Whether a command can run right now (greys it out in the palette and toolbar).
    /// Commands picked in the menu bar run like palette commands (unless
    /// greyed out, or a modal opened since).
    pub(crate) fn menu_commands(&self, cmds: Vec<Cmd>, actions: &mut Vec<Action>) {
        let modal = self.modal_open();
        for cmd in cmds {
            if self.menu_cmd_enabled(cmd, modal) {
                log::debug!("menu bar: {cmd:?}");
                actions.extend(self.cmd_actions(cmd));
            } else {
                log::debug!("menu bar: {cmd:?} ignored (disabled or modal open)");
            }
        }
    }

    /// A menu bar command can run now: while a modal is open only the
    /// ones that open another window or the browser (Settings, Help, the
    /// web links) do; the rest would act on the main window behind it.
    pub(crate) fn menu_cmd_enabled(&self, cmd: Cmd, modal: bool) -> bool {
        let behind_modal = matches!(
            cmd,
            Cmd::Settings
                | Cmd::Help
                | Cmd::Support
                | Cmd::SourceCode
                | Cmd::ReportIssue
                | Cmd::SuggestIdea
                | Cmd::ThirdPartyLicenses
                | Cmd::BrowsePlugins
        );
        (!modal || behind_modal) && self.cmd_enabled(cmd)
    }

    fn cmd_enabled(&self, cmd: Cmd) -> bool {
        if self.cmd_disabled_reason(cmd).is_some() {
            return false;
        }
        let pane = self.pane(self.active);
        let tab = pane.tab();
        let has_sel = tab.cursor.is_some() || !tab.selected.is_empty();
        let ro = tab.loc.read_only();
        match cmd {
            Cmd::Open | Cmd::CopyToOther | Cmd::OpenWithApp | Cmd::GetInfo => has_sel,
            Cmd::MoveToOther | Cmd::MoveToTrash | Cmd::DeletePermanently => has_sel && !ro,
            Cmd::EmptyTrash => self.trash.dir.is_some() && self.trash.status.full() != Some(false),
            Cmd::Undo => self.history.can_undo(),
            Cmd::Redo => self.history.can_redo(),
            Cmd::NewFolder | Cmd::NewFile => !ro,
            Cmd::Compress => has_sel && !ro,
            // Asked for every menu item every frame: no copies of the
            // selection (it may be 300,000 items).
            Cmd::ExtractHere => {
                let sum = tab.selection_summary();
                tab.loc.archive.is_some()
                    || if sum.view_idx.is_empty() {
                        tab.cursor_entry().is_some_and(crate::pane::is_archive_file)
                    } else {
                        sum.any_archive
                    }
            }
            Cmd::QuickLook => {
                let sum = tab.selection_summary();
                let first = || sum.view_idx.first().and_then(|&vi| tab.entry_at(vi));
                (self.quicklook.available()
                    && self.prefs.space_action == crate::prefs::SpaceAction::QuickLook
                    && crate::quicklook::tab_has_target(tab))
                    || tab
                        .cursor_entry()
                        .or_else(first)
                        .is_some_and(|e| !e.is_dir())
            }
            Cmd::Rename if ro => false,
            Cmd::Rename => match tab
                .selection_summary()
                .view_idx
                .len()
                .max(usize::from(tab.cursor_entry().is_some()))
            {
                0 => false,
                1 => true,
                _ => self.batch_rename_available(),
            },
            Cmd::Back => !tab.back.is_empty(),
            Cmd::Forward => !tab.forward.is_empty(),
            Cmd::EnclosingFolder => !tab.loc.path.is_root() || tab.loc.archive.is_some(),
            Cmd::CloseTab => pane.tabs.len() > 1,
            Cmd::SelectAll => !tab.view.is_empty(),
            Cmd::OpenShellInContainer => self.container_shell_command(self.active).is_some(),
            Cmd::FocusPaneLeft => {
                crate::pane_set::neighbour(&self.pane_order(), self.active, -1).is_some()
            }
            Cmd::FocusPaneRight => {
                crate::pane_set::neighbour(&self.pane_order(), self.active, 1).is_some()
            }
            _ => true,
        }
    }

    /// Why a command is off for reasons of the pane layout (one pane),
    /// shown as its tooltip; `None` if the layout allows it.
    pub(crate) fn cmd_disabled_reason(&self, cmd: Cmd) -> Option<&'static str> {
        if !cmd.needs_two_panes() || self.panes.len() > 1 {
            return None;
        }
        Some(if cmd == Cmd::ClosePane {
            t!("panes.last_pane")
        } else {
            t!("panes.needs_two")
        })
    }

    /// `docker exec -it … sh` (or kubectl) for the folder shown in `side`,
    /// if it is a running container.
    pub(crate) fn container_shell_command(&self, side: Side) -> Option<String> {
        let tab = self.pane(side).tab();
        if !tab.loc.fs.protocol().is_container()
            || tab.loc.fs.is_read_only()
            || tab.loc.archive.is_some()
        {
            return None;
        }
        let conn = self
            .connections
            .iter()
            .find(|c| Some(c.id) == tab.loc.conn)?;
        let target = manta_vfs::container::Target::from_config(&conn.config).ok()?;
        Some(target.shell_command(&tab.loc.path))
    }

    fn open_container_shell(&mut self, side: Side) {
        match self.container_shell_command(side) {
            Some(cmd) => self.plugins.run_in_terminal(Some(side), cmd, None),
            None => self.set_info(t!("status.container_shell")),
        }
    }

    /// Which of `paths` (on `fs`) are folders, as far as the open tabs'
    /// listings know.
    fn folder_paths(&self, fs: &SharedVfs, paths: &[VfsPath]) -> Vec<VfsPath> {
        self.panes
            .iter()
            .flat_map(|p| p.tabs.iter())
            .filter(|t| same_fs(&t.loc.fs, fs))
            .flat_map(|t| t.entries.iter())
            .filter(|e| e.is_dir() && paths.contains(&e.path))
            .map(|e| e.path.clone())
            .collect()
    }

    // ------------------------------------------------------------ go to path

    /// "Local", or "deploy@web-01 (SFTP)" for a remote tab.
    fn backend_label(&self, loc: &Location) -> String {
        if let Some(m) = &loc.archive {
            return t!("app.backend_archive", name = m.file_name);
        }
        match loc.conn {
            None => t!("app.local").into(),
            Some(_) => format!("{} ({})", self.conn_label(loc.conn), loc.fs.protocol()),
        }
    }

    fn show_goto(&mut self, side: Side) {
        self.activate(side);
        let loc = self.pane(side).tab().loc.clone();
        // Recent: folders opened from this popup, then the tabs' histories
        // and open tabs on the same backend.
        let mut recent: Vec<VfsPath> = Vec::new();
        let same = |l: &Location| same_fs(&l.fs, &loc.fs);
        let from_goto = self
            .goto_recent
            .iter()
            .filter(|(c, _)| *c == loc.conn)
            .map(|(_, p)| p.clone());
        let from_tabs = self.panes.iter().flat_map(|p| {
            p.tabs.iter().flat_map(|t| {
                t.back
                    .iter()
                    .rev()
                    .chain(std::iter::once(&t.loc))
                    .filter(|l| same(l))
                    .map(|l| l.path.clone())
            })
        });
        for path in from_goto.chain(from_tabs) {
            if path != loc.path && !recent.contains(&path) {
                recent.push(path);
            }
        }
        recent.truncate(8);
        let home = if loc.conn.is_none() {
            dirs::home_dir().map(|h| local_vfs_path(&h))
        } else {
            None
        };
        self.goto = Some(GotoPopup::new(
            side,
            loc.fs.clone(),
            loc.conn,
            loc.path.clone(),
            self.backend_label(&loc),
            home,
            recent,
        ));
    }

    fn goto_ui(&mut self, ctx: &egui::Context) {
        let Some(g) = &mut self.goto else {
            return;
        };
        let mut requests = Vec::new();
        let keep_open = g.ui(ctx, &mut requests);
        let (fs, base) = (g.fs.clone(), g.base.clone());
        if !keep_open {
            self.goto = None;
        }
        for r in requests {
            let (fs, base) = (fs.clone(), base.clone());
            match r {
                GotoRequest::Check {
                    gen,
                    input,
                    new_tab,
                } => self.worker.spawn(
                    "goto-check",
                    move || crate::goto::check_target(&fs, &input, &base),
                    move |result| WorkResult::GotoChecked {
                        gen,
                        new_tab,
                        result,
                    },
                ),
                GotoRequest::ListFolders { gen, dir_part } => {
                    let key = dir_part.clone();
                    self.worker.spawn(
                        "goto-complete",
                        move || crate::goto::list_folders(&fs, &dir_part, &base),
                        move |result| WorkResult::GotoFolders {
                            gen,
                            dir_part: key,
                            result,
                        },
                    )
                }
            }
        }
    }

    /// Open a checked "Go to Path" target: in a new tab of `side` (Enter) or
    /// in its current tab (⌘Enter). A file opens its folder with the file
    /// selected (hidden files are shown so the selection is visible).
    fn open_goto_target(
        &mut self,
        side: Side,
        conn: Option<u64>,
        fs: SharedVfs,
        target: GotoTarget,
        new_tab: bool,
    ) {
        self.goto_recent
            .retain(|(c, p)| !(*c == conn && *p == target.dir));
        self.goto_recent.insert(0, (conn, target.dir.clone()));
        self.goto_recent.truncate(20);
        let cur = &self.pane(side).tab().loc;
        let loc = if same_fs(&cur.fs, &fs) {
            cur.with_path(target.dir)
        } else {
            Location::new(fs, conn, target.dir)
        };
        let hidden_target = target.select.as_deref().is_some_and(|n| n.starts_with('.'));
        self.activate(side);
        if new_tab {
            let id = self.next_id();
            let pane = self.pane_mut(side);
            let mut tab = Tab::new(id, loc.clone());
            tab.sort = pane.tab().sort;
            tab.show_hidden = pane.tab().show_hidden;
            pane.tabs.push(tab);
            pane.active = pane.tabs.len() - 1;
            let worker = &self.worker;
            let pi = self.pos(side);
            let tab = self.panes[pi].tab_mut();
            start_listing(worker, tab, Some(loc.path));
        } else {
            self.navigate(side, loc, false);
        }
        let tab = self.pane_mut(side).tab_mut();
        if hidden_target {
            tab.show_hidden = true;
        }
        tab.pending_select = target.select;
    }

    /// The shell in the pane's Terminal was left with `exit` while the pane
    /// showed a connection: the pane goes back to where it was on this Mac
    /// (else the home folder), and the connection is closed unless another
    /// tab still shows it.
    pub(crate) fn leave_connection(&mut self, side: Side) {
        let Some(id) = self.pane(side).tab().loc.conn else {
            return;
        };
        let local = self
            .pane(side)
            .tab()
            .back
            .iter()
            .rev()
            .find(|l| l.conn.is_none() && l.archive.is_none())
            .cloned()
            .unwrap_or_else(|| Location {
                fs: self.local.clone(),
                conn: None,
                path: dirs::home_dir()
                    .map(|h| local_vfs_path(&h))
                    .unwrap_or_else(VfsPath::root),
                archive: None,
            });
        self.navigate(side, local, false);
        let shown = self
            .panes
            .iter()
            .flat_map(|p| &p.tabs)
            .any(|t| t.loc.conn == Some(id));
        if !shown {
            self.disconnect(id);
        }
    }

    fn disconnect(&mut self, id: u64) {
        let Some(pos) = self.connections.iter().position(|c| c.id == id) else {
            return;
        };
        let side = std::iter::once(self.active)
            .chain(self.mru.iter().copied())
            .chain(self.pane_order())
            .find(|s| self.has_pane(*s) && self.pane(*s).tab().loc.conn == Some(id))
            .unwrap_or(self.active);
        let conn_fs = self.connections[pos].fs.clone();
        let mut a = self.new_action(side, FileActionKind::Disconnect, &conn_fs, Vec::new(), None);
        if self.pane(side).tab().loc.conn == Some(id) {
            a.sources.push(self.pane(side).tab().loc.path.clone());
        }
        let action = self.start_action(a);
        self.finish_action(action, Ok(0));
        let conn = self.connections.remove(pos);
        let home = dirs::home_dir()
            .map(|h| local_vfs_path(&h))
            .unwrap_or_else(VfsPath::root);
        let local = self.local.clone();
        let worker = &self.worker;
        for pane in &mut self.panes {
            for tab in &mut pane.tabs {
                tab.back.retain(|l| l.conn != Some(id));
                tab.forward.retain(|l| l.conn != Some(id));
                if tab.loc.conn == Some(id) {
                    let loc = Location {
                        fs: local.clone(),
                        conn: None,
                        path: home.clone(),
                        archive: None,
                    };
                    navigate_tab(worker, tab, loc, false, false);
                }
            }
        }
        self.set_info(t!("status.disconnected_from", label = conn.label));
        // An SMB share Manta mounted is ejected now (the transfer manager
        // keeps the backend for later retries, so it isn't dropped).
        if smb_mount(&conn.fs).is_some() {
            let fs = conn.fs.clone();
            let _ = std::thread::Builder::new()
                .name("smb-unmount".into())
                .spawn(move || {
                    if let Some(s) = fs
                        .as_any()
                        .and_then(|a| a.downcast_ref::<manta_vfs::SmbFs>())
                    {
                        if let Err(e) = s.unmount_now() {
                            log::warn!("SMB: unmounting {}: {e}", s.mount_point().display());
                        }
                    }
                });
        }
        // Dropping the last Arc closes the connection (transfers may still hold one).
    }

    /// Eject a volume from the sidebar (a network share, disk image, USB
    /// stick or external drive; see [`manta_vfs::volumes::eject`]), off the
    /// UI thread. Panes showing it go back to the home folder.
    fn eject_volume(&mut self, path: PathBuf) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.set_info(t!("status.ejecting", name = name));
        let p = path.clone();
        self.worker.spawn(
            "eject",
            move || manta_vfs::volumes::eject(&p).map_err(|e| e.to_string()),
            move |result| WorkResult::Ejected { path, name, result },
        );
    }

    /// A local folder in a new tab of the active pane (the volume menu's
    /// "Open in New Tab", like Finder's "Open in New Window"; ⌘T with a
    /// folder set for new tabs).
    fn open_local_in_new_tab(&mut self, path: PathBuf) {
        let side = self.active;
        let id = self.next_id();
        let pane = self.pane_mut(side);
        let tab = pane.tab().duplicate(id);
        pane.tabs.push(tab);
        pane.active = pane.tabs.len() - 1;
        let loc = Location {
            fs: self.local.clone(),
            conn: None,
            path: local_vfs_path(&path),
            archive: None,
        };
        self.navigate(side, loc, false);
    }

    /// "Show in Enclosing Folder" of a volume: the disk image file it was
    /// opened from (`hdiutil info`, off the UI thread), else the volume
    /// itself in `/Volumes`.
    fn reveal_volume(&mut self, volume: PathBuf) {
        let v = volume.clone();
        self.worker.spawn(
            "reveal-volume",
            move || Ok::<_, String>(manta_vfs::volumes::disk_image_of(&v)),
            move |result| WorkResult::VolumeRevealed {
                volume,
                image: result.ok().flatten(),
            },
        );
    }

    /// Select `image` (or `volume`) in its folder in the active pane.
    fn on_volume_revealed(&mut self, volume: PathBuf, image: Option<PathBuf>) {
        let item = image.filter(|i| i.exists()).unwrap_or(volume);
        let (Some(dir), Some(name)) = (item.parent(), item.file_name()) else {
            return;
        };
        let loc = Location {
            fs: self.local.clone(),
            conn: None,
            path: local_vfs_path(dir),
            archive: None,
        };
        let side = self.active;
        self.navigate(side, loc, false);
        self.pane_mut(side).tab_mut().pending_select = Some(name.to_string_lossy().into_owned());
    }

    /// `docker start|restart|stop` in the background; the sidebar row shows
    /// "Starting …" etc. until it is done.
    fn docker_control(&mut self, id: String, name: String, op: DockerOp) {
        if !self.containers.begin(&id, op) {
            return;
        }
        let runner = crate::containers::discovery_env().runner;
        let (job_id, job_name) = (id.clone(), name.clone());
        self.worker.spawn(
            "docker-control",
            move || {
                manta_vfs::container::discover::docker_control(&*runner, op, &job_id, &job_name)
                    .map_err(|e| e.to_string())
            },
            move |result| WorkResult::DockerControlled {
                id,
                name,
                op,
                result,
            },
        );
    }

    /// The Docker container a connection opens, by name.
    fn docker_container_of(&self, conn: Option<u64>) -> Option<String> {
        let c = self.connections.iter().find(|c| Some(c.id) == conn)?;
        match manta_vfs::container::Target::from_config(&c.config).ok()? {
            manta_vfs::container::Target::Docker { container } => Some(container),
            _ => None,
        }
    }

    /// `docker start` for the container the pane shows (it stopped while
    /// open); the listing is loaded again once it runs.
    fn start_pane_container(&mut self, side: Side) {
        let Some(name) = self.docker_container_of(self.pane(side).tab().loc.conn) else {
            return;
        };
        // The sidebar keys its "Starting…" by id; `docker start` takes either.
        let id = match self.containers.snapshot().docker {
            Some(Ok(list)) => list
                .into_iter()
                .find(|c| c.name == name || c.id == name)
                .map(|c| c.id),
            _ => None,
        }
        .unwrap_or_else(|| name.clone());
        if self.containers.pending(&id).is_some() {
            return;
        }
        self.pane_mut(side).tab_mut().container_starting = true;
        self.docker_control(id, name, DockerOp::Start);
    }

    fn on_docker_controlled(
        &mut self,
        id: String,
        name: String,
        op: DockerOp,
        result: Result<(), String>,
    ) {
        self.containers.finish(&id);
        // Panes showing the container ("not running" page): it runs again.
        let conns: Vec<u64> = self
            .connections
            .iter()
            .map(|c| c.id)
            .filter(|&c| {
                self.docker_container_of(Some(c))
                    .is_some_and(|n| n == name || n == id)
            })
            .collect();
        let worker = &self.worker;
        for pane in &mut self.panes {
            for tab in &mut pane.tabs {
                if !tab.loc.conn.is_some_and(|c| conns.contains(&c)) {
                    continue;
                }
                let waited = std::mem::take(&mut tab.container_starting);
                if result.is_ok() && op != DockerOp::Stop && (waited || tab.error.is_some()) {
                    let p = tab.loc.path.clone();
                    start_listing(worker, tab, Some(p));
                }
            }
        }
        match result {
            Ok(()) => self.set_success(match op {
                DockerOp::Start => t!("status.container_started", name = name),
                DockerOp::Restart => t!("status.container_restarted", name = name),
                DockerOp::Stop => t!("status.container_stopped", name = name),
            }),
            Err(e) => self.set_error_detail(
                match op {
                    DockerOp::Start => t!("error.container_start", name = name),
                    DockerOp::Restart => t!("error.container_restart", name = name),
                    DockerOp::Stop => t!("error.container_stop", name = name),
                },
                e,
            ),
        }
    }

    /// The volume is gone: leave it in every tab, rescan the devices.
    fn on_ejected(&mut self, path: PathBuf, name: String, result: Result<(), String>) {
        if let Err(e) = result {
            self.set_error_detail(t!("error.eject", name = name), e);
            return;
        }
        let gone = local_vfs_path(&path);
        let home = dirs::home_dir()
            .map(|h| local_vfs_path(&h))
            .unwrap_or_else(VfsPath::root);
        let local = self.local.clone();
        let worker = &self.worker;
        for pane in &mut self.panes {
            for tab in &mut pane.tabs {
                if tab.loc.conn.is_none() && tab.loc.path.starts_with(&gone) {
                    let loc = Location {
                        fs: local.clone(),
                        conn: None,
                        path: home.clone(),
                        archive: None,
                    };
                    navigate_tab(worker, tab, loc, false, false);
                }
            }
        }
        let (l, d) = sidebar::discover_places();
        self.locations = l;
        self.devices = d;
        self.set_success(t!("status.ejected", name = name));
    }

    fn open_selected(&mut self, side: Side) {
        self.open_selected_with(side, true)
    }

    /// Open the selection; `browse_archives`: archives open as folders in
    /// the pane (if switched on in Settings) instead of in their app.
    fn open_selected_with(&mut self, side: Side, browse_archives: bool) {
        let tab = self.pane(side).tab();
        let entries = tab.selected_entries();
        let fs = tab.loc.fs.clone();
        if entries.is_empty() {
            return;
        }
        if entries.len() == 1 {
            let e = &entries[0];
            let is_dir = match e.kind {
                EntryKind::Directory => true,
                // Follow symlinks to directories; remote links are tried as directories.
                // Inside an archive the (in-memory) index knows.
                EntryKind::Symlink => fs
                    .local_path(&e.path)
                    .map(|p| p.is_dir())
                    .unwrap_or_else(|| !fs.is_read_only() || fs.list(&e.path).is_ok()),
                _ => false,
            };
            if is_dir {
                self.navigate_path(side, e.path.clone());
                return;
            }
            if browse_archives && self.maybe_browse_archive(side, &e.clone(), false) {
                return;
            }
        }
        let files: Vec<_> = entries.into_iter().filter(|e| !e.is_dir()).collect();
        let local: Vec<VfsPath> = files
            .iter()
            .filter(|e| fs.local_path(&e.path).is_some())
            .map(|e| e.path.clone())
            .collect();
        let local_action = if local.is_empty() {
            None
        } else {
            let a = self.new_action(side, FileActionKind::Open, &fs, local, None);
            Some(self.start_action(a))
        };
        let mut opened = 0;
        let mut first_error = None;
        for e in files {
            if let Some(p) = fs.local_path(&e.path) {
                // Checks readability first and logs the raw error.
                if let Err(err) = crate::logging::open_local(&p) {
                    first_error.get_or_insert_with(|| format!("{}: {err}", e.name));
                    self.set_error_detail(t!("error.couldnt_open", name = e.name), err);
                } else {
                    opened += 1;
                }
            } else {
                let a =
                    self.new_action(side, FileActionKind::Open, &fs, vec![e.path.clone()], None);
                let action = Some(self.start_action(a));
                // Remote: download to a temp file in the background, then open it.
                let fs = fs.clone();
                let name = e.name.clone();
                self.set_info(t!("status.downloading_to_open", name = name));
                self.worker.spawn(
                    "open-remote",
                    move || {
                        let local = worker::download_to_temp(&fs, &e.path)?;
                        open::that_detached(&local).map_err(|err| err.to_string())
                    },
                    move |result| WorkResult::FileOp {
                        description: t!("status.opened", name = name),
                        failure: t!("error.couldnt_open", name = name),
                        refresh: Vec::new(),
                        select: None,
                        action,
                        result,
                        undo: None,
                    },
                );
            }
        }
        if let Some(id) = local_action {
            self.finish_action(id, first_error.map_or(Ok(opened), Err));
        }
    }

    fn open_preview(&mut self, ctx: &egui::Context, side: Side) {
        // Apple's Quick Look for local items (Settings → General → Space opens).
        let use_ql = self.preview.is_none()
            && self.prefs.space_action == crate::prefs::SpaceAction::QuickLook;
        if use_ql || self.quicklook.is_open() {
            let tab = self.pane(side).tab();
            let key = crate::quicklook::QlKey::of(side.index(), tab);
            let target = crate::quicklook::tab_target(tab).filter(|_| use_ql);
            if target.is_some() || self.quicklook.is_open() {
                match self.quicklook.toggle(key, target) {
                    Ok(_) => return,
                    // Fall back to the built-in preview.
                    Err(e) => log::warn!("Quick Look unavailable, using the built-in preview: {e}"),
                }
            }
        }
        if let Some(p) = self.preview.take() {
            // Space toggles the preview like Quick Look.
            p.release(ctx);
            return;
        }
        let tab = self.pane(side).tab();
        let Some(entry) = tab
            .cursor_entry()
            .cloned()
            .or_else(|| tab.selected_entries().into_iter().next())
        else {
            return;
        };
        if entry.is_dir() {
            self.set_info(t!("status.ql_folder"));
            return;
        }
        let fs = tab.loc.fs.clone();
        let id = self.next_id();
        let e = entry.clone();
        self.preview = Some(Preview::new(id, entry));
        self.worker.spawn(
            "preview",
            move || worker::load_preview(&fs, &e),
            move |result| WorkResult::Preview { id, result },
        );
    }

    /// Say in the connect dialog which pane the server opens in.
    /// Open the connection favorite `i` (`config`) in `side`: reuse an open
    /// connection to the same server, else connect (asking for a secret the
    /// favorite doesn't keep).
    fn open_connection_favorite(
        &mut self,
        side: Side,
        i: usize,
        config: ConnectionConfig,
        keychain: bool,
    ) {
        // Reuse an existing connection to the same server.
        let existing = self.connections.iter().find(|c| {
            c.config.host == config.host
                && c.config.port == config.port
                && c.config.username == config.username
                && c.config.protocol == config.protocol
        });
        if let Some(c) = existing {
            let loc = Location {
                fs: c.fs.clone(),
                conn: Some(c.id),
                path: config
                    .initial_path
                    .as_deref()
                    .map(VfsPath::new)
                    .unwrap_or_else(VfsPath::root),
                archive: None,
            };
            let home = config.initial_path.is_none();
            self.navigate(side, loc, home);
        } else if keychain {
            self.start_connect(config, None, side, true);
        } else if favorites::needs_secret(&config) {
            let mut form = self.connect_form(ConnectForm::from_config(&config, side, Some(i)));
            form.error = Some(t!("error.enter_password").into());
            self.dialog = Some(Dialog::Connect(form));
        } else {
            self.start_connect(config, None, side, false);
        }
    }

    /// Show the SFTP server `user@host:port` in `side` (the Terminal saw an
    /// `ssh` to it): an open connection, else a favorite for it, else the
    /// Connect dialog filled in.
    fn open_sftp(&mut self, side: Side, user: String, host: String, port: u16) {
        self.activate(side);
        let same = |c: &ConnectionConfig| {
            c.protocol == manta_core::Protocol::Sftp
                && c.host.eq_ignore_ascii_case(&host)
                && c.port == port
                && c.username == user
        };
        if let Some(c) = self.connections.iter().find(|c| same(&c.config)) {
            let loc = Location {
                fs: c.fs.clone(),
                conn: Some(c.id),
                path: VfsPath::root(),
                archive: None,
            };
            self.navigate(side, loc, true);
            return;
        }
        let fav = self
            .favorites
            .items
            .iter()
            .enumerate()
            .find_map(|(i, f)| match f {
                Favorite::Connection { config, keychain } if same(config) => {
                    Some((i, config.clone(), *keychain))
                }
                _ => None,
            });
        if let Some((i, config, keychain)) = fav {
            self.open_connection_favorite(side, i, config, keychain);
            return;
        }
        let mut form = ConnectForm::new(side);
        form.protocol = manta_core::Protocol::Sftp;
        form.host = host;
        form.port = port.to_string();
        form.user = user;
        self.dialog = Some(Dialog::Connect(self.connect_form(form)));
    }

    fn connect_form(&self, mut f: ConnectForm) -> ConnectForm {
        f.pane_note = match self.panes.len() {
            1 => Some(t!("connect.subtitle.single").into()),
            2 if self.pos(f.side) == 0 => Some(t!("connect.subtitle.left").into()),
            2 => Some(t!("connect.subtitle.right").into()),
            _ => Some(t!("connect.subtitle.pane", n = self.pane_number(f.side))),
        };
        f.gdrive = self.settings.cloud.own_client();
        f
    }

    /// Why the items can't go to pane `to` (`None` if they can).
    fn transfer_problem(
        &self,
        src_fs: &SharedVfs,
        sources: &[VfsPath],
        to: Side,
    ) -> Option<String> {
        let dst = &self.pane(to).tab().loc;
        if dst.read_only() {
            let name = dst
                .archive
                .as_ref()
                .map(|m| m.file_name.clone())
                .unwrap_or_else(|| crate::i18n::fs_label(&dst.fs.label()));
            return Some(t!("archive.cant_paste", name = name));
        }
        let src_dir = sources.first()?.parent().unwrap_or_else(VfsPath::root);
        if same_fs(src_fs, &dst.fs) && src_dir == dst.path {
            return Some(t!("status.same_folder").into());
        }
        if same_fs(src_fs, &dst.fs) && sources.iter().any(|s| dst.path.starts_with(s)) {
            return Some(t!("status.into_itself").into());
        }
        None
    }

    /// The panes the copy dialog offers (all but the source), left to right.
    fn transfer_targets(
        &self,
        from: Side,
        src_fs: &SharedVfs,
        sources: &[VfsPath],
    ) -> Vec<dialogs::TransferTarget> {
        self.panes
            .iter()
            .enumerate()
            .filter(|(_, p)| p.id != from)
            .map(|(i, p)| {
                let loc = &p.tab().loc;
                dialogs::TransferTarget {
                    side: p.id,
                    number: i + 1,
                    folder: p.tab().title(),
                    backend: self.place_label(loc),
                    remote: loc.conn.is_some(),
                    fs: loc.fs.clone(),
                    dir: loc.path.clone(),
                    problem: self.transfer_problem(src_fs, sources, p.id),
                }
            })
            .collect()
    }

    /// Start a confirmed copy / move / extract job (the transfer dialog,
    /// drops from other apps).
    fn submit_transfer(&mut self, f: TransferForm) {
        let kind = match (f.mv, f.extract) {
            (true, _) => JobKind::Move,
            (false, true) => JobKind::Extract,
            (false, false) => JobKind::Copy,
        };
        let mut refresh = vec![(f.dst_fs.clone(), f.dst_dir.clone())];
        if f.mv {
            for s in &f.sources {
                let parent = s.parent().unwrap_or_else(VfsPath::root);
                if !refresh
                    .iter()
                    .any(|(fs, d)| same_fs(fs, &f.src_fs) && *d == parent)
                {
                    refresh.push((f.src_fs.clone(), parent));
                }
            }
        }
        let akind = match (f.mv, &f.archive) {
            (true, _) => FileActionKind::Move,
            (false, Some(_)) => FileActionKind::Extract,
            (false, None) => FileActionKind::Copy,
        };
        let mut a = self.new_action(
            f.src_side,
            akind,
            &f.src_fs,
            f.sources.clone(),
            Some((&f.dst_fs, f.dst_dir.clone())),
        );
        if let (Some(m), false) = (&f.archive, f.mv) {
            a.archive = Some(Box::new(self.archive_spec(m)));
        }
        let action = self.start_action(a);
        let job = Job {
            kind,
            src_fs: f.src_fs,
            sources: f.sources,
            dst_fs: Some(f.dst_fs),
            dst_dir: Some(f.dst_dir),
            conflict: f.conflict,
        };
        let job_id = self.enqueue(job, refresh);
        self.actions.attach_job(action, job_id);
    }

    /// F5 / F6 (`drop`: false) or a drop between panes: the copy / move
    /// dialog. A drop offers Copy / Move in it, or skips it when Settings
    /// say "Always copy" / "Always move".
    fn confirm_transfer(
        &mut self,
        from: Side,
        src_fs: SharedVfs,
        sources: Vec<VfsPath>,
        to: Side,
        mut mv: bool,
        drop: bool,
    ) {
        if sources.is_empty() {
            self.set_info(t!("status.select_copy_move"));
            return;
        }
        // With three or more panes the dialog offers the other targets, so
        // a default target that can't take the items isn't the end.
        let targets = if self.panes.len() > 2 {
            self.transfer_targets(from, &src_fs, &sources)
        } else {
            Vec::new()
        };
        let alternative = targets.iter().any(|t| t.side != to && t.problem.is_none());
        if !alternative && self.refuse_read_only(to, ReadOnlyOp::Paste) {
            return;
        }
        // Archives are extracted; read-only containers just copied.
        let extract = src_fs.is_read_only() && !src_fs.protocol().is_container();
        if src_fs.is_read_only() && mv {
            mv = false;
            self.set_info(if extract {
                t!("status.archive_extract_not_move")
            } else {
                t!("status.container_copy_not_move")
            });
        }
        let archive = Some(&self.pane(from).tab().loc)
            .filter(|l| same_fs(&l.fs, &src_fs))
            .and_then(|l| l.archive.clone());
        if !alternative {
            if let Some(problem) = self.transfer_problem(&src_fs, &sources, to) {
                self.set_info(problem);
                return;
            }
        }
        let dst = self.pane(to).tab();
        let folders = self.folder_paths(&src_fs, &sources);
        let form = TransferForm {
            extract,
            archive,
            src_side: from,
            dst_side: to,
            targets,
            folders,
            mv,
            src_fs,
            sources,
            dst_fs: dst.loc.fs.clone(),
            dst_dir: dst.loc.path.clone(),
            dst_label: self.place_label(&dst.loc),
            conflict: self.prefs.conflict.policy(),
            choose: drop,
        };
        self.ask_or_submit(form);
    }

    /// Show the copy / move dialog, or, for a drop with "Always copy" /
    /// "Always move" in Settings, start right away.
    pub(super) fn ask_or_submit(&mut self, form: TransferForm) {
        if form.choose && self.prefs.drop_action != crate::prefs::DropAction::Ask {
            self.submit_transfer(form);
        } else {
            self.dialog = Some(Dialog::Transfer(form));
        }
    }

    fn start_compare(&mut self) {
        let Some((a, b)) = self.sync_pair() else {
            return;
        };
        self.sync.pick = Some((a, b));
        self.sync.picker = self.panes.len() > 2;
        self.sync.choices = self
            .panes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let loc = &p.tab().loc;
                crate::sync_view::PaneChoice {
                    id: p.id,
                    number: i + 1,
                    folder: p.tab().title(),
                    backend: self.place_label(loc),
                    remote: loc.conn.is_some(),
                }
            })
            .collect();
        let l = self.pane(a).tab().loc.clone();
        let r = self.pane(b).tab().loc.clone();
        self.sync.left_label = self.place_and_path(&l);
        self.sync.right_label = self.place_and_path(&r);
        self.sync.gen += 1;
        self.sync.loading = true;
        self.sync.error = None;
        self.sync.items = None;
        self.sync.left = Some(l.clone());
        self.sync.right = Some(r.clone());
        let gen = self.sync.gen;
        let opts = self.sync.options;
        self.worker.spawn(
            "compare",
            move || {
                manta_ops::compare_dirs(&l.fs, &l.path, &r.fs, &r.path, &opts)
                    .map_err(|e| e.to_string())
            },
            move |result| WorkResult::Compare { gen, result },
        );
    }

    fn start_sync(&mut self) {
        let (Some(l), Some(r), Some(items)) = (&self.sync.left, &self.sync.right, &self.sync.items)
        else {
            return;
        };
        let jobs = manta_ops::plan_sync(
            items,
            &l.fs,
            &l.path,
            &r.fs,
            &r.path,
            self.sync.direction,
            self.sync.delete_extra,
        );
        let refresh = vec![
            (l.fs.clone(), l.path.clone()),
            (r.fs.clone(), r.path.clone()),
        ];
        let (a, b) = self.sync.pick.unwrap_or((self.active, self.active));
        let (from, to) = match self.sync.direction {
            manta_ops::SyncDirection::RightToLeft => ((b, r.clone()), (a, l.clone())),
            _ => ((a, l.clone()), (b, r.clone())),
        };
        let spec = SyncSpec {
            delete_extra: self.sync.delete_extra
                && self.sync.direction != manta_ops::SyncDirection::Bidirectional,
            both_ways: self.sync.direction == manta_ops::SyncDirection::Bidirectional,
        };
        let mut a = self.new_action(
            from.0,
            FileActionKind::Sync,
            &from.1.fs,
            vec![from.1.path.clone()],
            Some((&to.1.fs, to.1.path.clone())),
        );
        a.sync = Some(spec);
        let action = self.start_action(a);
        let n = jobs.len();
        for job in jobs {
            let job_id = self.enqueue(job, refresh.clone());
            self.actions.attach_job(action, job_id);
        }
        if n == 0 {
            self.finish_action(action, Ok(0));
        }
        self.set_info(t!("status.sync_started", count = n));
        self.sync.open = false;
    }

    fn dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(dialog) = &mut self.dialog else {
            return;
        };
        if let Dialog::Conflict(f) = dialog {
            if let Some(choice) = f.ui(ctx) {
                if let Some(Dialog::Conflict(f)) = self.dialog.take() {
                    self.conflict_answered(*f, choice);
                }
            }
            return;
        }
        if let Dialog::HostKey(f) = dialog {
            let result = f.ui(ctx);
            if matches!(result, DialogResult::Submit | DialogResult::Cancel) {
                if let Some(Dialog::HostKey(f)) = self.dialog.take() {
                    self.host_key_answered(*f, result == DialogResult::Submit);
                }
            }
            return;
        }
        let result = match dialog {
            Dialog::Connect(f) => f.ui(ctx),
            Dialog::Transfer(f) => f.ui(ctx),
            Dialog::Delete(f) => f.ui(ctx),
            Dialog::EmptyTrash(f) => f.ui(ctx),
            Dialog::WhatsNew(f) => f.ui(ctx),
            Dialog::Name(f) => f.ui(ctx),
            Dialog::Compress(f) => f.ui(ctx),
            Dialog::HostKey(_) | Dialog::Conflict(_) => return,
        };
        match result {
            DialogResult::Open => {}
            DialogResult::Cancel => {
                if matches!(self.dialog, Some(Dialog::WhatsNew(_))) {
                    self.mark_whats_new_seen();
                }
                self.dialog = None;
            }
            DialogResult::Submit => self.submit_dialog(),
            DialogResult::Test => self.test_connection(),
        }
    }

    /// Connect with the dialog's settings in the background, then drop the
    /// connection again; the result is shown inside the dialog.
    fn test_connection(&mut self) {
        let Some(Dialog::Connect(form)) = &mut self.dialog else {
            return;
        };
        let config = match form.to_config() {
            Ok(c) => c,
            Err(e) => {
                form.error = Some(e);
                form.show_validation = true;
                return;
            }
        };
        form.testing = true;
        form.test_result = None;
        form.error = None;
        self.worker.spawn(
            "test-connection",
            move || {
                let fs = match Checked::from_result(manta_vfs::connect(&config))? {
                    Checked::Done(fs) => fs,
                    Checked::Ask { info, changed } => return Ok(Checked::Ask { info, changed }),
                };
                let home = fs.home().map_err(|e| e.to_string())?;
                Ok(Checked::Done(home.to_string()))
            },
            |result| match result {
                Ok(Checked::Ask { info, changed }) => WorkResult::HostKey {
                    info,
                    changed,
                    retry: HostKeyRetry::Test,
                },
                Ok(Checked::Done(home)) => WorkResult::TestConnection { result: Ok(home) },
                Err(e) => WorkResult::TestConnection { result: Err(e) },
            },
        );
    }

    fn submit_dialog(&mut self) {
        let Some(dialog) = self.dialog.take() else {
            return;
        };
        match dialog {
            Dialog::Connect(mut form) => match form.to_config() {
                Err(e) => {
                    form.error = Some(e);
                    self.dialog = Some(Dialog::Connect(form));
                }
                Ok(cfg) => {
                    let favorite = form.save_favorite.then(|| {
                        if form.remember_password {
                            cfg.clone()
                        } else {
                            favorites::strip_secrets(&cfg)
                        }
                    });
                    // Saved in the Keychain and not typed again: use it.
                    let keychain = form.stored_secret
                        && form.remember_password
                        && favorites::secret_of(&cfg).is_none();
                    form.connecting = true;
                    form.error = None;
                    let side = form.side;
                    self.dialog = Some(Dialog::Connect(form));
                    self.start_connect(cfg, favorite, side, keychain);
                }
            },
            Dialog::Transfer(f) => self.submit_transfer(f),
            Dialog::Delete(f) => {
                let mut refresh: Vec<(SharedVfs, VfsPath)> = Vec::new();
                for p in &f.paths {
                    let parent = p.parent().unwrap_or_else(VfsPath::root);
                    if !refresh.iter().any(|(_, d)| *d == parent) {
                        refresh.push((f.fs.clone(), parent));
                    }
                }
                let a =
                    self.new_action(f.side, FileActionKind::Delete, &f.fs, f.paths.clone(), None);
                let action = self.start_action(a);
                let job = Job {
                    kind: JobKind::Delete,
                    src_fs: f.fs,
                    sources: f.paths,
                    dst_fs: None,
                    dst_dir: None,
                    conflict: ConflictPolicy::Skip,
                };
                let job_id = self.enqueue(job, refresh);
                self.actions.attach_job(action, job_id);
            }
            Dialog::Compress(f) => self.submit_compress(f),
            Dialog::EmptyTrash(_) => self.empty_trash(),
            Dialog::HostKey(f) => self.host_key_answered(*f, true),
            Dialog::Conflict(_) => {}
            Dialog::WhatsNew(_) => self.mark_whats_new_seen(),
            Dialog::Name(f) => {
                if let Err(f) = self.submit_name(f) {
                    self.dialog = Some(Dialog::Name(*f));
                }
            }
        }
    }

    /// Create the folder / file or rename as `f` says (the name dialog and
    /// in-place renaming). An invalid name comes back with its error set.
    fn submit_name(&mut self, mut f: NameForm) -> Result<(), Box<NameForm>> {
        match f.validated() {
            Err(e) => {
                f.error = Some(e);
                return Err(Box::new(f));
            }
            Ok(name) => {
                let fs = f.fs.clone();
                let dir = f.dir.clone();
                let target = dir.join(&name);
                // Made inside a folder of the listing: nothing to select
                // there; the folder itself changed too (its date, its size).
                let select = (!f.inside).then(|| (f.tab_id, name.clone()));
                let mut refresh = vec![(fs.clone(), dir.clone())];
                if f.inside {
                    refresh.push((fs.clone(), dir.parent().unwrap_or_else(VfsPath::root)));
                }
                let side = self.side_of_tab(f.tab_id);
                match f.original {
                    None if f.new_file => {
                        let a = self.new_action(
                            side,
                            FileActionKind::NewFile,
                            &fs,
                            vec![target.clone()],
                            None,
                        );
                        let action = Some(self.start_action(a));
                        let undo = Some(Op::remove_new(fs.clone(), target.clone(), false));
                        self.worker.spawn(
                            "touch",
                            move || create_empty_file(&*fs, &target).map_err(|e| e.to_string()),
                            move |result| WorkResult::FileOp {
                                description: t!("status.created_file", name = name),
                                failure: t!("error.create_file", name = name),
                                refresh,
                                select,
                                action,
                                result,
                                undo,
                            },
                        );
                    }
                    None => {
                        let a = self.new_action(
                            side,
                            FileActionKind::NewFolder,
                            &fs,
                            vec![target.clone()],
                            None,
                        );
                        let action = Some(self.start_action(a));
                        let undo = Some(Op::remove_new(fs.clone(), target.clone(), true));
                        self.worker.spawn(
                            "mkdir",
                            move || fs.create_dir(&target).map_err(|e| e.to_string()),
                            move |result| WorkResult::FileOp {
                                description: t!("status.created_folder", name = name),
                                failure: t!("error.create_folder", name = name),
                                refresh,
                                select,
                                action,
                                result,
                                undo,
                            },
                        );
                    }
                    Some(old) if old == name => {}
                    Some(old) => {
                        let from = dir.join(&old);
                        let mut a = self.new_action(
                            side,
                            FileActionKind::Rename,
                            &fs,
                            vec![from.clone()],
                            None,
                        );
                        a.destination = Some(target.clone());
                        let action = Some(self.start_action(a));
                        let undo = Some(Op::Rename {
                            fs: fs.clone(),
                            from: target.clone(),
                            to: from.clone(),
                        });
                        self.worker.spawn(
                            "rename",
                            move || fs.rename(&from, &target).map_err(|e| e.to_string()),
                            move |result| WorkResult::FileOp {
                                description: t!("status.renamed", old = old, name = name),
                                failure: t!("error.rename", name = old),
                                refresh,
                                select,
                                action,
                                result,
                                undo,
                            },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Click-to-rename: the name becomes editable in the List view; the
    /// other views (names under icons, in columns) use the rename dialog.
    /// Nothing happens in read-only places or under a modal.
    fn rename_in_place(&mut self, side: Side) {
        if self.pane(side).tab().loc.read_only() || self.modal_open() {
            return;
        }
        let Some(form) = self.rename_form(side) else {
            return;
        };
        if self.pane(side).view == crate::views::ViewMode::List {
            self.pane_mut(side).tab_mut().inline_rename = Some(pane::InlineRename::new(form));
        } else {
            self.dialog = Some(Dialog::Name(form));
        }
    }

    fn inline_rename_end(&mut self, side: Side, commit: bool, enter: bool) {
        let Some(ir) = self.pane_mut(side).tab_mut().inline_rename.take() else {
            return;
        };
        if !commit {
            return;
        }
        if let Err(f) = self.submit_name(ir.form) {
            if enter {
                // ↩ on an invalid name: keep editing (the field says why).
                self.pane_mut(side).tab_mut().inline_rename = Some(pane::InlineRename::new(*f));
            } else if let Some(e) = f.error {
                self.set_error(e);
            }
        }
    }

    /// Click-to-rename: start editing once the delay after the second
    /// click has passed and the item is still the only selection. The
    /// editor goes away when the pane leaves the List view.
    fn poll_rename_click(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let modal = self.modal_open();
        for pane in &mut self.panes {
            let side = pane.id;
            let list = pane.view == crate::views::ViewMode::List;
            let tab = pane.tab_mut();
            if !list {
                tab.inline_rename = None;
            }
            let Some((name, at)) = &tab.rename_click else {
                continue;
            };
            let left = pane::RENAME_CLICK_DELAY.saturating_sub(at.elapsed());
            if !left.is_zero() {
                ctx.request_repaint_after(left);
                continue;
            }
            let still = tab.selected.len() == 1 && tab.selected.contains(name);
            tab.rename_click = None;
            if still && !modal && tab.inline_rename.is_none() {
                actions.push(Action::RenameInPlace(side));
            }
        }
    }

    /// The rename form for the pane's single selected item (a search
    /// result is renamed in its own folder).
    fn rename_form(&self, side: Side) -> Option<NameForm> {
        let tab = self.pane(side).tab();
        let sel = tab.selected_entries();
        let [entry] = sel.as_slice() else {
            return None;
        };
        if tab.in_search() {
            let dir = entry.path.parent()?;
            // The names next to it (local folders only: no network on the
            // UI thread; servers refuse a clash).
            let existing = match tab.loc.fs.local_path(&dir) {
                Some(_) => tab
                    .loc
                    .fs
                    .list(&dir)
                    .map(|v| v.into_iter().map(|e| e.name).collect())
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            return Some(NameForm::new(
                tab.id,
                tab.loc.fs.clone(),
                dir,
                Some(entry.path.file_name().to_string()),
                existing,
            ));
        }
        let existing = tab.entries.iter().map(|e| e.name.clone()).collect();
        Some(NameForm::new(
            tab.id,
            tab.loc.fs.clone(),
            tab.loc.path.clone(),
            Some(entry.name.clone()),
            existing,
        ))
    }

    // ------------------------------------------------------------ plugins

    /// What plugins know about a pane.
    fn pane_state(&self, side: Side) -> PaneState {
        let tab = self.pane(side).tab();
        PaneState {
            pane: side,
            path: tab.loc.path.clone(),
            local_path: tab.loc.fs.local_path(&tab.loc.path),
            protocol: tab.loc.fs.protocol(),
            backend_label: self.backend_label(&tab.loc),
            selection: {
                // Paths only (no copies of whole entries: 300,000 selected).
                let sum = tab.selection_summary();
                if sum.view_idx.is_empty() {
                    tab.cursor_entry()
                        .map(|e| e.path.clone())
                        .into_iter()
                        .collect()
                } else {
                    sum.view_idx
                        .iter()
                        .filter_map(|&vi| tab.entry_at(vi))
                        .map(|e| e.path.clone())
                        .collect()
                }
            },
            active: side == self.active,
            remote: self.endpoint(&tab.loc.fs),
        }
    }

    /// Cheap fingerprint of everything in [`Self::pane_state`] plus the
    /// listing generation; only when it changes is the state rebuilt.
    fn pane_sig(&self, side: Side) -> u64 {
        use std::hash::{Hash, Hasher};
        let tab = self.pane(side).tab();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (tab.id, tab.loc.conn, &tab.loc.path, tab.listing_gen).hash(&mut h);
        (side == self.active).hash(&mut h);
        // The selection by generation, not by its names: hashing 300,000
        // names every frame was a large part of a frame.
        if tab.selected.is_empty() {
            tab.cursor_entry().map(|e| &e.name).hash(&mut h);
        } else {
            tab.selected.gen().hash(&mut h);
        }
        h.finish()
    }

    /// Diff the panes against what plugins last saw, then act on what the
    /// plugins asked for.
    fn sync_plugins(&mut self) {
        // The panes losing focus first, so `ActivePaneChanged` fires once.
        let mut order = self.pane_order();
        order.retain(|s| *s != self.active);
        order.push(self.active);
        for side in order {
            let sig = self.pane_sig(side);
            if self.pane_sigs.get(&side) != Some(&sig) {
                self.pane_sigs.insert(side, sig);
                let tab = self.pane(side).tab();
                let gen = (tab.id << 32) ^ tab.listing_gen;
                let state = self.pane_state(side);
                self.plugins.set_pane(state, gen);
            }
        }
        for ev in self.plugins.poll() {
            // Requests for a pane that was closed meanwhile are dropped (the
            // plugin got `PaneClosed`).
            let gone = match &ev {
                HostEvent::RenameMany { pane, .. }
                | HostEvent::ListDir { pane, .. }
                | HostEvent::ReadFile { pane, .. }
                | HostEvent::Navigate { pane, .. }
                | HostEvent::OpenSftp { pane, .. }
                | HostEvent::LeaveConnection { pane }
                | HostEvent::Select { pane, .. }
                | HostEvent::Refresh { pane: Some(pane) } => {
                    (!self.has_pane(*pane)).then_some(*pane)
                }
                _ => None,
            };
            if let Some(p) = gone {
                log::debug!("plugin request for closed pane {p} dropped");
                continue;
            }
            match ev {
                HostEvent::Toast { level, text } => {
                    if matches!(level, ToastLevel::Error) {
                        log::warn!("Plugin error toast: {text}");
                    }
                    self.toasts.push(toast_kind(level), text, None);
                }
                HostEvent::ActionToast {
                    level,
                    text,
                    button,
                    command,
                } => {
                    self.toasts.push_with_action(
                        toast_kind(level),
                        text,
                        None,
                        Some((button, command)),
                    );
                }
                HostEvent::RenameMany {
                    plugin,
                    pane,
                    dir,
                    renames,
                } => {
                    self.plugin_rename_many(plugin, pane, dir, renames);
                }
                HostEvent::ListDir { plugin, pane, dir } => self.plugin_list_dir(plugin, pane, dir),
                HostEvent::ReadFile {
                    plugin,
                    pane,
                    path,
                    max_bytes,
                } => self.plugin_read_file(plugin, pane, path, max_bytes),
                HostEvent::Refresh { pane } => {
                    let sides = match pane {
                        Some(p) => vec![p],
                        None => self.pane_order(),
                    };
                    for side in sides {
                        self.refresh(side);
                    }
                }
                HostEvent::Navigate { pane, path } => {
                    self.navigate_path(pane, path);
                }
                HostEvent::OpenSftp {
                    pane,
                    user,
                    host,
                    port,
                } => self.open_sftp(pane, user, host, port),
                HostEvent::LeaveConnection { pane } => self.leave_connection(pane),
                HostEvent::Select { pane, dir, names } => {
                    let side = pane;
                    self.activate(side);
                    let tab = self.pane_mut(side).tab_mut();
                    if tab.loc.path == dir && !tab.loading {
                        tab.select_names(&names);
                    } else {
                        tab.pending_select_many = names;
                        self.navigate_path(side, dir);
                    }
                }
                HostEvent::InstallReview(review) => {
                    self.settings
                        .show(Some(crate::settings::SettingsTab::Plugins));
                    // A newer review replaces one still open (its staging is removed).
                    if let Some(Confirm::Install(old)) = self.settings.confirm.take() {
                        self.plugins.cancel_install(&old);
                    }
                    self.settings.confirm = Some(Confirm::Install(review));
                }
                HostEvent::Installed { name, replaced, .. } => {
                    let text = if replaced {
                        t!("plugins.toast.updated", name = name)
                    } else {
                        t!("plugins.toast.installed", name = name)
                    };
                    self.toasts
                        .push(toast_kind(ToastLevel::Success), text, None);
                }
                HostEvent::InstallFailed { error } => {
                    log::warn!("plugin install failed: {error}");
                    self.toasts.push(
                        toast_kind(ToastLevel::Error),
                        t!("plugins.toast.install_failed", error = error),
                        None,
                    );
                }
                HostEvent::Uninstalled { name, error } => match error {
                    None => self.toasts.push(
                        toast_kind(ToastLevel::Success),
                        t!("plugins.toast.uninstalled", name = name),
                        None,
                    ),
                    Some(e) => self.toasts.push(
                        toast_kind(ToastLevel::Error),
                        t!("plugins.toast.uninstall_failed", name = name, error = e),
                        None,
                    ),
                },
                HostEvent::ConfirmReplace {
                    src,
                    name,
                    installed_version,
                    new_version,
                    ..
                } => {
                    self.settings
                        .show(Some(crate::settings::SettingsTab::Plugins));
                    self.settings.confirm = Some(Confirm::Replace {
                        src,
                        name,
                        installed_version,
                        new_version,
                    });
                }
            }
        }
        // Folder sizes measured by plugins.
        for side in self.pane_order() {
            let pid = side;
            let gen = self.plugins.sizes_gen(pid);
            let shows = self
                .plugins
                .pane(pid)
                .is_some_and(|p| p.path == self.pane(side).tab().loc.path);
            if shows && self.pane(side).tab().sizes_gen != Some(gen) {
                let sizes = self.plugins.sizes(pid).clone();
                self.pane_mut(side).tab_mut().set_dir_sizes(sizes, gen);
            }
        }
    }

    /// Palette rows: app commands, plugin commands, plugin panels.
    fn palette_entries(&self) -> Vec<PaletteEntry> {
        let mut v: Vec<PaletteEntry> = crate::commands::ALL
            .iter()
            .map(|&c| {
                let mut e = PaletteEntry::from_cmd(c, self.cmd_enabled(c));
                // Off with one pane: the row says why instead of its category.
                if let Some(why) = self.cmd_disabled_reason(c) {
                    e.category = why.into();
                }
                e
            })
            .collect();
        for c in self.plugins.commands() {
            v.push(PaletteEntry {
                id: PaletteId::Plugin(c.full_id.clone()),
                label: c.spec.title.clone(),
                category: c.plugin_name.clone(),
                shortcut: c.spec.shortcut.clone().unwrap_or_default(),
                icon: icons::PUZZLE_PIECE,
                keywords: format!("plugin {}", c.plugin_id),
                enabled: true,
            });
        }
        for placement in [PanelPlacement::PaneBottom, PanelPlacement::RightSidebar] {
            for p in self.plugins.panels(placement) {
                let pane = (placement == PanelPlacement::PaneBottom).then_some(self.active);
                let open = self.plugins.is_panel_open(&p.plugin_id, &p.spec.id, pane);
                v.push(PaletteEntry {
                    id: PaletteId::TogglePanel(p.plugin_id.clone(), p.spec.id.clone()),
                    label: if open {
                        t!("palette.hide_panel", title = p.spec.title)
                    } else {
                        t!("palette.show_panel", title = p.spec.title)
                    },
                    category: p.plugin_name.clone(),
                    shortcut: String::new(),
                    icon: icons::SIDEBAR_SIMPLE,
                    keywords: t!("palette.kw.panel").into(),
                    enabled: true,
                });
            }
        }
        v.push(PaletteEntry {
            id: PaletteId::ShowLogFile,
            label: t!("palette.show_log").into(),
            category: t!("palette.cat.help").into(),
            shortcut: String::new(),
            icon: icons::FILE_TEXT,
            keywords: t!("palette.kw.log").into(),
            enabled: true,
        });
        v.push(PaletteEntry {
            id: PaletteId::CheckForUpdates,
            label: if self.updates.enabled() {
                t!("menu.check_updates")
            } else {
                t!("menu.updates_off")
            }
            .into(),
            category: t!("palette.cat.help").into(),
            shortcut: String::new(),
            icon: icons::ARROWS_CLOCKWISE,
            keywords: t!("update.palette.kw").into(),
            enabled: true,
        });
        if self.viewer.supported() {
            let st = &self.viewer.status;
            v.push(PaletteEntry {
                id: PaletteId::ToggleDefaultViewer,
                label: if st.is_default {
                    t!("viewer.toggle.off", name = st.restore_to.name)
                } else {
                    t!("viewer.toggle.on").to_string()
                },
                category: t!("settings.title").into(),
                shortcut: String::new(),
                icon: if st.is_default {
                    icons::ARROW_U_UP_LEFT
                } else {
                    icons::FOLDER_OPEN
                },
                keywords: t!("viewer.palette.kw").into(),
                enabled: self.viewer.can_toggle(),
            });
            v.push(PaletteEntry {
                id: PaletteId::DefaultViewer,
                label: t!("viewer.palette").into(),
                category: t!("settings.title").into(),
                shortcut: String::new(),
                icon: icons::FOLDER_OPEN,
                keywords: t!("viewer.palette.kw").into(),
                enabled: true,
            });
        }
        v
    }

    fn palette_action(&self, id: PaletteId) -> Vec<Action> {
        match id {
            PaletteId::Cmd(c) => self.cmd_actions(c),
            PaletteId::Plugin(id) => vec![Action::RunPluginCommand(id)],
            PaletteId::ShowLogFile => vec![Action::ShowLogFile],
            PaletteId::DefaultViewer => vec![Action::ShowSettingsGeneral],
            PaletteId::ToggleDefaultViewer => vec![Action::ToggleDefaultViewer],
            PaletteId::CheckForUpdates => vec![Action::CheckForUpdates],
            PaletteId::TogglePanel(plugin, panel) => {
                let side = match self
                    .plugins
                    .panels(PanelPlacement::PaneBottom)
                    .iter()
                    .any(|p| p.plugin_id == plugin && p.spec.id == panel)
                {
                    true => Some(self.active),
                    false => None,
                };
                vec![Action::TogglePanel(plugin, panel, side)]
            }
        }
    }

    /// Engine status changes: the toast after the launch check found an
    /// update, quitting once an update is installed (the helper relaunches).
    fn update_events(&mut self, ctx: &egui::Context) {
        let busy = self.transfers.is_busy();
        for e in self.updates.tick(chrono::Utc::now(), busy) {
            match e {
                crate::updates::Event::Notice(v) => self.toasts.push_with_action(
                    ToastKind::Info,
                    t!("update.toast", version = v),
                    None,
                    Some((
                        t!("update.toast.show").into(),
                        crate::updates::TOAST_COMMAND.into(),
                    )),
                ),
                crate::updates::Event::Quit => {
                    log::info!("updates: closing the main window to quit");
                    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            }
        }
        if self.updates.selftest_active() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let before = self.prefs.clone();
        if self.settings.open && !self.settings_was_open {
            self.viewer.refresh();
        }
        self.settings_was_open = self.settings.open;
        self.settings.viewer = self.viewer.supported().then(|| self.viewer.status.clone());
        self.settings.updates = Some(self.updates.settings_info());
        self.settings.cloud.drive_accounts = self
            .cloud
            .locations
            .iter()
            .filter(|l| l.provider == manta_vfs::cloud_local::CloudProvider::GoogleDrive)
            .map(|l| l.account.clone().unwrap_or_else(|| l.display_name.clone()))
            .collect();
        let out = self.settings.ui(ctx, &mut self.plugins, &mut self.prefs);
        if let Some(url) = out.open_link {
            self.open_web_link(url);
        }
        if let Some(a) = out.update_action {
            self.updates.apply_settings(a);
        }
        if let Some(on) = out.viewer_switch {
            self.viewer.switch(on);
        }
        if let Some(e) = out.error {
            self.set_error(e);
        }
        if out.prefs_changed {
            self.prefs_changed(ctx, before);
        }
    }

    /// Finished switches and open / reveal requests from other apps (queued
    /// by the Apple Event handlers; applied here, between frames).
    fn default_viewer_requests(&mut self, ctx: &egui::Context) {
        if let Some(r) = self.viewer.poll() {
            match r {
                Ok(()) if self.viewer.status.is_default && self.viewer.status.folders => {
                    self.set_success(t!("viewer.toast.on"))
                }
                Ok(()) if self.viewer.status.is_default => {
                    self.set_success(t!("viewer.toast.on_reveal"))
                }
                Ok(()) => {
                    let name = self.viewer.status.holder.name.clone();
                    self.set_info(t!("viewer.toast.off", name = name));
                }
                Err(e) => self.set_error_detail(t!("viewer.toast.failed"), e),
            }
        }
        #[cfg(not(test))]
        {
            let reqs = crate::default_viewer::take_requests();
            if !reqs.is_empty() {
                self.apply_open_requests(reqs);
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        #[cfg(test)]
        let _ = ctx;
    }

    /// The first-run "Use Manta as the default file manager?" sheet.
    fn default_viewer_prompt(&mut self, ctx: &egui::Context) {
        if !self.viewer.prompt_open || self.dialog.is_some() {
            return;
        }
        if let Some(choice) =
            crate::default_viewer::prompt_ui(ctx, &self.viewer.status, self.viewer.prompt_manual)
        {
            self.viewer.answer(choice);
        }
    }

    /// Show what other apps asked for: each plan in a new tab of the active
    /// pane, except the very first request of a cold start, which replaces
    /// the start folder of the untouched first tab.
    pub(crate) fn apply_open_requests(&mut self, reqs: Vec<crate::default_viewer::OpenRequest>) {
        for mut req in reqs {
            // A double-clicked .mantaplugin file: the install review sheet
            // (Settings › Plugins opens with it).
            if req.kind == crate::default_viewer::RequestKind::Open {
                let (plugins, rest): (Vec<_>, Vec<_>) = req
                    .paths
                    .into_iter()
                    .partition(|p| crate::default_viewer::is_plugin_file(p));
                req.paths = rest;
                if let Some(first) = plugins.into_iter().next() {
                    log::info!("open: plugin file {}", first.display());
                    self.plugins.review_install(first);
                }
                if req.paths.is_empty() {
                    continue;
                }
            }
            let plans = crate::default_viewer::plan(&req, &|p: &std::path::Path| p.is_dir());
            log::info!("default viewer: {:?} request → {plans:?}", req.kind);
            for plan in plans {
                let replace = self.take_cold_start_slot();
                self.open_plan(plan, replace);
            }
        }
    }

    /// True once, for the first request, if it came in at launch while the
    /// active pane still shows its untouched start folder.
    fn take_cold_start_slot(&mut self) -> bool {
        if std::mem::replace(&mut self.cold_open_used, true) {
            return false;
        }
        let pane = self.pane(self.active);
        self.started.elapsed() < Duration::from_secs(10)
            && pane.tabs.len() == 1
            && pane.tab().back.is_empty()
    }

    fn open_plan(&mut self, plan: crate::default_viewer::OpenPlan, replace: bool) {
        let side = self.active;
        let dir = local_vfs_path(&plan.dir);
        let loc = Location::new(self.local.clone(), None, dir.clone());
        let hidden = plan.select.iter().any(|n| n.starts_with('.'));
        if replace {
            let worker = &self.worker;
            let pi = self.pos(side);
            navigate_tab(worker, self.panes[pi].tab_mut(), loc, false, false);
        } else {
            let id = self.next_id();
            let pane = self.pane_mut(side);
            let mut tab = Tab::new(id, loc);
            tab.sort = pane.tab().sort;
            tab.show_hidden = pane.tab().show_hidden;
            pane.tabs.push(tab);
            pane.active = pane.tabs.len() - 1;
            let worker = &self.worker;
            let pi = self.pos(side);
            start_listing(worker, self.panes[pi].tab_mut(), Some(dir));
        }
        let tab = self.pane_mut(side).tab_mut();
        if hidden {
            tab.show_hidden = true;
        }
        tab.pending_select_many = plan.select;
    }

    /// Apply and save preferences after a change from `before`.
    fn prefs_changed(&mut self, ctx: &egui::Context, before: Prefs) {
        if self.prefs.show_hidden != before.show_hidden {
            for tab in self.panes.iter_mut().flat_map(|p| p.tabs.iter_mut()) {
                tab.show_hidden = self.prefs.show_hidden;
                tab.rebuild_view();
            }
        }
        if (self.prefs.appearance, self.prefs.accent) != (before.appearance, before.accent) {
            theme::apply(ctx, self.prefs.appearance, self.prefs.accent);
        }
        if self.prefs.language != before.language {
            self.apply_language(ctx);
        }
        ctx.request_repaint();
        if let Some(path) = &self.prefs_path {
            if let Err(e) = self.prefs.save(path) {
                self.set_error_detail(t!("error.save_settings"), e);
            }
        }
    }

    /// Switch the UI language to the one in the preferences (and tell plugins).
    fn apply_language(&mut self, ctx: &egui::Context) {
        let l = self.prefs.language.resolve();
        crate::i18n::set_locale(l);
        self.plugins.set_locale(l.code());
        // Chinese needs its font in the chain (see `theme::font_definitions`).
        ctx.set_fonts(theme::font_definitions());
    }

    /// Open panels of `placement` (for `pane`), in plugin order.
    fn open_panels(&self, placement: PanelPlacement, pane: Option<PaneId>) -> Vec<PanelRef> {
        self.plugins
            .panels(placement)
            .into_iter()
            .filter(|p| self.plugins.is_panel_open(&p.plugin_id, &p.spec.id, pane))
            .collect()
    }

    /// The right sidebar with plugin panels (tabs if there are several).
    fn plugin_sidebar(&mut self, ui: &mut egui::Ui) {
        let open = self.open_panels(PanelPlacement::RightSidebar, None);
        if open.is_empty() {
            return;
        }
        let p = pal(ui);
        let resp = egui::Panel::right("plugin-sidebar")
            .resizable(true)
            .default_size(self.layout.sidebar_width)
            .size_range(220.0..=640.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(p.chrome)
                    .inner_margin(egui::Margin {
                        left: 0,
                        right: space::SM as i8,
                        top: space::SM as i8,
                        bottom: space::SM as i8,
                    }),
            )
            .show(ui, |ui| {
                let rect = ui.max_rect();
                // egui sizes a resizable panel to its content's min rect each
                // frame. Claim the whole area, or the panel shrinks by its
                // margins on every repaint (e.g. while the mouse moves).
                ui.set_min_size(rect.size());
                let r = CornerRadius::same(radius::PANEL);
                ui.painter().rect_filled(rect, r, p.content);
                let rects = draw_panel_area(
                    ui,
                    rect.shrink(1.0),
                    "sidebar",
                    &mut self.plugins,
                    &open,
                    &mut self.sidebar_tab,
                    None,
                    self.active,
                    radius::PANEL - 1,
                );
                self.plugin_panel_rects.extend(rects);
                ui.painter().rect_stroke(
                    rect,
                    r,
                    Stroke::new(1.0, p.separator),
                    egui::StrokeKind::Inside,
                );
            });
        let w = resp.response.rect.width();
        let dragging = ui.input(|i| i.pointer.any_down());
        if !dragging && (w - self.layout.sidebar_width).abs() > 1.0 {
            self.layout.sidebar_width = w;
            self.plugins.set_layout(self.layout);
        }
    }

    // ------------------------------------------------------------ plugin requests

    fn batch_rename_available(&self) -> bool {
        self.plugins
            .commands()
            .iter()
            .any(|c| c.full_id == BATCH_RENAME_CMD)
    }

    /// `HostRequest::RenameMany`: rename in the background on the pane's
    /// backend, report one file action, refresh, answer the plugin.
    fn plugin_rename_many(
        &mut self,
        plugin: String,
        pane: PaneId,
        dir: VfsPath,
        renames: Vec<(String, String)>,
    ) {
        let side = pane;
        let fs = self.pane(side).tab().loc.fs.clone();
        if renames.is_empty() {
            let result = RenameResult {
                pane,
                dir,
                ok: Vec::new(),
                failed: Vec::new(),
            };
            self.plugins
                .deliver(&plugin, PluginEvent::RenameResult(result));
            return;
        }
        let sources: Vec<VfsPath> = renames.iter().map(|(o, _)| dir.join(o)).collect();
        let first = dir.join(&renames[0].1);
        let mut a = self.new_action(
            side,
            FileActionKind::Rename,
            &fs,
            sources,
            Some((&fs, first)),
        );
        a.renames = renames
            .iter()
            .map(|(o, n)| (dir.join(o), dir.join(n)))
            .collect();
        let action = self.start_action(a);
        let names: Vec<String> = renames.iter().map(|(o, _)| o.clone()).collect();
        let (job_fs, job_dir) = (fs.clone(), dir.clone());
        self.worker.spawn(
            "rename-many",
            move || Ok(manta_ops::rename::rename_many(&*job_fs, &job_dir, &renames)),
            move |result| WorkResult::RenameMany {
                plugin,
                pane,
                dir,
                fs,
                action,
                outcome: result.unwrap_or_else(|e| manta_ops::rename::RenameOutcome {
                    ok: Vec::new(),
                    failed: names.into_iter().map(|n| (n, e.clone())).collect(),
                }),
            },
        );
    }

    /// `HostRequest::ListDir`: answer from the pane's listing if it shows
    /// `dir`, else list in the background.
    fn plugin_list_dir(&mut self, plugin: String, pane: PaneId, dir: VfsPath) {
        let tab = self.pane(pane).tab();
        if tab.loc.path == dir && !tab.loading && tab.error.is_none() {
            let entries = listed_entries(tab);
            let l = DirListing {
                pane,
                dir,
                entries,
                error: None,
            };
            self.plugins.deliver(&plugin, PluginEvent::DirListing(l));
            return;
        }
        let fs = tab.loc.fs.clone();
        let job_dir = dir.clone();
        self.worker.spawn(
            "plugin-list",
            move || worker::with_reconnect(&fs, || fs.list(&job_dir)).map_err(|e| e.to_string()),
            move |result| WorkResult::PluginListing {
                plugin,
                pane,
                dir,
                result,
            },
        );
    }

    /// `HostRequest::ReadFile`: read through the pane's backend (local or
    /// remote) on a worker.
    fn plugin_read_file(&mut self, plugin: String, pane: PaneId, path: VfsPath, max_bytes: u64) {
        let fs = self.pane(pane).tab().loc.fs.clone();
        let job_path = path.clone();
        self.worker.spawn(
            "plugin-read",
            move || {
                Ok(crate::worker::read_for_plugin(
                    &fs, pane, &job_path, max_bytes,
                ))
            },
            move |r: Result<manta_plugin::FileContent, String>| {
                // `Err` only if the backend panicked.
                let content = r.unwrap_or_else(|e| manta_plugin::FileContent {
                    pane,
                    path,
                    bytes: Vec::new(),
                    size: None,
                    truncated: false,
                    modified: None,
                    error: Some(e),
                });
                WorkResult::PluginFileRead { plugin, content }
            },
        );
    }

    /// `MANTA_AUX_SELFTEST=1`: open Settings as a native window, then
    /// close it with the red button (a `Close` command to its viewport) and
    /// quit, logging each step (checks the native window in a real run).
    #[cfg(not(test))]
    fn aux_selftest(&mut self, ctx: &egui::Context) {
        use crate::aux_window::AuxKind;
        static STEP: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("MANTA_AUX_SELFTEST").is_some()) {
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(100));
        let t = self.started.elapsed().as_secs_f32();
        let step = STEP.load(std::sync::atomic::Ordering::Relaxed);
        let vid = AuxKind::Settings.viewport_id();
        let say = |m: String| {
            log::info!("aux selftest: {m}");
            eprintln!("aux selftest: {m}");
        };
        let next = match step {
            0 if t > 1.0 => {
                say(format!(
                    "embed_viewports={} → opening Settings",
                    ctx.embed_viewports()
                ));
                self.settings.show(None);
                true
            }
            1 if t > 3.0 => {
                let rect = ctx.input_for(vid, |i| i.viewport().outer_rect);
                let main = ctx.input(|i| i.viewport().outer_rect);
                say(format!("Settings open={} native viewport {vid:?} outer rect {rect:?} (main window {main:?})", self.settings.open));
                say(
                    "closing Settings via its window's close button (ViewportCommand::Close)"
                        .into(),
                );
                ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Close);
                true
            }
            2 if t > 4.5 => {
                say(format!(
                    "after close: Settings open={}, main window still running",
                    self.settings.open
                ));
                ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                true
            }
            _ => false,
        };
        if next {
            STEP.store(step + 1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// `MANTA_MENU_SELFTEST=1`: after the menu bar is built, check that the
    /// generated menus list every command with its shortcut and perform
    /// none of them (keys stay with egui), log the result and quit.
    #[cfg(not(test))]
    fn menu_selftest(&mut self, ctx: &egui::Context) {
        static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("MANTA_MENU_SELFTEST").is_some())
            || DONE.load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(200));
        if self.started.elapsed() < Duration::from_millis(1500) {
            return;
        }
        DONE.store(true, std::sync::atomic::Ordering::Relaxed);
        let problems = crate::app_menu::selftest();
        if problems.is_empty() {
            log::info!("menu selftest: ok");
            eprintln!("menu selftest: ok");
        } else {
            for p in &problems {
                log::warn!("menu selftest: {p}");
                eprintln!("menu selftest: FAIL {p}");
            }
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
    }

    /// `MANTA_PANES_SELFTEST=1`: split to three panes, close the middle
    /// one, then quit, logging each step (checks the pane code in a real
    /// run; the log says whether anything went wrong).
    #[cfg(not(test))]
    fn panes_selftest(&mut self, ctx: &egui::Context) {
        static STEP: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("MANTA_PANES_SELFTEST").is_some()) {
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(100));
        let t = self.started.elapsed().as_secs_f32();
        let step = STEP.load(std::sync::atomic::Ordering::Relaxed);
        let say = |m: String| {
            log::info!("panes selftest: {m}");
            eprintln!("panes selftest: {m}");
        };
        let ids = |app: &Self| {
            app.pane_order()
                .iter()
                .map(|p| p.key())
                .collect::<Vec<_>>()
                .join(",")
        };
        let next = match step {
            0 if t > 1.0 => {
                say(format!(
                    "start: {} panes [{}], area {:.0} pt",
                    self.panes.len(),
                    ids(self),
                    self.panes_width
                ));
                while self.panes.len() < 3 {
                    let before = self.panes.len();
                    let last = *self.pane_order().last().unwrap_or(&self.active);
                    self.split_pane(last);
                    if self.panes.len() == before {
                        say("split refused (window too narrow)".into());
                        break;
                    }
                }
                say(format!(
                    "after split: {} panes [{}], active {}",
                    self.panes.len(),
                    ids(self),
                    self.active
                ));
                true
            }
            1 if t > 2.5 => {
                let order = self.pane_order();
                let middle = order[order.len() / 2];
                say(format!(
                    "closing pane {} (id {middle}); plugins know [{}]",
                    self.pane_number(middle),
                    self.plugins
                        .pane_ids()
                        .iter()
                        .map(|p| p.key())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
                self.close_pane(middle);
                say(format!(
                    "after close: {} panes [{}], active {}, target {:?}",
                    self.panes.len(),
                    ids(self),
                    self.active,
                    self.target_of(self.active).map(|p| p.key())
                ));
                true
            }
            2 if t > 3.5 => {
                say(format!(
                    "done: plugins know [{}]; quitting",
                    self.plugins
                        .pane_ids()
                        .iter()
                        .map(|p| p.key())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
                ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                true
            }
            _ => false,
        };
        if next {
            STEP.store(step + 1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// The auxiliary windows (Settings, Compare & Sync, preview) that are open.
    pub(crate) fn open_aux_windows(&self) -> Vec<crate::aux_window::AuxKind> {
        use crate::aux_window::AuxKind;
        [
            (AuxKind::Settings, self.settings.open),
            (AuxKind::Sync, self.sync.open),
            (AuxKind::Preview, self.preview.is_some()),
            (AuxKind::Update, self.updates.open),
            (AuxKind::Help, self.help.open),
        ]
        .into_iter()
        .filter_map(|(k, open)| open.then_some(k))
        .chain(self.info.kinds())
        .collect()
    }

    fn plugin_modal_open(&self) -> bool {
        !self.open_panels(PanelPlacement::Modal, None).is_empty()
    }

    /// The app-modal window that is open, if any (see `modal.rs`):
    /// Software Update (opened from Settings, so it is in front) or
    /// Settings. Help, Compare & Sync and the preview are not modal.
    pub(crate) fn modal_window(&self) -> Option<crate::aux_window::AuxKind> {
        use crate::aux_window::AuxKind;
        if self.updates.open {
            Some(AuxKind::Update)
        } else if self.settings.open {
            Some(AuxKind::Settings)
        } else {
            None
        }
    }

    /// A dialog, sheet or overlay inside the main window is up (a
    /// question, the palette, Go to Path, a plugin sheet, a confirmation
    /// in Settings).
    pub(crate) fn sheet_open(&self) -> bool {
        self.dialog.is_some()
            || self.palette.open
            || self.goto.is_some()
            || self.settings.modal_open()
            || self.plugin_modal_open()
            // The default-file-manager question: Esc / ↩ answer it.
            || self.viewer.prompt_open
    }

    /// Anything modal is open: the main window's shortcuts, menu bar
    /// commands, toast commands and file drops stay off until it closes.
    pub(crate) fn modal_open(&self) -> bool {
        self.sheet_open() || self.modal_window().is_some()
    }

    /// The user clicked or typed into the main window behind an app-modal
    /// window: bring that window back to the front, with the alert sound.
    fn bounce_to_modal_window(&mut self, ctx: &egui::Context) {
        use crate::aux_window::AuxKind;
        let Some(kind) = self.modal_window() else {
            return;
        };
        log::debug!("main window used while {kind:?} is open: bringing it to the front");
        match kind {
            AuxKind::Update => self.updates.window.request_focus(),
            _ => self.settings.bring_to_front(),
        }
        crate::modal::beep();
        ctx.request_repaint();
    }

    /// Per frame while an app-modal window is open: dim the main window
    /// and, when the window is drawn inside the main window (no native
    /// windows: tests, fallback), keep it above the dimming, make it egui's
    /// modal layer (nothing behind it reacts) and bounce clicks outside it.
    fn modal_window_frame(&mut self, ctx: &egui::Context) {
        let Some(kind) = self.modal_window() else {
            return;
        };
        let scrim = crate::modal::scrim(ctx, ctx.embed_viewports());
        if !ctx.embed_viewports() {
            // Native window: `raw_input_hook` keeps the input away.
            return;
        }
        let win = egui::LayerId::new(egui::Order::Middle, kind.window_id());
        let below = ctx.memory(|m| {
            let ids: Vec<egui::LayerId> = m.layer_ids().collect();
            let at = |l: egui::LayerId| ids.iter().position(|x| *x == l);
            at(win) < at(scrim)
        });
        if below {
            ctx.move_to_top(win);
        }
        ctx.memory_mut(|m| m.set_modal_layer(win));
        let press = ctx.input(|i| {
            i.pointer
                .any_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        });
        if let Some(pos) = press {
            let rect = ctx.data(|d| d.get_temp::<egui::Rect>(kind.window_id().with("rect")));
            // `layer_id_at` reports the modal layer for anything behind it.
            let behind = ctx.layer_id_at(pos).is_none_or(|l| l == win)
                && !rect.is_some_and(|r| r.contains(pos));
            if behind {
                self.bounce_to_modal_window(ctx);
            }
        }
    }

    /// The open `Modal` plugin panel (if any): a centred sheet over a
    /// dimmed window with a title bar. Like a macOS sheet it has no close
    /// button: Esc or the panel's own buttons close it.
    fn plugin_modal(&mut self, ctx: &egui::Context) {
        let Some(panel) = self
            .open_panels(PanelPlacement::Modal, None)
            .into_iter()
            .next()
        else {
            return;
        };
        let Some(state) = self.plugins.pane(self.active).cloned() else {
            return;
        };
        let p = theme::pal_ctx(ctx);
        let screen = ctx.content_rect();
        let [w, h] = panel.size.unwrap_or([560.0, 420.0]);
        let size = egui::vec2(
            w.min(screen.width() - 2.0 * space::XL * 2.0).max(320.0),
            h.min(screen.height() - 2.0 * space::XL * 2.0).max(220.0),
        );
        let (plugin, id) = (panel.plugin_id.clone(), panel.spec.id.clone());
        let modal = egui::Modal::new(egui::Id::unique(("plugin-modal", &plugin, &id)))
            .backdrop_color(crate::a11y::scrim(p.dark, 60, 110))
            .frame(
                egui::Frame::new()
                    .fill(p.elevated)
                    .stroke(Stroke::new(1.0, p.separator))
                    .corner_radius(CornerRadius::same(radius::WINDOW))
                    .shadow(ctx.global_style().visuals.window_shadow)
                    .inner_margin(egui::Margin::ZERO),
            );
        let plugins = &mut self.plugins;
        let resp = modal.show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let bar = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(rect.width(), plugin_ui::TITLE_BAR_H + 4.0),
            );
            let tabs = [plugin_ui::TitleTab {
                title: &panel.spec.title,
                glyph: plugin_ui::plugin_glyph(&plugin, plugin.starts_with("manta.")),
                plugin_name: &panel.plugin_name,
            }];
            let out = plugin_ui::title_bar(
                ui,
                bar,
                egui::Id::unique(("modal-title", &plugin, &id)),
                &tabs,
                0,
                radius::WINDOW,
                false,
            );
            let body = egui::Rect::from_min_max(egui::pos2(rect.left(), bar.bottom()), rect.max);
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(body)
                    .id_salt(("modal-body", &plugin, &id))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(body.intersect(ui.clip_rect()));
                    match plugins.panel_content(&plugin, &id, None) {
                        PanelContent::Native => plugins.panel_ui(&plugin, &id, &state, ui),
                        PanelContent::Empty => plugin_ui::empty_panel(ui, body),
                        PanelContent::Blocks(blocks) => {
                            let mut clicked = None;
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    egui::Frame::new()
                                        .inner_margin(egui::Margin::symmetric(
                                            space::XL as i8,
                                            space::LG as i8,
                                        ))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            plugin_ui::blocks_ui(ui, &blocks, &mut clicked);
                                        });
                                });
                            if let Some(action) = clicked {
                                plugins.panel_action(&plugin, &id, &action, &state);
                            }
                        }
                    }
                },
            );
            out.close
        });
        self.plugin_panel_rects.push(resp.response.rect);
        // Esc closes the sheet (unless one of its popups is open); a click
        // on the backdrop doesn't, so no work is lost by accident.
        let esc = !resp.any_popup_open
            && resp.is_top_modal
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if resp.inner || esc {
            self.plugins.set_panel_open(&plugin, &id, None, false);
        }
    }

    // ------------------------------------------------------------ keyboard

    /// Keyboard focus bookkeeping at the end of a frame: give the focus
    /// back to where it was when a dialog, sheet or modal window closes (no
    /// keyboard trap, no lost place), and draw the focus ring.
    fn keyboard_focus_frame(&mut self, ctx: &egui::Context) {
        // Give the focus back one frame after the dialog closed: until then
        // egui still treats its (gone) layer as modal and ignores focus
        // requests behind it.
        if let Some(before) = self.focus_restore.take() {
            match before {
                Some(id) => ctx.memory_mut(|m| m.request_focus(id)),
                None => {
                    if let Some(id) = ctx.memory(|m| m.focused()) {
                        ctx.memory_mut(|m| m.surrender_focus(id));
                    }
                }
            }
        }
        let focused = ctx.memory(|m| m.focused());
        match (self.modal_open(), self.focus_before_modal) {
            (true, None) => {
                // Remember where the keyboard was; a control behind the
                // dialog lets go, so Tab starts inside the dialog (egui's
                // modal layer keeps it there).
                let before = self.last_focus;
                self.focus_before_modal = Some(before);
                if let Some(id) = before.filter(|id| focused == Some(*id)) {
                    ctx.memory_mut(|m| m.surrender_focus(id));
                }
            }
            (false, Some(before)) => {
                self.focus_before_modal = None;
                self.focus_restore = Some(before);
                ctx.request_repaint();
            }
            _ => {}
        }
        self.last_focus = ctx.memory(|m| m.focused());
        crate::a11y::paint_focus_ring(ctx);
    }

    /// The mouse's side buttons go back and forward in the pane under the
    /// pointer (else the active one), like ⌘[ and ⌘]. Settings → Mouse
    /// side buttons swaps or turns them off.
    fn handle_mouse_nav(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let Some(step) = ctx.input(|i| self.prefs.mouse_nav.step(i)) else {
            return;
        };
        if self.sheet_open()
            || self.modal_window().is_some()
            || crate::aux_window::embedded_key(ctx).is_some()
        {
            return;
        }
        let pos = ctx.input(|i| i.pointer.interact_pos());
        let side = pos
            .and_then(|p| self.pane_rects.iter().find(|(_, r)| r.contains(p)))
            .map_or(self.active, |(s, _)| *s);
        self.activate(side);
        actions.push(match step {
            crate::prefs::NavStep::Back => Action::Back(side),
            crate::prefs::NavStep::Forward => Action::Forward(side),
        });
    }

    fn handle_keys(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let step = std::mem::take(&mut self.pending_tab_switch);
        if step != 0 && !self.modal_open() {
            self.activate(crate::pane_set::cycle(
                &self.pane_order(),
                self.active,
                step,
            ));
        }
        // Sheets handle their own keys (Esc, ↩, typing).
        if self.sheet_open() {
            return;
        }
        // An app-modal window, or (fallback mode, no native windows) an
        // auxiliary window with keyboard focus: the main window's shortcuts
        // stay off; ⌘, still brings Settings forward and ⌘? opens Help.
        if self.modal_window().is_some() || crate::aux_window::embedded_key(ctx).is_some() {
            if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Comma)) {
                actions.push(Action::ShowSettings);
            }
            if crate::aux_window::embedded_key(ctx) != Some(crate::aux_window::AuxKind::Help)
                && help_key(ctx)
            {
                actions.push(Action::ShowHelp(String::new()));
            }
            return;
        }
        let side = self.active;
        // Keyboard focus rules (see docs/plugins.md, "Keyboard focus"):
        // - while any widget has focus (a text field, the terminal, a
        //   focused button), shortcuts without ⌘ are left to that widget;
        // - in a text field only the palette, Go to Path, Settings and
        //   plugin chords work (the field needs ⌘A, ⌘←, ⌘⌫ …);
        // - in a plugin panel ⌘-shortcuts work except ⌘C, ⌘V, ⌘A, ⌘K,
        //   ⌘←, ⌘→ and ⌘⌫, which the panel (the terminal) handles.
        // A file list's own Tab stop counts as "no control has focus".
        let any_focus = ctx.memory(|m| m.focused().is_some()) && !pane::list_has_focus(ctx);
        let text_focus = ctx.text_edit_focused();
        let panel_focus = self.plugin_focus && any_focus;
        let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        let key = |mods: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(mods, k));
        // Plugin shortcuts are global; ones without ⌘/⌃ only when nothing
        // has focus (they would collide with typing).
        for c in self.plugins.commands() {
            if let Some(sc) = c.shortcut {
                if (sc.is_chord() || !any_focus) && ctx.input_mut(|i| sc.consume(i)) {
                    actions.push(Action::RunPluginCommand(c.full_id.clone()));
                }
            }
        }
        // The palette and "Go to Path" also open from inside text fields.
        if key(cmd_shift, Key::P) || key(Modifiers::COMMAND, Key::P) {
            actions.push(Action::ShowPalette);
            return;
        }
        if key(cmd_shift, Key::G) {
            actions.push(Action::GoToPath(side));
            return;
        }
        if key(Modifiers::COMMAND, Key::Comma) {
            actions.push(Action::ShowSettings);
            return;
        }
        if help_key(ctx) {
            actions.push(Action::ShowHelp(String::new()));
            return;
        }
        // ⌃F5 (macOS: "Move focus to the window toolbar") moves the keyboard
        // from the file list into the window's controls; Tab / ⇧Tab then go
        // through toolbar, sidebar, panes and status bar, Esc comes back.
        if key(Modifiers::CTRL, Key::F5) {
            if let Some(id) = self.toolbar_first {
                ctx.memory_mut(|m| m.request_focus(id));
            }
            return;
        }
        if text_focus {
            return;
        }

        // Most specific modifier combinations first (see `consume_key` docs).
        if key(cmd_shift, Key::N) {
            actions.push(Action::NewFolder(side));
        }
        if key(Modifiers::COMMAND | Modifiers::ALT, Key::N) {
            actions.push(Action::NewFile(side));
        }
        if key(cmd_shift, Key::S) {
            actions.push(Action::ShowSync);
        }
        if key(cmd_shift, Key::Period) {
            actions.push(Action::ToggleHidden(side));
        }
        if !panel_focus && key(Modifiers::COMMAND, Key::K) {
            actions.push(Action::ShowConnect);
        }
        if key(Modifiers::COMMAND | Modifiers::ALT, Key::C) {
            actions.push(Action::Compress(side));
        }
        if key(Modifiers::COMMAND | Modifiers::ALT, Key::E) {
            actions.push(Action::ExtractHere(side));
        }
        // Panes: ⇧⌘D before ⌘D (a pattern without ⇧ also matches with it).
        if key(cmd_shift, Key::D) {
            actions.push(Action::ClosePane(side));
        }
        if key(Modifiers::COMMAND, Key::D) {
            actions.push(Action::SplitPane(side));
        }
        // ⌘1 – ⌘4: the active pane's view (Finder's shortcuts).
        for (k, mode) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4]
            .into_iter()
            .zip(crate::views::ViewMode::ALL)
        {
            if key(Modifiers::COMMAND, k) {
                actions.push(Action::SetView(side, mode));
            }
        }
        if key(Modifiers::CTRL | Modifiers::COMMAND, Key::T) {
            actions.push(Action::AddCurrentToFavorites);
        }
        // ⌘T: new tab (Finder). ⇧⌘T, the old shortcut, still matches it.
        if key(Modifiers::COMMAND, Key::T) {
            actions.push(Action::NewTab(side));
        }
        // ⌥⌘L: Transfers (like Safari's downloads), before ⌘L.
        if key(Modifiers::COMMAND | Modifiers::ALT, Key::L) {
            actions.push(Action::ToggleTransfers);
        }
        // ⌘Y: Quick Look, as in Finder (Space too).
        if key(Modifiers::COMMAND, Key::Y) {
            actions.push(Action::Preview(side));
        }
        // ⌘I: Get Info (Finder).
        if key(Modifiers::COMMAND, Key::I) {
            actions.push(Action::GetInfo(side));
        }
        if key(Modifiers::COMMAND, Key::R) {
            actions.push(Action::Refresh(side));
        }
        if key(Modifiers::COMMAND, Key::W) {
            let active = self.pane(side).active;
            actions.push(Action::CloseTab(side, active));
        }
        if key(Modifiers::COMMAND, Key::L) {
            let tab = self.pane_mut(side).tab_mut();
            tab.path_edit = Some(tab.loc.path.to_string());
        }
        if key(Modifiers::COMMAND, Key::F) {
            self.pane_mut(side).tab_mut().filter_focus = true;
            ctx.memory_mut(|m| m.request_focus(pane::filter_field_id(side)));
            ctx.request_repaint();
        }
        if key(Modifiers::COMMAND, Key::ArrowUp) {
            actions.push(Action::Up(side));
        }
        if key(Modifiers::COMMAND, Key::ArrowDown) {
            actions.push(Action::OpenSelected(side));
        }
        // ⌥⌘← / ⌥⌘→ move the focus between panes (before ⌘← / ⌘→, which
        // would match them too).
        if key(Modifiers::COMMAND | Modifiers::ALT, Key::ArrowLeft) {
            actions.push(Action::FocusPane(-1));
        }
        if key(Modifiers::COMMAND | Modifiers::ALT, Key::ArrowRight) {
            actions.push(Action::FocusPane(1));
        }
        // A focused plugin panel (the terminal) keeps ⌘←, ⌘→, ⌘⌫, ⌘K, ⌘A.
        if key(Modifiers::COMMAND, Key::OpenBracket)
            || (!panel_focus && key(Modifiers::COMMAND, Key::ArrowLeft))
        {
            actions.push(Action::Back(side));
        }
        if key(Modifiers::COMMAND, Key::CloseBracket)
            || (!panel_focus && key(Modifiers::COMMAND, Key::ArrowRight))
        {
            actions.push(Action::Forward(side));
        }
        // ⇧⌘⌫ empties the Trash (asks), ⌥⌘⌫ deletes permanently (asks),
        // ⌘⌫ moves to the Trash (Finder). Most modifiers first: egui
        // ignores extra ⇧ / ⌥ when matching.
        if !panel_focus && key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Backspace) {
            actions.push(Action::EmptyTrash);
        }
        if !panel_focus && key(Modifiers::COMMAND | Modifiers::ALT, Key::Backspace) {
            actions.push(Action::DeletePermanently(side));
        }
        if !panel_focus && key(Modifiers::COMMAND, Key::Backspace) {
            actions.push(Action::Delete(side));
        }
        // ⇧⌘Z before ⌘Z (a pattern without ⇧ also matches with it). A
        // focused text field keeps both for its text (see `text_focus`).
        if !panel_focus && key(cmd_shift, Key::Z) {
            actions.push(Action::Redo);
        }
        if !panel_focus && key(Modifiers::COMMAND, Key::Z) {
            actions.push(Action::Undo);
        }
        if !panel_focus && key(Modifiers::COMMAND, Key::A) {
            self.pane_mut(side).tab_mut().select_all();
        }
        if any_focus {
            return;
        }
        if key(Modifiers::NONE, Key::Delete) {
            actions.push(Action::Delete(side));
        }
        let key = |mods: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(mods, k));
        if key(Modifiers::NONE, Key::F2) {
            actions.push(Action::Rename(side));
        }
        if key(Modifiers::NONE, Key::F5) {
            actions.push(Action::CopyToOther(side, false));
        }
        if key(Modifiers::NONE, Key::F6) {
            actions.push(Action::CopyToOther(side, true));
        }
        if key(Modifiers::ALT, Key::Enter) {
            actions.push(Action::OpenWithApp(side));
        }
        if key(Modifiers::NONE, Key::Enter) {
            actions.push(Action::OpenSelected(side));
        }
        if key(Modifiers::NONE, Key::Backspace) {
            actions.push(Action::Up(side));
        }
        // Type-to-select: letters typed into the file list jump to the
        // first matching name. While a word is being typed (within about a
        // second), Space belongs to it instead of opening Quick Look.
        let now = std::time::Instant::now();
        let typing = self.pane(side).tab().type_ahead_active(now);
        let typed: String = ctx.input_mut(|i| {
            let mut s = String::new();
            i.events.retain(|e| match e {
                egui::Event::Text(t) if !i.modifiers.command && !i.modifiers.ctrl => {
                    // Spaces are handled through Key::Space below (text and
                    // key events may arrive in different frames).
                    let t: String = t.chars().filter(|c| !c.is_control() && *c != ' ').collect();
                    s.push_str(&t);
                    false
                }
                _ => true,
            });
            s
        });
        if !typed.is_empty() {
            self.pane_mut(side).tab_mut().type_ahead(&typed, now);
        }
        if typing && key(Modifiers::NONE, Key::Space) {
            self.pane_mut(side).tab_mut().type_ahead(" ", now);
        }
        if key(Modifiers::NONE, Key::Space) {
            actions.push(Action::Preview(side));
        }
        // Esc that just took the focus from a control (egui drops it) hands
        // the keyboard back to the list and does nothing else.
        if self.last_focus.is_some()
            && self.last_focus != pane::list_anchor_focus(ctx)
            && key(Modifiers::NONE, Key::Escape)
        {
            return;
        }
        if key(Modifiers::NONE, Key::Escape) {
            if let Some(p) = self.preview.take() {
                p.release(ctx);
            } else if self.quicklook.is_open() {
                self.quicklook.close();
            } else {
                let tab = self.pane_mut(side).tab_mut();
                tab.selected.clear();
            }
        }
        // Arrows etc. move the cursor as the pane's view lays items out
        // (grid, columns, filmstrip); see `views::nav`.
        use crate::views::{Nav, NavOut};
        let pi = self.pos(side);
        let view = self.panes[pi].view;
        let tab = self.panes[pi].tab_mut();
        let keys = [
            (Key::ArrowDown, Nav::Down, true),
            (Key::ArrowUp, Nav::Up, true),
            (Key::ArrowLeft, Nav::Left, true),
            (Key::ArrowRight, Nav::Right, true),
            (Key::PageDown, Nav::PageDown, false),
            (Key::PageUp, Nav::PageUp, false),
            (Key::Home, Nav::Home, false),
            (Key::End, Nav::End, false),
        ];
        for (k, nav, shift_extends) in keys {
            let uses = crate::views::nav_uses(view, nav);
            if !uses {
                continue;
            }
            let out = if shift_extends && ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, k)) {
                crate::views::nav(view, tab, nav, true)
            } else if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k)) {
                crate::views::nav(view, tab, nav, false)
            } else {
                continue;
            };
            match out {
                NavOut::Parent => actions.push(Action::Up(side)),
                NavOut::Enter => actions.push(Action::ColumnsEnter(side)),
                NavOut::Done | NavOut::Unused => {}
            }
        }
    }

    // ------------------------------------------------------------ chrome

    fn toolbar(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let p = pal(ui);
        let side = self.active;
        let wide = ui.available_width() > 1000.0;
        let mut rects = Vec::new();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let mut first: Option<egui::Id> = None;
            let mut tb = |ui: &mut egui::Ui, cmd: Cmd, glyph: &str, selected: bool, label: &str| {
                let enabled = self.cmd_enabled(cmd);
                let r = widgets::icon_button(ui, glyph, enabled, selected);
                if enabled && first.is_none() {
                    first = Some(r.id);
                }
                rects.push((cmd, r.rect));
                // A two-pane command with one pane says why it is off.
                // Quick Look: Space and ⌘Y, as in Finder.
                let shortcut = match cmd {
                    Cmd::QuickLook => format!("{} / ⌘Y", cmd.shortcut()),
                    _ => cmd.shortcut().to_string(),
                };
                let r = match self.cmd_disabled_reason(cmd) {
                    Some(why) => widgets::tooltip(r, &format!("{label}\n{why}"), ""),
                    None => widgets::tooltip(r, label, &shortcut),
                };
                if r.clicked() {
                    actions.extend(self.cmd_actions(cmd));
                }
            };
            let group_gap = |ui: &mut egui::Ui| {
                ui.add_space(space::MD);
                let (r, _) = ui.allocate_exact_size(egui::vec2(1.0, 18.0), Sense::hover());
                ui.painter()
                    .vline(r.center().x, r.y_range(), Stroke::new(1.0, p.separator));
                ui.add_space(space::MD);
            };
            tb(ui, Cmd::Back, icons::CARET_LEFT, false, t!("cmd.back"));
            tb(
                ui,
                Cmd::Forward,
                icons::CARET_RIGHT,
                false,
                t!("cmd.forward"),
            );
            ui.add_space(space::XS);
            tb(
                ui,
                Cmd::EnclosingFolder,
                icons::ARROW_UP,
                false,
                t!("cmd.enclosing_folder"),
            );
            tb(
                ui,
                Cmd::GoToPath,
                icons::SIGNPOST,
                self.goto.is_some(),
                t!("cmd.go_to_path"),
            );
            tb(
                ui,
                Cmd::Refresh,
                icons::ARROW_CLOCKWISE,
                false,
                t!("cmd.refresh"),
            );
            group_gap(ui);
            tb(
                ui,
                Cmd::NewFolder,
                icons::FOLDER_PLUS,
                false,
                Cmd::NewFolder.label(),
            );
            tb(
                ui,
                Cmd::Rename,
                icons::PENCIL_SIMPLE,
                false,
                Cmd::Rename.label(),
            );
            // Servers have no Trash: the button deletes permanently there.
            let loc = &self.pane(side).tab().loc;
            let delete = if can_trash(&loc.fs) || loc.archive.is_some() {
                Cmd::MoveToTrash
            } else {
                Cmd::DeletePermanently
            };
            tb(ui, delete, icons::TRASH, false, delete.label());
            group_gap(ui);
            let (copy_tip, move_tip, move_icon) = self.copy_tips(side);
            tb(ui, Cmd::CopyToOther, icons::COPY, false, &copy_tip);
            tb(ui, Cmd::MoveToOther, move_icon, false, &move_tip);
            tb(ui, Cmd::QuickLook, icons::EYE, false, t!("cmd.quick_look"));
            group_gap(ui);
            let hidden = self.pane(side).tab().show_hidden;
            tb(
                ui,
                Cmd::ToggleHidden,
                // The eye is Quick Look; hidden files are a dashed file
                // (highlighted while they are shown).
                Cmd::ToggleHidden.icon(),
                hidden,
                if hidden {
                    t!("toolbar.hide_hidden")
                } else {
                    t!("cmd.toggle_hidden")
                },
            );
            tb(
                ui,
                Cmd::CompareSync,
                icons::ARROWS_LEFT_RIGHT,
                self.sync.open,
                Cmd::CompareSync.label(),
            );
            self.toolbar_first = first;

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = space::SM;
                // Settings, at the far right like most Mac toolbars.
                let r = widgets::icon_button(ui, icons::GEAR_SIX, true, self.settings.open);
                if widgets::tooltip(r, Cmd::Settings.label(), Cmd::Settings.shortcut()).clicked() {
                    actions.push(Action::ShowSettings);
                }
                // Command palette launcher, styled like a search field.
                let w = if wide { 210.0 } else { 34.0 };
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 26.0), Sense::click());
                let fill = if resp.hovered() {
                    ui.visuals().widgets.hovered.bg_fill
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };
                ui.painter().rect(
                    rect,
                    CornerRadius::same(radius::CONTROL + 2),
                    fill,
                    Stroke::new(1.0, p.separator),
                    egui::StrokeKind::Inside,
                );
                let ir = egui::Rect::from_min_size(rect.min, egui::vec2(30.0, rect.height()));
                widgets::paint_icon(
                    ui,
                    ir,
                    icons::MAGNIFYING_GLASS,
                    14.0,
                    p.text_secondary,
                    false,
                );
                if wide {
                    ui.painter().text(
                        egui::pos2(ir.right() - 2.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        t!("toolbar.commands"),
                        theme::regular(text::BODY),
                        p.text_tertiary,
                    );
                    ui.painter().text(
                        egui::pos2(rect.right() - space::MD, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        Cmd::CommandPalette.shortcut(),
                        theme::regular(text::SMALL),
                        p.text_tertiary,
                    );
                }
                if widgets::tooltip(
                    resp,
                    Cmd::CommandPalette.label(),
                    Cmd::CommandPalette.shortcut(),
                )
                .clicked()
                {
                    actions.push(Action::ShowPalette);
                }

                // Transfers, with a count badge while busy.
                let busy = self
                    .jobs
                    .iter()
                    .filter(|j| !transfers::is_finished(&j.state))
                    .count();
                let r = widgets::icon_button(ui, icons::ARROWS_DOWN_UP, true, self.show_transfers);
                if busy > 0 {
                    let c = r.rect.right_top() + egui::vec2(-6.0, 6.0);
                    let label = busy.min(99).to_string();
                    let g =
                        ui.painter()
                            .layout_no_wrap(label, theme::semibold(10.0), p.text_on_accent);
                    let br = egui::Rect::from_center_size(
                        c,
                        egui::vec2((g.size().x + 8.0).max(15.0), 15.0),
                    );
                    ui.painter()
                        .rect_filled(br, CornerRadius::same(radius::PILL), p.accent);
                    ui.painter()
                        .galley(br.center() - g.size() / 2.0, g, p.text_on_accent);
                }
                let tip = if busy > 0 {
                    t!("toolbar.transfers_active", n = busy)
                } else {
                    Cmd::Transfers.label().to_string()
                };
                if widgets::tooltip(r, &tip, Cmd::Transfers.shortcut()).clicked() {
                    actions.push(Action::ToggleTransfers);
                }

                let label = format!("{}  {}", icons::GLOBE, t!("toolbar.connect"));
                let r = widgets::button(ui, &label, widgets::ButtonKind::Secondary, true);
                if widgets::tooltip(r, Cmd::Connect.label(), Cmd::Connect.shortcut()).clicked() {
                    actions.push(Action::ShowConnect);
                }
            });
        });
        self.toolbar_rects = rects;
    }

    /// Toolbar tooltips of Copy / Move for the target of `side`: left /
    /// right with two panes, the pane's number and folder with more.
    fn copy_tips(&self, side: Side) -> (String, String, &'static str) {
        let Some(to) = self.target_of(side) else {
            return (
                Cmd::CopyToOther.label().into(),
                Cmd::MoveToOther.label().into(),
                icons::ARROW_LINE_RIGHT,
            );
        };
        let right = self.pos(to) > self.pos(side);
        let icon = if right {
            icons::ARROW_LINE_RIGHT
        } else {
            icons::ARROW_LINE_LEFT
        };
        if self.panes.len() == 2 {
            return if right {
                (
                    t!("toolbar.copy_right").into(),
                    t!("toolbar.move_right").into(),
                    icon,
                )
            } else {
                (
                    t!("toolbar.copy_left").into(),
                    t!("toolbar.move_left").into(),
                    icon,
                )
            };
        }
        let n = self.pane_number(to);
        let name = self.pane(to).tab().title();
        (
            t!("toolbar.copy_to_pane", n = n, name = name),
            t!("toolbar.move_to_pane", n = n, name = name),
            icon,
        )
    }

    fn status_bar(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let p = pal(ui);
        let space = self.free_space();
        let tab = self.pane(self.active).tab();
        let caption = theme::regular(text::CAPTION + 0.5);
        ui.horizontal_centered(|ui| {
            let total = tab.view.len();
            let mut s = t!("status.items", count = total);
            let hidden = util::listable_count(&tab.entries).saturating_sub(total);
            if hidden > 0 && tab.filter.is_empty() && !tab.show_hidden {
                s.push_str(&t!("status.hidden", n = util::format_count(hidden as u64)));
            }
            if !tab.filter.is_empty() {
                s.push_str(&t!("status.matching", filter = tab.filter.trim()));
            }
            let sel = tab.selected.len();
            if sel > 0 {
                let size = tab.selection_size();
                s.push_str(&t!("status.selected", n = util::format_count(sel as u64)));
                if size > 0 {
                    s.push_str(&format!(" ({})", util::format_size(size)));
                }
            }
            if let Some(sp) = space {
                s.push_str(&t!(
                    "status.available",
                    size = util::format_size(sp.available)
                ));
            }
            let r = ui.label(
                RichText::new(s)
                    .font(caption.clone())
                    .color(p.text_secondary),
            );
            if let Some(total) = space.and_then(|sp| sp.total.map(|t| (sp.available, t))) {
                r.on_hover_text(t!(
                    "status.available.tip",
                    available = util::format_size(total.0),
                    total = util::format_size(total.1)
                ));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Job progress while the transfers panel is hidden ("Kopiere
                // 3 Objekte … 42 %", "Fertig", errors until seen); else what
                // a cloud provider still uploads in a visible folder.
                let job_pill = (!self.show_transfers)
                    .then(|| self.transfer_pill.pill(&self.jobs, Instant::now()))
                    .flatten();
                let cloud_pill = if job_pill.is_none() {
                    self.cloud_upload_pill()
                } else {
                    None
                };
                // The pills keep their room; the palette hint and the
                // plugins' status items share the rest (see status_items::fit).
                let gap = ui.spacing().item_spacing.x;
                let mut pills_w = 0.0;
                if self.updates.notice.is_some() {
                    pills_w += crate::update_window::status_pill_width(ui, &self.updates)
                        + space::MD
                        + gap;
                }
                if let Some(pill) = job_pill.as_ref().or(cloud_pill.as_ref().map(|(_, p)| p)) {
                    pills_w += crate::transfer_pill::width(ui, pill) + space::LG + gap;
                }
                let avail = ui.available_width() - pills_w - space::XL;
                self.status_right(ui, avail, actions);
                if self.updates.notice.is_some() {
                    ui.add_space(space::MD);
                    if crate::update_window::status_pill(ui, &self.updates) {
                        actions.push(Action::ShowUpdate);
                    }
                }
                if let Some(pill) = job_pill {
                    ui.add_space(space::LG);
                    let r = crate::transfer_pill::show(ui, &pill);
                    let tip = crate::transfer_pill::tooltip(&self.jobs);
                    if widgets::tooltip(r, &tip, Cmd::Transfers.shortcut()).clicked() {
                        actions.push(Action::ToggleTransfers);
                    }
                } else if let Some((provider, pill)) = cloud_pill {
                    ui.add_space(space::LG);
                    let r = crate::transfer_pill::show(ui, &pill);
                    let tip = if pill.kind == crate::transfer_pill::PillKind::UploadError {
                        t!("pill.upload_errors_tip", provider = provider)
                    } else {
                        t!("pill.uploading_tip", provider = provider)
                    };
                    r.on_hover_text(tip);
                }
            });
        });
    }

    /// Show a notice and return true if the selection of `side` holds an
    /// app's folder in iCloud Drive or one of Apple's apps: like in Finder,
    /// those can't be renamed, moved or deleted (`removal`: moving or
    /// deleting).
    fn refuse_app_folders(&mut self, side: Side, removal: bool) -> bool {
        let tab = self.pane(side).tab();
        if tab.loc.conn.is_some() || tab.loc.archive.is_some() {
            return false;
        }
        let paths: Vec<VfsPath> = tab.selected_entries().into_iter().map(|e| e.path).collect();
        self.refuse_app_folder_paths(&paths, removal)
    }

    fn refuse_app_folder_paths(&mut self, paths: &[VfsPath], removal: bool) -> bool {
        // Apple's apps (Applications lists them, like Finder): macOS
        // protects them.
        if let Some(p) = paths
            .iter()
            .find(|p| util::sip_protected(std::path::Path::new(p.as_str())))
        {
            let name = p.file_name().to_string();
            let title = if removal {
                t!("system.cant_remove", name = name)
            } else {
                t!("system.cant_rename", name = name)
            };
            self.toasts.push(
                ToastKind::Info,
                title,
                Some(t!("system.protected_detail").into()),
            );
            return true;
        }
        let Some(name) = paths
            .iter()
            .map(|p| std::path::Path::new(p.as_str()))
            .find(|p| manta_vfs::icloud::is_app_library(p))
            .and_then(manta_vfs::icloud::folder_label)
        else {
            return false;
        };
        let title = if removal {
            t!("icloud.cant_remove_app_folder", name = name)
        } else {
            t!("icloud.cant_rename_app_folder", name = name)
        };
        self.toasts.push(
            ToastKind::Info,
            title,
            Some(t!("icloud.app_folder_detail").into()),
        );
        true
    }

    /// Free space where the active pane is; asks for it (again) when it
    /// is unknown or old. Shows the last answer meanwhile.
    fn free_space(&mut self) -> Option<manta_core::Space> {
        if cfg!(test) && !self.test_space {
            return None;
        }
        let loc = self.pane(self.active).tab().loc.clone();
        let key = crate::space::SpaceCache::key(&loc)?;
        if self.space.should_ask(&key, Instant::now()) {
            let (fs, path, k) = (loc.fs, loc.path, key.clone());
            self.worker.spawn(
                "space",
                move || fs.space(&path).map_err(|e| e.to_string()),
                move |result| WorkResult::Space { key: k, result },
            );
        }
        self.space.get(&key)
    }

    /// The right end of the status bar (in a right-to-left layout): the
    /// command palette hint and, left of it, the plugins' status items,
    /// shrunk to fit `avail` points.
    fn status_right(&self, ui: &mut egui::Ui, avail: f32, actions: &mut Vec<Action>) {
        use crate::status_items::{self as si, HintForm, ItemForm};
        let p = pal(ui);
        let caption = theme::regular(text::CAPTION + 0.5);
        let items = self.plugins.status_items();
        let full: Vec<f32> = items
            .iter()
            .map(|(_, it)| si::width(ui, it, ItemForm::Full))
            .collect();
        let compact: Vec<Option<f32>> = items
            .iter()
            .map(|(_, it)| {
                it.compact
                    .as_ref()
                    .map(|_| si::width(ui, it, ItemForm::Compact))
            })
            .collect();
        let priority: Vec<i32> = items.iter().map(|(_, it)| it.priority).collect();
        let shortcut = Cmd::CommandPalette.shortcut();
        let hint_full = t!("status.palette_hint", shortcut = shortcut);
        let measure = |s: &str| {
            ui.painter()
                .layout_no_wrap(s.to_string(), caption.clone(), p.text_tertiary)
                .size()
                .x
        };
        let hint_w = [measure(&hint_full), measure(shortcut)];
        let (forms, hint_form) = si::fit(&full, &compact, &priority, hint_w, avail);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let mut first = true;
            if hint_form != HintForm::Hidden {
                let text = if hint_form == HintForm::Full {
                    hint_full.clone()
                } else {
                    shortcut.to_string()
                };
                let hint = ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .font(caption.clone())
                            .color(p.text_tertiary),
                    )
                    .sense(Sense::click())
                    .selectable(false),
                );
                crate::a11y::name(
                    &hint,
                    egui::accesskit::Role::Button,
                    Cmd::CommandPalette.label(),
                );
                hint.ctx.accesskit_node_builder(hint.id, |n| {
                    n.set_keyboard_shortcut(Cmd::CommandPalette.shortcut())
                });
                let hint = if hint_form == HintForm::Short {
                    hint.on_hover_text(&hint_full)
                } else {
                    hint
                };
                if hint
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    actions.push(Action::ShowPalette);
                }
                first = false;
            }
            // Right to left: the last item is drawn first.
            for (i, (plugin, item)) in items.iter().enumerate().rev() {
                if forms[i] == ItemForm::Hidden {
                    continue;
                }
                if !first {
                    ui.add_space(si::GAP);
                }
                first = false;
                let r = si::show(
                    ui,
                    item,
                    forms[i],
                    egui::Id::unique(("status-item", plugin, &item.id)),
                );
                if r.clicked() {
                    if let Some(a) = &item.action {
                        actions.push(Action::StatusItem(plugin.clone(), a.clone()));
                    }
                }
            }
        });
    }

    /// The panes side by side, with a draggable divider between each pair
    /// of neighbours (double-click: equal widths).
    fn panes_ui(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let rect = ui.max_rect();
        let gap = PANE_GAP;
        let n = self.panes.len();
        self.panes_width = rect.width();
        let content_w = (rect.width() - gap * (n as f32 - 1.0)).max(1.0);
        let weights: Vec<f32> = self.panes.iter().map(|p| p.weight).collect();
        let widths = crate::pane_set::widths(&weights, content_w, crate::pane_set::MIN_PANE_W);
        let mut rects = Vec::with_capacity(n);
        let mut x = rect.left();
        for (i, w) in widths.iter().enumerate() {
            let right = if i + 1 == n { rect.right() } else { x + w };
            rects.push(egui::Rect::from_min_max(
                egui::pos2(x, rect.top()),
                egui::pos2(right, rect.bottom()),
            ));
            x = right + gap;
        }

        // Dividers.
        for i in 0..n.saturating_sub(1) {
            let handle = egui::Rect::from_min_max(
                egui::pos2(rects[i].right(), rect.top()),
                egui::pos2(rects[i + 1].left(), rect.bottom()),
            );
            let resp = ui.interact(
                handle.expand2(egui::vec2(2.0, 0.0)),
                egui::Id::unique(("pane-splitter", i)),
                Sense::CLICK | Sense::DRAG,
            );
            crate::a11y::name(
                &resp,
                egui::accesskit::Role::Splitter,
                t!("a11y.pane_divider"),
            );
            if resp.hovered() || resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                let p = pal(ui);
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(handle.center(), egui::vec2(3.0, 36.0)),
                    CornerRadius::same(2),
                    p.border,
                );
            }
            if resp.dragged() {
                let dx = resp.drag_delta().x;
                if dx != 0.0 {
                    let mut w: Vec<f32> = self.panes.iter().map(|p| p.weight).collect();
                    crate::pane_set::drag_divider(
                        &mut w,
                        i,
                        dx,
                        content_w,
                        crate::pane_set::MIN_PANE_W,
                    );
                    for (p, w) in self.panes.iter_mut().zip(w) {
                        p.weight = w;
                    }
                }
            }
            if resp.double_clicked() {
                let mut w: Vec<f32> = self.panes.iter().map(|p| p.weight).collect();
                crate::pane_set::equalize(&mut w);
                for (p, w) in self.panes.iter_mut().zip(w) {
                    p.weight = w;
                }
            }
            let _ = widgets::tooltip(resp, t!("app.resize_tip"), "");
        }

        let active = self.active;
        self.pane_rects = self
            .panes
            .iter()
            .map(|p| p.id)
            .zip(rects.iter().copied())
            .collect();
        let target = self.target_highlight();
        let sync_pick = (self.sync.open && n > 2)
            .then_some(self.sync.pick)
            .flatten();
        let context_cmds: Vec<crate::pane::ContextCmd> = self
            .plugins
            .context_menu_commands()
            .map(|c| crate::pane::ContextCmd {
                id: c.full_id.clone(),
                title: c.spec.title.clone(),
                shortcut: c.spec.shortcut.clone().unwrap_or_default(),
                plugin: c.plugin_name.clone(),
            })
            .collect();
        struct Per {
            place: crate::place::PlaceChip,
            bottoms: Vec<PanelRef>,
            chips: Vec<(String, manta_plugin::Chip)>,
        }
        let per: Vec<Per> = self
            .panes
            .iter()
            .map(|p| {
                let loc = &p.tab().loc;
                let mut chips = Vec::new();
                if let Some(m) = &loc.archive {
                    chips.push(("manta.archive".to_string(), archive_chip(m)));
                } else if let Some(c) = container_chip(&loc.fs) {
                    chips.push(("manta.container".to_string(), c));
                }
                chips.extend(self.plugins.chips(p.id));
                Per {
                    place: self.place_chip(loc),
                    bottoms: self.open_panels(PanelPlacement::PaneBottom, Some(p.id)),
                    chips,
                }
            })
            .collect();
        let mut height = self.layout.pane_bottom_height;
        let mut resize_done = false;
        let mut panel_rects: Vec<egui::Rect> = Vec::new();
        let plugins = &mut self.plugins;
        let bottom_tab = &mut self.bottom_tab;
        let thumbs = &self.thumbs;
        let tag_store = &self.tags;
        let remote_thumbs = self.prefs.remote_thumbnails;
        // Off in tests (renders stay the same) unless a test asks for them.
        let type_icons = self.prefs.system_icons && (cfg!(not(test)) || self.test_type_icons);
        for (i, pane) in self.panes.iter_mut().enumerate() {
            let side = pane.id;
            let r = rects[i];
            let info_of = &per[i];
            let open = &info_of.bottoms;
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(r).id_salt(("pane", side)),
                |ui| {
                    // Decorations are read from the host, panels drawn by it:
                    // copy what the pane needs first.
                    let mut decorations = plugins.decorations(side).clone();
                    for name in &pane.tab().locked {
                        let d = decorations.entry(name.clone()).or_default();
                        d.badges.push(manta_plugin::Badge {
                            text: icons::LOCK_SIMPLE.to_string(),
                            tone: manta_plugin::Tone::Warning,
                        });
                        let tip = t!("app.encrypted_tip");
                        d.tooltip = Some(match d.tooltip.take() {
                            Some(t) => format!("{tip}\n{t}"),
                            None => tip.to_string(),
                        });
                    }
                    let info = PaneInfo {
                        side,
                        is_active: active == side,
                        count: n,
                        is_last: i + 1 == n,
                        is_target: target == Some(side),
                        sync_pick: sync_pick.map(|(a, b)| {
                            if side == a {
                                Some(crate::pane::SyncSlot::A)
                            } else if side == b {
                                Some(crate::pane::SyncSlot::B)
                            } else {
                                None
                            }
                        }),
                        place: &info_of.place,
                        decorations: &decorations,
                        chips: &info_of.chips,
                        context_cmds: &context_cmds,
                        thumbs: Some(thumbs),
                        remote_thumbs,
                        type_icons,
                        tags: Some(tag_store),
                    };
                    let mut tab_sel = bottom_tab.get(&side).cloned();
                    let height_now = height;
                    let mut draw = |ui: &mut egui::Ui, area: egui::Rect| {
                        let (dy, stopped) = plugin_ui::resize_handle_top(
                            ui,
                            area,
                            egui::Id::unique(("pane-bottom-resize", side)),
                        );
                        if dy != 0.0 {
                            // Start from the drawn (possibly automatic) height.
                            let current = if height > 0.0 { height } else { area.height() };
                            height = (current - dy).clamp(pane::MIN_BOTTOM_H, 2000.0);
                        }
                        resize_done |= stopped;
                        panel_rects.extend(draw_panel_area(
                            ui,
                            area,
                            &format!("bottom-{}", side.key()),
                            plugins,
                            open,
                            &mut tab_sel,
                            Some(side),
                            side,
                            0,
                        ));
                    };
                    let bottom = (!open.is_empty()).then_some(pane::BottomArea {
                        height: height_now,
                        draw: &mut draw,
                    });
                    pane::pane_ui(ui, info, pane, actions, bottom);
                    if let Some(t) = tab_sel {
                        bottom_tab.insert(side, t);
                    }
                },
            );
        }
        self.plugin_panel_rects.extend(panel_rects);
        if height != self.layout.pane_bottom_height {
            self.layout.pane_bottom_height = height;
        }
        if resize_done {
            self.plugins.set_layout(self.layout);
        }
    }
}

/// "Read-only" chip of a stopped (or shell-less) container.
/// Where an SMB connection's share is mounted.
fn smb_mount(fs: &SharedVfs) -> Option<PathBuf> {
    fs.as_any()
        .and_then(|a| a.downcast_ref::<manta_vfs::SmbFs>())
        .map(|s| s.mount_point())
}

fn container_chip(fs: &SharedVfs) -> Option<manta_plugin::Chip> {
    use manta_vfs::container::{ContainerFs, Mode};
    let c = fs.as_any()?.downcast_ref::<ContainerFs>()?;
    let (text, tip) = match c.mode() {
        Mode::Exec => return None,
        Mode::Stopped => (t!("app.chip.stopped"), t!("app.chip.stopped_tip")),
        Mode::NoShell => (t!("app.chip.read_only"), t!("app.chip.no_shell_tip")),
    };
    Some(manta_plugin::Chip {
        text: text.into(),
        tone: manta_plugin::Tone::Neutral,
        icon: Some("lock".into()),
        tooltip: Some(tip.into()),
        command: None,
    })
}

/// The path-bar chip of an archive pane.
fn archive_chip(m: &pane::ArchiveMount) -> manta_plugin::Chip {
    manta_plugin::Chip {
        text: t!("app.chip.read_only").into(),
        tone: manta_plugin::Tone::Neutral,
        icon: Some("lock".into()),
        tooltip: Some(t!(
            "app.chip.archive_tip",
            name = m.file_name,
            format = m.format
        )),
        command: None,
    }
}

/// Draw a plugin panel area (title bar with tabs + body) in `rect`.
/// `pane` is the instance for PaneBottom panels; `state_pane` is the pane
/// whose state the body gets. Returns the rect for focus tracking.
#[allow(clippy::too_many_arguments)]
fn draw_panel_area(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    salt: &str,
    plugins: &mut PluginHost,
    open: &[PanelRef],
    selected: &mut Option<(String, String)>,
    pane: Option<PaneId>,
    state_pane: PaneId,
    top_radius: u8,
) -> Option<egui::Rect> {
    if open.is_empty() {
        return None;
    }
    let mut idx = selected
        .as_ref()
        .and_then(|(pl, pa)| {
            open.iter()
                .position(|p| &p.plugin_id == pl && &p.spec.id == pa)
        })
        .unwrap_or(0);
    let tabs: Vec<plugin_ui::TitleTab<'_>> = open
        .iter()
        .map(|p| plugin_ui::TitleTab {
            title: &p.spec.title,
            glyph: plugin_ui::plugin_glyph(&p.plugin_id, p.plugin_id.starts_with("manta.")),
            plugin_name: &p.plugin_name,
        })
        .collect();
    let bar = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), plugin_ui::TITLE_BAR_H));
    let out = plugin_ui::title_bar(
        ui,
        bar,
        egui::Id::unique(("panel-title", salt)),
        &tabs,
        idx,
        top_radius,
        true,
    );
    if let Some(i) = out.selected {
        idx = i;
    }
    let current = &open[idx];
    *selected = Some((current.plugin_id.clone(), current.spec.id.clone()));
    if out.close {
        plugins.set_panel_open(&current.plugin_id, &current.spec.id, pane, false);
    }
    let body = egui::Rect::from_min_max(egui::pos2(rect.left(), bar.bottom()), rect.max);
    let Some(state) = plugins.pane(state_pane).cloned() else {
        return Some(rect);
    };
    let (plugin, panel) = (current.plugin_id.clone(), current.spec.id.clone());
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(body)
            .id_salt(("panel-body", salt, &plugin, &panel))
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.set_clip_rect(body.intersect(ui.clip_rect()));
            match plugins.panel_content(&plugin, &panel, pane) {
                PanelContent::Native => plugins.panel_ui(&plugin, &panel, &state, ui),
                PanelContent::Empty => plugin_ui::empty_panel(ui, body),
                PanelContent::Blocks(blocks) => {
                    let mut clicked = None;
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Frame::new()
                                .inner_margin(egui::Margin::symmetric(
                                    space::LG as i8,
                                    space::MD as i8 + 2,
                                ))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    plugin_ui::blocks_ui(ui, &blocks, &mut clicked);
                                });
                        });
                    if let Some(action) = clicked {
                        plugins.panel_action(&plugin, &panel, &action, &state);
                    }
                }
            }
        },
    );
    Some(rect)
}

impl eframe::App for MantaApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Settings / Software Update are open as native windows: the main
        // window takes no input; trying brings the modal window back.
        if self.modal_window().is_some() && !ctx.embed_viewports() {
            if crate::modal::block_input(raw_input) {
                self.bounce_to_modal_window(ctx);
            }
            return;
        }
        // No file drops onto panes or the sidebar behind a modal.
        if self.modal_open() {
            // Settings drawn inside this window (embedded viewports): plugin
            // files dropped on it go to its Plugins tab. A native Settings
            // window gets its drops in its own input.
            if ctx.embed_viewports()
                && self.settings.open
                && self.settings.tab == crate::settings::SettingsTab::Plugins
            {
                self.settings.hovered = raw_input
                    .hovered_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect();
                self.settings.dropped.extend(
                    raw_input
                        .dropped_files
                        .iter()
                        .map(|f| f.path().to_path_buf()),
                );
            }
            raw_input.hovered_files.clear();
            raw_input.dropped_files.clear();
        }
        crate::modal::dismiss_popup_on_outside_press(
            ctx,
            raw_input,
            &mut self.popup_swallow_release,
        );
        // "Go to Path" uses Tab for completion: take it before egui moves focus.
        if let Some(g) = &mut self.goto {
            raw_input.events.retain(|e| match e {
                egui::Event::Key {
                    key: Key::Tab,
                    pressed,
                    modifiers,
                    ..
                } if modifiers.is_none() => {
                    if *pressed {
                        g.tab_pressed = true;
                    }
                    false
                }
                _ => true,
            });
            return;
        }
        // Intercept Tab (pane switching) before egui uses it for focus navigation,
        // but only when no text field or dialog wants it.
        if self.modal_open()
            || ctx.memory(|m| m.focused().is_some())
            || crate::aux_window::embedded_key(ctx).is_some()
        {
            return;
        }
        // ⇧Tab goes to the previous pane (egui would focus the last widget).
        raw_input.events.retain(|e| match e {
            egui::Event::Key {
                key: Key::Tab,
                pressed,
                modifiers,
                ..
            } if modifiers.is_none() || modifiers.shift_only() => {
                if *pressed {
                    self.pending_tab_switch += if modifiers.shift { -1 } else { 1 };
                }
                false
            }
            _ => true,
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }

    fn on_exit(&mut self) {
        self.save_session();
        // SMB shares Manta mounted go away with Manta (as on disconnect;
        // the background unmount of a drop would not finish in time).
        for c in &self.connections {
            if let Some(smb) =
                c.fs.as_any()
                    .and_then(|a| a.downcast_ref::<manta_vfs::SmbFs>())
                    .filter(|s| s.mounted_by_manta())
            {
                if let Err(e) = smb.unmount_now() {
                    log::warn!("SMB: unmounting at exit: {e}");
                }
            }
        }
        // Send `shutdown` to every plugin; external processes get a short
        // grace period, then their supervisors kill them.
        self.plugins.shutdown(Duration::from_millis(300));
        self.quicklook.close();
    }
}

impl MantaApp {
    /// What a cloud provider still uploads (or failed to) in the folders
    /// the panes show: the first pane with any, as (provider, pill).
    fn cloud_upload_pill(&self) -> Option<(String, crate::transfer_pill::Pill)> {
        use manta_vfs::cloud_local::SyncState;
        self.panes.iter().find_map(|p| {
            let tab = p.tab();
            let states = tab.cloud.as_ref()?;
            let uploading = states
                .values()
                .filter(|i| i.state == SyncState::Uploading)
                .count();
            let failed = states
                .values()
                .filter(|i| i.state == SyncState::UploadError)
                .count();
            if uploading + failed == 0 {
                return None;
            }
            let dir = tab.loc.fs.local_path(&tab.loc.path)?;
            let loc = self.cloud.location_of(&dir)?;
            let provider = crate::cloud::split_display_name(&loc.display_name)
                .0
                .to_string();
            let pill = crate::transfer_pill::cloud_pill(&provider, uploading, failed)?;
            Some((provider, pill))
        })
    }

    /// One frame of the whole UI (separate from `eframe::App::ui` so tests
    /// can drive it without an `eframe::Frame`).
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        crate::a11y::frame(&ctx);
        // Another frame: panics reported so far didn't end the app.
        crate::crash::survived();
        crate::native_drag::set_drops_move_by_default(
            &ctx,
            self.prefs.drop_action.moves_by_default(),
        );
        self.poll_worker();
        self.live_refresh_frame();
        self.trash_frame(&ctx);
        self.cloud_frame(&ctx);
        self.search_frame(&ctx);
        self.tags_frame(&ctx);
        self.poll_connection_notices();
        self.jobs = self.transfers.snapshot();
        #[cfg(test)]
        if let Some(jobs) = &self.fake_jobs {
            self.jobs = jobs.clone();
        }
        self.process_finished_jobs();
        let now = Instant::now();
        self.transfer_pill.update(&self.jobs, now);
        if self.show_transfers {
            // The panel shows it all: nothing left to point at.
            self.transfer_pill.acknowledge();
        } else if let Some(d) = self.transfer_pill.repaint_in(now) {
            ctx.request_repaint_after(d);
        }
        self.conflict_frame();
        self.transfer_resume_frame(&ctx);
        self.thumbs.begin_frame(&ctx);
        self.apply_folder_views();
        self.sync_plugins();
        self.default_viewer_requests(&ctx);

        let mut actions: Vec<Action> = Vec::new();
        // Behind a modal the menu bar's commands are greyed out (like macOS
        // does behind app-modal panels) and ignored if one slips through.
        let modal = self.modal_open();
        let vs = &self.viewer.status;
        crate::app_menu::set_viewer_state(
            self.viewer.supported().then_some(vs.is_default),
            self.viewer.can_toggle() && !modal,
        );
        crate::app_menu::sync(&ctx);
        if crate::app_menu::take_viewer_toggle_request() {
            if modal {
                log::debug!("menu bar: default file manager ignored (modal open)");
            } else {
                log::debug!("menu bar: default file manager");
                actions.push(Action::ToggleDefaultViewer);
            }
        }
        self.menu_commands(crate::app_menu::take_commands(), &mut actions);
        // Which menu items are greyed out (a few times a second is enough,
        // right away when a modal opens or closes).
        if self.menu_modal != modal
            || self
                .menu_checked
                .is_none_or(|t| t.elapsed() >= Duration::from_millis(250))
        {
            self.menu_checked = Some(Instant::now());
            self.menu_modal = modal;
            crate::app_menu::set_enabled(|c| self.menu_cmd_enabled(c, modal));
        }
        if crate::app_menu::take_settings_request() {
            log::debug!("menu bar: Settings…");
            actions.push(Action::ShowSettings);
        }
        if crate::app_menu::take_help_request() {
            log::debug!("menu bar: Manta Manual");
            actions.push(Action::ShowHelp(String::new()));
        }
        if crate::app_menu::take_support_request() {
            log::debug!("menu bar: Manta Website & Support");
            actions.push(Action::OpenWebLink(crate::help::WEBSITE_URL.to_string()));
        }
        if crate::app_menu::take_source_request() {
            log::debug!("menu bar: Source Code on GitHub");
            actions.push(Action::OpenWebLink(crate::help::SOURCE_URL.to_string()));
        }
        if crate::app_menu::take_log_request() {
            log::debug!("menu bar: Show Log File in Finder");
            actions.push(Action::ShowLogFile);
        }
        if crate::app_menu::take_licenses_request() {
            log::debug!("menu bar: Third-Party Licences");
            actions.push(Action::ShowHelp(
                crate::help::THIRD_PARTY_LICENSES.to_string(),
            ));
        }
        crate::app_menu::set_updates_enabled(self.updates.enabled());
        if crate::app_menu::take_update_request() {
            log::debug!("menu bar: Check for Updates…");
            actions.push(Action::CheckForUpdates);
        }
        self.update_events(&ctx);
        #[cfg(not(test))]
        self.aux_selftest(&ctx);
        crate::aux_window::track_embedded_focus(&ctx, &self.open_aux_windows());
        self.handle_keys(&ctx, &mut actions);
        self.handle_mouse_nav(&ctx, &mut actions);
        let p = pal(ui);

        egui::Panel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(p.chrome)
                    .inner_margin(egui::Margin::symmetric(
                        space::MD as i8 + 2,
                        space::SM as i8,
                    )),
            )
            .show(ui, |ui| {
                crate::a11y::region(ui, egui::accesskit::Role::Toolbar, t!("a11y.toolbar"));
                self.toolbar(ui, &mut actions)
            });
        // The status bar's room is taken now (it spans the window's width);
        // its contents are drawn after the panes, so Tab reaches them last:
        // toolbar → sidebar → panes → status bar.
        let status_rect = egui::Panel::bottom("statusbar")
            .exact_size(24.0)
            .frame(
                egui::Frame::new()
                    .fill(p.chrome)
                    .inner_margin(egui::Margin::symmetric(space::MD as i8 + 4, 0)),
            )
            .show(ui, |ui| ui.max_rect())
            .inner;

        if self.show_transfers {
            egui::Panel::bottom("transfers")
                .resizable(true)
                .default_size(210.0)
                .min_size(110.0)
                .frame(
                    egui::Frame::new()
                        .fill(p.content)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ui, |ui| {
                    // See plugin_sidebar: keep the panel from shrinking.
                    ui.set_min_size(ui.max_rect().size());
                    if self.transfers_view.ui(ui, &self.transfers, &self.jobs) {
                        actions.push(Action::ToggleTransfers);
                    }
                });
        }

        let connections: Vec<ConnectionInfo> = self
            .connections
            .iter()
            .map(|c| ConnectionInfo {
                id: c.id,
                label: c.label.clone(),
                key: (
                    c.config.host.clone(),
                    c.config.port,
                    c.config.username.clone(),
                    c.config.protocol,
                ),
                mount: smb_mount(&c.fs),
            })
            .collect();
        let tab = self.pane(self.active).tab();
        let current = sidebar::Current {
            local: tab.loc.conn.is_none().then(|| tab.loc.path.to_string()),
            conn: tab.loc.conn,
        };
        // The Tags section: the user's tags; the one searched for is selected.
        let sidebar_tags = sidebar::TagRows {
            tags: self.tags.sidebar(),
            current: tab.search.as_ref().and_then(|s| match &s.query {
                crate::search::Query::Tag(n) => Some(n.clone()),
                _ => None,
            }),
        };
        let flash = self.fav_flash_now(&ctx);
        egui::Panel::left("sidebar")
            .resizable(true)
            .default_size(200.0)
            .size_range(150.0..=320.0)
            .frame(
                egui::Frame::new()
                    .fill(p.sidebar)
                    .inner_margin(egui::Margin::symmetric(space::MD as i8, 0)),
            )
            .show(ui, |ui| {
                // See plugin_sidebar: keep the panel from shrinking.
                ui.set_min_size(ui.max_rect().size());
                crate::a11y::region(ui, egui::accesskit::Role::Navigation, t!("a11y.sidebar"));
                let trash = self.trash.dir.clone().map(|dir| sidebar::TrashRow {
                    dir,
                    items: self.trash.status.items,
                });
                let rects = sidebar::sidebar_ui(
                    ui,
                    &self.favorites,
                    &self.locations,
                    &self.devices,
                    trash.as_ref(),
                    &self.cloud.locations,
                    !self.prefs.gdrive_hint_dismissed && !self.cloud.has_google_drive(),
                    &connections,
                    &current,
                    Some(&self.thumbs),
                    &mut self.containers,
                    flash,
                    &sidebar_tags,
                    &mut actions,
                );
                self.fav_rects = rects.favorites;
                self.trash_rect = rects.trash;
            });

        self.plugin_panel_rects.clear();
        self.plugin_sidebar(ui);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(p.chrome)
                    .inner_margin(egui::Margin::same(space::SM as i8)),
            )
            .show(ui, |ui| self.panes_ui(ui, &mut actions));
        {
            let mut status_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("statusbar-content")
                    .max_rect(status_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            crate::a11y::region(
                &status_ui,
                egui::accesskit::Role::Group,
                t!("a11y.status_bar"),
            );
            self.status_bar(&mut status_ui, &mut actions);
        }

        // Floating windows.
        if self.sync.open {
            match self.sync.ui(&ctx) {
                Some(SyncRequest::Compare) => self.start_compare(),
                Some(SyncRequest::Pick(slot, side)) => {
                    self.handle_action(&ctx, Action::SetSyncPane(side, slot))
                }
                Some(SyncRequest::Sync) => self.start_sync(),
                None => {}
            }
        }
        if let Some(pv) = &mut self.preview {
            if !pv.ui(&ctx, &mut self.preview_window) {
                pv.release(&ctx);
                self.preview = None;
            }
        }
        if self.preview.is_none() {
            self.preview_window
                .hidden(&ctx, crate::aux_window::AuxKind::Preview);
        }
        self.info_frame(&ctx);
        self.modal_window_frame(&ctx);
        self.settings_window(&ctx);
        crate::update_window::window(&ctx, &mut self.updates);
        self.help.mouse_nav = self.prefs.mouse_nav;
        let help = self.help.ui(&ctx);
        if let Some(url) = help.open_url {
            self.open_web_link(url);
        }
        self.dialog_ui(&ctx);
        self.default_viewer_prompt(&ctx);
        self.plugin_modal(&ctx);
        if self.palette.open {
            let entries = self.palette_entries();
            if let Some(id) = self.palette.ui(&ctx, &entries) {
                actions.extend(self.palette_action(id));
            }
        }
        self.goto_ui(&ctx);

        // Does a widget inside a plugin panel hold keyboard focus? (Used
        // by the shortcut rules next frame.)
        self.plugin_focus = ctx
            .memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            .is_some_and(|r| {
                self.plugin_panel_rects
                    .iter()
                    .any(|pr| pr.contains(r.rect.center()))
            });

        // Toasts sit above the status bar (and the transfers panel), and
        // above the crash notice when there is one.
        let screen = ctx.content_rect();
        let mut bottom =
            screen.bottom() - 24.0 - space::MD - if self.show_transfers { 210.0 } else { 0.0 };
        if let Some(notice) = &self.crash_notice {
            let (choice, top) = crate::crash::notice_ui(
                &ctx,
                notice,
                egui::pos2(screen.right() - space::LG, bottom),
            );
            bottom = top - space::SM;
            if let Some(choice) = choice {
                self.crash_choice(choice);
            }
        }
        if let Some(cmd) = self
            .toasts
            .ui(&ctx, egui::pos2(screen.right() - space::LG, bottom))
        {
            if cmd == crate::updates::TOAST_COMMAND {
                actions.push(Action::ShowUpdate);
            } else if self.modal_open() {
                log::debug!("toast command {cmd} ignored (modal open)");
            } else {
                actions.push(Action::RunPluginCommand(cmd));
            }
        }

        // "?" buttons anywhere in the UI (Settings, dialogs, panels).
        if let Some(target) = crate::help::take_request(&ctx) {
            actions.push(Action::ShowHelp(target));
        }
        self.poll_rename_click(&ctx, &mut actions);
        for a in actions {
            self.handle_action(&ctx, a);
        }
        self.keyboard_focus_frame(&ctx);
        self.native_drag_frame(&ctx);

        // An open Quick Look panel follows the cursor (and closes when the
        // folder changes or the pane has nothing local to show).
        if self.quicklook.is_open() {
            let tab = self.pane(self.active).tab();
            let key = crate::quicklook::QlKey::of(self.active.index(), tab);
            let target = crate::quicklook::tab_target(tab);
            self.quicklook.follow(key, target);
        }

        if self.transfers.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        // Tabs and folders change all the time: save them every few seconds
        // (and at exit) when something changed.
        if self.session_checked.elapsed() > Duration::from_secs(3) {
            self.session_checked = Instant::now();
            self.save_session();
        }
        #[cfg(not(test))]
        self.panes_selftest(&ctx);
        #[cfg(not(test))]
        self.menu_selftest(&ctx);
    }
}

/// Where tests "download" the plugin directory from: nothing listens there.
#[cfg(test)]
pub(crate) const TEST_REGISTRY_URL: &str = "http://127.0.0.1:9/registry.json";

#[cfg(test)]
#[path = "ux_tests.rs"]
mod ux_tests;

#[cfg(test)]
#[path = "duplicates_tests.rs"]
mod duplicates_tests;

/// ⌘? (the Help shortcut): `?` is ⇧/ on US keyboards and ⇧ß on German
/// ones; egui reports the typed character as `Questionmark` either way
/// (extra ⇧ is ignored). ⇧⌘/ covers layouts that report the physical key.
fn help_key(ctx: &egui::Context) -> bool {
    ctx.input_mut(|i| {
        i.consume_key(Modifiers::COMMAND, Key::Questionmark)
            || i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Slash)
    })
}
