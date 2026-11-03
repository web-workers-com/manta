//! [`PluginHost`]: the registry of built-in and external plugins, their
//! enable state, event dispatch and the contributions (decorations, chips,
//! commands, panels) the app draws.
//!
//! Everything here runs on the UI thread and returns quickly: plugin
//! processes start and stop on their own threads, installs and uninstalls
//! copy/remove files on a background thread. Reading the small config
//! files (`plugins.json`, `plugin.toml`) happens inline, like the app's
//! favorites.
//!
//! Files (under `dirs::config_dir()/Manta/`):
//! - `plugins/<folder>/plugin.toml`: installed external plugins
//! - `plugins.json`: enable state, trusted plugins, panel state, layout
//! - `plugin-data/<id>/`: per-plugin data dir (`plugin.log` for externals)

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use manta_core::VfsPath;
use serde::{Deserialize, Serialize};

use crate::bundled::{self, BundledPlugin};
use crate::external::ExternalPlugin;
use crate::install::{self, PluginReview};
use crate::manifest::{Manifest, MANIFEST_FILE};
use crate::protocol::Block;
use crate::registry::{self, Entry, Fetched, Incompatible};
use crate::sandbox::SandboxState;
use crate::shortcut::{self, Shortcut};
use crate::{
    Badge, Chip, CommandContext, CommandSpec, Decoration, DirListing, FileAction, FileContent,
    HostRequest, PaneId, PaneState, PanelPlacement, PanelSpec, Plugin, PluginEvent, PluginHandle,
    PluginInfo, PluginStatus, RenameResult, SizeInfo, StatusItem, ToastLevel, Tone,
};

/// Where the host keeps its files.
#[derive(Debug, Clone)]
pub struct HostPaths {
    /// Installed external plugins, one folder each.
    pub plugins_dir: PathBuf,
    /// Parent of the per-plugin data dirs.
    pub data_root: PathBuf,
    /// `plugins.json`.
    pub state_file: PathBuf,
    /// Parent of the per-plugin cache dirs (may be deleted at any time).
    pub cache_root: PathBuf,
}

impl HostPaths {
    /// Everything below `root` (e.g. `~/Library/Application Support/Manta`).
    pub fn under(root: &Path) -> Self {
        HostPaths {
            plugins_dir: root.join("plugins"),
            data_root: root.join("plugin-data"),
            state_file: root.join("plugins.json"),
            cache_root: root.join("plugin-cache"),
        }
    }

    /// `dirs::config_dir()/Manta`, if there is a config dir; caches in
    /// `dirs::cache_dir()/Manta` (`~/Library/Caches`, not backed up).
    pub fn default_paths() -> Option<Self> {
        let mut p = Self::under(&dirs::config_dir()?.join("Manta"));
        if let Some(c) = dirs::cache_dir() {
            p.cache_root = c.join("Manta").join("plugin-cache");
        }
        Some(p)
    }
}

/// Sizes of the plugin panel areas (points), remembered across launches.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PanelLayout {
    pub pane_bottom_height: f32,
    pub sidebar_width: f32,
}

impl Default for PanelLayout {
    fn default() -> Self {
        PanelLayout {
            // 0 = automatic (35 % of the pane) until the user resizes it.
            pane_bottom_height: 0.0,
            sidebar_width: 300.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SavedState {
    /// Explicit choices; missing = default (built-ins on, externals off).
    #[serde(default)]
    enabled: BTreeMap<String, bool>,
    /// External plugins the user confirmed the security note for.
    #[serde(default)]
    trusted: BTreeSet<String>,
    /// "plugin/panel/left|right|sidebar" → open.
    #[serde(default)]
    panels: BTreeMap<String, bool>,
    /// "plugin/panel" whose `default_open` was applied once.
    #[serde(default)]
    seen_panels: BTreeSet<String>,
    #[serde(default)]
    layout: PanelLayout,
    /// Retired external plugins (see [`RETIRED_EXTERNAL`]) whose "is now
    /// built in" toast was shown.
    #[serde(default)]
    retired_notified: BTreeSet<String>,
}

/// Built-in, external, or an external folder that couldn't be loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginKind {
    Builtin,
    External,
    /// An external plugin folder with a broken `plugin.toml`.
    Invalid,
}

enum SlotKind {
    Builtin(Box<dyn Plugin>),
    External(Box<ExternalPlugin>),
    Invalid,
}

struct Slot {
    info: PluginInfo,
    kind: SlotKind,
    /// `enable` was called and `disable` wasn't (yet).
    running: bool,
    /// Panic message (built-ins) or manifest error (invalid folders).
    error: Option<String>,
    /// External: plugin folder.
    dir: Option<PathBuf>,
    capabilities: Vec<String>,
    commands: Vec<CommandSpec>,
    panels: Vec<PanelSpec>,
    /// Preferred sizes of `Modal` panels.
    panel_sizes: HashMap<String, [f32; 2]>,
    /// Loaded from a development folder (`--plugin-dev`), not installed.
    dev: bool,
}

impl Slot {
    fn plugin(&mut self) -> Option<&mut dyn Plugin> {
        match &mut self.kind {
            SlotKind::Builtin(p) => Some(p.as_mut()),
            SlotKind::External(p) => Some(p.as_mut()),
            SlotKind::Invalid => None,
        }
    }

    fn kind(&self) -> PluginKind {
        match self.kind {
            SlotKind::Builtin(_) => PluginKind::Builtin,
            SlotKind::External(_) => PluginKind::External,
            SlotKind::Invalid => PluginKind::Invalid,
        }
    }
}

/// One plugin as shown in Settings → Plugins.
#[derive(Debug, Clone)]
pub struct PluginRow {
    pub info: PluginInfo,
    pub kind: PluginKind,
    /// The user's choice (the toggle).
    pub enabled: bool,
    pub status: PluginStatus,
    /// Last log lines (externals: stderr and `log` messages).
    pub log: Vec<String>,
    /// Folder of an external plugin.
    pub dir: Option<PathBuf>,
    pub capabilities: Vec<String>,
    /// An external plugin that a built-in replaced ([`RETIRED_EXTERNAL`]):
    /// the built-in's id. It is kept switched off; Settings offers to
    /// uninstall it.
    pub replaced_by: Option<String>,
    /// Loaded from a development folder (`manta --plugin-dev <dir>`): runs
    /// from that folder, trusted and switched on for this session only.
    pub dev: bool,
    /// External plugins: whether the macOS sandbox enforces `permissions`.
    pub sandbox: Option<SandboxState>,
    /// External plugins: the declared permissions.
    pub permissions: Vec<String>,
}

/// A plugin that ships with Manta, as shown in Settings → Plugins.
#[derive(Debug, Clone, PartialEq)]
pub struct BundledRow {
    /// From the bundled `plugin.toml` (`version` = the bundled version).
    pub info: PluginInfo,
    /// Version of the installed copy, if one is installed.
    pub installed_version: Option<String>,
    /// The bundled version is newer than the installed one.
    pub update_available: bool,
    /// An install or update is running.
    pub installing: bool,
}

/// Settings → Plugins → Discover: the community directory.
#[derive(Debug, Clone, Default)]
pub struct Discover {
    /// A download of the directory is running.
    pub loading: bool,
    /// The directory was asked for in this session ([`PluginHost::open_discover`]).
    pub requested: bool,
    /// The entries (from the last download or the saved copy).
    pub plugins: Vec<Entry>,
    /// `plugins` is the saved copy (offline, or not refreshed this session).
    pub from_cache: bool,
    /// When `plugins` was downloaded.
    pub fetched_at: Option<std::time::SystemTime>,
    /// Why the last download failed.
    pub error: Option<String>,
}

/// A directory entry as shown in Discover.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryRow {
    pub entry: Entry,
    /// In the app's language.
    pub name: String,
    pub description: String,
    /// Version of the installed copy.
    pub installed_version: Option<String>,
    /// The directory has a newer version than the installed one.
    pub update_available: bool,
    /// Why it can't be installed here.
    pub incompatible: Option<Incompatible>,
    /// Downloading and checking the release.
    pub downloading: bool,
}

/// Decorations of all plugins for one entry, merged.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MergedDecoration {
    /// One per plugin that set a badge, in plugin order.
    pub badges: Vec<Badge>,
    /// The first plugin's name tint wins.
    pub name_tone: Option<Tone>,
    /// Dimmed if any plugin dims it.
    pub dimmed: bool,
    /// All tooltips, one per line.
    pub tooltip: Option<String>,
}

/// A plugin command, namespaced as `plugin_id:command_id`.
#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub full_id: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub spec: CommandSpec,
    /// Parsed `spec.shortcut` (`None` if absent or invalid).
    pub shortcut: Option<Shortcut>,
}

#[derive(Debug, Clone)]
pub struct PanelRef {
    pub plugin_id: String,
    pub plugin_name: String,
    pub spec: PanelSpec,
    /// Preferred size of a `Modal` panel ([`Plugin::panel_size`]).
    pub size: Option<[f32; 2]>,
}

/// What a panel's body is drawn from.
pub enum PanelContent {
    /// The plugin draws it ([`PluginHost::panel_ui`]).
    Native,
    /// Declarative blocks from an external plugin.
    Blocks(Arc<Vec<Block>>),
    /// An external plugin that hasn't sent content yet.
    Empty,
}

/// Something the app has to act on, returned by [`PluginHost::poll`].
#[derive(Debug, Clone, PartialEq)]
pub enum HostEvent {
    Toast {
        level: ToastLevel,
        text: String,
    },
    Navigate {
        pane: PaneId,
        path: VfsPath,
    },
    /// An install found a plugin with the same id: ask, then call
    /// [`PluginHost::install`] again with `replace = true`.
    ConfirmReplace {
        src: PathBuf,
        id: String,
        name: String,
        installed_version: String,
        new_version: String,
    },
    /// A toast with a button; clicking it runs `command`
    /// (`plugin_id:command_id`, see [`PluginHost::run_command`]).
    ActionToast {
        level: ToastLevel,
        text: String,
        button: String,
        command: String,
    },
    /// A plugin asked to rename entries of `dir` in `pane`. Do it in the
    /// background, then report with [`PluginHost::deliver`] and
    /// [`PluginEvent::RenameResult`].
    RenameMany {
        plugin: String,
        pane: PaneId,
        dir: VfsPath,
        renames: Vec<(String, String)>,
    },
    /// A plugin asked for the listing of `dir` in `pane`; answer with
    /// [`PluginHost::deliver`] and [`PluginEvent::DirListing`].
    ListDir {
        plugin: String,
        pane: PaneId,
        dir: VfsPath,
    },
    /// A plugin asked to list the folder of `pane` again (`None` = both).
    Refresh {
        pane: Option<PaneId>,
    },
    /// A plugin asked to show this SFTP server in `pane`
    /// ([`HostRequest::OpenSftp`]).
    OpenSftp {
        pane: PaneId,
        user: String,
        host: String,
        port: u16,
    },
    /// The Terminal's shell in `pane` was left while it showed a connection
    /// ([`HostRequest::LeaveConnection`]).
    LeaveConnection {
        pane: PaneId,
    },
    /// A plugin asked for the bytes of `path` in `pane`'s backend (`max_bytes`
    /// already capped). Read it in the background, then answer with
    /// [`PluginHost::deliver`] and [`PluginEvent::FileContent`].
    ReadFile {
        plugin: String,
        pane: PaneId,
        path: VfsPath,
        max_bytes: u64,
    },
    /// A plugin asked to select `names` in the folder `dir` of `pane`
    /// (a local pane): navigate there if needed, then select and reveal them.
    Select {
        pane: PaneId,
        dir: VfsPath,
        names: Vec<String>,
    },
    /// [`PluginHost::review_install`] checked a folder or archive: show
    /// the review, then call [`PluginHost::confirm_install`] or
    /// [`PluginHost::cancel_install`].
    InstallReview(Box<PluginReview>),
    /// An install or update finished (the plugin is switched off unless it
    /// was on before an update).
    Installed {
        id: String,
        name: String,
        replaced: bool,
    },
    /// An install, review or update failed; `error` is technical English
    /// (manifest problems, file errors).
    InstallFailed {
        error: String,
    },
    /// An external plugin was uninstalled (`error`: its folder couldn't be
    /// removed completely).
    Uninstalled {
        name: String,
        error: Option<String>,
    },
}

/// Id of the built-in Terminal plugin ([`HostRequest::RunInTerminal`]).
pub const TERMINAL_PLUGIN: &str = "manta.terminal";
/// Its panel below each pane.
pub const TERMINAL_PANEL: &str = "terminal";

/// External plugins that became built-ins: (old external id, new built-in
/// id). An installed copy of the old plugin is switched off at startup (not
/// deleted), a toast says once that it is now built in, Settings marks it
/// "Replaced by built-in" with an Uninstall button, and it can't be switched
/// on again. When the built-in starts and its data dir is still empty, the
/// files of `plugin-data/<old id>/` are copied over (user settings such as
/// `actions.json` or `recent.json`).
pub const RETIRED_EXTERNAL: &[(&str, &str)] = &[
    ("quick-actions", "manta.quickactions"),
    ("project-info", "manta.projectinfo"),
    ("image-tools", "manta.imagetools"),
    ("duplicates", "manta.duplicates"),
];

/// `HostRequest::Select` → the folder of the first path and the names of
/// the paths in it (others are dropped; `None` if nothing is left).
fn select_event(pane: PaneId, paths: &[VfsPath]) -> Option<HostEvent> {
    let dir = paths.first()?.parent()?;
    let mut names: Vec<String> = Vec::new();
    for p in paths {
        if p.parent().as_ref() == Some(&dir) {
            let name = p.file_name().to_string();
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    (!names.is_empty()).then_some(HostEvent::Select { pane, dir, names })
}

enum Job {
    Review(Box<PluginReview>),
    Registry(Result<Fetched, String>),
    /// A directory download for `id` finished (review or error).
    Downloaded {
        id: String,
        result: Result<Box<PluginReview>, String>,
    },
    Installed {
        id: String,
        name: String,
        replaced: bool,
    },
    NeedsReplace {
        src: PathBuf,
        id: String,
        name: String,
        new_version: String,
    },
    InstallFailed(String),
    Uninstalled {
        name: String,
        result: Result<(), String>,
    },
}

/// "plugin/panel/left", ".../right", ".../2" (more panes) or ".../sidebar".
fn panel_key(plugin: &str, panel: &str, pane: Option<PaneId>) -> String {
    let where_ = match pane {
        Some(p) => p.key(),
        None => "sidebar".to_string(),
    };
    format!("{plugin}/{panel}/{where_}")
}

fn panic_message(e: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = e.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".into()
    }
}

/// Decorations/chips of one plugin for one pane, valid while it shows `dir`.
struct PaneData<T> {
    dir: VfsPath,
    data: T,
}

pub struct PluginHost {
    ctx: egui::Context,
    paths: Option<HostPaths>,
    slots: Vec<Slot>,
    state: SavedState,
    tx: Sender<(String, HostRequest)>,
    rx: Receiver<(String, HostRequest)>,
    jobs_tx: Sender<Job>,
    jobs_rx: Receiver<Job>,
    /// The panes the app showed last, by id (see [`PaneId`]).
    panes: BTreeMap<PaneId, PaneState>,
    listing_gen: HashMap<PaneId, u64>,
    decorations: HashMap<(String, PaneId), PaneData<HashMap<String, Decoration>>>,
    merged: HashMap<PaneId, HashMap<String, MergedDecoration>>,
    chips: HashMap<(String, PaneId), PaneData<Vec<Chip>>>,
    /// Status bar items by plugin id.
    status_items: HashMap<String, Vec<StatusItem>>,
    sizes: HashMap<(String, PaneId), PaneData<HashMap<String, SizeInfo>>>,
    merged_sizes: HashMap<PaneId, HashMap<String, SizeInfo>>,
    /// Bumped whenever `merged_sizes` of a pane changes (never reset, so a
    /// reused pane id doesn't repeat an old generation).
    sizes_gen: HashMap<PaneId, u64>,
    /// Returned for panes without decorations / sizes.
    empty_merged: HashMap<String, MergedDecoration>,
    empty_sizes: HashMap<String, SizeInfo>,
    commands: Vec<PluginCommand>,
    out: Vec<HostEvent>,
    /// External plugins to enable again once a replacing install is done.
    reenable_after_install: BTreeSet<String>,
    /// Open `Modal` panels ("plugin/panel"); not saved across launches.
    modal_open: BTreeSet<String>,
    /// Plugins shipped with the app ([`Self::set_bundled`]).
    bundled: Vec<BundledPlugin>,
    /// Bundled plugins being installed (ids).
    installing: BTreeSet<String>,
    /// Development folders ([`Self::add_dev_plugin`]), loaded before the
    /// installed plugins.
    dev_dirs: Vec<PathBuf>,
    /// Development plugins the user switched off in this session.
    dev_off: BTreeSet<String>,
    /// Counter for staging folder names.
    review_seq: u64,
    /// The community directory (Discover).
    discover: Discover,
    /// Where the directory is downloaded from ([`registry::registry_url`]).
    registry_url: String,
    /// Directory entries being downloaded (ids).
    downloading: BTreeSet<String>,
}

impl PluginHost {
    /// Create the host with the app's built-in plugins and (if `paths` is
    /// set) the external plugins installed there, and enable those that
    /// are switched on.
    pub fn new(
        ctx: egui::Context,
        builtins: Vec<Box<dyn Plugin>>,
        paths: Option<HostPaths>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let (jobs_tx, jobs_rx) = mpsc::channel();
        let state = paths
            .as_ref()
            .and_then(|p| std::fs::read_to_string(&p.state_file).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let mut host = PluginHost {
            ctx,
            paths,
            slots: Vec::new(),
            state,
            tx,
            rx,
            jobs_tx,
            jobs_rx,
            panes: BTreeMap::new(),
            listing_gen: HashMap::new(),
            decorations: HashMap::new(),
            merged: HashMap::new(),
            chips: HashMap::new(),
            status_items: HashMap::new(),
            sizes: HashMap::new(),
            merged_sizes: HashMap::new(),
            sizes_gen: HashMap::new(),
            empty_merged: HashMap::new(),
            empty_sizes: HashMap::new(),
            commands: Vec::new(),
            out: Vec::new(),
            reenable_after_install: BTreeSet::new(),
            modal_open: BTreeSet::new(),
            bundled: Vec::new(),
            installing: BTreeSet::new(),
            dev_dirs: Vec::new(),
            dev_off: BTreeSet::new(),
            review_seq: 0,
            discover: Discover::default(),
            registry_url: registry::registry_url(),
            downloading: BTreeSet::new(),
        };
        // The saved directory (no network): updates of installed plugins
        // are offered without opening Discover.
        if let Some(f) = host.registry_cache().and_then(|c| registry::load_cache(&c)) {
            host.discover.plugins = f.registry.plugins;
            host.discover.from_cache = true;
            host.discover.fetched_at = f.fetched_at;
        }
        // Staging folders of reviews that were never confirmed.
        if let Some(dir) = host.plugins_dir() {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for e in rd.flatten() {
                    if e.file_name()
                        .to_string_lossy()
                        .starts_with(install::STAGING_PREFIX)
                    {
                        // Folders, and downloads from the directory.
                        let _ = std::fs::remove_dir_all(e.path())
                            .or_else(|_| std::fs::remove_file(e.path()));
                    }
                }
            }
        }
        for b in builtins {
            let info = b.info();
            if host.slots.iter().any(|s| s.info.id == info.id) {
                log::warn!("duplicate built-in plugin id {}", info.id);
                continue;
            }
            host.slots.push(Slot {
                info,
                kind: SlotKind::Builtin(b),
                running: false,
                error: None,
                dir: None,
                capabilities: Vec::new(),
                commands: Vec::new(),
                panels: Vec::new(),
                panel_sizes: HashMap::new(),
                dev: false,
            });
        }
        host.scan_externals();
        host.retire_externals();
        let ids: Vec<String> = host.slots.iter().map(|s| s.info.id.clone()).collect();
        for id in ids {
            if host.wants_enabled(&id) {
                host.start(&id);
            }
        }
        host
    }

    /// Add an external plugin that isn't on disk (tests and previews).
    #[doc(hidden)]
    pub fn add_external_for_test(&mut self, plugin: ExternalPlugin) -> &mut ExternalPlugin {
        let m = plugin.manifest().clone();
        self.slots.push(Slot {
            info: m.info(),
            dir: Some(plugin.dir().to_path_buf()),
            capabilities: m.capabilities.clone(),
            commands: plugin.commands(),
            panels: plugin.panels(),
            panel_sizes: HashMap::new(),
            kind: SlotKind::External(Box::new(plugin)),
            running: false,
            error: None,
            dev: false,
        });
        self.rebuild_commands();
        match &mut self.slots.last_mut().expect("just pushed").kind {
            SlotKind::External(p) => p,
            _ => unreachable!(),
        }
    }

    /// Test seam: mark a plugin's panels as running contributions without
    /// enabling it (external plugins injected with state).
    #[doc(hidden)]
    pub fn set_running_for_test(&mut self, id: &str, running: bool) {
        if let Some(s) = self.slots.iter_mut().find(|s| s.info.id == id) {
            s.running = running;
        }
        self.state.enabled.insert(id.to_string(), running);
        self.refresh_contributions(id);
    }

    /// The built-in that replaced the external plugin `id`
    /// ([`RETIRED_EXTERNAL`]), if `id` is an installed retired external
    /// plugin and that built-in is present.
    pub fn replaced_by(&self, id: &str) -> Option<&'static str> {
        let &(_, new) = RETIRED_EXTERNAL.iter().find(|(old, _)| *old == id)?;
        let installed = self
            .slots
            .iter()
            .any(|s| s.info.id == id && s.kind() == PluginKind::External);
        let builtin = self
            .slots
            .iter()
            .any(|s| s.info.id == new && s.kind() == PluginKind::Builtin);
        (installed && builtin).then_some(new)
    }

    /// Switch off installed retired external plugins (kept on disk) and
    /// say once per plugin that it is built in now.
    fn retire_externals(&mut self) {
        let mut changed = false;
        for &(old, new) in RETIRED_EXTERNAL {
            if self.replaced_by(old).is_none() {
                continue;
            }
            if self.state.enabled.get(old) == Some(&true) {
                self.state.enabled.insert(old.to_string(), false);
                changed = true;
            }
            if self.state.retired_notified.insert(old.to_string()) {
                changed = true;
                let name = self
                    .slots
                    .iter()
                    .find(|s| s.info.id == new)
                    .map(|s| s.info.name.clone())
                    .unwrap_or_default();
                use crate::i18n::Lang;
                let l = crate::i18n::lang();
                let q = l.quote(&name);
                let text = match l {
                    Lang::En => format!("{q} is now built in"),
                    Lang::De => format!("{q} ist jetzt eingebaut"),
                    Lang::Zh => format!("{q}现已内置"),
                    Lang::Es => format!("{q} ahora viene integrado"),
                    Lang::Fr => format!("{q} est désormais intégré"),
                    Lang::It => format!("{q} ora è integrato"),
                    Lang::Ru => format!("{q} теперь встроен"),
                };
                self.out.push(HostEvent::Toast {
                    level: ToastLevel::Info,
                    text,
                });
            }
        }
        if changed {
            self.save();
        }
    }

    pub fn plugins_dir(&self) -> Option<&Path> {
        self.paths.as_ref().map(|p| p.plugins_dir.as_path())
    }

    // ------------------------------------------------------------ state

    fn save(&self) {
        let Some(p) = &self.paths else { return };
        let Ok(json) = serde_json::to_string_pretty(&self.state) else {
            return;
        };
        if let Some(dir) = p.state_file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = p.state_file.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(&tmp, &p.state_file);
        }
    }

    /// The user's choice: built-ins default on, external plugins off.
    pub fn wants_enabled(&self, id: &str) -> bool {
        if self.replaced_by(id).is_some() {
            return false;
        }
        if self.is_dev(id) {
            return !self.dev_off.contains(id);
        }
        match self.state.enabled.get(id) {
            Some(on) => *on,
            None => self
                .slots
                .iter()
                .find(|s| s.info.id == id)
                .is_some_and(|s| s.kind() == PluginKind::Builtin),
        }
    }

    /// External plugins need a one-time confirmation before they run.
    pub fn is_trusted(&self, id: &str) -> bool {
        self.is_dev(id)
            || self.state.trusted.contains(id)
            || self
                .slots
                .iter()
                .find(|s| s.info.id == id)
                .is_some_and(|s| s.kind() == PluginKind::Builtin)
    }

    pub fn trust(&mut self, id: &str) {
        self.state.trusted.insert(id.to_string());
        self.save();
    }

    pub fn layout(&self) -> PanelLayout {
        self.state.layout
    }

    pub fn set_layout(&mut self, layout: PanelLayout) {
        if self.state.layout != layout {
            self.state.layout = layout;
            self.save();
        }
    }

    // ------------------------------------------------------------ enabling

    /// Switch a plugin on or off (and remember it). An untrusted external
    /// plugin is trusted by this call: ask the user first.
    pub fn set_enabled(&mut self, id: &str, on: bool) {
        if on && self.replaced_by(id).is_some() {
            return; // a built-in does its job now
        }
        if self.is_dev(id) {
            // Session only: nothing is saved for development folders.
            if on {
                self.dev_off.remove(id);
                self.start(id);
            } else {
                self.dev_off.insert(id.to_string());
                self.stop(id);
            }
            return;
        }
        self.state.enabled.insert(id.to_string(), on);
        if on && !self.is_trusted(id) {
            self.state.trusted.insert(id.to_string());
        }
        self.save();
        if on {
            if let Some(s) = self.slots.iter_mut().find(|s| s.info.id == id) {
                s.error = None;
            }
            self.start(id);
        } else {
            self.stop(id);
        }
    }

    fn start(&mut self, id: &str) {
        let Some(i) = self.slots.iter().position(|s| s.info.id == id) else {
            return;
        };
        if self.slots[i].running || self.slots[i].kind() == PluginKind::Invalid {
            return;
        }
        if self.slots[i].kind() == PluginKind::External
            && (!self.is_trusted(id) || self.replaced_by(id).is_some())
        {
            return;
        }
        let data_dir = self
            .paths
            .as_ref()
            .map(|p| p.data_root.join(id))
            .unwrap_or_else(|| std::env::temp_dir().join("manta-plugin-data").join(id));
        if self.slots[i].kind() == PluginKind::Builtin {
            if let (Some(paths), Some(&(old, _))) = (
                &self.paths,
                RETIRED_EXTERNAL.iter().find(|(_, new)| *new == id),
            ) {
                migrate_data(&paths.data_root.join(old), &data_dir);
            }
        }
        let _ = std::fs::create_dir_all(&data_dir);
        let mut handle =
            PluginHandle::new(id.to_string(), self.tx.clone(), self.ctx.clone(), data_dir);
        if let Some(p) = &self.paths {
            handle = handle.with_cache_dir(p.cache_root.join(id));
        }
        self.slots[i].running = true;
        if self.call(i, "enable", |p| p.enable(handle)).is_none() {
            return;
        }
        self.refresh_contributions(id);
        // Then a PaneOpened and a DirectoryChanged for each known pane.
        for p in self.panes.values().cloned().collect::<Vec<_>>() {
            if self
                .call(i, "on_event", |pl| {
                    pl.on_event(&PluginEvent::PaneOpened(p.pane))
                })
                .is_none()
            {
                return;
            }
            if self
                .call(i, "on_event", |pl| {
                    pl.on_event(&PluginEvent::DirectoryChanged(p.clone()))
                })
                .is_none()
            {
                return;
            }
        }
    }

    fn stop(&mut self, id: &str) {
        let Some(i) = self.slots.iter().position(|s| s.info.id == id) else {
            return;
        };
        if self.slots[i].running {
            self.slots[i].running = false;
            if let Some(p) = self.slots[i].plugin() {
                let r = catch_unwind(AssertUnwindSafe(|| p.disable()));
                if let Err(e) = r {
                    log::error!("plugin {id} panicked in disable: {}", panic_message(&*e));
                }
            }
        }
        self.forget_contributions(id);
    }

    /// Drop everything a plugin contributed.
    fn forget_contributions(&mut self, id: &str) {
        self.decorations.retain(|(p, _), _| p != id);
        self.chips.retain(|(p, _), _| p != id);
        self.status_items.remove(id);
        self.sizes.retain(|(p, _), _| p != id);
        let prefix = format!("{id}/");
        self.modal_open.retain(|k| !k.starts_with(&prefix));
        if let Some(s) = self.slots.iter_mut().find(|s| s.info.id == id) {
            s.commands.clear();
            s.panels.clear();
            s.panel_sizes.clear();
        }
        self.rebuild_all_panes();
        self.rebuild_commands();
    }

    /// Call into a plugin, catching panics. A panicking plugin is switched
    /// off (for this session) and shows the panic in Settings.
    fn call<R>(&mut self, i: usize, what: &str, f: impl FnOnce(&mut dyn Plugin) -> R) -> Option<R> {
        let slot = &mut self.slots[i];
        let p = slot.plugin()?;
        match catch_unwind(AssertUnwindSafe(|| f(p))) {
            Ok(r) => Some(r),
            Err(e) => {
                let msg = panic_message(&*e);
                let id = slot.info.id.clone();
                let name = slot.info.name.clone();
                log::error!("plugin {id} panicked in {what}: {msg}");
                slot.error = Some(format!("Crashed in {what}: {msg}"));
                if slot.running {
                    slot.running = false;
                    if let Some(p) = slot.plugin() {
                        let _ = catch_unwind(AssertUnwindSafe(|| p.disable()));
                    }
                }
                self.forget_contributions(&id);
                self.out.push(HostEvent::Toast {
                    level: ToastLevel::Error,
                    text: format!("Plugin “{name}” stopped after an internal error"),
                });
                None
            }
        }
    }

    fn running_indices(&self) -> Vec<usize> {
        (0..self.slots.len())
            .filter(|&i| self.slots[i].running)
            .collect()
    }

    // ------------------------------------------------------------ events

    /// Tell the host the current state of a pane (every frame, or whenever
    /// it may have changed). `listing_gen` changes whenever a listing
    /// arrives. The host diffs against the last state and sends
    /// `DirectoryChanged` / `ListingRefreshed` / `SelectionChanged` /
    /// `ActivePaneChanged`.
    ///
    /// The first state of a pane id the host hasn't seen (or saw closed)
    /// also sends [`PluginEvent::PaneOpened`].
    pub fn set_pane(&mut self, state: PaneState, listing_gen: u64) {
        let prev = self.panes.remove(&state.pane);
        let prev_gen = self
            .listing_gen
            .insert(state.pane, listing_gen)
            .unwrap_or(0);
        let mut events = Vec::new();
        if prev.is_none() {
            events.push(PluginEvent::PaneOpened(state.pane));
        }
        let dir_changed = match &prev {
            None => true,
            Some(p) => {
                p.path != state.path
                    || p.backend_label != state.backend_label
                    || p.protocol != state.protocol
                    || p.local_path != state.local_path
            }
        };
        if dir_changed {
            events.push(PluginEvent::DirectoryChanged(state.clone()));
        } else if let Some(p) = &prev {
            if listing_gen != prev_gen {
                events.push(PluginEvent::ListingRefreshed(state.clone()));
            }
            if p.selection != state.selection {
                events.push(PluginEvent::SelectionChanged(state.clone()));
            }
        }
        if state.active && prev.as_ref().is_some_and(|p| !p.active) {
            events.push(PluginEvent::ActivePaneChanged(state.pane));
        }
        let pane = state.pane;
        let path = state.path.clone();
        self.panes.insert(pane, state);
        if dir_changed {
            // Drop what was set for another folder (or another backend).
            let backend_changed = prev.is_some();
            self.decorations
                .retain(|(_, p), d| *p != pane || (d.dir == path && !backend_changed));
            self.chips
                .retain(|(_, p), d| *p != pane || (d.dir == path && !backend_changed));
            self.sizes
                .retain(|(_, p), d| *p != pane || (d.dir == path && !backend_changed));
            self.rebuild_merged(pane);
            self.rebuild_sizes(pane);
        }
        for ev in events {
            self.dispatch(&ev);
        }
    }

    /// The last state set for a pane.
    pub fn pane(&self, pane: PaneId) -> Option<&PaneState> {
        self.panes.get(&pane)
    }

    /// The panes the host knows (set with [`Self::set_pane`] and not closed).
    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.panes.keys().copied().collect()
    }

    /// The app closed a pane: drop its state, decorations, chips, sizes and
    /// `PaneBottom` panel instances, and send [`PluginEvent::PaneClosed`]
    /// (the Terminal hangs up the pane's shells). Unknown ids are ignored.
    pub fn close_pane(&mut self, pane: PaneId) {
        if self.panes.remove(&pane).is_none() {
            return;
        }
        self.listing_gen.remove(&pane);
        self.decorations.retain(|(_, p), _| *p != pane);
        self.chips.retain(|(_, p), _| *p != pane);
        self.sizes.retain(|(_, p), _| *p != pane);
        self.merged.remove(&pane);
        self.merged_sizes.remove(&pane);
        if let Some(g) = self.sizes_gen.get_mut(&pane) {
            *g += 1;
        }
        let suffix = format!("/{}", pane.key());
        let before = self.state.panels.len();
        self.state.panels.retain(|k, _| !k.ends_with(&suffix));
        if self.state.panels.len() != before {
            self.save();
        }
        self.dispatch(&PluginEvent::PaneClosed(pane));
        self.ctx.request_repaint();
    }

    /// Open the same `PaneBottom` panels in `to` as in `from` (a split pane
    /// starts like the pane it was split from).
    pub fn copy_pane_panels(&mut self, from: PaneId, to: PaneId) {
        let open: Vec<(String, String)> = self
            .panels(PanelPlacement::PaneBottom)
            .into_iter()
            .filter(|p| self.is_panel_open(&p.plugin_id, &p.spec.id, Some(from)))
            .map(|p| (p.plugin_id, p.spec.id))
            .collect();
        for (plugin, panel) in open {
            self.set_panel_open(&plugin, &panel, Some(to), true);
        }
    }

    fn active_pane(&self) -> Option<PaneId> {
        self.panes.values().find(|p| p.active).map(|p| p.pane)
    }

    fn rebuild_all_panes(&mut self) {
        let ids: Vec<PaneId> = self
            .panes
            .keys()
            .chain(self.merged.keys())
            .chain(self.merged_sizes.keys())
            .copied()
            .collect();
        for p in ids {
            self.rebuild_merged(p);
            self.rebuild_sizes(p);
        }
    }

    /// Report a file action done in the GUI (started or finished) to all
    /// running plugins.
    pub fn file_action(&mut self, action: FileAction) {
        self.dispatch(&PluginEvent::FileAction(action));
    }

    /// Send an event to one running plugin only (answers to its requests,
    /// e.g. [`PluginEvent::RenameResult`]). Ignored if it isn't running.
    pub fn deliver(&mut self, plugin_id: &str, ev: PluginEvent) {
        if let Some(i) = self
            .slots
            .iter()
            .position(|s| s.running && s.info.id == plugin_id)
        {
            self.call(i, "on_event", |p| p.on_event(&ev));
        }
    }

    /// Set the UI language (BCP 47 code like `"de"`) for every plugin: built-in
    /// ones read it with [`crate::locale`] / [`crate::PluginHandle::locale`],
    /// external ones get it in `init`. Running plugins get
    /// [`PluginEvent::LocaleChanged`] when it differs from before.
    pub fn set_locale(&mut self, code: &str) {
        if crate::locale() == code {
            return;
        }
        crate::set_global_locale(code);
        self.relocalize();
    }

    /// Re-read everything plugins translate (names, descriptions, command,
    /// panel and sheet titles) in the current language and tell them it
    /// changed. [`Self::set_locale`] does this; tests and renders that
    /// switch only their thread's language call it directly.
    pub fn relocalize(&mut self) {
        self.refresh_infos();
        self.dispatch(&PluginEvent::LocaleChanged(crate::locale()));
        // Built-ins may now return translated command and panel titles.
        let ids: Vec<String> = self
            .running_indices()
            .into_iter()
            .map(|i| self.slots[i].info.id.clone())
            .collect();
        for id in ids {
            self.refresh_contributions(&id);
        }
    }

    /// Re-read plugin names and descriptions (they may be translated, see
    /// `[localized.<lang>]` in plugin.toml). [`Self::set_locale`] does this;
    /// tests that switch only their thread's language call it directly.
    pub fn refresh_infos(&mut self) {
        for slot in &mut self.slots {
            let (id, builtin) = (slot.info.id.clone(), slot.info.builtin);
            match &slot.kind {
                SlotKind::Builtin(p) => {
                    let info = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.info()));
                    if let Ok(info) = info {
                        slot.info = info;
                    }
                }
                SlotKind::External(p) => slot.info = p.manifest().info(),
                SlotKind::Invalid => {}
            }
            slot.info.id = id;
            slot.info.builtin = builtin;
        }
        self.rebuild_commands();
    }

    fn dispatch(&mut self, ev: &PluginEvent) {
        for i in self.running_indices() {
            self.call(i, "on_event", |p| p.on_event(ev));
        }
    }

    /// Handle plugin requests and finished background jobs. Call once per
    /// frame; act on the returned events.
    pub fn poll(&mut self) -> Vec<HostEvent> {
        while let Ok((id, req)) = self.rx.try_recv() {
            self.handle_request(&id, req);
        }
        while let Ok(job) = self.jobs_rx.try_recv() {
            self.handle_job(job);
        }
        std::mem::take(&mut self.out)
    }

    fn handle_request(&mut self, id: &str, req: HostRequest) {
        // Late requests from a plugin that was switched off are ignored.
        let Some(slot) = self.slots.iter().find(|s| s.info.id == id) else {
            return;
        };
        if !slot.running {
            return;
        }
        // External plugins only get what their declared permissions allow
        // (the same rules the sandbox enforces on their process).
        if let SlotKind::External(e) = &slot.kind {
            if let Some(perm) = required_permission(&req) {
                if !e.manifest().permissions.iter().any(|p| p == perm) {
                    e.note(&format!(
                        "refused {}: the plugin doesn't declare the \"{perm}\" permission",
                        request_name(&req)
                    ));
                    self.refuse(id, req, perm);
                    return;
                }
            }
        }
        match req {
            HostRequest::SetDecorations { pane, dir, entries } => {
                if self.panes.get(&pane).is_none_or(|p| p.path != dir) {
                    return; // stale
                }
                let map: HashMap<String, Decoration> = entries
                    .into_iter()
                    .take(crate::protocol::MAX_DECORATIONS)
                    .collect();
                self.decorations
                    .insert((id.to_string(), pane), PaneData { dir, data: map });
                self.rebuild_merged(pane);
            }
            HostRequest::SetChips {
                pane,
                dir,
                mut chips,
            } => {
                if self.panes.get(&pane).is_none_or(|p| p.path != dir) {
                    return;
                }
                chips.truncate(crate::protocol::MAX_CHIPS);
                self.chips
                    .insert((id.to_string(), pane), PaneData { dir, data: chips });
            }
            HostRequest::Toast { level, text } => {
                let text = if text.chars().count() > 500 {
                    text.chars().take(500).collect::<String>() + "…"
                } else {
                    text
                };
                self.out.push(HostEvent::Toast { level, text });
            }
            HostRequest::Navigate { pane, path } => {
                // Only for local panes (paths there are OS paths); ignored
                // when unchanged.
                match self.panes.get(&pane) {
                    Some(p) if p.local_path.is_some() && p.path != path => {}
                    _ => return,
                }
                self.out.push(HostEvent::Navigate { pane, path });
            }
            HostRequest::NavigateIn { pane, remote, path } => {
                // Only while the pane still shows that backend (local: an OS
                // folder; remote: the same server/container), and the path
                // changed.
                match self.panes.get(&pane) {
                    Some(p)
                        if p.path != path
                            && p.remote == remote
                            && (remote.is_some() || p.local_path.is_some()) => {}
                    _ => return,
                }
                self.out.push(HostEvent::Navigate { pane, path });
            }
            HostRequest::SetPanelOpen { panel, pane, open } => {
                self.set_panel_open(id, &panel, pane, open);
            }
            HostRequest::ContributionsChanged => self.refresh_contributions(id),
            HostRequest::SetSizes { pane, dir, sizes } => {
                if self.panes.get(&pane).is_none_or(|p| p.path != dir) {
                    return; // stale
                }
                let slot = self
                    .sizes
                    .entry((id.to_string(), pane))
                    .or_insert_with(|| PaneData {
                        dir: dir.clone(),
                        data: HashMap::new(),
                    });
                if slot.dir != dir {
                    *slot = PaneData {
                        dir,
                        data: HashMap::new(),
                    };
                }
                for (name, info) in sizes {
                    if slot.data.len() >= crate::protocol::MAX_DECORATIONS
                        && !slot.data.contains_key(&name)
                    {
                        continue;
                    }
                    slot.data.insert(name, info);
                }
                self.rebuild_sizes(pane);
            }
            HostRequest::TogglePanel { panel, pane } => {
                let pane = pane.or_else(|| self.active_pane());
                let open = self.is_panel_open(id, &panel, pane);
                let target = match self.placement_of(id, &panel) {
                    Some(PanelPlacement::PaneBottom) => pane,
                    _ => None,
                };
                self.set_panel_open(id, &panel, target, !open);
            }
            HostRequest::RenameMany { pane, dir, renames } => {
                self.out.push(HostEvent::RenameMany {
                    plugin: id.to_string(),
                    pane,
                    dir,
                    renames,
                });
            }
            HostRequest::ListDir { pane, dir } => {
                self.out.push(HostEvent::ListDir {
                    plugin: id.to_string(),
                    pane,
                    dir,
                });
            }
            HostRequest::ActionToast {
                level,
                text,
                button,
                command,
            } => {
                let text: String = text.chars().take(500).collect();
                let button: String = button.chars().take(24).collect();
                self.out.push(HostEvent::ActionToast {
                    level,
                    text,
                    button,
                    command: format!("{id}:{command}"),
                });
            }
            HostRequest::RunInTerminal { pane, command, cwd } => {
                self.run_in_terminal(pane, command, cwd)
            }
            HostRequest::RefreshPane { pane } => self.out.push(HostEvent::Refresh { pane }),
            HostRequest::OpenSftp {
                pane,
                user,
                host,
                port,
            } => self.out.push(HostEvent::OpenSftp {
                pane,
                user,
                host,
                port,
            }),
            HostRequest::LeaveConnection { pane, remote } => {
                // Only while the pane still shows that server or share.
                if self
                    .panes
                    .get(&pane)
                    .is_some_and(|p| p.remote.as_ref() == Some(&remote))
                {
                    self.out.push(HostEvent::LeaveConnection { pane });
                }
            }
            HostRequest::ReadFile {
                pane,
                path,
                max_bytes,
            } => self.out.push(HostEvent::ReadFile {
                plugin: id.to_string(),
                pane,
                path,
                max_bytes: max_bytes.min(crate::MAX_READ_BYTES),
            }),
            HostRequest::Select { pane, paths } => {
                let pane = pane.or_else(|| self.active_pane());
                let Some(pane) = pane else { return };
                if self.panes.get(&pane).is_none_or(|p| p.local_path.is_none()) {
                    return; // remote or archive pane
                }
                if let Some(event) = select_event(pane, &paths) {
                    self.out.push(event);
                }
            }
            HostRequest::SetStatusItems { mut items } => {
                crate::protocol::sanitize_status_items(&mut items);
                if items.is_empty() {
                    self.status_items.remove(id);
                } else {
                    self.status_items.insert(id.to_string(), items);
                }
            }
        }
    }

    /// Answer a request an external plugin isn't allowed to make with an
    /// error, so it doesn't wait for an answer that never comes.
    fn refuse(&mut self, id: &str, req: HostRequest, perm: &str) {
        let error = format!("not allowed: the plugin doesn't declare the \"{perm}\" permission");
        let answer = match req {
            HostRequest::ReadFile { pane, path, .. } => PluginEvent::FileContent(FileContent {
                pane,
                path,
                bytes: Vec::new(),
                size: None,
                truncated: false,
                modified: None,
                error: Some(error),
            }),
            HostRequest::ListDir { pane, dir } => PluginEvent::DirListing(DirListing {
                pane,
                dir,
                entries: Vec::new(),
                error: Some(error),
            }),
            HostRequest::RenameMany { pane, dir, renames } => {
                PluginEvent::RenameResult(RenameResult {
                    pane,
                    dir,
                    ok: Vec::new(),
                    failed: renames
                        .into_iter()
                        .map(|(old, _)| (old, error.clone()))
                        .collect(),
                })
            }
            _ => return,
        };
        self.deliver(id, answer);
    }

    /// Status bar items of the running plugins, with the plugin id each
    /// belongs to, in plugin order (built-ins first).
    pub fn status_items(&self) -> Vec<(String, StatusItem)> {
        let mut v = Vec::new();
        for slot in self.slots.iter().filter(|s| s.running) {
            if let Some(items) = self.status_items.get(&slot.info.id) {
                v.extend(items.iter().map(|it| (slot.info.id.clone(), it.clone())));
            }
        }
        v
    }

    /// Route a [`HostRequest::RunInTerminal`] to the Terminal plugin: open
    /// its panel below the pane, then deliver the command.
    pub fn run_in_terminal(&mut self, pane: Option<PaneId>, command: String, cwd: Option<VfsPath>) {
        if command.trim().is_empty() {
            return;
        }
        let running = self
            .slots
            .iter()
            .any(|s| s.running && s.info.id == TERMINAL_PLUGIN);
        if !running {
            let mut short: String = command.chars().take(60).collect();
            if short.len() < command.len() {
                short.push('…');
            }
            self.out.push(HostEvent::Toast {
                level: ToastLevel::Warning,
                text: format!(
                    "Can't run “{short}”: the Terminal plugin is off. Switch it on in Settings → Plugins."
                ),
            });
            return;
        }
        let pane = pane.or_else(|| self.active_pane()).unwrap_or(PaneId::LEFT);
        self.set_panel_open(TERMINAL_PLUGIN, TERMINAL_PANEL, Some(pane), true);
        self.deliver(
            TERMINAL_PLUGIN,
            PluginEvent::RunInTerminal { pane, command, cwd },
        );
        self.ctx.request_repaint();
    }

    fn rebuild_merged(&mut self, pane: PaneId) {
        let mut merged: HashMap<String, MergedDecoration> = HashMap::new();
        // Plugin order = slot order, so badges keep a stable order.
        for slot in &self.slots {
            let Some(d) = self.decorations.get(&(slot.info.id.clone(), pane)) else {
                continue;
            };
            for (name, dec) in &d.data {
                let m = merged.entry(name.clone()).or_default();
                if let Some(b) = &dec.badge {
                    m.badges.push(b.clone());
                }
                if m.name_tone.is_none() {
                    m.name_tone = dec.name_tone;
                }
                m.dimmed |= dec.dimmed;
                if let Some(t) = dec.tooltip.as_deref().filter(|t| !t.is_empty()) {
                    m.tooltip = Some(match m.tooltip.take() {
                        Some(prev) => format!("{prev}\n{t}"),
                        None => t.to_string(),
                    });
                }
            }
        }
        if merged.is_empty() {
            self.merged.remove(&pane);
        } else {
            self.merged.insert(pane, merged);
        }
    }

    fn rebuild_sizes(&mut self, pane: PaneId) {
        let mut merged: HashMap<String, SizeInfo> = HashMap::new();
        // The first plugin (slot order) wins for a name.
        for slot in &self.slots {
            if let Some(d) = self.sizes.get(&(slot.info.id.clone(), pane)) {
                for (name, s) in &d.data {
                    merged.entry(name.clone()).or_insert(*s);
                }
            }
        }
        let old = self.merged_sizes.get(&pane).unwrap_or(&self.empty_sizes);
        if merged != *old {
            if merged.is_empty() {
                self.merged_sizes.remove(&pane);
            } else {
                self.merged_sizes.insert(pane, merged);
            }
            *self.sizes_gen.entry(pane).or_default() += 1;
        }
    }

    /// Plugin-provided sizes of the entries in `pane` (by file name), for
    /// the folder the pane showed when they were set.
    pub fn sizes(&self, pane: PaneId) -> &HashMap<String, SizeInfo> {
        self.merged_sizes.get(&pane).unwrap_or(&self.empty_sizes)
    }

    /// Changes whenever [`sizes`](Self::sizes) of `pane` changes.
    pub fn sizes_gen(&self, pane: PaneId) -> u64 {
        self.sizes_gen.get(&pane).copied().unwrap_or(0)
    }

    /// Merged decorations of the entries in `pane` (by file name).
    pub fn decorations(&self, pane: PaneId) -> &HashMap<String, MergedDecoration> {
        self.merged.get(&pane).unwrap_or(&self.empty_merged)
    }

    /// Path-bar chips for `pane`, with the plugin id each belongs to.
    pub fn chips(&self, pane: PaneId) -> Vec<(String, Chip)> {
        let Some(state) = self.panes.get(&pane) else {
            return Vec::new();
        };
        let mut v = Vec::new();
        for slot in &self.slots {
            if let Some(c) = self.chips.get(&(slot.info.id.clone(), pane)) {
                if c.dir == state.path {
                    v.extend(
                        c.data
                            .iter()
                            .map(|chip| (slot.info.id.clone(), chip.clone())),
                    );
                }
            }
        }
        v
    }

    // ------------------------------------------------------------ commands

    fn refresh_contributions(&mut self, id: &str) {
        let Some(i) = self.slots.iter().position(|s| s.info.id == id) else {
            return;
        };
        if !self.slots[i].running {
            return;
        }
        let Some(commands) = self.call(i, "commands", |p| p.commands()) else {
            return;
        };
        let Some(panels) = self.call(i, "panels", |p| p.panels()) else {
            return;
        };
        let mut sizes = HashMap::new();
        for panel in panels
            .iter()
            .filter(|p| p.placement == PanelPlacement::Modal)
        {
            if let Some(Some(sz)) = self.call(i, "panel_size", |p| p.panel_size(&panel.id)) {
                sizes.insert(panel.id.clone(), sz);
            }
        }
        // Apply `default_open` the first time a panel shows up.
        let mut changed = false;
        for panel in &panels {
            let seen = format!("{id}/{}", panel.id);
            if self.state.seen_panels.insert(seen) {
                changed = true;
                if panel.default_open && panel.placement != PanelPlacement::Modal {
                    let instances = match panel.placement {
                        PanelPlacement::PaneBottom => self.pane_instances(),
                        _ => vec![None],
                    };
                    for pane in instances {
                        self.state
                            .panels
                            .insert(panel_key(id, &panel.id, pane), true);
                    }
                }
            }
        }
        if changed {
            self.save();
        }
        let slot = &mut self.slots[i];
        slot.commands = commands;
        slot.panels = panels;
        slot.panel_sizes = sizes;
        self.rebuild_commands();
    }

    fn rebuild_commands(&mut self) {
        let mut v = Vec::new();
        for slot in self.slots.iter().filter(|s| s.running) {
            for c in &slot.commands {
                let shortcut = c
                    .shortcut
                    .as_deref()
                    .and_then(|s| match shortcut::parse(s) {
                        Ok(sc) => Some(sc),
                        Err(e) => {
                            log::warn!("plugin {}: {e}", slot.info.id);
                            None
                        }
                    });
                v.push(PluginCommand {
                    full_id: format!("{}:{}", slot.info.id, c.id),
                    plugin_id: slot.info.id.clone(),
                    plugin_name: slot.info.name.clone(),
                    spec: c.clone(),
                    shortcut,
                });
            }
        }
        self.commands = v;
    }

    /// Commands of all running plugins.
    pub fn commands(&self) -> &[PluginCommand] {
        &self.commands
    }

    /// Commands for the file list's context menu.
    pub fn context_menu_commands(&self) -> impl Iterator<Item = &PluginCommand> {
        self.commands
            .iter()
            .filter(|c| c.spec.context == CommandContext::FileContextMenu)
    }

    /// Run `plugin_id:command_id` with the active pane's state.
    pub fn run_command(&mut self, full_id: &str, pane: &PaneState) {
        let Some((plugin, cmd)) = full_id.split_once(':') else {
            return;
        };
        let Some(i) = self
            .slots
            .iter()
            .position(|s| s.running && s.info.id == plugin)
        else {
            return;
        };
        let cmd = cmd.to_string();
        self.call(i, "run_command", |p| p.run_command(&cmd, pane));
    }

    // ------------------------------------------------------------ panels

    /// Panels of running plugins with the given placement.
    pub fn panels(&self, placement: PanelPlacement) -> Vec<PanelRef> {
        self.slots
            .iter()
            .filter(|s| s.running)
            .flat_map(|s| {
                s.panels
                    .iter()
                    .filter(|p| p.placement == placement)
                    .map(|p| PanelRef {
                        plugin_id: s.info.id.clone(),
                        plugin_name: s.info.name.clone(),
                        spec: p.clone(),
                        size: s.panel_sizes.get(&p.id).copied(),
                    })
            })
            .collect()
    }

    fn placement_of(&self, plugin: &str, panel: &str) -> Option<PanelPlacement> {
        self.slots
            .iter()
            .find(|s| s.info.id == plugin)?
            .panels
            .iter()
            .find(|p| p.id == panel)
            .map(|p| p.placement)
    }

    /// Is a panel instance open? `pane` is ignored for sidebar panels.
    pub fn is_panel_open(&self, plugin: &str, panel: &str, pane: Option<PaneId>) -> bool {
        let pane = match self.placement_of(plugin, panel) {
            Some(PanelPlacement::Modal) => {
                return self.modal_open.contains(&format!("{plugin}/{panel}"))
            }
            Some(PanelPlacement::RightSidebar) | None => None,
            Some(PanelPlacement::PaneBottom) => pane,
        };
        self.state
            .panels
            .get(&panel_key(plugin, panel, pane))
            .copied()
            .unwrap_or(false)
    }

    /// Open or close a panel. For `PaneBottom` panels, `pane: None` means
    /// every pane.
    pub fn set_panel_open(&mut self, plugin: &str, panel: &str, pane: Option<PaneId>, open: bool) {
        let instances: Vec<Option<PaneId>> = match self.placement_of(plugin, panel) {
            None => return,
            Some(PanelPlacement::Modal) => {
                let key = format!("{plugin}/{panel}");
                let changed = if open {
                    self.modal_open.insert(key)
                } else {
                    self.modal_open.remove(&key)
                };
                if changed {
                    self.ctx.request_repaint();
                }
                return;
            }
            Some(PanelPlacement::RightSidebar) => vec![None],
            Some(PanelPlacement::PaneBottom) => match pane {
                Some(p) => vec![Some(p)],
                None => self.pane_instances(),
            },
        };
        let mut changed = false;
        for inst in instances {
            let key = panel_key(plugin, panel, inst);
            changed |= self.state.panels.insert(key, open) != Some(open);
        }
        if changed {
            self.save();
            self.ctx.request_repaint();
        }
    }

    /// Every `PaneBottom` instance: the known panes (at least the classic
    /// two, which exist before the app reports its panes).
    fn pane_instances(&self) -> Vec<Option<PaneId>> {
        let mut v: BTreeSet<PaneId> = self.panes.keys().copied().collect();
        if v.is_empty() {
            v.extend([PaneId::LEFT, PaneId::RIGHT]);
        }
        v.into_iter().map(Some).collect()
    }

    pub fn panel_content(&self, plugin: &str, panel: &str, pane: Option<PaneId>) -> PanelContent {
        match self
            .slots
            .iter()
            .find(|s| s.info.id == plugin)
            .map(|s| &s.kind)
        {
            Some(SlotKind::External(e)) => match e.panel_blocks(panel, pane) {
                Some(b) => PanelContent::Blocks(b),
                None => PanelContent::Empty,
            },
            Some(SlotKind::Builtin(p)) => {
                match catch_unwind(AssertUnwindSafe(|| p.panel_blocks(panel, pane))) {
                    Ok(Some(b)) => PanelContent::Blocks(b),
                    Ok(None) => PanelContent::Native,
                    Err(e) => {
                        log::error!(
                            "plugin {plugin} panicked in panel_blocks: {}",
                            panic_message(&*e)
                        );
                        PanelContent::Empty
                    }
                }
            }
            _ => PanelContent::Empty,
        }
    }

    /// Draw a built-in plugin's panel body (panics are caught).
    pub fn panel_ui(&mut self, plugin: &str, panel: &str, pane: &PaneState, ui: &mut egui::Ui) {
        let Some(i) = self
            .slots
            .iter()
            .position(|s| s.running && s.info.id == plugin)
        else {
            return;
        };
        self.call(i, "panel_ui", |p| p.panel_ui(panel, pane, ui));
    }

    /// A click on a block row or button with an action (external plugins,
    /// and built-ins that draw with [`Plugin::panel_blocks`]).
    pub fn panel_action(&mut self, plugin: &str, panel: &str, action: &str, pane: &PaneState) {
        let Some(i) = self
            .slots
            .iter()
            .position(|s| s.running && s.info.id == plugin)
        else {
            return;
        };
        match &mut self.slots[i].kind {
            SlotKind::External(e) => e.panel_action(panel, action, pane),
            SlotKind::Builtin(_) => {
                self.call(i, "panel_action", |p| p.panel_action(panel, action, pane));
            }
            SlotKind::Invalid => {}
        }
    }

    // ------------------------------------------------------------ settings

    pub fn rows(&self) -> Vec<PluginRow> {
        self.slots
            .iter()
            .map(|s| {
                let (status, log) = match &s.kind {
                    SlotKind::External(e) => {
                        let st = match (&s.error, s.running) {
                            (Some(err), _) => PluginStatus::Error(err.clone()),
                            (None, true) => e.status(),
                            (None, false) => match e.status() {
                                PluginStatus::Error(m) => PluginStatus::Error(m),
                                _ => PluginStatus::Stopped,
                            },
                        };
                        (st, e.log_tail())
                    }
                    _ => (
                        match (&s.error, s.running) {
                            (Some(err), _) => PluginStatus::Error(err.clone()),
                            (None, true) => PluginStatus::Running,
                            (None, false) => PluginStatus::Stopped,
                        },
                        Vec::new(),
                    ),
                };
                PluginRow {
                    info: s.info.clone(),
                    kind: s.kind(),
                    enabled: self.wants_enabled(&s.info.id),
                    status,
                    log,
                    dir: s.dir.clone(),
                    capabilities: s.capabilities.clone(),
                    replaced_by: self.replaced_by(&s.info.id).map(str::to_string),
                    dev: s.dev,
                    sandbox: match &s.kind {
                        SlotKind::External(e) => Some(e.sandbox()),
                        _ => None,
                    },
                    permissions: match &s.kind {
                        SlotKind::External(e) => e.manifest().permissions.clone(),
                        _ => Vec::new(),
                    },
                }
            })
            .collect()
    }

    /// Draw a plugin's own settings; `false` if it has none.
    pub fn settings_ui(&mut self, id: &str, ui: &mut egui::Ui) -> bool {
        let Some(i) = self.slots.iter().position(|s| s.info.id == id) else {
            return false;
        };
        self.call(i, "settings_ui", |p| p.settings_ui(ui))
            .unwrap_or(false)
    }

    /// Restart a plugin (externals: re-read `plugin.toml`).
    pub fn reload(&mut self, id: &str) {
        let Some(i) = self.slots.iter().position(|s| s.info.id == id) else {
            return;
        };
        self.stop(id);
        self.slots[i].error = None;
        if let Some(dir) = self.slots[i].dir.clone() {
            self.slots[i] = load_slot(&dir);
            if self.slots[i].info.id != id {
                // The id changed in plugin.toml: rescan to sort it out.
                self.rescan();
                return;
            }
        }
        if self.wants_enabled(id) {
            self.start(id);
        }
        self.rebuild_commands();
    }

    fn scan_externals(&mut self) {
        let dir = self
            .paths
            .as_ref()
            .map(|p| p.plugins_dir.clone())
            .unwrap_or_default();
        let mut folders: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.is_dir()
                            && !self.dev_dirs.contains(p)
                            && !p
                                .file_name()
                                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                    })
                    .collect()
            })
            .unwrap_or_default();
        folders.sort();
        let mut found: Vec<Slot> = self
            .dev_dirs
            .iter()
            .map(|d| {
                let mut s = load_slot(d);
                s.dev = true;
                s
            })
            .collect();
        let dev_ids: BTreeSet<String> = found
            .iter()
            .filter(|s| s.kind() != PluginKind::Invalid)
            .map(|s| s.info.id.clone())
            .collect();
        found.extend(
            folders
                .iter()
                .map(|f| load_slot(f))
                .filter(|s| !dev_ids.contains(&s.info.id)),
        );
        // Duplicate ids: the first folder wins.
        let mut seen: BTreeMap<String, PathBuf> = self
            .slots
            .iter()
            .filter(|s| s.kind() == PluginKind::Builtin)
            .map(|s| (s.info.id.clone(), PathBuf::new()))
            .collect();
        for s in &mut found {
            if s.kind() == PluginKind::Invalid {
                continue;
            }
            if let Some(other) = seen.get(&s.info.id) {
                let msg = format!(
                    "Another plugin already uses the id “{}”{}",
                    s.info.id,
                    other
                        .file_name()
                        .map(|n| format!(" (folder “{}”)", n.to_string_lossy()))
                        .unwrap_or_default()
                );
                s.info.id = format!(
                    "{}#{}",
                    s.info.id,
                    s.dir
                        .as_ref()
                        .and_then(|d| d.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                );
                s.kind = SlotKind::Invalid;
                s.error = Some(msg);
            } else {
                seen.insert(s.info.id.clone(), s.dir.clone().unwrap_or_default());
            }
        }
        found.sort_by_key(|s| (!s.dev, s.info.name.to_lowercase()));
        self.slots.extend(found);
    }

    /// Re-read the plugins folder: pick up added, removed and changed
    /// plugins. Unchanged running plugins keep running.
    pub fn rescan(&mut self) {
        let old: Vec<Slot> = self
            .slots
            .iter()
            .position(|s| s.kind() != PluginKind::Builtin)
            .map(|i| self.slots.split_off(i))
            .unwrap_or_default();
        let builtins = self.slots.len();
        self.scan_externals();
        self.retire_externals();
        let mut old = old;
        for i in builtins..self.slots.len() {
            let id = self.slots[i].info.id.clone();
            let same = old.iter().position(|o| {
                o.info.id == id
                    && o.dir == self.slots[i].dir
                    && match (&o.kind, &self.slots[i].kind) {
                        (SlotKind::External(a), SlotKind::External(b)) => {
                            a.manifest() == b.manifest()
                        }
                        _ => false,
                    }
            });
            if let Some(j) = same {
                // Keep the running instance (and its process).
                self.slots[i] = old.remove(j);
            }
        }
        // Whatever is left in `old` is gone or changed: stop it.
        for mut o in old {
            if o.running {
                if let Some(p) = o.plugin() {
                    let _ = catch_unwind(AssertUnwindSafe(|| p.disable()));
                }
            }
            let id = o.info.id.clone();
            self.decorations.retain(|(p, _), _| *p != id);
            self.chips.retain(|(p, _), _| *p != id);
            self.status_items.remove(&id);
        }
        let ids: Vec<String> = self.slots[builtins..]
            .iter()
            .map(|s| s.info.id.clone())
            .collect();
        for id in ids {
            if self.wants_enabled(&id) {
                self.start(&id);
            }
        }
        self.rebuild_all_panes();
        self.rebuild_commands();
    }

    /// Install the plugin in `src` (validated, then copied into the plugins
    /// folder on a background thread). If a plugin with the same id is
    /// installed, [`HostEvent::ConfirmReplace`] comes back from
    /// [`poll`](Self::poll) unless `replace` is set.
    pub fn install(&mut self, src: PathBuf, replace: bool) {
        let Some(plugins_dir) = self.plugins_dir().map(Path::to_path_buf) else {
            self.out.push(HostEvent::Toast {
                level: ToastLevel::Error,
                text: "Plugins can't be installed: no configuration folder".into(),
            });
            return;
        };
        // When replacing a running plugin, stop it before its files change.
        let mut was_running = false;
        if replace {
            if let Ok(m) = Manifest::load(&src) {
                if let Some(s) = self.slots.iter().find(|s| s.info.id == m.id) {
                    was_running = s.running;
                }
                if was_running {
                    self.stop(&m.id);
                    self.reenable_after_install.insert(m.id.clone());
                }
            }
        }
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            if was_running {
                // Let the old process exit (it gets SHUTDOWN_GRACE).
                std::thread::sleep(crate::external::SHUTDOWN_GRACE + Duration::from_millis(100));
            }
            let job = install_files(&src, &plugins_dir, replace);
            let _ = tx.send(job);
            ctx.request_repaint();
        });
    }

    /// Check a plugin folder or a `.zip` / `.mantaplugin` archive the user
    /// picked, without installing anything: archives are unpacked into a
    /// hidden staging folder. [`HostEvent::InstallReview`] (or
    /// [`HostEvent::InstallFailed`]) comes back from [`poll`](Self::poll).
    pub fn review_install(&mut self, src: PathBuf) {
        let Some(plugins_dir) = self.plugins_dir().map(Path::to_path_buf) else {
            self.out.push(HostEvent::InstallFailed {
                error: "no configuration folder".into(),
            });
            return;
        };
        self.review_seq += 1;
        let staging = plugins_dir.join(format!(
            "{}{}-{}",
            install::STAGING_PREFIX,
            std::process::id(),
            self.review_seq
        ));
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let _ = std::fs::create_dir_all(&plugins_dir);
            let job = match install::prepare(&src, &staging) {
                Ok(r) => Job::Review(Box::new(r)),
                Err(e) => Job::InstallFailed(e),
            };
            let _ = tx.send(job);
            ctx.request_repaint();
        });
    }

    /// Install what [`Self::review_install`] reviewed (replacing an
    /// installed plugin with the same id). The staging folder is removed
    /// afterwards.
    pub fn confirm_install(&mut self, review: &PluginReview) {
        let Some(plugins_dir) = self.plugins_dir().map(Path::to_path_buf) else {
            return;
        };
        let was_running = self
            .slots
            .iter()
            .any(|s| s.info.id == review.id && s.running && !s.dev);
        if was_running {
            self.stop(&review.id);
            self.reenable_after_install.insert(review.id.clone());
        }
        let root = review.root.clone();
        let staging = review.staging.clone();
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            if was_running {
                std::thread::sleep(crate::external::SHUTDOWN_GRACE + Duration::from_millis(100));
            }
            let job = install_files(&root, &plugins_dir, true);
            if let Some(dir) = staging {
                let _ = std::fs::remove_dir_all(dir);
            }
            let _ = tx.send(job);
            ctx.request_repaint();
        });
    }

    /// The user cancelled a review: remove its staging folder.
    pub fn cancel_install(&mut self, review: &PluginReview) {
        if let Some(dir) = review.staging.clone() {
            std::thread::spawn(move || {
                let _ = std::fs::remove_dir_all(dir);
            });
        }
    }

    /// What the trust sheet shows about an installed external plugin
    /// (`None` for built-ins and broken folders).
    pub fn review(&self, id: &str) -> Option<PluginReview> {
        let s = self.slots.iter().find(|s| s.info.id == id)?;
        let SlotKind::External(e) = &s.kind else {
            return None;
        };
        Some(PluginReview::of_folder(e.dir(), e.manifest()))
    }

    /// Remove macOS's quarantine flag from an installed plugin's files,
    /// after the user confirmed the trust sheet that says so. Returns how
    /// many files had it.
    pub fn clear_quarantine(&mut self, id: &str) -> usize {
        let Some(dir) = self
            .slots
            .iter()
            .find(|s| s.info.id == id && s.kind() == PluginKind::External)
            .and_then(|s| s.dir.clone())
        else {
            return 0;
        };
        let n = install::clear_quarantine(&dir);
        if n > 0 {
            log::info!("plugin {id}: removed the quarantine flag from {n} files after the user's confirmation");
        }
        n
    }

    /// Run the plugin in `dir` straight from that folder for this session
    /// (`manta --plugin-dev <dir>`): not copied, trusted and switched on
    /// without asking, nothing saved. It replaces an installed plugin with
    /// the same id while it is loaded. Returns the plugin's id.
    pub fn add_dev_plugin(&mut self, dir: &Path) -> Result<String, String> {
        let dir = std::fs::canonicalize(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let m = Manifest::load(&dir).map_err(|e| e.to_string())?;
        if !self.dev_dirs.contains(&dir) {
            self.dev_dirs.push(dir);
        }
        // An installed copy with the same id steps aside for the session
        // (scan_externals leaves it out; rescan stops it).
        self.rescan();
        Ok(m.id)
    }

    /// The plugin was loaded with [`Self::add_dev_plugin`].
    pub fn is_dev(&self, id: &str) -> bool {
        self.slots.iter().any(|s| s.info.id == id && s.dev)
    }

    /// Restart every running external plugin (re-reading its
    /// `plugin.toml`) and rescan the plugins folder: the "Reload Plugins"
    /// command, handy while developing.
    pub fn reload_all(&mut self) {
        let ids: Vec<String> = self
            .slots
            .iter()
            .filter(|s| s.kind() == PluginKind::External && s.running)
            .map(|s| s.info.id.clone())
            .collect();
        for id in ids {
            self.reload(&id);
        }
        self.rescan();
    }

    /// The plugins that ship with the app (usually
    /// [`bundled::embedded`]), offered in Settings → Plugins → Available.
    pub fn set_bundled(&mut self, plugins: Vec<BundledPlugin>) {
        self.bundled = plugins;
    }

    /// Bundled plugins with their install state, in name order.
    pub fn bundled_rows(&self) -> Vec<BundledRow> {
        let mut rows: Vec<BundledRow> = self
            .bundled
            .iter()
            .map(|b| {
                let id = &b.manifest.id;
                let installed_version = self
                    .slots
                    .iter()
                    .find(|s| &s.info.id == id && s.kind() == PluginKind::External)
                    .map(|s| s.info.version.clone());
                let update_available = installed_version.as_deref().is_some_and(|v| {
                    bundled::compare_versions(&b.manifest.version, v) == std::cmp::Ordering::Greater
                });
                BundledRow {
                    info: b.manifest.info(),
                    installed_version,
                    update_available,
                    installing: self.installing.contains(id),
                }
            })
            .collect();
        rows.sort_by_key(|r| r.info.name.to_lowercase());
        rows
    }

    /// Install (or update) a bundled plugin: its files are written to a
    /// staging folder and installed from there on a background thread. A
    /// new plugin starts switched off (the usual trust confirmation applies
    /// when it is switched on); an update keeps the on/off state.
    pub fn install_bundled(&mut self, id: &str) {
        let Some(b) = self.bundled.iter().find(|b| b.manifest.id == id).cloned() else {
            return;
        };
        let Some(plugins_dir) = self.plugins_dir().map(Path::to_path_buf) else {
            self.out.push(HostEvent::Toast {
                level: ToastLevel::Error,
                text: "Plugins can't be installed: no configuration folder".into(),
            });
            return;
        };
        if !self.installing.insert(id.to_string()) {
            return;
        }
        let installed = self
            .slots
            .iter()
            .any(|s| s.info.id == id && s.kind() == PluginKind::External);
        let was_running = self.slots.iter().any(|s| s.info.id == id && s.running);
        if was_running {
            self.stop(id);
            self.reenable_after_install.insert(id.to_string());
        }
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            if was_running {
                std::thread::sleep(crate::external::SHUTDOWN_GRACE + Duration::from_millis(100));
            }
            let staging = plugins_dir.join(format!(".bundled-{}", b.manifest.id));
            let _ = std::fs::remove_dir_all(&staging);
            let job = match b.write_to(&staging) {
                Ok(()) => install_files(&staging, &plugins_dir, installed),
                Err(e) => Job::InstallFailed(format!("couldn't unpack the plugin: {e}")),
            };
            let _ = std::fs::remove_dir_all(&staging);
            let job = match job {
                // The id was checked above; a race with a manual install
                // of the same id simply replaces it.
                Job::NeedsReplace { .. } => {
                    Job::InstallFailed("another copy was installed meanwhile".into())
                }
                j => j,
            };
            let _ = tx.send(job);
            ctx.request_repaint();
        });
    }

    // ------------------------------------------------------------ directory

    /// `plugin-registry.json` next to `plugins.json`.
    fn registry_cache(&self) -> Option<PathBuf> {
        self.paths
            .as_ref()
            .map(|p| p.state_file.with_file_name(registry::CACHE_FILE))
    }

    /// Use another directory URL (tests).
    pub fn set_registry_url(&mut self, url: impl Into<String>) {
        self.registry_url = url.into();
    }

    /// Test seam: show these entries without downloading anything.
    #[doc(hidden)]
    pub fn set_discover_for_test(&mut self, discover: Discover) {
        self.discover = discover;
    }

    /// The Discover tab was opened: download the directory once per session
    /// (the only time Manta contacts the directory on its own; see
    /// [`registry`]).
    pub fn open_discover(&mut self) {
        if !self.discover.requested {
            self.refresh_registry();
        }
    }

    /// Download the directory again (Refresh in Discover).
    pub fn refresh_registry(&mut self) {
        self.discover.requested = true;
        if self.discover.loading {
            return;
        }
        self.discover.loading = true;
        let url = self.registry_url.clone();
        let cache = self.registry_cache();
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let result = registry::fetch(&url, cache.as_deref());
            let _ = tx.send(Job::Registry(result));
            ctx.request_repaint();
        });
    }

    pub fn discover(&self) -> &Discover {
        &self.discover
    }

    fn registry_row(&self, e: &Entry) -> RegistryRow {
        let installed_version = self
            .slots
            .iter()
            .find(|s| s.info.id == e.id && s.kind() == PluginKind::External && !s.dev)
            .map(|s| s.info.version.clone());
        let update_available = installed_version.as_deref().is_some_and(|v| {
            bundled::compare_versions(&e.latest.version, v) == std::cmp::Ordering::Greater
        });
        RegistryRow {
            name: e.name(),
            description: e.description(),
            installed_version,
            update_available,
            incompatible: e.incompatibility(),
            downloading: self.downloading.contains(&e.id),
            entry: e.clone(),
        }
    }

    /// The directory's entries with their install state (name order).
    pub fn registry_rows(&self) -> Vec<RegistryRow> {
        self.discover
            .plugins
            .iter()
            .map(|e| self.registry_row(e))
            .collect()
    }

    /// The directory entry of an installed plugin when it offers a newer,
    /// compatible version.
    pub fn registry_update(&self, id: &str) -> Option<RegistryRow> {
        let e = self.discover.plugins.iter().find(|e| e.id == id)?;
        let row = self.registry_row(e);
        (row.update_available && row.incompatible.is_none()).then_some(row)
    }

    /// Install (or update) a directory entry: download the release over
    /// HTTPS, check its SHA-256 against the entry (a mismatch stops here),
    /// unpack it into a staging folder and check that its `plugin.toml`
    /// matches the entry. Then [`HostEvent::InstallReview`] comes back and
    /// the usual review → install → trust flow follows.
    pub fn install_from_registry(&mut self, id: &str) {
        let Some(entry) = self.discover.plugins.iter().find(|e| e.id == id).cloned() else {
            return;
        };
        if let Some(why) = entry.incompatibility() {
            let error = match why {
                Incompatible::NeedsManta(v) => {
                    format!("{} needs Manta {v} or newer", entry.id)
                }
                Incompatible::Platform(p) => {
                    format!("{} doesn't run on this Mac ({})", entry.id, p.join(", "))
                }
            };
            self.out.push(HostEvent::InstallFailed { error });
            return;
        }
        let Some(plugins_dir) = self.plugins_dir().map(Path::to_path_buf) else {
            self.out.push(HostEvent::InstallFailed {
                error: "no configuration folder".into(),
            });
            return;
        };
        if !self.downloading.insert(entry.id.clone()) {
            return;
        }
        self.review_seq += 1;
        let tag = format!("{}-{}", std::process::id(), self.review_seq);
        let staging = plugins_dir.join(format!("{}{tag}", install::STAGING_PREFIX));
        let file = plugins_dir.join(format!("{}{tag}.mantaplugin", install::STAGING_PREFIX));
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let result = download_and_prepare(&entry, &plugins_dir, &file, &staging);
            let _ = std::fs::remove_file(&file);
            let _ = tx.send(Job::Downloaded {
                id: entry.id,
                result: result.map(Box::new),
            });
            ctx.request_repaint();
        });
    }

    /// Stop and delete an external plugin (files removed in the background).
    pub fn uninstall(&mut self, id: &str) {
        let Some(i) = self.slots.iter().position(|s| s.info.id == id) else {
            return;
        };
        if self.slots[i].kind() == PluginKind::Builtin || self.slots[i].dev {
            return;
        }
        self.stop(id);
        let slot = self.slots.remove(i);
        self.state.enabled.remove(id);
        self.state.trusted.remove(id);
        let prefix = format!("{id}/");
        self.state.panels.retain(|k, _| !k.starts_with(&prefix));
        self.state.seen_panels.retain(|k| !k.starts_with(&prefix));
        self.save();
        self.rebuild_commands();
        let Some(dir) = slot.dir else { return };
        let name = slot.info.name;
        let tx = self.jobs_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            // Give the process time to exit before its folder disappears.
            std::thread::sleep(crate::external::SHUTDOWN_GRACE);
            let result = std::fs::remove_dir_all(&dir).map_err(|e| e.to_string());
            let _ = tx.send(Job::Uninstalled { name, result });
            ctx.request_repaint();
        });
    }

    fn handle_job(&mut self, job: Job) {
        match job {
            Job::Installed { id, name, replaced } => {
                self.installing.remove(&id);
                self.rescan();
                let reenable = self.reenable_after_install.remove(&id);
                if reenable && self.wants_enabled(&id) {
                    self.start(&id);
                }
                self.out.push(HostEvent::Installed { id, name, replaced });
            }
            Job::Registry(result) => {
                self.discover.loading = false;
                match result {
                    Ok(f) => {
                        self.discover.plugins = f.registry.plugins;
                        self.discover.from_cache = f.from_cache;
                        self.discover.fetched_at = f.fetched_at;
                        self.discover.error = f.error;
                    }
                    Err(e) => self.discover.error = Some(e),
                }
            }
            Job::Downloaded { id, result } => {
                self.downloading.remove(&id);
                match result {
                    Ok(r) => self.handle_job(Job::Review(r)),
                    Err(e) => self.handle_job(Job::InstallFailed(e)),
                }
            }
            Job::Review(mut r) => {
                r.installed_version = self
                    .slots
                    .iter()
                    .find(|s| s.info.id == r.id && s.kind() == PluginKind::External && !s.dev)
                    .map(|s| s.info.version.clone());
                self.out.push(HostEvent::InstallReview(r));
            }
            Job::NeedsReplace {
                src,
                id,
                name,
                new_version,
            } => {
                let installed_version = self
                    .slots
                    .iter()
                    .find(|s| s.info.id == id)
                    .map(|s| s.info.version.clone())
                    .unwrap_or_default();
                self.out.push(HostEvent::ConfirmReplace {
                    src,
                    id,
                    name,
                    installed_version,
                    new_version,
                });
            }
            Job::InstallFailed(e) => {
                self.installing.clear();
                // A replace that failed: start the old version again.
                for id in std::mem::take(&mut self.reenable_after_install) {
                    if self.wants_enabled(&id) {
                        self.start(&id);
                    }
                }
                self.out.push(HostEvent::InstallFailed { error: e });
            }
            Job::Uninstalled { name, result } => self.out.push(HostEvent::Uninstalled {
                name,
                error: result.err(),
            }),
        }
    }

    /// Switch every plugin off (app exit). Waits at most `wait` for
    /// external processes to exit; the rest are killed by their own
    /// supervisor threads shortly after.
    pub fn shutdown(&mut self, wait: Duration) {
        let ids: Vec<String> = self
            .slots
            .iter()
            .filter(|s| s.running)
            .map(|s| s.info.id.clone())
            .collect();
        for id in &ids {
            self.stop(id);
        }
        let start = std::time::Instant::now();
        for s in &self.slots {
            if let SlotKind::External(e) = &s.kind {
                let left = wait.saturating_sub(start.elapsed());
                e.wait_stopped(left);
            }
        }
    }
}

impl Drop for PluginHost {
    fn drop(&mut self) {
        for i in 0..self.slots.len() {
            if self.slots[i].running {
                self.slots[i].running = false;
                if let Some(p) = self.slots[i].plugin() {
                    let _ = catch_unwind(AssertUnwindSafe(|| p.disable()));
                }
            }
        }
    }
}

/// Background part of [`PluginHost::install_from_registry`].
fn download_and_prepare(
    entry: &Entry,
    plugins_dir: &Path,
    file: &Path,
    staging: &Path,
) -> Result<PluginReview, String> {
    std::fs::create_dir_all(plugins_dir).map_err(|e| e.to_string())?;
    registry::download(&entry.latest.url, &entry.latest.sha256, file)?;
    // Downloaded from the internet: quarantined like a browser download,
    // removed only after the user confirms the trust sheet.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    install::set_quarantine(file, format!("0083;{now:08x};Manta;").as_bytes());
    let mut r = install::prepare(file, staging)?;
    if r.id != entry.id || r.version != entry.latest.version {
        if let Some(s) = &r.staging {
            let _ = std::fs::remove_dir_all(s);
        }
        return Err(format!(
            "the download doesn't match the plugin directory: its plugin.toml says {} {}, \
             the directory says {} {}",
            r.id, r.version, entry.id, entry.latest.version
        ));
    }
    r.source = PathBuf::from(
        entry
            .latest
            .url
            .rsplit('/')
            .next()
            .filter(|n| !n.is_empty())
            .unwrap_or("plugin.mantaplugin"),
    );
    r.verified_download = Some(entry.latest.url.clone());
    Ok(r)
}

/// The permission an external plugin must declare to make `req`: reading
/// files and listing folders through a pane's connection (`read_files`),
/// renaming (`write_files`), typing into the built-in Terminal
/// (`terminal`).
pub fn required_permission(req: &HostRequest) -> Option<&'static str> {
    match req {
        HostRequest::ReadFile { .. } | HostRequest::ListDir { .. } => Some("read_files"),
        HostRequest::RenameMany { .. } => Some("write_files"),
        HostRequest::RunInTerminal { .. } => Some("terminal"),
        _ => None,
    }
}

/// The protocol name of a request [`required_permission`] checks.
fn request_name(req: &HostRequest) -> &'static str {
    match req {
        HostRequest::ReadFile { .. } => "read_file",
        HostRequest::ListDir { .. } => "list_dir",
        HostRequest::RenameMany { .. } => "rename_many",
        HostRequest::RunInTerminal { .. } => "run_in_terminal",
        _ => "a request",
    }
}

/// Load one plugin folder (an `Invalid` slot if its manifest is broken).
fn load_slot(dir: &Path) -> Slot {
    match Manifest::load(dir) {
        Ok(m) => {
            let info = m.info();
            let capabilities = m.capabilities.clone();
            let plugin = ExternalPlugin::new(m, dir.to_path_buf());
            Slot {
                info,
                commands: Vec::new(),
                panels: Vec::new(),
                panel_sizes: HashMap::new(),
                kind: SlotKind::External(Box::new(plugin)),
                running: false,
                error: None,
                dir: Some(dir.to_path_buf()),
                capabilities,
                dev: false,
            }
        }
        Err(e) => {
            let folder = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            Slot {
                info: PluginInfo {
                    id: format!("invalid#{folder}"),
                    name: folder,
                    version: String::new(),
                    description: String::new(),
                    author: String::new(),
                    builtin: false,
                },
                kind: SlotKind::Invalid,
                running: false,
                error: Some(e.message),
                dir: Some(dir.to_path_buf()),
                capabilities: Vec::new(),
                commands: Vec::new(),
                panels: Vec::new(),
                panel_sizes: HashMap::new(),
                dev: false,
            }
        }
    }
}

/// Copy a retired external plugin's data (`old`) into its built-in
/// successor's data dir (`new`) if `new` has no files yet. The external
/// plugin's `plugin.log` stays behind. Small files only (settings,
/// `recent.json`); errors are logged.
fn migrate_data(old: &Path, new: &Path) {
    let has_files = std::fs::read_dir(new).is_ok_and(|mut rd| rd.next().is_some());
    if has_files || !old.is_dir() {
        return;
    }
    let Ok(rd) = std::fs::read_dir(old) else {
        return;
    };
    if let Err(e) = std::fs::create_dir_all(new) {
        log::warn!("can't create {}: {e}", new.display());
        return;
    }
    for entry in rd.flatten() {
        let name = entry.file_name();
        let n = name.to_string_lossy();
        if n.starts_with("plugin.log") || skip_on_copy(&n) {
            continue;
        }
        let (from, to) = (entry.path(), new.join(&name));
        let r = match entry.file_type() {
            Ok(t) if t.is_dir() => copy_dir(&from, &to, 0),
            Ok(t) if t.is_file() => std::fs::copy(&from, &to).map(|_| ()),
            _ => Ok(()),
        };
        match r {
            Ok(()) => log::info!("migrated {} to {}", from.display(), to.display()),
            Err(e) => log::warn!("can't migrate {}: {e}", from.display()),
        }
    }
}

/// Skipped when copying a plugin folder.
fn skip_on_copy(name: &str) -> bool {
    matches!(name, ".git" | "__pycache__" | ".DS_Store")
}

fn copy_dir(src: &Path, dst: &Path, depth: usize) -> std::io::Result<()> {
    if depth > 32 {
        return Err(std::io::Error::other("folder nesting too deep"));
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if skip_on_copy(&name.to_string_lossy()) {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::fs::read_link(&from)?, &to)?;
            #[cfg(not(unix))]
            std::fs::copy(&from, &to).map(|_| ())?;
        } else if ft.is_dir() {
            copy_dir(&from, &to, depth + 1)?;
        } else {
            // `copy` keeps permissions, so executables stay executable.
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Validate `src` and copy it to `<plugins_dir>/<id>` (background thread).
fn install_files(src: &Path, plugins_dir: &Path, replace: bool) -> Job {
    let m = match Manifest::load(src) {
        Ok(m) => m,
        Err(e) => return Job::InstallFailed(e.message),
    };
    let dest = plugins_dir.join(&m.id);
    let canon = |p: &Path| std::fs::canonicalize(p).ok();
    if canon(src).is_some() && canon(src) == canon(&dest) {
        return Job::InstallFailed("this folder is already the installed copy".into());
    }
    if canon(src)
        .zip(canon(plugins_dir))
        .is_some_and(|(s, d)| d.starts_with(&s))
    {
        return Job::InstallFailed(
            "can't install a folder that contains the plugins folder".into(),
        );
    }
    // An installed plugin with the same id may live in another folder.
    let existing = std::fs::read_dir(plugins_dir).ok().and_then(|rd| {
        rd.flatten()
            // Hidden folders (staging, half-done installs) aren't plugins.
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .map(|e| e.path())
            .find(|p| Manifest::load(p).is_ok_and(|x| x.id == m.id))
    });
    if existing.is_some() && !replace {
        return Job::NeedsReplace {
            src: src.to_path_buf(),
            id: m.id,
            name: m.name,
            new_version: m.version,
        };
    }
    let tmp = plugins_dir.join(format!(".installing-{}", m.id));
    let _ = std::fs::remove_dir_all(&tmp);
    if let Err(e) = std::fs::create_dir_all(plugins_dir).and_then(|_| copy_dir(src, &tmp, 0)) {
        let _ = std::fs::remove_dir_all(&tmp);
        return Job::InstallFailed(format!("copying failed: {e}"));
    }
    let replaced = existing.is_some();
    let mut old_dirs: Vec<PathBuf> = existing.into_iter().collect();
    if !old_dirs.contains(&dest) {
        old_dirs.push(dest.clone());
    }
    for old in old_dirs.iter().filter(|d| d.exists()) {
        if let Err(e) = std::fs::remove_dir_all(old) {
            let _ = std::fs::remove_dir_all(&tmp);
            return Job::InstallFailed(format!(
                "couldn't remove the old version in {}: {e}",
                old.display()
            ));
        }
    }
    if let Err(e) = std::fs::rename(&tmp, &dest) {
        let _ = std::fs::remove_dir_all(&tmp);
        return Job::InstallFailed(format!("couldn't move the plugin into place: {e}"));
    }
    if !dest.join(MANIFEST_FILE).exists() {
        return Job::InstallFailed("the copy has no plugin.toml".into());
    }
    install::make_command_executable(&dest, &m);
    Job::Installed {
        id: m.id,
        name: m.name,
        replaced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use manta_core::Protocol;
    use std::sync::Mutex;

    /// What a fake plugin saw.
    #[derive(Default)]
    struct Log(Mutex<Vec<String>>);

    struct Fake {
        id: &'static str,
        badge: Option<(&'static str, Tone)>,
        dim: bool,
        panic_on_run: bool,
        handle: Option<PluginHandle>,
        log: Arc<Log>,
    }

    impl Fake {
        fn new(id: &'static str, log: Arc<Log>) -> Self {
            Fake {
                id,
                badge: None,
                dim: false,
                panic_on_run: false,
                handle: None,
                log,
            }
        }
    }

    impl Plugin for Fake {
        fn info(&self) -> PluginInfo {
            PluginInfo {
                id: self.id.into(),
                name: self.id.to_uppercase(),
                version: "1".into(),
                description: String::new(),
                author: String::new(),
                builtin: true,
            }
        }
        fn enable(&mut self, handle: PluginHandle) {
            self.handle = Some(handle);
            self.log
                .0
                .lock()
                .unwrap()
                .push(format!("{}:enable", self.id));
        }
        fn disable(&mut self) {
            self.handle = None;
            self.log
                .0
                .lock()
                .unwrap()
                .push(format!("{}:disable", self.id));
        }
        fn on_event(&mut self, ev: &PluginEvent) {
            let name = match ev {
                PluginEvent::DirectoryChanged(_) => "dir",
                PluginEvent::ListingRefreshed(_) => "listing",
                PluginEvent::SelectionChanged(_) => "selection",
                PluginEvent::ActivePaneChanged(_) => "active",
                PluginEvent::FileAction(_) => "file_action",
                PluginEvent::RenameResult(_) => "rename_result",
                PluginEvent::DirListing(_) => "dir_listing",
                PluginEvent::FileContent(_) => "file_content",
                PluginEvent::LocaleChanged(_) => "locale",
                PluginEvent::PaneOpened(_) => "opened",
                PluginEvent::PaneClosed(_) => "closed",
                PluginEvent::RunInTerminal { pane, command, cwd } => {
                    let cwd = cwd.as_ref().map_or("-", |c| c.as_str());
                    self.log.0.lock().unwrap().push(format!(
                        "{}:run_in_terminal:{pane}:{command}:{cwd}",
                        self.id
                    ));
                    return;
                }
            };
            self.log
                .0
                .lock()
                .unwrap()
                .push(format!("{}:{name}", self.id));
            if let PluginEvent::DirectoryChanged(p) = ev {
                let h = self.handle.as_ref().unwrap();
                let dec = Decoration {
                    badge: self.badge.map(|(t, tone)| Badge {
                        text: t.into(),
                        tone,
                    }),
                    name_tone: Some(Tone::Accent),
                    dimmed: self.dim,
                    tooltip: Some(format!("from {}", self.id)),
                };
                h.send(HostRequest::SetDecorations {
                    pane: p.pane,
                    dir: p.path.clone(),
                    entries: vec![("a.txt".into(), dec)],
                });
                h.send(HostRequest::SetChips {
                    pane: p.pane,
                    dir: p.path.clone(),
                    chips: vec![Chip {
                        text: self.id.into(),
                        tone: Tone::Info,
                        icon: None,
                        tooltip: None,
                        command: Some("go".into()),
                    }],
                });
            }
        }
        fn commands(&self) -> Vec<CommandSpec> {
            vec![CommandSpec {
                id: "go".into(),
                title: "Go".into(),
                shortcut: Some("⌃`".into()),
                context: CommandContext::FileContextMenu,
            }]
        }
        fn run_command(&mut self, id: &str, _pane: &PaneState) {
            if self.panic_on_run {
                panic!("boom");
            }
            self.log
                .0
                .lock()
                .unwrap()
                .push(format!("{}:run:{id}", self.id));
        }
        fn panels(&self) -> Vec<PanelSpec> {
            vec![PanelSpec {
                id: if self.id == TERMINAL_PLUGIN {
                    TERMINAL_PANEL.into()
                } else {
                    "p".into()
                },
                title: "P".into(),
                placement: PanelPlacement::PaneBottom,
                default_open: self.id == "a",
            }]
        }
    }

    fn pane(id: PaneId, path: &str, active: bool) -> PaneState {
        PaneState {
            remote: None,
            pane: id,
            path: VfsPath::new(path),
            local_path: Some(PathBuf::from(path)),
            protocol: Protocol::Local,
            backend_label: "Local".into(),
            selection: Vec::new(),
            active,
        }
    }

    fn host_with(plugins: Vec<Box<dyn Plugin>>, root: &Path) -> PluginHost {
        PluginHost::new(
            egui::Context::default(),
            plugins,
            Some(HostPaths::under(root)),
        )
    }

    #[test]
    fn leave_connection_only_while_the_pane_shows_it() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new(TERMINAL_PLUGIN, log))], tmp.path());
        let nas = crate::RemoteEndpoint {
            protocol: Protocol::Smb,
            user: "anna".into(),
            host: "nas.local".into(),
            port: 445,
            context: String::new(),
        };
        let mut left = pane(PaneId::Left, "/Volumes/media", true);
        left.protocol = Protocol::Smb;
        left.remote = Some(nas.clone());
        host.set_pane(left, 1);
        host.set_pane(pane(PaneId::Right, "/y", false), 1);
        host.poll();
        let tx = host.tx.clone();
        let send = |r: HostRequest| tx.send((TERMINAL_PLUGIN.into(), r)).unwrap();
        send(HostRequest::LeaveConnection {
            pane: PaneId::Left,
            remote: nas.clone(),
        });
        assert_eq!(
            host.poll(),
            vec![HostEvent::LeaveConnection { pane: PaneId::Left }]
        );
        // The other pane is local; another share: dropped.
        send(HostRequest::LeaveConnection {
            pane: PaneId::Right,
            remote: nas.clone(),
        });
        send(HostRequest::LeaveConnection {
            pane: PaneId::Left,
            remote: crate::RemoteEndpoint {
                host: "backup.local".into(),
                ..nas
            },
        });
        assert!(host.poll().is_empty());
    }

    #[test]
    fn navigate_in_requires_the_same_backend() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new(TERMINAL_PLUGIN, log))], tmp.path());
        let web = crate::RemoteEndpoint {
            protocol: Protocol::Docker,
            user: String::new(),
            host: "web".into(),
            port: 0,
            context: String::new(),
        };
        let mut left = pane(PaneId::Left, "/srv", true);
        left.local_path = None;
        left.protocol = Protocol::Docker;
        left.remote = Some(web.clone());
        host.set_pane(left, 1);
        host.set_pane(pane(PaneId::Right, "/y", false), 1);
        host.poll();
        let tx = host.tx.clone();
        let send = |r: HostRequest| tx.send((TERMINAL_PLUGIN.into(), r)).unwrap();
        // Plain Navigate stays local-only.
        send(HostRequest::Navigate {
            pane: PaneId::Left,
            path: VfsPath::new("/etc"),
        });
        assert!(host.poll().is_empty());
        // NavigateIn for the container the pane shows: accepted.
        send(HostRequest::NavigateIn {
            pane: PaneId::Left,
            remote: Some(web.clone()),
            path: VfsPath::new("/etc"),
        });
        assert_eq!(
            host.poll(),
            vec![HostEvent::Navigate {
                pane: PaneId::Left,
                path: VfsPath::new("/etc")
            }]
        );
        // Another container (the user switched meanwhile): dropped.
        let db = crate::RemoteEndpoint {
            host: "db".into(),
            ..web.clone()
        };
        send(HostRequest::NavigateIn {
            pane: PaneId::Left,
            remote: Some(db),
            path: VfsPath::new("/etc"),
        });
        assert!(host.poll().is_empty());
        // Unchanged path: dropped.
        send(HostRequest::NavigateIn {
            pane: PaneId::Left,
            remote: Some(web.clone()),
            path: VfsPath::new("/srv"),
        });
        assert!(host.poll().is_empty());
        // Local pane: NavigateIn with `None` works like Navigate; a remote
        // endpoint for a local pane is dropped.
        send(HostRequest::NavigateIn {
            pane: PaneId::Right,
            remote: None,
            path: VfsPath::new("/z"),
        });
        assert_eq!(
            host.poll(),
            vec![HostEvent::Navigate {
                pane: PaneId::Right,
                path: VfsPath::new("/z")
            }]
        );
        send(HostRequest::NavigateIn {
            pane: PaneId::Right,
            remote: Some(web),
            path: VfsPath::new("/q"),
        });
        assert!(host.poll().is_empty());
    }

    #[test]
    fn run_in_terminal_goes_to_the_terminal_plugin() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let term = Fake::new(TERMINAL_PLUGIN, log.clone());
        let other = Fake::new("qa", log.clone());
        let mut host = host_with(vec![Box::new(term), Box::new(other)], tmp.path());
        host.set_pane(pane(PaneId::Left, "/x", false), 1);
        host.set_pane(pane(PaneId::Right, "/y", true), 1);
        host.poll();
        host.set_panel_open(TERMINAL_PLUGIN, TERMINAL_PANEL, None, false);
        let tx = host.tx.clone();
        // No pane: the active one (right). The panel opens there only.
        tx.send((
            "qa".into(),
            HostRequest::RunInTerminal {
                pane: None,
                command: "ls -la".into(),
                cwd: Some(VfsPath::new("/y/sub")),
            },
        ))
        .unwrap();
        assert!(host.poll().is_empty());
        assert!(host.is_panel_open(TERMINAL_PLUGIN, TERMINAL_PANEL, Some(PaneId::Right)));
        assert!(!host.is_panel_open(TERMINAL_PLUGIN, TERMINAL_PANEL, Some(PaneId::Left)));
        let seen: Vec<String> = log
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.contains("run_in_terminal"))
            .cloned()
            .collect();
        assert_eq!(
            seen,
            vec![format!(
                "{TERMINAL_PLUGIN}:run_in_terminal:right:ls -la:/y/sub"
            )],
            "only the terminal gets it"
        );
        // Refresh requests come back to the app.
        tx.send((
            "qa".into(),
            HostRequest::RefreshPane {
                pane: Some(PaneId::Left),
            },
        ))
        .unwrap();
        assert_eq!(
            host.poll(),
            vec![HostEvent::Refresh {
                pane: Some(PaneId::Left)
            }]
        );
        // File reads come back to the app, capped.
        tx.send((
            "qa".into(),
            HostRequest::ReadFile {
                pane: PaneId::Right,
                path: VfsPath::new("/y/a.md"),
                max_bytes: u64::MAX,
            },
        ))
        .unwrap();
        assert_eq!(
            host.poll(),
            vec![HostEvent::ReadFile {
                plugin: "qa".into(),
                pane: PaneId::Right,
                path: VfsPath::new("/y/a.md"),
                max_bytes: crate::MAX_READ_BYTES
            }]
        );

        // Terminal off: a toast that says so.
        host.set_enabled(TERMINAL_PLUGIN, false);
        tx.send((
            "qa".into(),
            HostRequest::RunInTerminal {
                pane: Some(PaneId::Left),
                command: "make".into(),
                cwd: None,
            },
        ))
        .unwrap();
        let ev = host.poll();
        assert!(
            matches!(&ev[..], [HostEvent::Toast { level: ToastLevel::Warning, text }] if text.contains("Terminal plugin is off") && text.contains("make")),
            "{ev:?}"
        );
    }

    #[test]
    fn select_resolves_the_folder_and_names() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new("dup", log))], tmp.path());
        host.set_pane(pane(PaneId::Left, "/x", false), 1);
        host.set_pane(pane(PaneId::Right, "/y", true), 1);
        host.poll();
        let tx = host.tx.clone();
        let paths = [
            "/x/sub/a.txt",
            "/x/sub/b c.txt",
            "/x/other/d.txt",
            "/x/sub/a.txt",
        ]
        .map(VfsPath::new)
        .to_vec();
        tx.send(("dup".into(), HostRequest::Select { pane: None, paths }))
            .unwrap();
        assert_eq!(
            host.poll(),
            vec![HostEvent::Select {
                pane: PaneId::Right,
                dir: VfsPath::new("/x/sub"),
                names: vec!["a.txt".into(), "b c.txt".into()]
            }],
            "the active pane; only the first path's folder, no duplicates"
        );
        tx.send((
            "dup".into(),
            HostRequest::Select {
                pane: Some(PaneId::Left),
                paths: vec![],
            },
        ))
        .unwrap();
        assert!(host.poll().is_empty());
        let mut remote = pane(PaneId::Left, "/srv", false);
        remote.local_path = None;
        host.set_pane(remote, 2);
        host.poll();
        tx.send((
            "dup".into(),
            HostRequest::Select {
                pane: Some(PaneId::Left),
                paths: vec![VfsPath::new("/srv/a")],
            },
        ))
        .unwrap();
        assert!(host.poll().is_empty(), "remote panes are ignored");
    }

    #[test]
    fn merges_decorations_and_namespaces_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut a = Fake::new("a", log.clone());
        a.badge = Some(("M", Tone::Warning));
        let mut b = Fake::new("b", log.clone());
        b.badge = Some(("L", Tone::Danger));
        b.dim = true;
        let mut host = host_with(vec![Box::new(a), Box::new(b)], tmp.path());
        host.set_pane(pane(PaneId::Left, "/x", true), 1);
        host.set_pane(pane(PaneId::Right, "/y", false), 1);
        host.poll();

        let m = &host.decorations(PaneId::Left)["a.txt"];
        let texts: Vec<&str> = m.badges.iter().map(|b| b.text.as_str()).collect();
        assert_eq!(texts, vec!["M", "L"], "badges side by side, plugin order");
        assert!(m.dimmed);
        assert_eq!(m.name_tone, Some(Tone::Accent));
        assert_eq!(m.tooltip.as_deref(), Some("from a\nfrom b"));
        assert_eq!(host.chips(PaneId::Right).len(), 2);

        let ids: Vec<&str> = host.commands().iter().map(|c| c.full_id.as_str()).collect();
        assert_eq!(ids, vec!["a:go", "b:go"]);
        assert!(host.commands()[0].shortcut.is_some());
        assert_eq!(host.context_menu_commands().count(), 2);
        host.run_command("b:go", &pane(PaneId::Left, "/x", true));
        assert!(log.0.lock().unwrap().contains(&"b:run:go".to_string()));

        // default_open applies to a's panel on both panes, not b's.
        assert!(host.is_panel_open("a", "p", Some(PaneId::Left)));
        assert!(host.is_panel_open("a", "p", Some(PaneId::Right)));
        assert!(!host.is_panel_open("b", "p", Some(PaneId::Left)));
        host.set_panel_open("b", "p", Some(PaneId::Right), true);
        // TogglePanel from a plugin: the active pane (left) by default.
        let tx = host.tx.clone();
        tx.send((
            "b".into(),
            HostRequest::TogglePanel {
                panel: "p".into(),
                pane: None,
            },
        ))
        .unwrap();
        host.poll();
        assert!(host.is_panel_open("b", "p", Some(PaneId::Left)));
        tx.send((
            "b".into(),
            HostRequest::TogglePanel {
                panel: "p".into(),
                pane: Some(PaneId::Left),
            },
        ))
        .unwrap();
        host.poll();
        assert!(!host.is_panel_open("b", "p", Some(PaneId::Left)));
        assert!(host.is_panel_open("b", "p", Some(PaneId::Right)));
        assert!(!host.is_panel_open("b", "p", Some(PaneId::Left)));
    }

    #[test]
    fn status_items_are_kept_per_plugin_until_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(
            vec![
                Box::new(Fake::new("a", log.clone())),
                Box::new(Fake::new("b", log.clone())),
            ],
            tmp.path(),
        );
        host.poll();
        assert!(host.status_items().is_empty());
        let item = |id: &str, meter: f32| crate::StatusItem {
            meter: Some(meter),
            ..crate::StatusItem::new(id, id)
        };
        host.tx
            .send((
                "b".into(),
                HostRequest::SetStatusItems {
                    items: vec![item("b1", 0.5)],
                },
            ))
            .unwrap();
        host.tx
            .send((
                "a".into(),
                HostRequest::SetStatusItems {
                    items: vec![item("a1", 2.0), item("a2", 0.1)],
                },
            ))
            .unwrap();
        host.poll();
        let got: Vec<(String, String)> = host
            .status_items()
            .into_iter()
            .map(|(p, i)| (p, i.id))
            .collect();
        assert_eq!(
            got,
            [("a", "a1"), ("a", "a2"), ("b", "b1")].map(|(p, i)| (p.to_string(), i.to_string()))
        );
        assert_eq!(host.status_items()[0].1.meter, Some(1.0), "clamped");
        // Replaced, then removed with an empty list.
        host.tx
            .send((
                "a".into(),
                HostRequest::SetStatusItems {
                    items: vec![item("a3", 0.0)],
                },
            ))
            .unwrap();
        host.poll();
        assert_eq!(host.status_items().len(), 2);
        host.tx
            .send(("a".into(), HostRequest::SetStatusItems { items: vec![] }))
            .unwrap();
        host.poll();
        assert_eq!(host.status_items().len(), 1);
        // Switching a plugin off drops its items; late ones are ignored.
        host.set_enabled("b", false);
        assert!(host.status_items().is_empty());
        host.tx
            .send((
                "b".into(),
                HostRequest::SetStatusItems {
                    items: vec![item("b2", 0.5)],
                },
            ))
            .unwrap();
        host.poll();
        assert!(host.status_items().is_empty());
    }

    #[test]
    fn panes_open_and_close() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new("a", log.clone()))], tmp.path());
        let take = |log: &Log| std::mem::take(&mut *log.0.lock().unwrap());
        let third = PaneId(2);
        for (id, dir) in [(PaneId::LEFT, "/x"), (PaneId::RIGHT, "/y"), (third, "/z")] {
            host.set_pane(pane(id, dir, id == third), 1);
        }
        host.poll();
        take(&log);
        assert_eq!(host.pane_ids(), vec![PaneId::LEFT, PaneId::RIGHT, third]);
        // A PaneBottom panel opened for "all panes" reaches the third one.
        host.set_panel_open("a", "p", None, true);
        assert!(host.is_panel_open("a", "p", Some(third)));
        assert!(
            host.decorations(third).contains_key("a.txt"),
            "decorations for pane 3"
        );
        host.close_pane(third);
        assert_eq!(take(&log), vec!["a:closed"]);
        assert!(host.pane(third).is_none());
        assert!(host.decorations(third).is_empty());
        assert!(host.chips(third).is_empty());
        assert!(
            !host.is_panel_open("a", "p", Some(third)),
            "its panel instance is gone"
        );
        assert!(
            host.is_panel_open("a", "p", Some(PaneId::RIGHT)),
            "other panes keep theirs"
        );
        // Late decorations for the closed pane are dropped.
        host.tx
            .send((
                "a".into(),
                HostRequest::SetChips {
                    pane: third,
                    dir: VfsPath::new("/z"),
                    chips: vec![],
                },
            ))
            .unwrap();
        host.poll();
        assert!(host.chips(third).is_empty());
        host.close_pane(third);
        assert!(take(&log).is_empty(), "closing twice is a no-op");
        // The id can come back: opened again, before its directory.
        host.copy_pane_panels(PaneId::RIGHT, third);
        host.set_pane(pane(third, "/w", false), 1);
        assert_eq!(take(&log), vec!["a:opened", "a:dir"]);
        assert!(
            host.is_panel_open("a", "p", Some(third)),
            "copied from the pane it was split from"
        );
    }

    #[test]
    fn events_are_diffed() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new("a", log.clone()))], tmp.path());
        let take = |log: &Log| std::mem::take(&mut *log.0.lock().unwrap());
        take(&log);
        host.set_pane(pane(PaneId::Left, "/x", true), 1);
        host.set_pane(pane(PaneId::Left, "/x", true), 1);
        assert_eq!(take(&log), vec!["a:opened", "a:dir"]);
        host.set_pane(pane(PaneId::Left, "/x", true), 2);
        assert_eq!(take(&log), vec!["a:listing"]);
        let mut s = pane(PaneId::Left, "/x", true);
        s.selection = vec![VfsPath::new("/x/a.txt")];
        host.set_pane(s, 2);
        assert_eq!(take(&log), vec!["a:selection"]);
        host.set_pane(pane(PaneId::Right, "/y", false), 1);
        take(&log);
        host.set_pane(pane(PaneId::Right, "/y", true), 1);
        assert_eq!(take(&log), vec!["a:active"]);
        host.file_action(FileAction::new(
            1,
            PaneId::Left,
            crate::FileActionKind::Delete,
            vec![VfsPath::new("/x/a.txt")],
        ));
        assert_eq!(take(&log), vec!["a:file_action"]);
    }

    #[test]
    fn locale_changes_reach_plugins() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new("a", log.clone()))], tmp.path());
        let take = |log: &Log| std::mem::take(&mut *log.0.lock().unwrap());
        take(&log);
        host.set_locale("en");
        assert!(take(&log).is_empty(), "same language: no event");
        host.set_locale("de");
        assert_eq!(take(&log), vec!["a:locale"]);
        assert_eq!(crate::locale(), "de");
        host.set_locale("de");
        assert!(take(&log).is_empty());
        host.set_locale("en");
        assert_eq!(take(&log), vec!["a:locale"]);
    }

    #[test]
    fn stale_decorations_are_dropped() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut a = Fake::new("a", log);
        a.badge = Some(("M", Tone::Warning));
        let mut host = host_with(vec![Box::new(a)], tmp.path());
        host.set_pane(pane(PaneId::Left, "/x", true), 1);
        // Navigate before the plugin's answer for /x is handled.
        host.set_pane(pane(PaneId::Left, "/z", true), 1);
        host.poll();
        // The /x answer was ignored, the /z one taken.
        assert_eq!(host.decorations(PaneId::Left).len(), 1);
        host.set_pane(pane(PaneId::Left, "/elsewhere", true), 1);
        // Without a poll, the old decorations are already gone.
        assert!(host.decorations(PaneId::Left).is_empty());
        assert!(host.chips(PaneId::Left).is_empty());
    }

    #[test]
    fn sizes_merge_and_are_dropped_on_navigation() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new("a", log))], tmp.path());
        host.set_pane(pane(PaneId::Left, "/x", true), 1);
        host.poll();
        let tx = host.tx.clone();
        let size = |bytes, complete| SizeInfo {
            bytes,
            complete,
            files: 3,
            ..Default::default()
        };
        let send = |dir: &str, sizes: Vec<(String, SizeInfo)>| {
            tx.send((
                "a".into(),
                HostRequest::SetSizes {
                    pane: PaneId::Left,
                    dir: VfsPath::new(dir),
                    sizes,
                },
            ))
            .unwrap();
        };
        let gen0 = host.sizes_gen(PaneId::Left);
        send(
            "/x",
            vec![
                ("src".into(), size(10, false)),
                ("doc".into(), size(5, true)),
            ],
        );
        host.poll();
        assert_eq!(host.sizes(PaneId::Left).len(), 2);
        assert!(host.sizes_gen(PaneId::Left) > gen0);
        // A partial update merges.
        send("/x", vec![("src".into(), size(42, true))]);
        host.poll();
        assert_eq!(host.sizes(PaneId::Left)["src"], size(42, true));
        assert_eq!(host.sizes(PaneId::Left)["doc"].bytes, 5);
        // Same values again: no new generation.
        let g = host.sizes_gen(PaneId::Left);
        send("/x", vec![("src".into(), size(42, true))]);
        host.poll();
        assert_eq!(host.sizes_gen(PaneId::Left), g);
        // Stale folder: ignored. Navigation drops them.
        send("/elsewhere", vec![("zzz".into(), size(1, true))]);
        host.poll();
        assert!(!host.sizes(PaneId::Left).contains_key("zzz"));
        host.set_pane(pane(PaneId::Left, "/y", true), 1);
        assert!(host.sizes(PaneId::Left).is_empty());
        // Switching the plugin off drops them too.
        send("/y", vec![("a".into(), size(1, true))]);
        host.poll();
        assert_eq!(host.sizes(PaneId::Left).len(), 1);
        host.set_enabled("a", false);
        assert!(host.sizes(PaneId::Left).is_empty());
    }

    #[test]
    fn panicking_plugin_is_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut a = Fake::new("a", log.clone());
        a.panic_on_run = true;
        let mut host = host_with(vec![Box::new(a), Box::new(Fake::new("b", log))], tmp.path());
        host.set_pane(pane(PaneId::Left, "/x", true), 1);
        host.poll();
        host.run_command("a:go", &pane(PaneId::Left, "/x", true));
        let events = host.poll();
        assert!(events.iter().any(|e| matches!(
            e,
            HostEvent::Toast {
                level: ToastLevel::Error,
                ..
            }
        )));
        let rows = host.rows();
        assert!(matches!(&rows[0].status, PluginStatus::Error(m) if m.contains("boom")));
        assert_eq!(rows[1].status, PluginStatus::Running);
        assert!(host.commands().iter().all(|c| c.plugin_id == "b"));
        let m = &host.decorations(PaneId::Left)["a.txt"];
        assert_eq!(m.tooltip.as_deref(), Some("from b"));
    }

    #[test]
    fn enable_state_is_persisted() {
        let tmp = tempfile::tempdir().unwrap();
        let log = Arc::new(Log::default());
        let mut host = host_with(vec![Box::new(Fake::new("a", log.clone()))], tmp.path());
        assert!(host.wants_enabled("a"));
        host.set_enabled("a", false);
        assert_eq!(host.rows()[0].status, PluginStatus::Stopped);
        drop(host);
        let host = host_with(vec![Box::new(Fake::new("a", log))], tmp.path());
        assert!(!host.wants_enabled("a"));
        assert_eq!(host.rows()[0].status, PluginStatus::Stopped);
    }

    const MANIFEST: &str = "id = \"ext\"\nname = \"Ext\"\nversion = \"1.0\"\napi_version = 1\n\
        command = [\"/bin/cat\"]\n";

    #[test]
    fn scans_installs_and_uninstalls_externals() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        std::fs::create_dir_all(paths.plugins_dir.join("broken")).unwrap();
        std::fs::write(paths.plugins_dir.join("broken/plugin.toml"), "id = 3").unwrap();
        let mut host = host_with(Vec::new(), tmp.path());
        let rows = host.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, PluginKind::Invalid);
        assert!(matches!(&rows[0].status, PluginStatus::Error(_)));

        // Install from a source folder.
        let src = tmp.path().join("src-plugin");
        std::fs::create_dir_all(src.join(".git")).unwrap();
        std::fs::write(src.join("plugin.toml"), MANIFEST).unwrap();
        std::fs::write(src.join("main.py"), "print()").unwrap();
        host.install(src.clone(), false);
        let wait = |host: &mut PluginHost| {
            for _ in 0..500 {
                let ev = host.poll();
                if !ev.is_empty() {
                    return ev;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            panic!("no event");
        };
        let ev = wait(&mut host);
        assert!(
            matches!(
                &ev[0],
                HostEvent::Installed {
                    replaced: false,
                    ..
                }
            ),
            "{ev:?}"
        );
        assert!(paths.plugins_dir.join("ext/main.py").exists());
        assert!(!paths.plugins_dir.join("ext/.git").exists());
        let row = host
            .rows()
            .into_iter()
            .find(|r| r.info.id == "ext")
            .unwrap();
        assert_eq!(row.kind, PluginKind::External);
        assert!(!row.enabled, "new external plugins start disabled");
        assert!(!host.is_trusted("ext"));

        // Same id again: asks to replace.
        host.install(src.clone(), false);
        let ev = wait(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::ConfirmReplace { id, .. } if id == "ext"),
            "{ev:?}"
        );
        std::fs::write(src.join("plugin.toml"), MANIFEST.replace("1.0", "2.0")).unwrap();
        host.install(src, true);
        wait(&mut host);
        let row = host
            .rows()
            .into_iter()
            .find(|r| r.info.id == "ext")
            .unwrap();
        assert_eq!(row.info.version, "2.0");

        host.uninstall("ext");
        assert!(host.rows().iter().all(|r| r.info.id != "ext"));
        wait(&mut host);
        assert!(!paths.plugins_dir.join("ext").exists());
    }

    fn wait_events(host: &mut PluginHost) -> Vec<HostEvent> {
        for _ in 0..500 {
            let ev = host.poll();
            if !ev.is_empty() {
                return ev;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no event");
    }

    /// Install from an archive: review first (nothing installed, staging
    /// hidden), then confirm; an update is reviewed with the installed
    /// version; cancelling removes the staging folder.
    #[test]
    fn reviews_archives_before_installing() {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let mut host = host_with(Vec::new(), tmp.path());
        let archive = |version: &str| {
            let path = tmp.path().join(format!("ext-{version}.mantaplugin"));
            let mut z = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
            let opts = zip::write::SimpleFileOptions::default().unix_permissions(0o644);
            z.start_file("ext/plugin.toml", opts).unwrap();
            let manifest = MANIFEST
                .replace("1.0", version)
                .replace("[\"/bin/cat\"]", "[\"./run.sh\"]")
                + "permissions = [\"network\"]\nauthor = \"Anna\"\n";
            z.write_all(manifest.as_bytes()).unwrap();
            // Packed without the executable bit: the installer sets it.
            z.start_file("ext/run.sh", opts).unwrap();
            z.write_all(b"#!/bin/sh\ncat\n").unwrap();
            z.finish().unwrap();
            path
        };
        host.review_install(archive("1.0.0"));
        let ev = wait_events(&mut host);
        let HostEvent::InstallReview(r) = &ev[0] else {
            panic!("{ev:?}")
        };
        assert_eq!(
            (r.id.as_str(), r.version.as_str(), r.author.as_str()),
            ("ext", "1.0.0", "Anna")
        );
        assert_eq!(r.permissions, ["network"]);
        assert_eq!(r.installed_version, None);
        assert!(
            !paths.plugins_dir.join("ext").exists(),
            "nothing installed before the user confirms"
        );
        assert!(
            host.rows().iter().all(|row| row.info.id != "ext"),
            "staging folders are hidden"
        );
        let review = r.clone();
        host.confirm_install(&review);
        let ev = wait_events(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::Installed { id, replaced: false, .. } if id == "ext"),
            "{ev:?}"
        );
        assert!(
            !review.staging.as_ref().unwrap().exists(),
            "staging removed"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(paths.plugins_dir.join("ext/run.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_ne!(mode & 0o100, 0, "the plugin's program is executable");
        }
        assert!(
            !host.wants_enabled("ext") && !host.is_trusted("ext"),
            "starts off and untrusted"
        );
        assert_eq!(host.review("ext").map(|r| r.version), Some("1.0.0".into()));

        // An update shows the installed version; cancel cleans up.
        host.review_install(archive("2.0.0"));
        let ev = wait_events(&mut host);
        let HostEvent::InstallReview(r) = &ev[0] else {
            panic!("{ev:?}")
        };
        assert_eq!(r.installed_version.as_deref(), Some("1.0.0"));
        let staging = r.staging.clone().unwrap();
        host.cancel_install(r);
        for _ in 0..200 {
            if !staging.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!staging.exists());
        assert_eq!(host.review("ext").map(|r| r.version), Some("1.0.0".into()));

        // A broken pick reports why.
        std::fs::write(tmp.path().join("notes.txt"), "x").unwrap();
        host.review_install(tmp.path().join("notes.txt"));
        let ev = wait_events(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::InstallFailed { error } if error.contains(".mantaplugin")),
            "{ev:?}"
        );
    }

    /// A `.mantaplugin` with `ext/plugin.toml` (+ run.sh) of `version`.
    fn ext_archive(version: &str) -> Vec<u8> {
        use std::io::Write;
        let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default().unix_permissions(0o755);
        z.start_file("ext/plugin.toml", opts).unwrap();
        let manifest = MANIFEST
            .replace("1.0", version)
            .replace("[\"/bin/cat\"]", "[\"./run.sh\"]")
            + "permissions = [\"read_files\"]\nauthor = \"Anna\"\n";
        z.write_all(manifest.as_bytes()).unwrap();
        z.start_file("ext/run.sh", opts).unwrap();
        z.write_all(b"#!/bin/sh\ncat\n").unwrap();
        z.finish().unwrap().into_inner()
    }

    fn sha(bytes: &[u8]) -> String {
        use sha2::Digest;
        sha2::Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    fn wait_registry(host: &mut PluginHost) {
        for _ in 0..500 {
            host.poll();
            if !host.discover().loading {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("the directory never arrived");
    }

    /// Discover: nothing is fetched until the tab is opened; installing
    /// downloads, verifies the SHA-256 and goes through the usual review;
    /// a newer version in the directory is offered as an update; the saved
    /// copy is shown offline.
    #[test]
    fn installs_from_the_directory() {
        use crate::registry::tests::{entry_json, registry_json, TestServer};
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let v1 = ext_archive("1.0.0");
        let server = TestServer::start(vec![("/ext-1.0.0.mantaplugin", v1.clone())]);
        let reg = |version: &str, sha: &str| {
            registry_json(vec![entry_json(
                "ext",
                version,
                &server.url(&format!("/ext-{version}.mantaplugin")),
                sha,
            )])
        };
        server.set("/registry.json", reg("1.0.0", &sha(&v1)));

        let mut host = host_with(Vec::new(), tmp.path());
        host.set_registry_url(server.url("/registry.json"));
        host.poll();
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(
            server.hits.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "no network before Discover"
        );
        assert!(host.registry_rows().is_empty());

        host.open_discover();
        assert!(host.discover().loading && host.discover().requested);
        wait_registry(&mut host);
        host.open_discover(); // once per session
        assert!(!host.discover().loading);
        let rows = host.registry_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].name.as_str(), rows[0].installed_version.clone()),
            ("Plugin ext", None)
        );
        assert!(!host.discover().from_cache && host.discover().error.is_none());

        host.install_from_registry("ext");
        assert!(host.registry_rows()[0].downloading);
        let ev = wait_events(&mut host);
        let HostEvent::InstallReview(r) = &ev[0] else {
            panic!("{ev:?}")
        };
        assert_eq!((r.id.as_str(), r.version.as_str()), ("ext", "1.0.0"));
        assert_eq!(r.source, PathBuf::from("ext-1.0.0.mantaplugin"));
        assert!(r.verified_download.is_some());
        if cfg!(target_os = "macos") {
            assert!(r.quarantined, "downloads are quarantined until trusted");
        }
        assert!(!host.registry_rows()[0].downloading);
        assert!(!paths.plugins_dir.join("ext").exists(), "review first");
        let review = r.clone();
        host.confirm_install(&review);
        let ev = wait_events(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::Installed { id, .. } if id == "ext"),
            "{ev:?}"
        );
        assert_eq!(
            host.registry_rows()[0].installed_version.as_deref(),
            Some("1.0.0")
        );
        assert!(host.registry_update("ext").is_none());
        let leftovers: Vec<_> = std::fs::read_dir(&paths.plugins_dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(leftovers, ["ext"], "no downloads or staging left");

        // A newer version in the directory.
        let v2 = ext_archive("2.0.0");
        server.set("/ext-2.0.0.mantaplugin", v2.clone());
        server.set("/registry.json", reg("2.0.0", &sha(&v2)));
        host.refresh_registry();
        wait_registry(&mut host);
        let up = host.registry_update("ext").expect("update offered");
        assert_eq!(up.entry.latest.version, "2.0.0");
        host.install_from_registry("ext");
        let ev = wait_events(&mut host);
        let HostEvent::InstallReview(r) = &ev[0] else {
            panic!("{ev:?}")
        };
        assert_eq!(r.installed_version.as_deref(), Some("1.0.0"));
        host.cancel_install(r);

        // Offline next launch: the saved copy, no network needed.
        drop(server);
        let mut host = host_with(Vec::new(), tmp.path());
        assert!(host.discover().from_cache);
        assert!(
            host.registry_update("ext").is_some(),
            "updates from the saved copy"
        );
        host.set_registry_url("http://127.0.0.1:9/registry.json");
        host.open_discover();
        wait_registry(&mut host);
        assert!(host.discover().from_cache && host.discover().error.is_some());
        assert_eq!(host.registry_rows().len(), 1);
    }

    #[test]
    fn directory_downloads_that_dont_match_are_refused() {
        use crate::registry::tests::{entry_json, registry_json, TestServer};
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let v1 = ext_archive("1.0.0");
        let server = TestServer::start(vec![("/ext.mantaplugin", v1.clone())]);
        let url = server.url("/ext.mantaplugin");
        let mut host = host_with(Vec::new(), tmp.path());
        host.set_registry_url(server.url("/registry.json"));

        // Wrong checksum: refused before anything is unpacked.
        server.set(
            "/registry.json",
            registry_json(vec![entry_json("ext", "1.0.0", &url, &"ab".repeat(32))]),
        );
        host.refresh_registry();
        wait_registry(&mut host);
        host.install_from_registry("ext");
        let ev = wait_events(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::InstallFailed { error } if error.contains("SHA-256 mismatch")),
            "{ev:?}"
        );
        // The archive is fine but isn't the version the directory names.
        server.set(
            "/registry.json",
            registry_json(vec![entry_json("ext", "3.0.0", &url, &sha(&v1))]),
        );
        host.refresh_registry();
        wait_registry(&mut host);
        host.install_from_registry("ext");
        let ev = wait_events(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::InstallFailed { error } if error.contains("doesn't match")),
            "{ev:?}"
        );
        let left = std::fs::read_dir(&paths.plugins_dir)
            .map(|rd| rd.count())
            .unwrap_or(0);
        assert_eq!(left, 0, "nothing installed, nothing left behind");
        assert!(!host.registry_rows()[0].downloading);
    }

    /// External plugins can't ask Manta for what their permissions don't
    /// cover: the request is refused, logged and answered with an error.
    #[test]
    fn host_requests_need_the_declared_permission() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let dir = paths.plugins_dir.join("asker");
        std::fs::create_dir_all(&dir).unwrap();
        let script = r#"echo '{"type":"list_dir","pane":"left","dir":"/"}'
echo '{"type":"run_in_terminal","command":"ls"}'
while read -r line; do
  case "$line" in *dir_listing*) echo "answer: $line" >&2 ;; esac
done"#;
        std::fs::write(
            dir.join("plugin.toml"),
            format!(
                "id = \"asker\"\nname = \"Asker\"\nversion = \"1\"\napi_version = 1\n\
                 command = [\"/bin/sh\", \"-c\", {}]\n",
                toml::Value::String(script.into())
            ),
        )
        .unwrap();
        let mut host = host_with(Vec::new(), tmp.path());
        host.set_enabled("asker", true);
        let mut events = Vec::new();
        for _ in 0..500 {
            events.extend(host.poll());
            let log = host.rows()[0].log.join("\n");
            if log.contains("answer:") && log.contains("refused run_in_terminal") {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let row = &host.rows()[0];
        let log = row.log.join("\n");
        assert!(
            log.contains(
                "refused list_dir: the plugin doesn't declare the \"read_files\" permission"
            ),
            "{log}"
        );
        assert!(log.contains("not allowed"), "answered with an error: {log}");
        assert!(log.contains("refused run_in_terminal"), "{log}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, HostEvent::ListDir { .. })),
            "{events:?}"
        );
        assert!(row.sandbox.is_some());
        host.shutdown(Duration::from_secs(2));
    }

    /// `--plugin-dev`: runs from its folder, trusted and on for the
    /// session only, shadows an installed copy, can't be uninstalled.
    #[test]
    fn dev_plugins_run_from_their_folder_for_one_session() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        std::fs::create_dir_all(paths.plugins_dir.join("ext")).unwrap();
        std::fs::write(paths.plugins_dir.join("ext/plugin.toml"), MANIFEST).unwrap();
        let dev = tmp.path().join("work/ext");
        std::fs::create_dir_all(&dev).unwrap();
        std::fs::write(dev.join("plugin.toml"), MANIFEST.replace("1.0", "1.1-dev")).unwrap();
        let mut host = host_with(Vec::new(), tmp.path());
        assert!(!host.wants_enabled("ext"));
        assert_eq!(host.add_dev_plugin(&dev).unwrap(), "ext");
        let rows: Vec<PluginRow> = host
            .rows()
            .into_iter()
            .filter(|r| r.info.id == "ext")
            .collect();
        assert_eq!(rows.len(), 1, "the installed copy steps aside");
        assert!(rows[0].dev && rows[0].enabled && rows[0].info.version == "1.1-dev");
        assert!(host.is_trusted("ext"));
        assert_ne!(rows[0].status, PluginStatus::Stopped);
        host.set_enabled("ext", false);
        assert!(!host.wants_enabled("ext"));
        host.uninstall("ext");
        assert!(
            dev.exists() && host.rows().iter().any(|r| r.info.id == "ext"),
            "dev folders are never deleted"
        );
        host.reload_all();
        assert!(host.rows().iter().any(|r| r.info.id == "ext" && r.dev));
        drop(host);
        // Nothing was saved: next launch is the installed copy, off.
        let state = std::fs::read_to_string(&paths.state_file).unwrap_or_default();
        assert!(!state.contains("\"ext\""), "{state}");
        let mut host = host_with(Vec::new(), tmp.path());
        let row = host
            .rows()
            .into_iter()
            .find(|r| r.info.id == "ext")
            .unwrap();
        assert!(!row.dev && !row.enabled && row.info.version == "1.0");
        assert!(host.add_dev_plugin(&tmp.path().join("missing")).is_err());
    }

    #[test]
    fn installs_and_updates_bundled_plugins() {
        use crate::bundled::BundledFile;
        use std::borrow::Cow;
        let bundle = |version: &str| {
            BundledPlugin::from_files(
                "ext",
                vec![
                    BundledFile {
                        path: "plugin.toml".into(),
                        bytes: Cow::Owned(MANIFEST.replace("1.0", version).into_bytes()),
                        executable: false,
                    },
                    BundledFile {
                        path: "bin/run.sh".into(),
                        bytes: Cow::Borrowed(b"#!/bin/sh\ncat\n"),
                        executable: true,
                    },
                ],
            )
            .unwrap()
        };
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let mut host = host_with(Vec::new(), tmp.path());
        host.set_bundled(vec![bundle("1.0.0")]);
        let rows = host.bundled_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (
                rows[0].installed_version.as_deref(),
                rows[0].update_available
            ),
            (None, false)
        );

        let wait = |host: &mut PluginHost| {
            for _ in 0..500 {
                let ev = host.poll();
                if !ev.is_empty() {
                    return ev;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            panic!("no event");
        };
        host.install_bundled("ext");
        assert!(host.bundled_rows()[0].installing);
        let ev = wait(&mut host);
        assert!(
            matches!(
                &ev[0],
                HostEvent::Installed {
                    replaced: false,
                    ..
                }
            ),
            "{ev:?}"
        );
        let row = &host.bundled_rows()[0];
        assert_eq!(row.installed_version.as_deref(), Some("1.0.0"));
        assert!(!row.installing && !row.update_available);
        assert!(
            !host.wants_enabled("ext") && !host.is_trusted("ext"),
            "installed switched off, untrusted"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(paths.plugins_dir.join("ext/bin/run.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }
        assert!(std::fs::read_dir(&paths.plugins_dir)
            .unwrap()
            .flatten()
            .all(|e| !e.file_name().to_string_lossy().starts_with('.')));

        // A newer bundled version: offered as an update, keeps the on state.
        host.trust("ext");
        host.set_enabled("ext", true);
        host.set_bundled(vec![bundle("1.2.0")]);
        assert!(host.bundled_rows()[0].update_available);
        host.install_bundled("ext");
        let ev = wait(&mut host);
        assert!(
            matches!(&ev[0], HostEvent::Installed { replaced: true, .. }),
            "{ev:?}"
        );
        let row = &host.bundled_rows()[0];
        assert_eq!(row.installed_version.as_deref(), Some("1.2.0"));
        assert!(!row.update_available);
        assert!(host.wants_enabled("ext"));
        assert!(host
            .rows()
            .iter()
            .any(|r| r.info.id == "ext" && r.status != PluginStatus::Stopped));
        host.shutdown(Duration::from_secs(1));
    }

    /// An installed external plugin a built-in replaced: switched off at
    /// startup (files kept), one toast ever, can't be switched on, marked
    /// in the rows; its data is copied to the built-in's empty data dir.
    #[test]
    fn retired_externals_are_switched_off_once_and_their_data_moves() {
        let (old, new) = RETIRED_EXTERNAL[0];
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let dir = paths.plugins_dir.join(old);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plugin.toml"),
            MANIFEST.replace("\"ext\"", &format!("\"{old}\"")),
        )
        .unwrap();
        // The user had it on, with their own data.
        let state = serde_json::json!({ "enabled": { old: true }, "trusted": [old] });
        std::fs::create_dir_all(&paths.data_root).unwrap();
        std::fs::write(&paths.state_file, state.to_string()).unwrap();
        let old_data = paths.data_root.join(old);
        std::fs::create_dir_all(old_data.join("cache")).unwrap();
        std::fs::write(old_data.join("actions.json"), "{\"mine\": true}").unwrap();
        std::fs::write(old_data.join("cache/x.bin"), "x").unwrap();
        std::fs::write(old_data.join("plugin.log"), "log").unwrap();
        let log = Arc::new(Log::default());
        let builtin = |log: &Arc<Log>| -> Vec<Box<dyn Plugin>> {
            let new: &'static str = RETIRED_EXTERNAL[0].1;
            vec![Box::new(Fake::new(new, log.clone()))]
        };

        let mut host = host_with(builtin(&log), tmp.path());
        let toasts = host.poll();
        assert_eq!(toasts.len(), 1, "{toasts:?}");
        assert!(
            matches!(&toasts[0], HostEvent::Toast { level: ToastLevel::Info, text } if text.contains("is now built in")),
            "{toasts:?}"
        );
        let row = host.rows().into_iter().find(|r| r.info.id == old).unwrap();
        assert_eq!(row.replaced_by.as_deref(), Some(new));
        assert!(!row.enabled && row.status == PluginStatus::Stopped);
        assert!(dir.join("plugin.toml").exists(), "not deleted");
        assert!(host
            .rows()
            .iter()
            .all(|r| r.info.id == old || r.replaced_by.is_none()));
        host.set_enabled(old, true);
        assert!(!host.wants_enabled(old), "can't be switched on again");
        // The built-in got the user's files (not the external's log).
        let new_data = paths.data_root.join(new);
        assert_eq!(
            std::fs::read_to_string(new_data.join("actions.json")).unwrap(),
            "{\"mine\": true}"
        );
        assert!(new_data.join("cache/x.bin").exists());
        assert!(!new_data.join("plugin.log").exists());
        drop(host);

        // Next launch: no second toast; the built-in's own data is kept.
        std::fs::write(new_data.join("actions.json"), "edited").unwrap();
        let mut host = host_with(builtin(&log), tmp.path());
        assert!(host.poll().is_empty(), "the toast is shown once");
        assert_eq!(
            std::fs::read_to_string(new_data.join("actions.json")).unwrap(),
            "edited"
        );
        assert!(!host.wants_enabled(old));

        // Without the built-in (e.g. an old build) nothing is retired.
        let host = host_with(Vec::new(), tmp.path());
        assert!(host.rows().iter().all(|r| r.replaced_by.is_none()));
    }

    #[test]
    fn retired_toast_is_german_in_german() {
        let (old, new) = RETIRED_EXTERNAL[1];
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        let dir = paths.plugins_dir.join(old);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plugin.toml"),
            MANIFEST.replace("\"ext\"", &format!("\"{old}\"")),
        )
        .unwrap();
        let log = Arc::new(Log::default());
        let toasts = crate::i18n::with_lang(crate::i18n::Lang::De, || {
            let mut host = host_with(vec![Box::new(Fake::new(new, log.clone()))], tmp.path());
            host.poll()
        });
        let want = format!("„{}“ ist jetzt eingebaut", new.to_uppercase());
        assert!(
            matches!(&toasts[..], [HostEvent::Toast { text, .. }] if *text == want),
            "{toasts:?}"
        );
    }

    /// A built-in that draws with blocks: the host hands them out like an
    /// external plugin's and routes the clicks back.
    #[test]
    fn builtin_panel_blocks_and_actions() {
        struct Blocky(Arc<Mutex<Vec<String>>>);
        impl Plugin for Blocky {
            fn info(&self) -> PluginInfo {
                PluginInfo {
                    id: "blocky".into(),
                    name: "Blocky".into(),
                    version: "1".into(),
                    description: String::new(),
                    author: String::new(),
                    builtin: true,
                }
            }
            fn enable(&mut self, _h: PluginHandle) {}
            fn disable(&mut self) {}
            fn panels(&self) -> Vec<PanelSpec> {
                vec![PanelSpec {
                    id: "p".into(),
                    title: "P".into(),
                    placement: PanelPlacement::RightSidebar,
                    default_open: false,
                }]
            }
            fn panel_blocks(&self, panel: &str, _pane: Option<PaneId>) -> Option<Arc<Vec<Block>>> {
                (panel == "p").then(|| Arc::new(vec![Block::Heading { text: "Hi".into() }]))
            }
            fn panel_action(&mut self, panel: &str, action: &str, _pane: &PaneState) {
                self.0.lock().unwrap().push(format!("{panel}:{action}"));
            }
        }
        let seen = Arc::new(Mutex::new(Vec::new()));
        let tmp = tempfile::tempdir().unwrap();
        let mut host = host_with(vec![Box::new(Blocky(seen.clone()))], tmp.path());
        assert!(
            matches!(host.panel_content("blocky", "p", None), PanelContent::Blocks(b) if b.len() == 1)
        );
        assert!(matches!(
            host.panel_content("blocky", "other", None),
            PanelContent::Native
        ));
        host.panel_action("blocky", "p", "go", &pane(PaneId::Left, "/tmp", true));
        assert_eq!(*seen.lock().unwrap(), vec!["p:go".to_string()]);
    }

    #[test]
    fn duplicate_external_ids_are_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = HostPaths::under(tmp.path());
        for f in ["one", "two"] {
            std::fs::create_dir_all(paths.plugins_dir.join(f)).unwrap();
            std::fs::write(paths.plugins_dir.join(f).join("plugin.toml"), MANIFEST).unwrap();
        }
        let host = host_with(Vec::new(), tmp.path());
        let rows = host.rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter()
                .filter(|r| r.kind == PluginKind::Invalid)
                .count(),
            1
        );
    }
}
