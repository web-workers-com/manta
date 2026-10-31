//! A file pane: tabs, each tab showing one directory of one backend.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, CornerRadius, Sense, Stroke, StrokeKind};
use manta_core::{Entry, SharedVfs, VfsPath};
use manta_plugin::host::MergedDecoration;
use manta_plugin::{Chip, SizeInfo};

use crate::app::{Action, Side};
use crate::commands::Cmd;
use crate::icons;
use crate::plugin_ui;
use crate::search::{filter_changed, scope_bar_hold_id};
use crate::theme::{self, pal, radius, size, space, text, Palette};
use crate::util::{self, SortColumn, SortSpec};
use crate::widgets::{self, ButtonKind};

/// Where a tab points to: backend + directory.
#[derive(Clone)]
pub struct Location {
    pub fs: SharedVfs,
    /// Connection id for remote backends (`None` = local). Inside an
    /// archive: the connection of the folder the archive is in.
    pub conn: Option<u64>,
    pub path: VfsPath,
    /// Set when `fs` is an archive opened as a folder.
    pub archive: Option<Arc<ArchiveMount>>,
}

impl Location {
    pub fn new(fs: SharedVfs, conn: Option<u64>, path: VfsPath) -> Self {
        Location {
            fs,
            conn,
            path,
            archive: None,
        }
    }

    /// The same backend (and archive) at another path.
    pub fn with_path(&self, path: VfsPath) -> Self {
        Location {
            path,
            ..self.clone()
        }
    }

    /// Nothing can be written here (an archive).
    pub fn read_only(&self) -> bool {
        self.fs.is_read_only()
    }

    /// The location outside all archives (the real folder, local or remote).
    pub fn outermost(&self) -> &Location {
        match &self.archive {
            Some(m) => m.outer.outermost(),
            None => self,
        }
    }
}

/// An archive opened as a folder: where its file is.
pub struct ArchiveMount {
    /// The folder that contains the archive file (itself possibly inside
    /// another archive).
    pub outer: Location,
    /// The archive file's name in `outer.path`.
    pub file_name: String,
    /// e.g. "zip", "tar.gz".
    pub format: String,
}

impl ArchiveMount {
    /// The archive file's path in the outer backend.
    pub fn file_path(&self) -> VfsPath {
        self.outer.path.join(&self.file_name)
    }
}

/// An archive being downloaded / indexed for a tab (shown in the pane).
pub struct ArchiveOpening {
    pub id: u64,
    pub name: String,
    /// Bytes downloaded or indexed so far.
    pub done: Arc<AtomicU64>,
    /// Size of the archive file (0 = unknown).
    pub total: u64,
    /// Downloading (remote or nested archive) rather than indexing.
    pub downloading: Arc<AtomicBool>,
    pub cancel: Arc<AtomicBool>,
    pub since: Instant,
}

/// One breadcrumb segment.
#[derive(Clone)]
pub struct Crumb {
    pub label: String,
    /// The archive file itself (drawn with an archive icon).
    pub archive: bool,
    pub target: CrumbTarget,
}

#[derive(Clone)]
pub enum CrumbTarget {
    /// A folder in the tab's current backend.
    Here(VfsPath),
    /// A folder outside the current archive (select the name, if any).
    Outer(Location, Option<String>),
}

/// Folders shown under another name in the breadcrumb and tab title, like
/// in Finder: the user's Trash (`~/.Trash`) as „Papierkorb“ / "Trash",
/// iCloud Drive's top folder as "iCloud Drive" and the apps' folders in
/// it by their app's name („Obsidian“).
fn folder_label(conn: Option<u64>, archive: bool, path: &VfsPath) -> Option<String> {
    if conn.is_some() || archive {
        return None;
    }
    let p = std::path::Path::new(path.as_str());
    if dirs::home_dir().is_some_and(|h| p == h.join(".Trash")) {
        return Some(t!("sidebar.trash").to_string());
    }
    manta_vfs::icloud::folder_label(p)
}

/// The breadcrumb of a location: `… › Downloads › backup.zip › docs`.
pub fn crumbs(loc: &Location) -> Vec<Crumb> {
    let mut out = Vec::new();
    if let Some(m) = &loc.archive {
        let outer = crumbs(&m.outer);
        let n = outer.len();
        for (i, c) in outer.into_iter().enumerate() {
            let target = match c.target {
                CrumbTarget::Here(p) => {
                    let select = (i + 1 == n).then(|| m.file_name.clone());
                    CrumbTarget::Outer(m.outer.with_path(p), select)
                }
                other => other,
            };
            out.push(Crumb { target, ..c });
        }
        out.push(Crumb {
            label: m.file_name.clone(),
            archive: true,
            target: CrumbTarget::Here(VfsPath::root()),
        });
    }
    // Inside iCloud Drive the location chip stands for
    // `~/Library/Mobile Documents/com~apple~CloudDocs`: start after it.
    let skip = (loc.conn.is_none() && loc.archive.is_none())
        .then(|| manta_vfs::icloud::crumb_base(std::path::Path::new(loc.path.as_str())))
        .flatten()
        .map_or(0, |base| base.components().count() - 1);
    let mut acc = VfsPath::root();
    for (i, c) in loc.path.components().into_iter().enumerate() {
        acc = acc.join(c);
        if i < skip {
            continue;
        }
        let label =
            folder_label(loc.conn, loc.archive.is_some(), &acc).unwrap_or_else(|| c.to_string());
        out.push(Crumb {
            label,
            archive: false,
            target: CrumbTarget::Here(acc.clone()),
        });
    }
    out
}

/// Payload carried while dragging files between panes (and onto the
/// sidebar's favorites).
pub struct DragPayload {
    pub from: Side,
    pub fs: SharedVfs,
    pub paths: Vec<VfsPath>,
    /// Per path: is it a folder (only folders can become favorites).
    pub dirs: Vec<bool>,
    /// The connection the items are on (`None` = this Mac).
    pub conn: Option<u64>,
    /// The items are inside an archive (they can't become favorites).
    pub archive: bool,
}

impl DragPayload {
    /// Plain files from `from` (tests; real drags come from [`Self::of_tab`]).
    #[cfg(test)]
    pub fn files(from: Side, fs: SharedVfs, paths: Vec<VfsPath>) -> Self {
        let dirs = vec![false; paths.len()];
        DragPayload {
            from,
            fs,
            paths,
            dirs,
            conn: None,
            archive: false,
        }
    }

    /// The selected entries of `tab`.
    pub fn of_tab(from: Side, tab: &Tab) -> Option<Self> {
        let sel = tab.selected_entries();
        if sel.is_empty() {
            return None;
        }
        Some(DragPayload {
            from,
            fs: tab.loc.fs.clone(),
            dirs: sel.iter().map(|e| e.is_dir()).collect(),
            paths: sel.into_iter().map(|e| e.path).collect(),
            conn: tab.loc.conn,
            archive: tab.loc.archive.is_some(),
        })
    }

    /// The dragged folders (what the favorites accept).
    pub fn folders(&self) -> Vec<VfsPath> {
        if self.archive {
            return Vec::new();
        }
        self.paths
            .iter()
            .zip(&self.dirs)
            .filter(|(_, d)| **d)
            .map(|(p, _)| p.clone())
            .collect()
    }
}

/// The selected entry names of a tab (names are unique within a folder).
///
/// Reads go through `Deref` to the set; every change goes through a method
/// that gives the selection a new generation, so what is derived from it
/// (status bar size, menus, plugin state) is computed once per change
/// instead of once per frame — with 300,000 selected items that is the
/// difference between 2 and 200 ms a frame.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    names: BTreeSet<String>,
    gen: u64,
}

/// Generations are unique across all selections (a new selection never
/// looks like an old one).
fn next_selection_gen() -> u64 {
    static GEN: AtomicU64 = AtomicU64::new(1);
    GEN.fetch_add(1, Ordering::Relaxed)
}

impl std::ops::Deref for Selection {
    type Target = BTreeSet<String>;
    fn deref(&self) -> &BTreeSet<String> {
        &self.names
    }
}

impl From<BTreeSet<String>> for Selection {
    fn from(names: BTreeSet<String>) -> Self {
        Selection {
            names,
            gen: next_selection_gen(),
        }
    }
}

impl FromIterator<String> for Selection {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        BTreeSet::from_iter(iter).into()
    }
}

impl Selection {
    /// Changes whenever the selection changes.
    pub fn gen(&self) -> u64 {
        self.gen
    }

    /// The names as a plain set.
    pub fn names(&self) -> &BTreeSet<String> {
        &self.names
    }

    fn touch(&mut self) {
        self.gen = next_selection_gen();
    }

    pub fn clear(&mut self) {
        if !self.names.is_empty() {
            self.names.clear();
            self.touch();
        }
    }

    pub fn insert(&mut self, name: String) -> bool {
        let new = self.names.insert(name);
        if new {
            self.touch();
        }
        new
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let had = self.names.remove(name);
        if had {
            self.touch();
        }
        had
    }

    pub fn extend<I: IntoIterator<Item = String>>(&mut self, names: I) {
        self.names.extend(names);
        self.touch();
    }

    pub fn retain(&mut self, keep: impl FnMut(&String) -> bool) {
        let before = self.names.len();
        self.names.retain(keep);
        if self.names.len() != before {
            self.touch();
        }
    }
}

/// Frees a replaced listing: a big one on a background thread (freeing
/// 300,000 entries took tens of milliseconds of a frame).
fn drop_later(old: Vec<Entry>) {
    if old.len() < 10_000 {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("manta-free-listing".into())
        .spawn(move || drop(old));
}

/// What is derived from a tab's selection (see [`Tab::selection_summary`]).
#[derive(Debug, Default)]
pub struct SelectionSummary {
    /// View indices of the selected entries, in view order.
    pub view_idx: Vec<usize>,
    /// Bytes of the selected entries (folders once their size is known).
    pub size: u64,
    /// One of them is an archive file (Extract Here).
    pub any_archive: bool,
}

/// A file that Manta opens as an archive.
pub fn is_archive_file(e: &Entry) -> bool {
    !e.is_dir() && manta_vfs::ArchiveFormat::detect(&e.name).is_some()
}

/// The key a [`SelectionSummary`] was computed for: selection, listing,
/// view order and folder sizes.
type SummaryKey = (u64, u64, u64, Option<u64>);

/// [`Tab::tag_menu`]: the menu, for the summary key, the folder's tags
/// and the known tags it was made from.
type TagMenuCache = (
    SummaryKey,
    Option<std::sync::Arc<crate::tags::FolderTags>>,
    std::sync::Arc<crate::tags::KnownTags>,
    Option<crate::tags::TagMenu>,
);

/// How long after the second click on the selected item its name becomes
/// editable; longer than a double-click, so that one still opens the item.
pub const RENAME_CLICK_DELAY: std::time::Duration = std::time::Duration::from_millis(600);

/// A name edited in place in the List view (click-to-rename).
pub struct InlineRename {
    pub form: crate::dialogs::NameForm,
    /// Focus and the selection of the stem are set up.
    pub started: bool,
}

impl InlineRename {
    pub fn new(form: crate::dialogs::NameForm) -> Self {
        InlineRename {
            form,
            started: false,
        }
    }
}

pub struct Tab {
    pub id: u64,
    pub loc: Location,
    /// The listing (sorted by `sort` when it was listed; `view` is what is
    /// shown, filtered and sorted).
    pub entries: Vec<Entry>,
    /// Indices into `entries`, filtered and sorted.
    pub view: Vec<usize>,
    /// Selected entry names (names are unique within a directory).
    pub selected: Selection,
    /// Bumped whenever `view` is rebuilt (sort, filter, listing).
    pub view_gen: u64,
    /// The selection summary and what it was computed for.
    sel_summary: std::cell::RefCell<Option<(SummaryKey, std::rc::Rc<SelectionSummary>)>>,
    /// The Tags menu for the selection and what it was made for (the menus
    /// are built every frame).
    tag_menu: std::cell::RefCell<Option<TagMenuCache>>,
    /// Anchor for Shift-click range selection (index into `view`).
    pub anchor: Option<usize>,
    /// Keyboard cursor (index into `view`).
    pub cursor: Option<usize>,
    pub back: Vec<Location>,
    pub forward: Vec<Location>,
    pub loading: bool,
    /// When the current listing started (spinner only after a short delay).
    pub loading_since: Option<Instant>,
    /// Scroll position / viewport of the file list in the last frame.
    pub scroll_offset: f32,
    pub viewport_h: f32,
    /// Generation counter to drop stale listing results.
    pub gen: u64,
    pub error: Option<String>,
    pub sort: SortSpec,
    pub show_hidden: bool,
    pub filter: String,
    /// The filter field is open in a pane too narrow to show it all the
    /// time (it's a magnifier button there); closes again once it's empty
    /// and unfocused.
    pub filter_open: bool,
    /// Focus the filter field once it is drawn (⌘F, the magnifier).
    pub filter_focus: bool,
    /// `Some` while the path field is in text-edit mode.
    pub path_edit: Option<String>,
    pub scroll_to_cursor: bool,
    /// Height egui gave the scrolled rows last frame (should be rows × ROW).
    pub content_h: f32,
    /// Type-to-select: the letters typed so far and when the last one came.
    pub type_ahead: String,
    pub type_ahead_at: Option<std::time::Instant>,
    /// Click-to-rename (Finder): a plain click on the item that already was
    /// the only selection, and when. [`RENAME_CLICK_DELAY`] later its name
    /// becomes editable, unless a double-click, a drag or another click came.
    pub rename_click: Option<(String, std::time::Instant)>,
    /// The name being edited in place (List view).
    pub inline_rename: Option<InlineRename>,
    /// "Start Container" was clicked on the "not running" page; the button
    /// reads "Starting…" until `docker start` is done.
    pub container_starting: bool,
    /// Name to select once the next listing arrives.
    pub pending_select: Option<String>,
    /// Names to select (all of them) once the next listing arrives
    /// (a plugin's `select` request).
    pub pending_select_many: Vec<String>,
    /// Bumped whenever a listing arrives (plugins get `ListingRefreshed`).
    pub listing_gen: u64,
    /// Folder sizes measured by plugins, by entry name.
    pub dir_sizes: HashMap<String, SizeInfo>,
    /// The host's sizes generation `dir_sizes` was copied at.
    pub sizes_gen: Option<u64>,
    /// An archive being opened in this tab.
    pub opening: Option<ArchiveOpening>,
    /// Names of encrypted archive entries in the listing (lock badge).
    pub locked: HashSet<String>,
    /// Select the first entry once the next listing arrives (→ in the
    /// Columns view).
    pub pending_select_first: bool,
    /// State of the Icons, Columns and Gallery views.
    pub view_state: crate::views::ViewState,
    /// When the newest listing was started (live refresh skips events
    /// older than that).
    pub listed_at: Option<Instant>,
    /// Sync states of the items (by name) when the folder is in a cloud
    /// folder (iCloud Drive, `~/Library/CloudStorage`); set by the app.
    pub cloud: Option<std::sync::Arc<crate::cloud::FolderSync>>,
    /// Finder tags of the items (by name): read in the background for
    /// local folders (see [`crate::tags::TagStore`]), or the search
    /// results' tags. Set by the app.
    pub tags: Option<std::sync::Arc<crate::tags::FolderTags>>,
    /// The `listing_gen` the tags were read for (a new listing reads them
    /// again).
    pub tags_listing: u64,
    /// The scope bar's choice: This Folder (the filter), Subfolders or
    /// This Mac (a search).
    pub scope: crate::search::Scope,
    /// The search shown instead of the folder's items (a virtual location).
    pub search: Option<crate::search::Search>,
    /// When the typed query (re)starts the search (typing is debounced).
    pub search_due: Option<Instant>,
    /// Live refresh saw changes while the tab was in the background; it is
    /// reloaded (quietly) as soon as it is shown again. The time of the
    /// newest change.
    pub stale: Option<Instant>,
}

impl Tab {
    pub fn new(id: u64, loc: Location) -> Self {
        Tab {
            id,
            loc,
            entries: Vec::new(),
            view: Vec::new(),
            selected: Selection::default(),
            view_gen: 0,
            sel_summary: Default::default(),
            tag_menu: Default::default(),
            anchor: None,
            cursor: None,
            back: Vec::new(),
            forward: Vec::new(),
            loading: false,
            loading_since: None,
            scroll_offset: 0.0,
            viewport_h: 0.0,
            gen: 0,
            error: None,
            sort: SortSpec::default(),
            show_hidden: false,
            filter: String::new(),
            filter_open: false,
            filter_focus: false,
            path_edit: None,
            scroll_to_cursor: false,
            content_h: 0.0,
            type_ahead: String::new(),
            type_ahead_at: None,
            rename_click: None,
            inline_rename: None,
            container_starting: false,
            pending_select: None,
            pending_select_many: Vec::new(),
            listing_gen: 0,
            dir_sizes: HashMap::new(),
            sizes_gen: None,
            opening: None,
            locked: HashSet::new(),
            pending_select_first: false,
            listed_at: None,
            cloud: None,
            tags: None,
            tags_listing: 0,
            scope: crate::search::Scope::Folder,
            search: None,
            search_due: None,
            stale: None,
            view_state: Default::default(),
        }
    }

    /// The pane shows search results (a virtual location).
    pub fn in_search(&self) -> bool {
        self.search.is_some()
    }

    /// The tags of the entry named `name`.
    /// The Tags menu for the selection (`None` where items can't be
    /// tagged or nothing is selected); made again only when the selection,
    /// the listing, the items' tags or the known tags change.
    pub fn tag_menu(&self, ctx: &egui::Context) -> Option<crate::tags::TagMenu> {
        if !crate::tags::taggable(&self.loc) {
            return None;
        }
        let known = crate::tags::known(ctx);
        let key = (
            self.selected.gen(),
            self.listing_gen,
            self.view_gen,
            self.sizes_gen,
        );
        let same_tags = |a: &Option<std::sync::Arc<crate::tags::FolderTags>>| match (a, &self.tags)
        {
            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if let Some((k, t, kn, menu)) = &*self.tag_menu.borrow() {
            if *k == key && same_tags(t) && **kn == *known {
                return menu.clone();
            }
        }
        let sum = self.selection_summary();
        let sel: Vec<&Entry> = if sum.view_idx.is_empty() {
            self.cursor_entry().into_iter().collect()
        } else {
            sum.view_idx
                .iter()
                .filter_map(|&vi| self.entry_at(vi))
                .collect()
        };
        let menu = (!sel.is_empty()).then(|| {
            let selection: Vec<&[crate::tags::Tag]> =
                sel.iter().map(|e| self.tags_of(&e.name)).collect();
            crate::tags::TagMenu::new(&known, &selection)
        });
        *self.tag_menu.borrow_mut() = Some((key, self.tags.clone(), known, menu.clone()));
        menu
    }

    pub fn tags_of(&self, name: &str) -> &[crate::tags::Tag] {
        self.tags
            .as_deref()
            .and_then(|t| t.get(name))
            .map_or(&[], |v| v.as_slice())
    }

    /// Show search results from now on: the folder's items go, the results
    /// come in as they are found ([`Self::add_results`]).
    pub fn begin_search(&mut self, search: crate::search::Search) {
        // A listing still on its way is stale now.
        self.gen += 1;
        self.loading = false;
        self.loading_since = None;
        self.error = None;
        self.entries.clear();
        self.view.clear();
        self.selected.clear();
        self.cursor = None;
        self.anchor = None;
        self.scroll_offset = 0.0;
        self.dir_sizes.clear();
        self.tags = Some(search.tags.clone());
        self.search = Some(search);
        self.listing_gen += 1;
    }

    /// Results that arrived: add them, keep the selection.
    pub fn add_results(&mut self, hits: Vec<crate::search::Hit>) {
        if hits.is_empty() {
            return;
        }
        self.entries.extend(hits.into_iter().map(|h| h.entry));
        if let Some(s) = &self.search {
            self.tags = Some(s.tags.clone());
        }
        self.listing_gen += 1;
        self.rebuild_view();
    }

    /// Leave the search (the caller lists the folder again).
    pub fn end_search(&mut self) {
        if let Some(mut s) = self.search.take() {
            s.stop();
        }
        self.search_due = None;
        self.tags = None;
        self.tags_listing = 0;
        self.entries.clear();
        self.view.clear();
        self.selected.clear();
        self.cursor = None;
        self.anchor = None;
    }

    /// New tab at the same location with the same view settings.
    pub fn duplicate(&self, id: u64) -> Self {
        let mut t = Tab::new(id, self.loc.clone());
        t.sort = self.sort;
        t.show_hidden = self.show_hidden;
        t
    }

    pub fn title(&self) -> String {
        if let Some(s) = &self.search {
            return s.query.title();
        }
        if let Some(t) = folder_label(self.loc.conn, self.loc.archive.is_some(), &self.loc.path) {
            return t;
        }
        let name = self.loc.path.file_name();
        if let (true, Some(m)) = (name.is_empty(), &self.loc.archive) {
            return m.file_name.clone();
        }
        if name.is_empty() {
            "/".to_string()
        } else {
            name.to_string()
        }
    }

    pub fn begin_loading(&mut self) {
        self.loading = true;
        self.loading_since = Some(Instant::now());
    }

    /// Entries in view order. Never panics, even if `view` is stale.
    fn view_entries(&self) -> impl Iterator<Item = &Entry> {
        self.view.iter().filter_map(|&i| self.entries.get(i))
    }

    /// Re-filter and re-sort after a sort/filter/hidden change. Cursor and
    /// anchor follow their entries by name.
    pub fn rebuild_view(&mut self) {
        let cursor_name = self.cursor_entry().map(|e| e.name.clone());
        let anchor_name = self
            .anchor
            .and_then(|a| self.entry_at(a))
            .map(|e| e.name.clone());
        self.rebuild_view_keeping(cursor_name, anchor_name);
    }

    /// Rebuild `view` from `entries`; `cursor`/`anchor` are names captured
    /// *before* `entries` changed (the old `view` indices may be invalid).
    fn rebuild_view_keeping(&mut self, cursor_name: Option<String>, anchor_name: Option<String>) {
        // Search results already match the query (it is the filter text).
        let filter = if self.in_search() { "" } else { &self.filter };
        self.view = util::visible_indices_tagged(
            &self.entries,
            self.sort,
            self.show_hidden,
            filter,
            &self.dir_sizes,
            self.tags.as_deref(),
        );
        self.view_gen += 1;
        // Selected items that aren't shown any more leave the selection
        // (one pass over the view; hash lookups for big selections).
        if !self.selected.is_empty() {
            let big: Option<HashSet<&str>> = (self.selected.len() > 16)
                .then(|| self.selected.iter().map(String::as_str).collect());
            let shown: Vec<&str> = self
                .view_entries()
                .map(|e| e.name.as_str())
                .filter(|n| match &big {
                    Some(set) => set.contains(n),
                    None => self.selected.contains(*n),
                })
                .collect();
            if shown.len() != self.selected.len() {
                let keep: HashSet<String> = shown.into_iter().map(str::to_string).collect();
                self.selected.retain(|n| keep.contains(n));
            }
        }
        self.cursor = cursor_name.and_then(|n| self.view_index_of(&n));
        self.anchor = anchor_name.and_then(|n| self.view_index_of(&n));
    }

    pub fn view_index_of(&self, name: &str) -> Option<usize> {
        self.view
            .iter()
            .position(|&i| self.entries.get(i).is_some_and(|e| e.name == name))
    }

    pub fn entry_at(&self, view_idx: usize) -> Option<&Entry> {
        self.view.get(view_idx).and_then(|&i| self.entries.get(i))
    }

    pub fn cursor_entry(&self) -> Option<&Entry> {
        self.cursor.and_then(|c| self.entry_at(c))
    }

    /// The part of a folder read so far, while the listing goes on (a very
    /// large folder shows its first items at once); the complete listing
    /// replaces it through [`Tab::apply_listing`] and keeps the cursor and
    /// selection made meanwhile.
    pub fn apply_partial(&mut self, entries: Vec<Entry>) {
        let cursor_name = self.cursor_entry().map(|e| e.name.clone());
        let anchor_name = self
            .anchor
            .and_then(|a| self.entry_at(a))
            .map(|e| e.name.clone());
        drop_later(std::mem::replace(&mut self.entries, entries));
        self.view.clear();
        self.rebuild_view_keeping(cursor_name, anchor_name);
    }

    /// Replace the listing (keeps selection by name where possible).
    pub fn apply_listing(&mut self, path: VfsPath, entries: Vec<Entry>) {
        let same_dir = path == self.loc.path;
        // Capture names while `view` still matches the old `entries`.
        let (cursor_name, anchor_name) = if same_dir {
            (
                self.cursor_entry().map(|e| e.name.clone()),
                self.anchor
                    .and_then(|a| self.entry_at(a))
                    .map(|e| e.name.clone()),
            )
        } else {
            self.selected.clear();
            self.dir_sizes.clear();
            self.sizes_gen = None;
            (None, None)
        };
        self.loc.path = path;
        self.locked = match manta_vfs::archive::as_archive(&*self.loc.fs) {
            Some(a) => entries
                .iter()
                .filter(|e| a.is_encrypted(&e.path))
                .map(|e| e.name.clone())
                .collect(),
            None => HashSet::new(),
        };
        drop_later(std::mem::replace(&mut self.entries, entries));
        self.listing_gen += 1;
        self.view.clear();
        self.loading = false;
        self.loading_since = None;
        self.error = None;
        self.rebuild_view_keeping(cursor_name, anchor_name);
        if let Some(name) = self.pending_select.take() {
            if let Some(idx) = self.view_index_of(&name) {
                self.selected.clear();
                self.selected.insert(name);
                self.cursor = Some(idx);
                self.anchor = Some(idx);
                self.scroll_to_cursor = true;
            }
        }
        if !self.pending_select_many.is_empty() {
            let names = std::mem::take(&mut self.pending_select_many);
            self.select_names(&names);
        }
        if std::mem::take(&mut self.pending_select_first) && !self.view.is_empty() {
            self.select_only(0);
            self.scroll_to_cursor = true;
        }
    }

    /// Select exactly the entries named `names` that are in the view; the
    /// cursor goes to the first of them (in view order) and is scrolled to.
    /// Returns how many were selected (nothing changes if none is shown).
    pub fn select_names(&mut self, names: &[String]) -> usize {
        let wanted: HashSet<&str> = names.iter().map(String::as_str).collect();
        let idxs: Vec<usize> = self
            .view
            .iter()
            .enumerate()
            .filter(|(_, &i)| {
                self.entries
                    .get(i)
                    .is_some_and(|e| wanted.contains(e.name.as_str()))
            })
            .map(|(vi, _)| vi)
            .collect();
        if idxs.is_empty() {
            return 0;
        }
        self.selected = idxs
            .iter()
            .filter_map(|&vi| self.entry_at(vi))
            .map(|e| e.name.clone())
            .collect();
        self.cursor = Some(idxs[0]);
        self.anchor = Some(idxs[0]);
        self.scroll_to_cursor = true;
        idxs.len()
    }

    /// Selected entries in view order; falls back to the cursor entry.
    pub fn selected_entries(&self) -> Vec<Entry> {
        let sum = self.selection_summary();
        if sum.view_idx.is_empty() {
            self.cursor_entry().cloned().into_iter().collect()
        } else {
            sum.view_idx
                .iter()
                .filter_map(|&vi| self.entry_at(vi))
                .cloned()
                .collect()
        }
    }

    /// The selected entries (shown ones, in view order) and their size,
    /// computed once per change of the selection, listing, view or folder
    /// sizes (the status bar, menus and plugins ask for it every frame).
    pub fn selection_summary(&self) -> std::rc::Rc<SelectionSummary> {
        let key = (
            self.selected.gen(),
            self.listing_gen,
            self.view_gen,
            self.sizes_gen,
        );
        if let Some((k, s)) = &*self.sel_summary.borrow() {
            if *k == key {
                return s.clone();
            }
        }
        let mut sum = SelectionSummary::default();
        if !self.selected.is_empty() {
            // Hash lookups for big selections, the tree for small ones.
            let big: Option<HashSet<&str>> = (self.selected.len() > 16)
                .then(|| self.selected.iter().map(String::as_str).collect());
            for (vi, &i) in self.view.iter().enumerate() {
                let Some(e) = self.entries.get(i) else {
                    continue;
                };
                let hit = match &big {
                    Some(set) => set.contains(e.name.as_str()),
                    None => self.selected.contains(&e.name),
                };
                if hit {
                    sum.view_idx.push(vi);
                    sum.size += util::sort_size(e, &self.dir_sizes);
                    sum.any_archive = sum.any_archive || is_archive_file(e);
                }
            }
        }
        let sum = std::rc::Rc::new(sum);
        *self.sel_summary.borrow_mut() = Some((key, sum.clone()));
        sum
    }

    /// Bytes of the selected entries; folders count once their size is
    /// known (measured by a plugin, possibly still counting).
    pub fn selection_size(&self) -> u64 {
        self.selection_summary().size
    }

    /// Take new plugin folder sizes; re-sorts if sorted by size.
    pub fn set_dir_sizes(&mut self, sizes: HashMap<String, SizeInfo>, gen: u64) {
        self.sizes_gen = Some(gen);
        if sizes == self.dir_sizes {
            return;
        }
        self.dir_sizes = sizes;
        if self.sort.column == SortColumn::Size {
            self.rebuild_view();
        }
    }

    pub fn select_all(&mut self) {
        self.selected = self.view_entries().map(|e| e.name.clone()).collect();
    }

    pub fn select_only(&mut self, view_idx: usize) {
        self.selected.clear();
        if let Some(name) = self.entry_at(view_idx).map(|e| e.name.clone()) {
            self.selected.insert(name);
        }
        self.cursor = Some(view_idx);
        self.anchor = Some(view_idx);
    }

    /// Mouse click with modifiers: Cmd toggles, Shift selects a range.
    pub fn click(&mut self, view_idx: usize, mods: egui::Modifiers) {
        let Some(name) = self.entry_at(view_idx).map(|e| e.name.clone()) else {
            return;
        };
        if mods.shift {
            self.select_range(self.anchor.unwrap_or(view_idx), view_idx, mods.command);
            self.cursor = Some(view_idx);
        } else if mods.command {
            if !self.selected.remove(&name) {
                self.selected.insert(name);
            }
            self.cursor = Some(view_idx);
            self.anchor = Some(view_idx);
        } else {
            self.select_only(view_idx);
        }
    }

    fn select_range(&mut self, a: usize, b: usize, additive: bool) {
        if !additive {
            self.selected.clear();
        }
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let names: Vec<String> = self
            .view_entries()
            .skip(lo)
            .take(hi.saturating_sub(lo) + 1)
            .map(|e| e.name.clone())
            .collect();
        self.selected.extend(names);
    }

    /// Arrow-key navigation; Shift extends the selection.
    /// How long the typed letters stay "one word" (Finder uses about 1 s).
    pub const TYPE_AHEAD_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);

    /// True while typing letters still extends the current search.
    pub fn type_ahead_active(&self, now: std::time::Instant) -> bool {
        !self.type_ahead.is_empty()
            && self
                .type_ahead_at
                .is_some_and(|t| now.duration_since(t) < Self::TYPE_AHEAD_TIMEOUT)
    }

    /// Type-to-select like Finder: add `text` to the typed letters and
    /// select the first entry whose name starts with them (case-insensitive);
    /// without a prefix match, the first name that sorts after them.
    pub fn type_ahead(&mut self, text: &str, now: std::time::Instant) {
        if !self.type_ahead_active(now) {
            self.type_ahead.clear();
        }
        self.type_ahead.push_str(&text.to_lowercase());
        self.type_ahead_at = Some(now);
        let needle = self.type_ahead.as_str();
        let name = |vi: usize| self.entry_at(vi).map_or("", |e| e.name.as_str());
        let n = self.view.len();
        let hit = (0..n)
            .position(|vi| util::starts_with_lowercase(name(vi), needle))
            .or_else(|| {
                // Nearest following name in the current order (Finder-like).
                let mut best: Option<(usize, String)> = None;
                for vi in 0..n {
                    let nm = name(vi);
                    if util::cmp_lowercase(nm, needle) != std::cmp::Ordering::Greater {
                        continue;
                    }
                    let lower = || nm.to_lowercase();
                    let better = match &best {
                        None => true,
                        Some((_, b)) => util::cmp_lowercase(nm, b) == std::cmp::Ordering::Less,
                    };
                    if better {
                        best = Some((vi, lower()));
                    }
                }
                best.map(|(vi, _)| vi)
            });
        if let Some(i) = hit {
            self.select_only(i);
            self.scroll_to_cursor = true;
        }
    }

    pub fn move_cursor(&mut self, delta: isize, extend: bool) {
        if self.view.is_empty() {
            return;
        }
        let last = self.view.len() as isize - 1;
        let next = match self.cursor {
            Some(c) => (c as isize + delta).clamp(0, last),
            None if delta < 0 => last,
            None => 0,
        } as usize;
        if extend {
            let anchor = self.anchor.unwrap_or(next);
            self.anchor = Some(anchor);
            self.select_range(anchor, next, false);
            self.cursor = Some(next);
        } else {
            self.select_only(next);
        }
        self.scroll_to_cursor = true;
    }
}

pub struct Pane {
    /// Stable id while the pane exists (also its plugin [`manta_plugin::PaneId`]).
    pub id: Side,
    /// Share of the pane area (the panes' weights sum to 1).
    pub weight: f32,
    pub tabs: Vec<Tab>,
    pub active: usize,
    /// Widths of the Size, Modified and Permissions columns.
    pub col_widths: [f32; 3],
    /// How the pane shows its folder (⌘1 – ⌘4).
    pub view: crate::views::ViewMode,
    /// Icon size of the Icons view (points).
    pub icon_size: f32,
    /// Widths of the Columns view's columns (by position from the left).
    pub column_widths: Vec<f32>,
    /// The folder whose remembered view was applied last ("Remember view
    /// per folder").
    pub view_folder: Option<String>,
}

impl Pane {
    pub fn new(id: Side, tab: Tab) -> Self {
        Pane {
            id,
            weight: 1.0,
            tabs: vec![tab],
            active: 0,
            col_widths: [76.0, 120.0, 92.0],
            view: crate::views::ViewMode::List,
            icon_size: crate::views::ICON_DEFAULT,
            column_widths: Vec::new(),
            view_folder: None,
        }
    }

    pub fn tab(&self) -> &Tab {
        &self.tabs[self.active]
    }

    pub fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active]
    }
}

pub fn filter_field_id(side: Side) -> egui::Id {
    egui::Id::unique(("manta-filter", side))
}

fn path_field_id(side: Side) -> egui::Id {
    egui::Id::unique(("manta-path", side))
}

/// The Compare & Sync slots ("A" / "B") a pane can be chosen for while the
/// sync window is open with three or more panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncSlot {
    A,
    B,
}

/// What a pane needs to know about its surroundings.
pub struct PaneInfo<'a> {
    pub side: Side,
    pub is_active: bool,
    /// How many panes the window shows (close button, two-pane commands).
    pub count: usize,
    /// The rightmost pane (it has the split button and draws the drag badge).
    pub is_last: bool,
    /// Copy / move from the active pane go here (three or more panes, while
    /// the active pane has a selection): a faint outline and a "→" badge.
    pub is_target: bool,
    /// Compare & Sync is open with three or more panes: the A / B chooser in
    /// the path bar, with this pane's slot (if any).
    pub sync_pick: Option<Option<SyncSlot>>,
    /// Where the pane is, for the location chip.
    pub place: &'a crate::place::PlaceChip,
    /// Plugin decorations of this pane's entries, by file name.
    pub decorations: &'a HashMap<String, MergedDecoration>,
    /// Plugin chips for the path bar: (plugin id, chip).
    pub chips: &'a [(String, Chip)],
    /// Plugin commands for the context menu.
    pub context_cmds: &'a [ContextCmd],
    /// Thumbnails for the Icons, Columns and Gallery views.
    pub thumbs: Option<&'a crate::thumbs::Thumbs>,
    /// Load thumbnails of small images on servers (a setting).
    pub remote_thumbs: bool,
    /// Files show the icon macOS has for their type (a setting).
    pub type_icons: bool,
    /// Finder tags of the visible folders (the Columns view's other
    /// columns read theirs here).
    pub tags: Option<&'a crate::tags::TagStore>,
}

/// A plugin command in the file list's context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCmd {
    /// `plugin_id:command_id`.
    pub id: String,
    pub title: String,
    pub shortcut: String,
    /// The plugin's name: commands of a plugin with many of them share a
    /// submenu named after it.
    pub plugin: String,
}

/// More context commands than this from one plugin go into a submenu.
const MAX_INLINE_PLUGIN_CMDS: usize = 3;

/// Plugin commands grouped by plugin, in order: (plugin name, commands).
pub fn group_context_cmds(cmds: &[ContextCmd]) -> Vec<(&str, Vec<&ContextCmd>)> {
    let mut groups: Vec<(&str, Vec<&ContextCmd>)> = Vec::new();
    for c in cmds {
        match groups.iter_mut().find(|(p, _)| *p == c.plugin) {
            Some((_, v)) => v.push(c),
            None => groups.push((c.plugin.as_str(), vec![c])),
        }
    }
    groups
}

/// A plugin panel area at the bottom of the pane, drawn by the app.
pub struct BottomArea<'a> {
    pub height: f32,
    pub draw: &'a mut dyn FnMut(&mut egui::Ui, egui::Rect),
}

/// Smallest height left for the file list when a bottom panel is open.
pub const MIN_LIST_H: f32 = 120.0;
/// Smallest height of a bottom panel.
pub const MIN_BOTTOM_H: f32 = 120.0;
/// Height of a bottom panel the user hasn't resized: this share of the pane.
pub const DEFAULT_BOTTOM_SHARE: f32 = 0.35;

const PATH_BAR_H: f32 = size::PATH_BAR;
/// Horizontal text inset of a breadcrumb segment (and of the path editor).
const CRUMB_PAD: f32 = 5.0;
/// Gap between the location chip and the first breadcrumb. Leaves room for
/// the path editor, which starts one "/" further left (see `path_bar`).
/// Below this path-bar width the filter field folds into a magnifier.
pub const FILTER_COLLAPSE_W: f32 = 640.0;

const CRUMB_GAP: f32 = space::MD + 2.0;
const TAB_BAR_H: f32 = 28.0;

/// Draw one pane. Navigation and file operations are reported as [`Action`]s;
/// purely local state (selection, sorting, filter) is changed in place.
pub fn pane_ui(
    ui: &mut egui::Ui,
    info: PaneInfo<'_>,
    pane: &mut Pane,
    actions: &mut Vec<Action>,
    bottom: Option<BottomArea<'_>>,
) {
    let p = pal(ui);
    let side = info.side;
    let outer_rect = ui.max_rect();

    // Clicking anywhere inside the pane makes it active.
    let pressed_inside = ui.input(|i| {
        (i.pointer.any_pressed() || i.pointer.button_pressed(egui::PointerButton::Secondary))
            && i.pointer
                .interact_pos()
                .is_some_and(|pos| outer_rect.contains(pos))
    }) && ui.ctx().layer_id_at(ui.input(|i| i.pointer.interact_pos()).unwrap_or_default())
        // Not through a dialog, menu or modal window on top.
        == Some(ui.layer_id());
    if pressed_inside && !info.is_active {
        actions.push(Action::Activate(side));
    }

    let t = ui.ctx().animate_bool_with_time(
        egui::Id::unique(("pane-active", side)),
        info.is_active,
        crate::a11y::anim(0.12),
    );
    let stroke_color = lerp_color(p.separator, p.accent.gamma_multiply(0.85), t);
    let r = CornerRadius::same(radius::PANEL);
    ui.painter().rect_filled(outer_rect, r, p.content);

    let mut rect = outer_rect.shrink(1.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect);
        let mut region = t!("a11y.pane", folder = pane.tab().title());
        if info.is_active && info.count > 1 {
            region.push_str(", ");
            region.push_str(t!("a11y.active"));
        }
        crate::a11y::region(ui, egui::accesskit::Role::Group, &region);
        if pane.tabs.len() > 1 {
            let bar = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), TAB_BAR_H));
            tabs_bar(ui, bar, side, pane, actions);
            rect.min.y = bar.bottom();
        }
        let col_widths = &mut pane.col_widths;
        let view = pane.view;
        let icon_size = &mut pane.icon_size;
        let column_widths = &mut pane.column_widths;
        let tab = &mut pane.tabs[pane.active];
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), PATH_BAR_H));
        path_bar(ui, bar, &info, tab, view, icon_size, actions);
        rect.min.y = bar.bottom();
        ui.painter()
            .hline(rect.x_range(), rect.top(), Stroke::new(1.0, p.separator));
        // While searching (or about to): This Folder / Subfolders / This Mac.
        if crate::search::scope_bar_visible(ui.ctx(), side, tab) {
            let r = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(rect.width(), crate::search::SCOPE_BAR_H),
            );
            crate::search::scope_bar(ui, r, side, tab, actions);
            rect.min.y = r.bottom();
            ui.painter()
                .hline(rect.x_range(), rect.top(), Stroke::new(1.0, p.separator));
        }
        // Search results are a list (the Columns view shows folders).
        let view = if tab.in_search() && view == crate::views::ViewMode::Columns {
            crate::views::ViewMode::List
        } else {
            view
        };
        let mut list_rect = rect;
        if let Some(b) = bottom {
            let wanted = if b.height > 0.0 {
                b.height
            } else {
                rect.height() * DEFAULT_BOTTOM_SHARE
            };
            let h = wanted
                .max(MIN_BOTTOM_H)
                .min(rect.height() - MIN_LIST_H)
                .max(0.0);
            if h >= plugin_ui::TITLE_BAR_H + 20.0 {
                list_rect.max.y = rect.bottom() - h;
                let area =
                    egui::Rect::from_min_max(egui::pos2(rect.left(), list_rect.bottom()), rect.max);
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(area)
                        .id_salt(("pane-bottom", side)),
                    |ui| {
                        ui.set_clip_rect(area.intersect(ui.clip_rect()));
                        (b.draw)(ui, area);
                    },
                );
                ui.painter()
                    .hline(area.x_range(), area.top(), Stroke::new(1.0, p.separator));
            }
        }
        match view {
            crate::views::ViewMode::List => {
                file_list(ui, list_rect, &info, tab, col_widths, actions)
            }
            crate::views::ViewMode::Icons => {
                crate::views::icons_view(ui, list_rect, &info, tab, icon_size, actions)
            }
            crate::views::ViewMode::Columns => {
                crate::views::columns_view(ui, list_rect, &info, tab, column_widths, actions)
            }
            crate::views::ViewMode::Gallery => {
                crate::views::gallery_view(ui, list_rect, &info, tab, actions)
            }
        }
    });
    ui.painter().rect_stroke(
        outer_rect,
        r,
        Stroke::new(1.0 + 0.5 * t, stroke_color),
        StrokeKind::Inside,
    );

    if info.is_target && !info.is_active {
        // The copy target: a faint accent ring (the active pane has the
        // strong one).
        ui.painter().rect_stroke(
            outer_rect.shrink(1.5),
            CornerRadius::same(radius::PANEL.saturating_sub(1)),
            Stroke::new(
                1.0,
                p.accent.gamma_multiply(if p.dark { 0.45 } else { 0.35 }),
            ),
            StrokeKind::Inside,
        );
    }

    let loc = pane.tab().loc.clone();
    drop_target(ui, side, info.is_last, outer_rect, &loc, actions);
    external_drop_target(ui, side, outer_rect, &loc, actions);
}

/// A borderless icon button of the path bar (FIELD high), with a hover
/// fill and a tooltip.
fn bar_button(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    glyph: &str,
    tip: &str,
    shortcut: &str,
) -> bool {
    let p = pal(ui);
    let resp = ui.interact(rect, id, Sense::click());
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::CONTROL), p.hover);
    }
    let color = if resp.hovered() {
        p.text
    } else {
        p.text_secondary
    };
    widgets::paint_icon(ui, rect, glyph, 14.0, color, false);
    widgets::tooltip(resp, tip, shortcut).clicked()
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

// ------------------------------------------------------------------ tabs

/// Width of one cell of the A / B chooser.
const SYNC_CELL: f32 = 18.0;

/// The "A | B" chooser of Compare & Sync in a pane's path bar. The chosen
/// letter is filled; a click on the other one picks this pane for it.
fn sync_picker(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    side: Side,
    chosen: Option<SyncSlot>,
) -> Option<SyncSlot> {
    let p = pal(ui);
    let mut out = None;
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius::CONTROL),
        Stroke::new(1.0, p.separator),
        StrokeKind::Inside,
    );
    for (i, (slot, letter)) in [(SyncSlot::A, "A"), (SyncSlot::B, "B")]
        .into_iter()
        .enumerate()
    {
        let r = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 1.0 + i as f32 * SYNC_CELL, rect.top() + 1.0),
            egui::vec2(SYNC_CELL - 1.0, rect.height() - 2.0),
        );
        let resp = ui.interact(
            r,
            egui::Id::unique(("sync-pick", side, letter)),
            Sense::click(),
        );
        let on = chosen == Some(slot);
        let corner = CornerRadius::same(radius::CONTROL.saturating_sub(1));
        if on {
            ui.painter().rect_filled(r, corner, p.accent);
        } else if resp.hovered() {
            ui.painter().rect_filled(r, corner, p.hover);
        }
        let color = if on {
            p.text_on_accent
        } else if resp.hovered() {
            p.text
        } else {
            p.text_tertiary
        };
        ui.painter().text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            letter,
            theme::semibold(text::CAPTION),
            color,
        );
        let tip = match slot {
            SyncSlot::A => t!("panes.sync_pick_a"),
            SyncSlot::B => t!("panes.sync_pick_b"),
        };
        if resp.on_hover_text(tip).clicked() && !on {
            out = Some(slot);
        }
    }
    out
}

fn tabs_bar(
    ui: &mut egui::Ui,
    bar: egui::Rect,
    side: Side,
    pane: &Pane,
    actions: &mut Vec<Action>,
) {
    let p = pal(ui);
    ui.painter().rect_filled(bar, CornerRadius::ZERO, p.stripe);
    let n = pane.tabs.len();
    let tab_w = (bar.width() / n as f32).clamp(60.0, 220.0);
    for (i, tab) in pane.tabs.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            egui::pos2(bar.left() + tab_w * i as f32, bar.top()),
            egui::vec2(tab_w, bar.height()),
        );
        if r.left() > bar.right() {
            break;
        }
        let resp = ui.interact(r, egui::Id::unique(("tab", side, tab.id)), Sense::click());
        let active = i == pane.active;
        if active {
            ui.painter().rect_filled(r, CornerRadius::ZERO, p.content);
        } else if resp.hovered() {
            ui.painter().rect_filled(r, CornerRadius::ZERO, p.hover);
        }
        if i > 0 && !active && i != pane.active + 1 {
            ui.painter().vline(
                r.left(),
                r.y_range().shrink(6.0),
                Stroke::new(1.0, p.separator),
            );
        }
        let color = if active { p.text } else { p.text_secondary };
        let font = if active {
            theme::medium(text::SMALL)
        } else {
            theme::regular(text::SMALL)
        };
        let mut job = egui::text::LayoutJob::simple_singleline(tab.title(), font, color);
        job.wrap = egui::text::TextWrapping::truncate_at_width(r.width() - 44.0);
        let galley = ui.painter().layout_job(job);
        ui.painter()
            .galley(r.center() - galley.size() / 2.0, galley, color);
        if tab.loading {
            let dot = egui::pos2(r.right() - 14.0, r.center().y);
            ui.painter().circle_filled(dot, 2.5, p.accent);
        }
        // Close button on hover or for the active tab.
        let close = egui::Rect::from_center_size(
            egui::pos2(r.left() + 14.0, r.center().y),
            egui::vec2(18.0, 18.0),
        );
        let c = ui.interact(
            close,
            egui::Id::unique(("tab-close", side, tab.id)),
            Sense::click(),
        );
        if active || resp.hovered() || c.hovered() {
            if c.hovered() {
                ui.painter()
                    .rect_filled(close, CornerRadius::same(4), p.hover);
            }
            widgets::paint_icon(ui, close, icons::X, 11.0, p.text_secondary, false);
        }
        if widgets::tooltip(c, Cmd::CloseTab.label(), Cmd::CloseTab.shortcut()).clicked() {
            actions.push(Action::CloseTab(side, i));
        } else if resp.clicked() {
            actions.push(Action::SwitchTab(side, i));
        }
        if resp.middle_clicked() {
            actions.push(Action::CloseTab(side, i));
        }
        resp.on_hover_text(tab.loc.path.as_str());
    }
    ui.painter().hline(
        bar.x_range(),
        bar.bottom() - 0.5,
        Stroke::new(1.0, p.separator),
    );
}

// ------------------------------------------------------------------ path bar

fn path_bar(
    ui: &mut egui::Ui,
    bar: egui::Rect,
    info: &PaneInfo<'_>,
    tab: &mut Tab,
    view: crate::views::ViewMode,
    icon_size: &mut f32,
    actions: &mut Vec<Action>,
) {
    let p = pal(ui);
    let side = info.side;
    let narrow = bar.width() < 420.0;
    // Several narrow panes: the "Local" chip shrinks to its icon.
    let tight = bar.width() < 360.0;
    // At the minimum pane width (several panes) the folder name needs the
    // room: a short filter field, and the tab "+" goes (⇧⌘T still works).
    let minimal = bar.width() < 300.0;
    let filter_w = if minimal {
        64.0
    } else if bar.width() < 360.0 {
        84.0
    } else if narrow {
        110.0
    } else if bar.width() < 640.0 {
        // Room for the view switcher.
        124.0
    } else {
        160.0
    };
    let inner = bar.shrink2(egui::vec2(space::MD, 0.0));

    // Right side: [A|B] [→] [spinner] [filter] [+ tab] [split] [close]
    // Every item in the bar is FIELD high and centred on `cy`.
    let cy = bar.center().y;
    let mut right = inner.right();
    // The next FIELD-sized square leftwards.
    let slot = |right: &mut f32| {
        let r = egui::Rect::from_min_max(
            egui::pos2(*right - size::FIELD, cy - size::FIELD / 2.0),
            egui::pos2(*right, cy + size::FIELD / 2.0),
        );
        *right = r.left() - 2.0;
        r
    };
    if info.count > 1 {
        let r = slot(&mut right);
        if bar_button(
            ui,
            r,
            egui::Id::unique(("close-pane", side)),
            icons::X,
            Cmd::ClosePane.label(),
            Cmd::ClosePane.shortcut(),
        ) {
            actions.push(Action::ClosePane(side));
        }
    }
    if info.is_last {
        let r = slot(&mut right);
        if bar_button(
            ui,
            r,
            egui::Id::unique(("split-pane", side)),
            icons::COLUMNS_PLUS_RIGHT,
            Cmd::SplitPane.label(),
            Cmd::SplitPane.shortcut(),
        ) {
            actions.push(Action::SplitPane(side));
        }
    }
    if !minimal && (info.count > 1 || info.is_last) {
        // A hairline between the pane controls and the tab "+".
        let x = right - 1.0;
        ui.painter().vline(
            x,
            egui::Rangef::new(cy - 7.0, cy + 7.0),
            Stroke::new(1.0, p.separator),
        );
        right -= space::XS;
    }
    let plus_rect = if minimal {
        egui::Rect::from_min_max(
            egui::pos2(right + 2.0, cy - size::FIELD / 2.0),
            egui::pos2(right + 2.0, cy + size::FIELD / 2.0),
        )
    } else {
        let r = slot(&mut right);
        if bar_button(
            ui,
            r,
            egui::Id::unique(("new-tab", side)),
            icons::PLUS,
            Cmd::NewTab.label(),
            Cmd::NewTab.shortcut(),
        ) {
            actions.push(Action::NewTab(side));
        }
        r
    };

    // Narrow panes: the filter is a magnifier button that opens into the
    // field (click, ⌘F) and stays open while it has text or focus, so the
    // location chip and the breadcrumb keep their room.
    let collapsible = bar.width() < FILTER_COLLAPSE_W;
    let expanded = !collapsible || tab.filter_open || tab.filter_focus || !tab.filter.is_empty();
    let filter_rect = if expanded {
        let filter_rect = egui::Rect::from_min_size(
            egui::pos2(
                plus_rect.left() - space::XS - filter_w,
                cy - size::FIELD / 2.0,
            ),
            egui::vec2(filter_w, size::FIELD),
        );
        let id = filter_field_id(side);
        let hint = if tab.scope == crate::search::Scope::Folder {
            t!("pane.filter")
        } else {
            t!("search.placeholder")
        };
        let fresp = ui
            .scope_builder(egui::UiBuilder::new().max_rect(filter_rect), |ui| {
                widgets::search_field(ui, &mut tab.filter, hint, id, filter_w)
            })
            .inner;
        if std::mem::take(&mut tab.filter_focus) {
            fresp.request_focus();
            tab.filter_open = true;
        }
        if fresp.changed() {
            filter_changed(side, tab, actions);
        }
        if fresp.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && tab.scope != crate::search::Scope::Folder
            && !tab.filter.trim().is_empty()
        {
            // Return searches right away.
            tab.search_due = Some(Instant::now());
        }
        if fresp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            tab.filter.clear();
            filter_changed(side, tab, actions);
            // Esc closes the search entirely, like Finder.
            tab.scope = crate::search::Scope::Folder;
            fresp.surrender_focus();
        }
        if fresp.has_focus() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(scope_bar_hold_id(side), Instant::now()));
        }
        if collapsible && tab.filter.is_empty() && !ui.memory(|m| m.has_focus(id)) {
            tab.filter_open = false;
        }
        filter_rect
    } else {
        tab.filter_open = false;
        let r = egui::Rect::from_min_size(
            egui::pos2(
                plus_rect.left() - space::XS - size::FIELD,
                cy - size::FIELD / 2.0,
            ),
            egui::vec2(size::FIELD, size::FIELD),
        );
        let resp = ui.interact(r, egui::Id::unique(("filter-open", side)), Sense::click());
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, Cmd::Filter.label())
        });
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        let color = if resp.hovered() {
            p.text
        } else {
            p.text_secondary
        };
        widgets::paint_icon(ui, r, icons::MAGNIFYING_GLASS, 14.0, color, false);
        if widgets::tooltip(resp, Cmd::Filter.label(), Cmd::Filter.shortcut()).clicked() {
            tab.filter_focus = true;
            ui.ctx().request_repaint();
        }
        r
    };

    // The view switcher folds into one menu button in narrow panes, and
    // earlier while a folded filter is open.
    let compact_views = bar.width() < 470.0 || (collapsible && expanded);
    let switcher_left = crate::views::view_switcher(
        ui,
        filter_rect.left() - space::XS,
        cy,
        compact_views,
        side,
        tab,
        view,
        icon_size,
        actions,
    );
    let mut crumbs_right = switcher_left - space::SM;
    // Compare & Sync with three or more panes: which two are compared.
    if let Some(chosen) = info.sync_pick {
        let w = 2.0 * SYNC_CELL + 1.0;
        let r = egui::Rect::from_min_size(
            egui::pos2(crumbs_right - w, cy - SYNC_CELL / 2.0 - 1.0),
            egui::vec2(w, SYNC_CELL + 2.0),
        );
        if let Some(slot) = sync_picker(ui, r, side, chosen) {
            actions.push(Action::SetSyncPane(side, slot));
        }
        crumbs_right = r.left() - space::SM;
    }
    // Narrow panes keep the breadcrumb; the outline still marks the target.
    if info.is_target && !info.is_active && !tight {
        let g = ui.painter().layout_no_wrap(
            icons::ARROW_RIGHT.to_string(),
            theme::icon_font(12.0),
            p.accent_text,
        );
        let r = egui::Rect::from_min_size(
            egui::pos2(crumbs_right - 26.0, cy - 9.0),
            egui::vec2(26.0, 18.0),
        );
        ui.painter()
            .rect_filled(r, CornerRadius::same(radius::PILL), p.accent_soft);
        ui.painter()
            .galley(r.center() - g.size() / 2.0, g, p.accent_text);
        ui.interact(r, egui::Id::unique(("target-badge", side)), Sense::hover())
            .on_hover_text(t!("panes.target_tip"));
        crumbs_right = r.left() - space::SM;
    }
    if tab.loading {
        let sr = egui::Rect::from_center_size(
            egui::pos2(crumbs_right - 9.0, cy),
            egui::vec2(14.0, 14.0),
        );
        ui.put(sr, egui::Spinner::new().size(13.0).color(p.text_secondary));
        crumbs_right = sr.left() - space::SM;
    }

    // Plugin chips, right-aligned before the spinner / filter, as long as
    // the breadcrumb keeps a useful width.
    let baseline_chips = widgets::body_baseline(ui, cy);
    // Leave room for the location chip and at least "… › current" with
    // the current folder's name in full (up to a point).
    let current_w = ui
        .painter()
        .layout_no_wrap(tab.title(), theme::medium(text::BODY), p.text)
        .size()
        .x
        .min(if narrow { 120.0 } else { 200.0 });
    // The leading "…" of the breadcrumb gives way to the chip.
    let mut min_crumbs = current_w + 2.0 * CRUMB_PAD;
    // Inside an archive its name stays visible too: "… › backup.zip › docs".
    if let Some(m) = &tab.loc.archive {
        min_crumbs += 17.0;
        if !tab.loc.path.is_root() {
            let w = ui
                .painter()
                .layout_no_wrap(m.file_name.clone(), theme::regular(text::BODY), p.text)
                .size()
                .x
                .min(if narrow { 110.0 } else { 180.0 });
            min_crumbs += w + 2.0 * CRUMB_PAD + 16.0;
        }
    }
    // The location chip: „Lokal“, „Docker · shop-web“, „Google Drive“ …,
    // cut with "…" (full text in the tooltip), only its icon in tight
    // panes. Anything but „Lokal“ has a "back to local" button after it.
    let away = !info.place.local;
    let chip_font = theme::medium(text::SMALL);
    let mut label = if tight { "" } else { info.place.label.as_str() };
    let label_w = ui
        .painter()
        .layout_no_wrap(label.to_string(), chip_font.clone(), p.text)
        .size()
        .x;
    let chip_max = if narrow { 120.0 } else { 180.0 };
    let mut chip_w =
        (label_w + 26.0 + if label.is_empty() { 0.0 } else { space::SM }).min(chip_max);
    let back_w = if away { size::FIELD + 2.0 } else { 0.0 };
    // The breadcrumb keeps "… › current folder": the chip gives way, cut
    // first to the name alone („shop-web“; the icon says Docker), then
    // cut with "…", then down to its icon.
    let room = crumbs_right - inner.left() - back_w - CRUMB_GAP - min_crumbs;
    if chip_w > room {
        let short = label.split_once(" · ").map_or(label, |(_, name)| name);
        let short_w = ui
            .painter()
            .layout_no_wrap(short.to_string(), chip_font.clone(), p.text)
            .size()
            .x
            + 26.0
            + space::SM;
        if short_w <= room {
            label = short;
            chip_w = short_w;
        } else if room >= 70.0 {
            label = short;
            chip_w = room;
        } else {
            chip_w = 26.0;
            label = "";
        }
    }
    let crumbs_left_estimate = inner.left() + chip_w + back_w + CRUMB_GAP;
    // Chips degrade instead of vanishing: full → cut text → icon only;
    // hidden only when not even the icon fits.
    let full: Vec<f32> = info
        .chips
        .iter()
        .map(|(_, c)| plugin_ui::chip_width(ui, c))
        .collect();
    let min_text: Vec<f32> = info
        .chips
        .iter()
        .map(|(_, c)| plugin_ui::chip_min_text_width(ui, c))
        .collect();
    let avail = crumbs_right - (crumbs_left_estimate + min_crumbs);
    let forms = plugin_ui::layout_chips(&full, &min_text, space::SM, avail);
    for (i, (plugin, chip)) in info.chips.iter().enumerate().rev() {
        let form = forms[i];
        if form == plugin_ui::ChipForm::Hidden {
            continue;
        }
        let w = form.width(full[i]);
        let r = egui::Rect::from_min_size(
            egui::pos2(crumbs_right - w, cy - plugin_ui::CHIP_H / 2.0),
            egui::vec2(w, plugin_ui::CHIP_H),
        );
        let sense = if chip.command.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let resp = ui.interact(r, egui::Id::unique(("plugin-chip", side, i)), sense);
        let clickable = chip.command.is_some();
        plugin_ui::paint_chip(ui, r, chip, clickable && resp.hovered(), baseline_chips);
        let resp = match plugin_ui::chip_tooltip(chip, form) {
            Some(t) => resp.on_hover_text(t),
            None => resp,
        };
        if clickable {
            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            if resp.clicked() {
                if let Some(cmd) = &chip.command {
                    actions.push(Action::RunPluginCommand(format!("{plugin}:{cmd}")));
                }
            }
        }
        crumbs_right = r.left() - space::SM;
    }

    // Location chip (click = root).
    // FTPS continued with clear data connections: say so on the chip.
    let notice = tab.loc.fs.notice().map(|n| match n {
        manta_core::ConnectionNotice::ClearDataChannel => t!("pane.notice.clear_data"),
    });
    let chip_icon = if notice.is_some() {
        icons::SHIELD_WARNING
    } else {
        info.place.icon
    };
    let chip_fg = if info.is_active {
        p.accent_text
    } else {
        p.text_secondary
    };
    let chip_bg = if info.is_active {
        p.accent_soft
    } else {
        p.stripe
    };
    let baseline = widgets::body_baseline(ui, cy);
    let chip = egui::Rect::from_min_size(
        egui::pos2(inner.left(), cy - size::FIELD / 2.0),
        egui::vec2(chip_w, size::FIELD),
    );
    let chip_resp = ui.interact(chip, egui::Id::unique(("chip", side)), Sense::click());
    ui.painter()
        .rect_filled(chip, CornerRadius::same(radius::PILL), chip_bg);
    if chip_resp.hovered() {
        ui.painter()
            .rect_filled(chip, CornerRadius::same(radius::PILL), p.hover);
    }
    let ir = egui::Rect::from_min_size(
        chip.min + egui::vec2(4.0, 0.0),
        egui::vec2(18.0, chip.height()),
    );
    let icon_fg = if notice.is_some() { p.warning } else { chip_fg };
    widgets::paint_icon(ui, ir, chip_icon, 13.0, icon_fg, false);
    // Every text in the bar sits on the body baseline the breadcrumbs and
    // the filter field use, whatever its size (`baseline`, computed above).
    if !label.is_empty() {
        let mut job =
            egui::text::LayoutJob::simple_singleline(label.to_string(), chip_font, chip_fg);
        job.wrap = egui::text::TextWrapping::truncate_at_width(chip.width() - 30.0);
        let g = ui.painter().layout_job(job);
        widgets::paint_on_baseline(ui, ir.right() + 1.0, baseline, g, chip_fg);
    }
    chip_resp
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &info.place.label));
    let mut tip = format!("{}\n{}", info.place.tooltip, t!("pane.chip_tip"));
    if let Some(n) = notice {
        tip.push('\n');
        tip.push_str(n);
    }
    if widgets::tooltip(chip_resp, &tip, "").clicked() {
        actions.push(if tab.loc.archive.is_some() {
            Action::NavigateTo(side, tab.loc.outermost().with_path(VfsPath::root()), None)
        } else if let Some(root) = &info.place.cloud_root {
            Action::NavigateTo(
                side,
                tab.loc.with_path(VfsPath::new(&root.to_string_lossy())),
                None,
            )
        } else {
            Action::Navigate(side, VfsPath::root())
        });
    }
    let mut crumbs_left = chip.right();
    if away {
        let r = egui::Rect::from_min_size(
            egui::pos2(chip.right() + 2.0, cy - size::FIELD / 2.0),
            egui::vec2(size::FIELD, size::FIELD),
        );
        let resp = ui.interact(r, egui::Id::unique(("back-local", side)), Sense::click());
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, t!("pane.back_local"))
        });
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        let color = if resp.hovered() {
            p.text
        } else {
            p.text_secondary
        };
        widgets::paint_icon(ui, r, icons::ARROW_U_UP_LEFT, 14.0, color, false);
        if widgets::tooltip(
            resp.on_hover_cursor(egui::CursorIcon::PointingHand),
            t!("pane.back_local"),
            "",
        )
        .clicked()
        {
            actions.push(Action::GoLocal(side));
        }
        crumbs_left = r.right();
    }

    let crumb_area = egui::Rect::from_min_max(
        egui::pos2(crumbs_left + CRUMB_GAP, bar.top()),
        egui::pos2(crumbs_right, bar.bottom()),
    );
    if crumb_area.width() < 20.0 {
        return;
    }
    if let Some(text_val) = &mut tab.path_edit {
        // The editor shows the full path ("/usr/local"), the breadcrumb only
        // the components ("usr › local"). Start the field one "/" further
        // left, with the crumbs' text inset, so the first component's text
        // stays exactly where it was: switching modes doesn't move it.
        let slash_w = ui
            .painter()
            .layout_no_wrap("/".into(), theme::regular(text::BODY), p.text)
            .size()
            .x;
        let r = egui::Rect::from_x_y_ranges(
            egui::Rangef::new(crumb_area.left() - slash_w, crumb_area.right()),
            egui::Rangef::new(cy - size::FIELD / 2.0, cy + size::FIELD / 2.0),
        );
        let id = path_field_id(side);
        widgets::field_background(ui, r, ui.memory(|m| m.has_focus(id)));
        let resp = widgets::put_input(
            ui,
            r,
            widgets::bare_input(text_val, r, CRUMB_PAD).id(id),
            t!("pane.path_hint"),
        );
        if resp.lost_focus() {
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let target = VfsPath::new(text_val.trim());
                actions.push(Action::Navigate(side, target));
            }
            tab.path_edit = None;
        } else if !resp.has_focus() {
            resp.request_focus();
        }
    } else {
        breadcrumb(ui, crumb_area, side, tab, actions);
    }
}

fn breadcrumb(
    ui: &mut egui::Ui,
    area: egui::Rect,
    side: Side,
    tab: &mut Tab,
    actions: &mut Vec<Action>,
) {
    let p = pal(ui);
    if tab.in_search() {
        // A virtual location: "Search: “report” in Projects".
        crate::search::search_crumb(ui, area, side, tab);
        return;
    }
    let comps = crumbs(&tab.loc);
    let sep_w = 16.0;
    let pad = CRUMB_PAD;
    let icon_w = 17.0;
    let n = comps.len();
    let font_of = |last: bool| {
        if last {
            theme::medium(text::BODY)
        } else {
            theme::regular(text::BODY)
        }
    };
    let galleys: Vec<_> = comps
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let last = i + 1 == n;
            let color = if last { p.text } else { p.text_secondary };
            ui.painter()
                .layout_no_wrap(c.label.clone(), font_of(last), color)
        })
        .collect();
    let widths: Vec<f32> = galleys
        .iter()
        .zip(&comps)
        .map(|(g, c)| g.size().x + 2.0 * pad + if c.archive { icon_w } else { 0.0 })
        .collect();
    let ellipsis_w = 22.0 + sep_w;
    // Drop leading components until the rest fits.
    let mut start = 0;
    let total = |from: usize| -> f32 {
        widths[from..].iter().sum::<f32>()
            + sep_w * (n - from).saturating_sub(1) as f32
            + if from > 0 { ellipsis_w } else { 0.0 }
    };
    while start + 1 < n && total(start) > area.width() {
        start += 1;
    }
    // Only the current folder left and no room for "… ›" before it: the
    // folder keeps the room (↑ and the path field still lead up).
    let show_more = start > 0 && (total(start) <= area.width() || start + 1 < n);
    let action_for = |c: &Crumb| match &c.target {
        CrumbTarget::Here(path) => Action::Navigate(side, path.clone()),
        CrumbTarget::Outer(loc, select) => Action::NavigateTo(side, loc.clone(), select.clone()),
    };
    let archive_name = tab.loc.archive.as_ref().map(|m| m.file_name.clone());
    let target_text = |c: &Crumb| match &c.target {
        CrumbTarget::Here(path) if archive_name.is_some() => {
            format!("{} › {}", archive_name.as_deref().unwrap_or_default(), path)
        }
        CrumbTarget::Here(path) => path.to_string(),
        CrumbTarget::Outer(loc, _) => loc.path.to_string(),
    };

    let mut x = area.left();
    let cy = area.center().y;
    let baseline = widgets::body_baseline(ui, cy);
    if show_more {
        let r =
            egui::Rect::from_center_size(egui::pos2(x + 11.0, cy), egui::vec2(22.0, size::FIELD));
        let resp = ui.interact(r, egui::Id::unique(("crumb-more", side)), Sense::click());
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        widgets::paint_icon(ui, r, icons::DOTS_THREE, 14.0, p.text_secondary, false);
        let resp = widgets::tooltip(resp, t!("pane.enclosing_folders"), "");
        egui::Popup::menu(&resp).show(|ui| {
            let root = tab.loc.outermost().with_path(VfsPath::root());
            if ui.button(format!("{}  /", icons::HARD_DRIVE)).clicked() {
                actions.push(if tab.loc.archive.is_some() {
                    Action::NavigateTo(side, root, None)
                } else {
                    Action::Navigate(side, VfsPath::root())
                });
            }
            for c in &comps[..start] {
                let glyph = if c.archive {
                    icons::FILE_ARCHIVE
                } else {
                    icons::FOLDER_SIMPLE
                };
                if ui.button(format!("{glyph}  {}", c.label)).clicked() {
                    actions.push(action_for(c));
                }
            }
        });
        x += 22.0;
        widgets::paint_icon(
            ui,
            egui::Rect::from_center_size(egui::pos2(x + sep_w / 2.0, cy), egui::vec2(sep_w, 16.0)),
            icons::CARET_RIGHT,
            10.0,
            p.text_tertiary,
            false,
        );
        x += sep_w;
    }
    for (i, galley) in galleys.into_iter().enumerate().skip(start) {
        let c = &comps[i];
        let last = i + 1 == n;
        let w = widths[i].min(area.right() - x);
        if w <= 2.0 * pad {
            break;
        }
        let r = egui::Rect::from_min_size(
            egui::pos2(x, cy - size::FIELD / 2.0),
            egui::vec2(w, size::FIELD),
        );
        let resp = ui.interact(r, egui::Id::unique(("crumb", side, i)), Sense::click());
        // VoiceOver: the folder's name (the tooltip adds the full path).
        crate::a11y::name(&resp, egui::accesskit::Role::Link, &c.label);
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        let color = if last { p.text } else { p.text_secondary };
        let mut text_x = r.left() + pad;
        if c.archive {
            let ir = egui::Rect::from_min_size(
                egui::pos2(text_x - 1.0, r.top()),
                egui::vec2(icon_w, r.height()),
            );
            widgets::paint_icon(ui, ir, icons::FILE_ARCHIVE, 13.0, p.archive, false);
            text_x += icon_w;
        }
        let text_w = w - (text_x - r.left()) - pad;
        if galley.size().x > text_w {
            // Truncate a crumb that alone is too wide.
            let mut job =
                egui::text::LayoutJob::simple_singleline(c.label.clone(), font_of(last), color);
            job.wrap = egui::text::TextWrapping::truncate_at_width(text_w.max(8.0));
            let g = ui.painter().layout_job(job);
            widgets::paint_on_baseline(ui, text_x, baseline, g, color);
        } else {
            widgets::paint_on_baseline(ui, text_x, baseline, galley, color);
        }
        if resp.clicked() {
            actions.push(action_for(c));
        }
        if resp.double_clicked() || resp.secondary_clicked() {
            tab.path_edit = Some(tab.loc.path.to_string());
        }
        let hint = if c.archive {
            t!("pane.crumb.archive_tip")
        } else {
            t!("pane.crumb.edit_tip")
        };
        resp.on_hover_text(format!(
            "{}\n{hint} ({})",
            target_text(c),
            Cmd::EditPath.shortcut()
        ));
        x += w;
        if !last {
            widgets::paint_icon(
                ui,
                egui::Rect::from_center_size(
                    egui::pos2(x + sep_w / 2.0, cy),
                    egui::vec2(sep_w, 16.0),
                ),
                icons::CARET_RIGHT,
                10.0,
                p.text_tertiary,
                false,
            );
            x += sep_w;
        }
    }
    // Empty space after the crumbs: double-click edits the path, like Finder's path bar.
    if x < area.right() {
        let rest = egui::Rect::from_min_max(egui::pos2(x, area.top()), area.max);
        // Double-click only (⌘L does the same from the keyboard): no Tab stop.
        let resp = ui.interact(rest, egui::Id::unique(("crumb-rest", side)), Sense::CLICK);
        if resp.double_clicked() {
            tab.path_edit = Some(tab.loc.path.to_string());
        }
    }
}

// ------------------------------------------------------------------ file list

struct Columns {
    name: egui::Rangef,
    size: Option<egui::Rangef>,
    modified: Option<egui::Rangef>,
    perms: Option<egui::Rangef>,
    /// Search results: the folder each one is in (instead of Permissions).
    path: Option<egui::Rangef>,
    /// Search results at the top level are "in" this folder (its name).
    root: String,
}

const ROW_INSET: f32 = 6.0;

fn columns(rect: egui::Rect, widths: &[f32; 3], search: bool) -> Columns {
    let w = rect.width();
    let show_perms = w >= 640.0 && !search;
    let show_path = w >= 420.0 && search;
    let show_mod = w >= 380.0;
    let show_size = w >= 260.0;
    let mut right = rect.right() - ROW_INSET - space::SM;
    let mut take = |show: bool, width: f32| -> Option<egui::Rangef> {
        show.then(|| {
            let r = egui::Rangef::new(right - width, right);
            right -= width + space::MD;
            r
        })
    };
    let perms = take(show_perms, widths[2]);
    let path = take(show_path, (w * 0.32).clamp(120.0, 380.0));
    let modified = take(show_mod, widths[1]);
    let size = take(show_size, widths[0]);
    Columns {
        name: egui::Rangef::new(rect.left() + ROW_INSET + space::SM, right),
        size,
        modified,
        perms,
        path,
        root: String::new(),
    }
}

fn file_list(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    info: &PaneInfo<'_>,
    tab: &mut Tab,
    widths: &mut [f32; 3],
    actions: &mut Vec<Action>,
) {
    let row_icons = crate::views::RowIcons::of(info, &tab.loc);
    let side = info.side;

    if blocking_state(ui, rect, side, tab, actions) {
        return;
    }

    let mut cols = columns(rect, widths, tab.in_search());
    if tab.in_search() {
        cols.root = crate::search::root_label(&tab.loc);
    }
    let header = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), size::HEADER));
    header_row(ui, header, &cols, side, tab, widths);
    let body_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), header.bottom()), rect.max);

    // Background: click on empty space clears the selection.
    let bg = ui.interact(
        body_rect,
        egui::Id::unique(("files-bg", side, tab.id)),
        Sense::click(),
    );
    if bg.clicked() {
        tab.selected.clear();
    }
    list_anchor(&bg, side, tab, actions);
    let menu = MenuCtx::of(ui.ctx(), tab, info.count < 2);
    bg.context_menu(|ui| background_menu(ui, side, actions, &menu));

    if empty_states(ui, body_rect, side, tab, actions) {
        return;
    }

    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(("files", side, tab.id))
        .auto_shrink([false, false]);
    if tab.scroll_to_cursor {
        if let Some(c) = tab.cursor {
            let top = c as f32 * size::ROW;
            let view = (body_rect.height() - space::XS).max(size::ROW);
            if top < tab.scroll_offset {
                scroll = scroll.vertical_scroll_offset(top);
            } else if top + size::ROW > tab.scroll_offset + view {
                scroll = scroll.vertical_scroll_offset(top + size::ROW - view);
            }
        }
        tab.scroll_to_cursor = false;
    }

    let mut clicked: Option<(usize, egui::Modifiers)> = None;
    let mut double: Option<usize> = None;
    let mut drag_start: Option<usize> = None;
    let mut context: Option<usize> = None;
    let active = info.is_active;
    let now = chrono::Local::now();
    let multi = tab.selected.len() > 1;
    let menu = MenuCtx::of(ui.ctx(), tab, info.count < 2);
    let editing = tab
        .inline_rename
        .as_ref()
        .and_then(|r| r.form.original.clone());

    let out = ui
        .scope_builder(
            egui::UiBuilder::new().max_rect(body_rect.shrink2(egui::vec2(0.0, 0.0))),
            |ui| {
                // show_rows computes the row pitch as ROW + item_spacing.y of
                // *this* ui. Pin it to 0 (rows are exactly ROW high): with a
                // plugin panel open the inherited spacing differed, so egui
                // mapped scroll offsets to the wrong rows (type-to-select did
                // not reveal its match) and left an empty dark strip below
                // the last row.
                ui.spacing_mut().item_spacing.y = 0.0;
                scroll.show_rows(ui, size::ROW, tab.view.len(), |ui, range| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    crate::a11y::list(ui, &list_name(tab), tab.view.len());
                    let mut edit_row = None;
                    for vi in range {
                        // Rows are not Tab stops (the list is one, see
                        // `list_anchor`); VoiceOver reaches them as rows.
                        let (row, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), size::ROW),
                            Sense::CLICK | Sense::DRAG,
                        );
                        let Some(entry) = tab.entry_at(vi) else {
                            continue;
                        };
                        a11y_entry(&resp, info, tab, vi, now);
                        if entry.is_dir() {
                            note_folder_target(ui, side, row, &entry.path);
                        }
                        let is_selected = tab.selected.contains(&entry.name);
                        let prev_sel = vi > 0
                            && tab
                                .entry_at(vi - 1)
                                .is_some_and(|e| tab.selected.contains(&e.name));
                        let next_sel = tab
                            .entry_at(vi + 1)
                            .is_some_and(|e| tab.selected.contains(&e.name));
                        let is_cursor = tab.cursor == Some(vi);
                        let deco = info.decorations.get(&entry.name);
                        let dir_size = entry
                            .is_dir()
                            .then(|| tab.dir_sizes.get(&entry.name))
                            .flatten();
                        let sync_info = tab.cloud.as_ref().and_then(|c| c.get(&entry.name));
                        let sync = sync_info.map(|i| i.state);
                        let icon = row_icons.get(ui, entry, 18.0);
                        let (badge_rect, sync_rect) = paint_row(
                            ui,
                            row,
                            RowState {
                                entry,
                                index: vi,
                                selected: is_selected,
                                prev_selected: prev_sel,
                                next_selected: next_sel,
                                cursor: is_cursor && active,
                                multi,
                                active,
                                hovered: resp.hovered(),
                                dir_size,
                                sync,
                                tags: tab.tags_of(&entry.name),
                                icon,
                            },
                            &cols,
                            now,
                            deco,
                        );
                        if editing.as_deref() == Some(entry.name.as_str()) {
                            edit_row = Some(row);
                        }
                        let over_badge = badge_rect.is_some_and(|r| {
                            resp.hover_pos()
                                .is_some_and(|pos| r.expand(2.0).contains(pos))
                        });
                        let over_size = dir_size.is_some()
                            && cols.size.is_some_and(|r| {
                                resp.hover_pos().is_some_and(|pos| {
                                    pos.x >= r.min - space::LG && pos.x <= r.max + space::XS
                                })
                            });
                        let over_sync = sync_rect.is_some_and(|r| {
                            resp.hover_pos()
                                .is_some_and(|pos| r.expand(2.0).contains(pos))
                        });
                        let dragging = egui::DragAndDrop::has_any_payload(ui.ctx())
                            || external_drag_active(ui.ctx());
                        let resp = match (deco.and_then(|d| d.tooltip.as_ref()), &entry.link_target)
                        {
                            _ if dragging => resp,
                            _ if over_sync => match sync_info.and_then(crate::cloud::tooltip) {
                                Some(tip) => resp.on_hover_text(tip),
                                None => resp,
                            },
                            _ if over_size => match dir_size {
                                Some(d) => resp.on_hover_text(size_tooltip(d)),
                                None => resp,
                            },
                            (Some(tip), _) if over_badge => resp.on_hover_text(tip),
                            (Some(tip), None) if badge_rect.is_none() => resp.on_hover_text(tip),
                            (_, Some(target)) => {
                                resp.on_hover_text(t!("pane.alias_of", target = target))
                            }
                            _ => resp,
                        };
                        if resp.double_clicked() {
                            double = Some(vi);
                        } else if resp.clicked() {
                            clicked = Some((vi, resp.ctx.input(|i| i.modifiers)));
                        }
                        if resp.drag_started() {
                            drag_start = Some(vi);
                        }
                        if resp.secondary_clicked() {
                            context = Some(vi);
                        }
                        resp.context_menu(|ui| {
                            context_menu(ui, side, actions, info.context_cmds, &menu)
                        });
                    }
                    if let Some(row) = edit_row {
                        inline_rename_ui(ui, row, &cols, side, tab, actions);
                    }
                })
            },
        )
        .inner;
    tab.scroll_offset = out.state.clamped_offset().y;
    tab.viewport_h = out.inner_rect.height();
    tab.content_h = out.content_size.y;

    if let Some((vi, mods)) = clicked {
        // Click-to-rename: a plain click on the item that already was the
        // only selection of the active pane.
        let name = tab.entry_at(vi).map(|e| e.name.clone());
        let again = active
            && mods.is_none()
            && tab.selected.len() == 1
            && name.as_ref().is_some_and(|n| tab.selected.contains(n));
        tab.click(vi, mods);
        tab.rename_click = name
            .filter(|_| again)
            .map(|n| (n, std::time::Instant::now()));
    }
    if double.is_some() || drag_start.is_some() || context.is_some() {
        tab.rename_click = None;
    }
    if let Some(vi) = double {
        tab.select_only(vi);
        actions.push(Action::OpenSelected(side));
    }
    if let Some(vi) = context {
        let name = tab.entry_at(vi).map(|e| e.name.clone());
        if let Some(name) = name {
            if !tab.selected.contains(&name) {
                tab.select_only(vi);
            }
        }
    }
    if let Some(vi) = drag_start {
        let name = tab.entry_at(vi).map(|e| e.name.clone()).unwrap_or_default();
        if !tab.selected.contains(&name) {
            tab.select_only(vi);
        }
        if let Some(payload) = DragPayload::of_tab(side, tab) {
            egui::DragAndDrop::set_payload(ui.ctx(), payload);
        }
    }
}

/// Left inset of the name in the in-place editor, so the typed text sits
/// where the name was painted.
const INLINE_PAD: f32 = 4.0;

/// The in-place name editor over the name column of `row` (List view).
/// ↩ renames, Esc cancels, clicking elsewhere renames too (Finder).
fn inline_rename_ui(
    ui: &mut egui::Ui,
    row: egui::Rect,
    cols: &Columns,
    side: Side,
    tab: &mut Tab,
    actions: &mut Vec<Action>,
) {
    let id = egui::Id::unique(("inline-rename", side, tab.id));
    let Some(ir) = tab.inline_rename.as_mut() else {
        return;
    };
    let p = pal(ui);
    let r = egui::Rect::from_x_y_ranges(
        (cols.name.min + 24.0 - INLINE_PAD)..=cols.name.max,
        row.y_range(),
    )
    .shrink2(egui::vec2(0.0, 2.0));
    // ↩ / Esc while the field has focus (egui reports the lost focus a
    // frame later, when the key is no longer pressed).
    let focused = ui.memory(|m| m.has_focus(id));
    let (enter, esc) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Escape),
        )
    });
    if focused && (enter || esc) {
        actions.push(Action::InlineRenameEnd {
            side,
            commit: !esc,
            enter,
        });
        ui.memory_mut(|m| m.surrender_focus(id));
        return;
    }
    let problem = ir.form.validated().err();
    let stroke = match problem {
        Some(_) => p.danger,
        None => p.focus_ring,
    };
    ui.painter().rect(
        r,
        CornerRadius::same(radius::ROW),
        p.content,
        Stroke::new(1.5, stroke),
        egui::StrokeKind::Outside,
    );
    let resp = ui.put(
        r,
        widgets::bare_input(&mut ir.form.name, r, INLINE_PAD)
            .id(id)
            .text_color(p.text)
            // Esc stays with the field (egui would drop the focus first,
            // which reads as "clicked elsewhere": rename).
            .event_filter(egui::EventFilter {
                escape: true,
                ..Default::default()
            }),
    );
    let resp = resp.on_hover_text(t!("dialog.rename.field"));
    if !ir.started {
        resp.request_focus();
        crate::dialogs::select_name(ui.ctx(), id, &ir.form.name, true);
        ir.started = true;
    }
    if let Some(e) = &problem {
        resp.show_tooltip_text(e);
    }
    // Focus went elsewhere (a click outside): rename, like Finder.
    if resp.lost_focus() {
        actions.push(Action::InlineRenameEnd {
            side,
            commit: true,
            enter: false,
        });
    }
}

/// The AccessKit node of the file at view index `vi` (a row in every view):
/// its spoken description, selection and position; the cursor row of the
/// active pane is what VoiceOver follows.
pub(crate) fn a11y_entry(
    resp: &egui::Response,
    info: &PaneInfo<'_>,
    tab: &Tab,
    vi: usize,
    now: chrono::DateTime<chrono::Local>,
) {
    let Some(e) = tab.entry_at(vi) else {
        return;
    };
    let mut label = crate::a11y::entry_label(
        e,
        now,
        e.is_dir().then(|| tab.dir_sizes.get(&e.name)).flatten(),
        tab.cloud
            .as_ref()
            .and_then(|c| c.get(&e.name))
            .map(|i| i.state),
        info.decorations.get(&e.name),
    );
    if let Some(t) = crate::tags::spoken(tab.tags_of(&e.name)) {
        label.push_str(", ");
        label.push_str(&t);
    }
    crate::a11y::file_item(
        resp,
        &label,
        tab.selected.contains(&e.name),
        vi,
        tab.view.len(),
    );
    if info.is_active && tab.cursor == Some(vi) {
        crate::a11y::set_active_descendant(&resp.ctx, resp.id);
        let row = resp.id.accesskit_id();
        if let Some(anchor) = resp
            .ctx
            .data(|d| d.get_temp::<egui::Id>(anchor_key(info.side)))
        {
            resp.ctx
                .accesskit_node_builder(anchor, |n| n.set_active_descendant(row));
        }
    }
}

/// The accessible name of a pane's file list: "Files in Projects".
pub(crate) fn list_name(tab: &Tab) -> String {
    t!("a11y.file_list", folder = tab.title())
}

/// The file list's single Tab stop (the background behind the rows, in
/// every view). While it has the keyboard focus the pane is active and the
/// file list keys work as usual (arrows, Return, Space, typing; see
/// [`list_has_focus`]); Tab moves on to the next control. VoiceOver follows
/// the cursor row (the list's active descendant, see [`a11y_entry`]).
pub(crate) fn list_anchor(bg: &egui::Response, side: Side, tab: &Tab, actions: &mut Vec<Action>) {
    crate::a11y::name(bg, egui::accesskit::Role::List, &list_name(tab));
    let ctx = &bg.ctx;
    ctx.data_mut(|d| d.insert_temp(anchor_key(side), bg.id));
    // The arrow keys move the cursor, not the focus.
    ctx.memory_mut(|m| {
        m.set_focus_lock_filter(
            bg.id,
            egui::EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                tab: false,
                escape: false,
            },
        )
    });
    if bg.has_focus() {
        ctx.data_mut(|d| d.insert_temp(focused_anchor_key(), bg.id));
        if bg.gained_focus() {
            actions.push(Action::Activate(side));
        }
    }
}

fn anchor_key(side: Side) -> egui::Id {
    egui::Id::unique(("manta-list-anchor", side.index()))
}

fn focused_anchor_key() -> egui::Id {
    egui::Id::unique("manta-focused-list-anchor")
}

/// The file list Tab stop that had the focus most recently.
pub(crate) fn list_anchor_focus(ctx: &egui::Context) -> Option<egui::Id> {
    ctx.data(|d| d.get_temp::<egui::Id>(focused_anchor_key()))
}

/// Does a file list (its Tab stop) hold the keyboard focus? Then the file
/// list shortcuts apply as if no control had focus.
pub(crate) fn list_has_focus(ctx: &egui::Context) -> bool {
    let focused = ctx.memory(|m| m.focused());
    focused.is_some() && focused == ctx.data(|d| d.get_temp::<egui::Id>(focused_anchor_key()))
}

/// An archive being opened or a listing error: drawn over the whole list
/// area (every view). Returns whether one was drawn.
pub(crate) fn blocking_state(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    side: Side,
    tab: &mut Tab,
    actions: &mut Vec<Action>,
) -> bool {
    let p = pal(ui);
    if let Some(o) = &tab.opening {
        opening_overlay(ui, rect, side, o, actions);
        return true;
    }

    if let Some(err) = tab.error.clone() {
        // The Trash without Full Disk Access: say what to do about it.
        if crate::trash::is_permission_error(&err) && crate::app::shows_trash(ui.ctx(), &tab.loc) {
            let buttons = [
                (t!("trash.fda.open_settings"), ButtonKind::Primary),
                (t!("trash.fda.show_in_finder"), ButtonKind::Secondary),
            ];
            match widgets::empty_state(
                ui,
                rect,
                icons::LOCK_SIMPLE,
                p.text_tertiary,
                t!("trash.fda.title"),
                t!("trash.fda.body"),
                &buttons,
            ) {
                Some(0) => actions.push(Action::OpenFullDiskAccess),
                Some(1) => actions.push(Action::ShowTrashInFinder),
                _ => {}
            }
            return true;
        }
        // A stopped Docker container: start it rather than try again (a pod
        // can't be started; it keeps Try Again).
        let docker_down = tab.loc.fs.protocol() == manta_core::Protocol::Docker
            && err.to_lowercase().contains("container is not running");
        let (title, body) = if docker_down {
            (
                t!("connect.err.container_down"),
                t!("pane.err.container_down.body"),
            )
        } else {
            friendly_listing_error(&err)
        };
        let primary = match (docker_down, tab.container_starting) {
            (true, true) => t!("pane.starting_container"),
            (true, false) => t!("pane.start_container"),
            _ => t!("pane.try_again"),
        };
        let mut buttons = vec![(primary, ButtonKind::Primary)];
        let back = !tab.back.is_empty();
        if back {
            buttons.push((t!("pane.go_back"), ButtonKind::Secondary));
        }
        // The plain explanation is enough when there is one (the path is in
        // the breadcrumb); otherwise the backend's message is all we have.
        let detail = if body.is_empty() {
            crate::i18n::localize_error(&err)
        } else {
            body.to_string()
        };
        match widgets::empty_state(
            ui,
            rect,
            icons::WARNING_CIRCLE,
            p.danger,
            title,
            &detail,
            &buttons,
        ) {
            Some(0) if docker_down => {
                if !tab.container_starting {
                    actions.push(Action::StartContainer(side));
                }
            }
            Some(0) => actions.push(Action::Refresh(side)),
            Some(1) => actions.push(Action::Back(side)),
            _ => {}
        }
        return true;
    }

    false
}

/// Loading, "no match" and "empty folder" states of an empty view.
/// Returns whether the view is empty (and a state was drawn).
pub(crate) fn empty_states(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    side: Side,
    tab: &mut Tab,
    actions: &mut Vec<Action>,
) -> bool {
    let p = pal(ui);
    if !tab.view.is_empty() {
        return false;
    }

    if tab.loading {
        if tab
            .loading_since
            .is_some_and(|t| t.elapsed() > Duration::from_millis(200))
        {
            let c = body_rect.center() - egui::vec2(0.0, 30.0);
            ui.put(
                egui::Rect::from_center_size(c, egui::vec2(24.0, 24.0)),
                egui::Spinner::new().size(22.0).color(p.text_secondary),
            );
            ui.painter().text(
                c + egui::vec2(0.0, 26.0),
                egui::Align2::CENTER_TOP,
                t!("pane.loading", name = tab.title()),
                theme::regular(text::BODY),
                p.text_secondary,
            );
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(220));
        }
    } else if let Some(s) = &tab.search {
        if s.running {
            let c = body_rect.center() - egui::vec2(0.0, 30.0);
            ui.put(
                egui::Rect::from_center_size(c, egui::vec2(24.0, 24.0)),
                egui::Spinner::new().size(22.0).color(p.text_secondary),
            );
            ui.painter().text(
                c + egui::vec2(0.0, 26.0),
                egui::Align2::CENTER_TOP,
                t!("search.searching_title"),
                theme::regular(text::BODY),
                p.text_secondary,
            );
        } else {
            let title = t!("search.no_results");
            // Nothing below this folder: offer the whole Mac.
            let offer_mac = s.scope == crate::search::Scope::Subfolders
                && crate::search::mac_available(&tab.loc);
            let buttons: Vec<(&str, ButtonKind)> = if offer_mac {
                vec![(t!("search.try_mac"), ButtonKind::Secondary)]
            } else {
                Vec::new()
            };
            if widgets::empty_state(
                ui,
                body_rect,
                icons::MAGNIFYING_GLASS,
                p.text_tertiary,
                title,
                "",
                &buttons,
            )
            .is_some()
            {
                actions.push(Action::SetSearchScope(side, crate::search::Scope::Mac));
            }
        }
    } else if !tab.filter.is_empty() {
        let title = t!("pane.no_match", filter = tab.filter.trim());
        if widgets::empty_state(
            ui,
            body_rect,
            icons::MAGNIFYING_GLASS,
            p.text_tertiary,
            &title,
            "",
            &[(t!("pane.clear_filter"), ButtonKind::Secondary)],
        )
        .is_some()
        {
            tab.filter.clear();
            tab.rebuild_view();
        }
    } else {
        let hidden = util::listable_count(&tab.entries);
        let read_only = tab.loc.read_only();
        let body = if hidden > 0 && !tab.show_hidden {
            t!(
                "pane.hidden_items",
                count = hidden,
                shortcut = Cmd::ToggleHidden.shortcut()
            )
        } else if read_only {
            t!("pane.archive_folder_empty").to_string()
        } else {
            t!("pane.empty_hint", shortcut = Cmd::NewFolder.shortcut())
        };
        let buttons: Vec<(&str, ButtonKind)> = if read_only {
            Vec::new()
        } else {
            vec![(t!("pane.new_folder"), ButtonKind::Secondary)]
        };
        if widgets::empty_state(
            ui,
            body_rect,
            icons::FOLDER_DASHED,
            p.text_tertiary,
            t!("pane.empty_title"),
            &body,
            &buttons,
        )
        .is_some()
        {
            actions.push(Action::NewFolder(side));
        }
    }
    true
}

/// An archive being downloaded / indexed: progress card in the pane.
fn opening_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    side: Side,
    o: &ArchiveOpening,
    actions: &mut Vec<Action>,
) {
    let p = pal(ui);
    ui.ctx().request_repaint_after(Duration::from_millis(100));
    if o.since.elapsed() < Duration::from_millis(150) {
        return;
    }
    let downloading = o.downloading.load(Ordering::Relaxed);
    let done = o.done.load(Ordering::Relaxed);
    let card = egui::Rect::from_center_size(
        rect.center() - egui::vec2(0.0, 20.0),
        egui::vec2(rect.width().min(320.0) - 24.0, 150.0),
    );
    ui.painter()
        .rect_filled(card, CornerRadius::same(radius::WINDOW), p.elevated);
    ui.painter().rect_stroke(
        card,
        CornerRadius::same(radius::WINDOW),
        Stroke::new(1.0, p.separator),
        StrokeKind::Inside,
    );
    let top = card.top() + space::LG;
    let icon_r = egui::Rect::from_center_size(
        egui::pos2(card.center().x, top + 14.0),
        egui::vec2(28.0, 28.0),
    );
    ui.put(
        icon_r,
        egui::Spinner::new().size(22.0).color(p.text_secondary),
    );
    let mut title = egui::text::LayoutJob::simple_singleline(
        t!("pane.opening", name = o.name),
        theme::medium(text::BODY),
        p.text,
    );
    title.wrap = egui::text::TextWrapping::truncate_at_width(card.width() - 2.0 * space::LG);
    let g = ui.painter().layout_job(title);
    ui.painter().galley(
        egui::pos2(card.center().x - g.size().x / 2.0, top + 36.0),
        g,
        p.text,
    );
    let detail = if downloading {
        if o.total > 0 {
            t!(
                "pane.downloading_of",
                done = util::format_size(done),
                total = util::format_size(o.total)
            )
        } else {
            t!("pane.downloading", done = util::format_size(done))
        }
    } else {
        t!("pane.reading_archive").to_string()
    };
    ui.painter().text(
        egui::pos2(card.center().x, top + 60.0),
        egui::Align2::CENTER_TOP,
        detail,
        theme::regular(text::SMALL),
        p.text_secondary,
    );
    // Progress line.
    let track = egui::Rect::from_min_size(
        egui::pos2(card.left() + space::XL, top + 82.0),
        egui::vec2(card.width() - 2.0 * space::XL, 4.0),
    );
    ui.painter()
        .rect_filled(track, CornerRadius::same(2), p.stripe);
    if o.total > 0 {
        let f = (done as f32 / o.total as f32).clamp(0.0, 1.0);
        let fill =
            egui::Rect::from_min_size(track.min, egui::vec2(track.width() * f, track.height()));
        ui.painter()
            .rect_filled(fill, CornerRadius::same(2), p.accent);
    }
    let br = egui::Rect::from_center_size(
        egui::pos2(card.center().x, card.bottom() - space::LG - 10.0),
        egui::vec2(90.0, 24.0),
    );
    let clicked = ui
        .scope_builder(
            egui::UiBuilder::new()
                .max_rect(br)
                .layout(egui::Layout::top_down(egui::Align::Center)),
            |ui| widgets::button(ui, t!("common.cancel"), ButtonKind::Secondary, true).clicked(),
        )
        .inner;
    if clicked {
        actions.push(Action::CancelArchiveOpen(side));
    }
}

fn header_row(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    cols: &Columns,
    side: Side,
    tab: &mut Tab,
    widths: &mut [f32; 3],
) {
    let p = pal(ui);
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, p.separator),
    );
    let entries: [(&str, Option<egui::Rangef>, SortColumn, bool); 5] = [
        (
            t!("pane.col.name"),
            Some(egui::Rangef::new(cols.name.min + 24.0, cols.name.max)),
            SortColumn::Name,
            false,
        ),
        (t!("pane.col.size"), cols.size, SortColumn::Size, true),
        (
            t!("pane.col.modified"),
            cols.modified,
            SortColumn::Modified,
            false,
        ),
        (
            t!("pane.col.permissions"),
            cols.perms,
            SortColumn::Permissions,
            false,
        ),
        // Search results: where each one is (sized with the pane).
        (t!("search.col.path"), cols.path, SortColumn::Path, false),
    ];
    for (idx, (label, range, col, right)) in entries.into_iter().enumerate() {
        let Some(range) = range else { continue };
        let r = egui::Rect::from_x_y_ranges(
            egui::Rangef::new(range.min - 4.0, range.max + 4.0),
            rect.y_range(),
        );
        let resp = ui.interact(r, egui::Id::unique(("hdr", side, idx)), Sense::click());
        let sorted = tab.sort.column == col;
        if resp.hovered() {
            ui.painter().rect_filled(
                r.shrink2(egui::vec2(0.0, 3.0)),
                CornerRadius::same(radius::CONTROL),
                p.hover,
            );
        }
        let color = if sorted { p.text } else { p.text_secondary };
        let font = if sorted {
            theme::semibold(text::CAPTION)
        } else {
            theme::medium(text::CAPTION)
        };
        let g = ui.painter().layout_no_wrap(label.to_string(), font, color);
        let caret_w = if sorted { 12.0 } else { 0.0 };
        let x = if right {
            range.max - g.size().x - caret_w
        } else {
            range.min
        };
        let gw = g.size().x;
        ui.painter()
            .galley(egui::pos2(x, rect.center().y - g.size().y / 2.0), g, color);
        if sorted {
            let glyph = if tab.sort.ascending {
                icons::CARET_UP
            } else {
                icons::CARET_DOWN
            };
            let cr = egui::Rect::from_center_size(
                egui::pos2(x + gw + 7.0, rect.center().y),
                egui::vec2(10.0, 10.0),
            );
            widgets::paint_icon(ui, cr, glyph, 9.0, p.accent_text, false);
        }
        let resp = widgets::tooltip(resp, &t!("pane.sort_by", column = label), "");
        if resp.clicked() {
            tab.sort = tab.sort.clicked(col);
            tab.rebuild_view();
        }
        // Resize handle on the left edge of the fixed-width columns.
        if (1..=3).contains(&idx) {
            let h = egui::Rect::from_x_y_ranges(
                egui::Rangef::new(range.min - 9.0, range.min - 3.0),
                rect.y_range(),
            );
            let hr = ui.interact(h, egui::Id::unique(("hdr-resize", side, idx)), Sense::DRAG);
            if hr.hovered() || hr.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                ui.painter().vline(
                    h.center().x,
                    rect.y_range().shrink(5.0),
                    Stroke::new(1.0, p.border),
                );
            }
            if hr.dragged() {
                let w = &mut widths[idx - 1];
                *w = (*w - hr.drag_delta().x).clamp(50.0, 320.0);
            }
            if hr.double_clicked() {
                widths[idx - 1] = [76.0, 120.0, 92.0][idx - 1];
            }
        }
    }
}

struct RowState<'a> {
    entry: &'a Entry,
    index: usize,
    selected: bool,
    prev_selected: bool,
    next_selected: bool,
    cursor: bool,
    multi: bool,
    active: bool,
    hovered: bool,
    /// Plugin-measured size of a folder.
    dir_size: Option<&'a SizeInfo>,
    /// Sync state in a cloud folder (badge for cloud-only items etc.).
    sync: Option<manta_vfs::cloud_local::SyncState>,
    /// Finder tags (coloured dots after the name).
    tags: &'a [crate::tags::Tag],
    /// The app's own icon (app bundles), drawn instead of the glyph.
    icon: Option<egui::TextureHandle>,
}

/// Tooltip of a measured folder size: both sizes and the file count.
pub fn size_tooltip(s: &SizeInfo) -> String {
    let mut lines = vec![t!("pane.size.files", count = s.files)];
    lines.push(format!(
        "{} ({})",
        util::format_size(s.bytes),
        manta_plugin::i18n::format_bytes(s.bytes)
    ));
    if let Some(d) = s.on_disk {
        lines.push(t!("pane.size.on_disk", size = util::format_size(d)));
    }
    if !s.complete {
        lines.push(t!("pane.size.counting").into());
    } else if s.unreadable > 0 {
        lines.push(t!("pane.size.unreadable", count = s.unreadable));
    }
    lines.join("\n")
}

/// Text of a measured folder size in the Size column.
pub fn size_label(s: &SizeInfo) -> String {
    if !s.complete {
        format!("≥ {}…", util::format_size(s.bytes))
    } else if s.unreadable > 0 {
        format!("≥ {}", util::format_size(s.bytes))
    } else {
        util::format_size(s.bytes)
    }
}

/// Paint one file row. Returns the rect of the plugin badges, if any.
fn paint_row(
    ui: &egui::Ui,
    row: egui::Rect,
    s: RowState<'_>,
    cols: &Columns,
    now: chrono::DateTime<chrono::Local>,
    deco: Option<&MergedDecoration>,
) -> (Option<egui::Rect>, Option<egui::Rect>) {
    let p: &Palette = pal(ui);
    let painter = ui.painter();
    let e = s.entry;
    if s.index % 2 == 1 {
        painter.rect_filled(row, CornerRadius::ZERO, p.stripe);
    }
    let inset = row.shrink2(egui::vec2(ROW_INSET, 0.0));
    let rr = radius::ROW;
    if s.selected {
        let r = CornerRadius {
            nw: if s.prev_selected { 0 } else { rr },
            ne: if s.prev_selected { 0 } else { rr },
            sw: if s.next_selected { 0 } else { rr },
            se: if s.next_selected { 0 } else { rr },
        };
        painter.rect_filled(
            inset,
            r,
            if s.active {
                p.selection
            } else {
                p.selection_inactive
            },
        );
    } else if s.hovered {
        painter.rect_filled(inset, CornerRadius::same(rr), p.hover);
    }
    if s.cursor && (!s.selected || s.multi) {
        let c = if s.selected {
            p.text_on_accent.gamma_multiply(0.7)
        } else {
            p.focus_ring
        };
        painter.rect_stroke(
            inset.shrink(1.0),
            CornerRadius::same(rr),
            Stroke::new(1.5, c),
            StrokeKind::Inside,
        );
    }

    let on_accent = s.selected && s.active;
    let hidden = e.hidden || e.name.starts_with('.');
    let (glyph, tint, fill) = icons::entry_icon(e, p);
    let mut icon_color = if on_accent { p.text_on_accent } else { tint };
    let mut name_color = match deco.and_then(|d| d.name_tone) {
        _ if on_accent => p.text_on_accent,
        Some(tone) => theme::tone_text(p, tone),
        None => p.text,
    };
    let mut sub_color = if on_accent {
        p.text_on_accent.gamma_multiply(0.85)
    } else {
        p.text_secondary
    };
    if hidden {
        icon_color = icon_color.gamma_multiply(0.5);
        name_color = if on_accent {
            p.text_on_accent.gamma_multiply(0.75)
        } else {
            p.text_tertiary
        };
    }
    // A plugin dims the row (e.g. git-ignored files): fade every part.
    let dim = if deco.is_some_and(|d| d.dimmed) {
        if on_accent {
            0.75
        } else {
            0.5
        }
    } else {
        1.0
    };
    if dim < 1.0 {
        icon_color = icon_color.gamma_multiply(dim);
        name_color = name_color.gamma_multiply(dim);
        sub_color = sub_color.gamma_multiply(dim);
    }

    let cy = row.center().y;
    let ir =
        egui::Rect::from_center_size(egui::pos2(cols.name.min + 9.0, cy), egui::vec2(18.0, 18.0));
    match &s.icon {
        Some(tex) => {
            let tint = Color32::WHITE.gamma_multiply(if hidden { 0.5 * dim } else { dim });
            painter.image(
                tex.id(),
                ir,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        }
        None => widgets::paint_icon(ui, ir, glyph, size::ICON, icon_color, fill),
    }
    // Backends report a link to a folder as a folder with `link_target` set.
    if e.kind == manta_core::EntryKind::Symlink || e.link_target.is_some() {
        let b = egui::Rect::from_center_size(
            ir.right_bottom() - egui::vec2(2.0, 2.0),
            egui::vec2(10.0, 10.0),
        );
        let bg = if on_accent { p.selection } else { p.content };
        painter.circle_filled(b.center(), 5.0, bg);
        widgets::paint_icon(
            ui,
            b,
            icons::ARROW_U_UP_LEFT,
            8.0,
            if on_accent {
                p.text_on_accent
            } else {
                p.text_secondary
            },
            false,
        );
    }

    let text_left = cols.name.min + 24.0;
    let badges: &[manta_plugin::Badge] = deco.map_or(&[], |d| d.badges.as_slice());
    // Badges follow the name; a long name is truncated to make room.
    let badges_w = plugin_ui::badges_width(ui, badges);
    let mut reserve = if badges_w > 0.0 {
        badges_w + space::SM
    } else {
        0.0
    };
    // Cloud sync badge at the right end of the name column (like Finder).
    let sync_badge = s.sync.and_then(crate::cloud::badge);
    let sync_rect = sync_badge.map(|_| {
        egui::Rect::from_center_size(egui::pos2(cols.name.max - 10.0, cy), egui::vec2(16.0, 16.0))
    });
    if sync_rect.is_some() {
        reserve += 20.0;
    }
    // Finder tags: coloured dots right after the name.
    const DOT: f32 = 9.0;
    let dots_w = crate::tags::dots_width(s.tags, DOT);
    if dots_w > 0.0 {
        reserve += dots_w + space::SM;
    }
    let mut job = egui::text::LayoutJob::simple_singleline(
        util::display_name(&e.name).to_string(),
        theme::regular(text::BODY),
        name_color,
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(
        (cols.name.max - text_left - reserve).max(10.0),
    );
    let g = painter.layout_job(job);
    let mut name_w = g.size().x;
    painter.galley(egui::pos2(text_left, cy - g.size().y / 2.0), g, name_color);
    if dots_w > 0.0 {
        // On the selection the dots get a light ring (a blue tag stays
        // visible on the blue row).
        let ring = if s.selected {
            if s.active {
                p.text_on_accent
            } else {
                p.selection_inactive
            }
        } else if s.index % 2 == 1 {
            p.stripe
        } else {
            p.content
        };
        crate::tags::paint_dots(
            painter,
            text_left + name_w + space::SM,
            cy,
            s.tags,
            DOT,
            ring,
            p.dark,
        );
        name_w += dots_w + space::SM;
    }
    let badge_rect = (badges_w > 0.0 && cols.name.max - text_left > badges_w + 24.0).then(|| {
        plugin_ui::paint_badges(
            ui,
            text_left + name_w + space::SM,
            cy,
            badges,
            on_accent,
            dim,
        )
    });
    if let (Some(r), Some(st)) = (sync_rect, s.sync) {
        let c = if on_accent {
            p.text_on_accent
        } else {
            p.text_secondary
        };
        crate::cloud::paint_badge(ui, r, st, 14.0, c.gamma_multiply(dim));
    }

    let small = theme::regular(text::SMALL + 0.5);
    if let (Some(r), Some(ds)) = (cols.size, s.dir_size) {
        let c = if ds.complete {
            sub_color
        } else if on_accent {
            sub_color.gamma_multiply(0.8)
        } else {
            p.text_tertiary
        };
        // While counting: "≥ 1.2 GB" with a tiny spinner, or "≥ 1.2 GB…"
        // when the column is too narrow for the spinner.
        let mut g = painter.layout_no_wrap(size_label(ds), small.clone(), c);
        let mut spinner = false;
        if !ds.complete {
            let short = painter.layout_no_wrap(
                format!("≥ {}", util::format_size(ds.bytes)),
                small.clone(),
                c,
            );
            if short.size().x + 14.0 <= r.span() {
                g = short;
                spinner = true;
            }
        }
        let gw = g.size().x;
        let gh = g.size().y;
        painter.galley(egui::pos2(r.max - gw, cy - gh / 2.0), g, c);
        if spinner {
            // A tiny spinner left of the value while counting.
            let center = egui::pos2(r.max - gw - 8.0, cy);
            let t = ui.input(|i| i.time) as f32;
            let start = t * 5.0;
            let pts: Vec<egui::Pos2> = (0..=12)
                .map(|k| {
                    let a = start + k as f32 / 12.0 * 4.4;
                    center + 3.5 * egui::vec2(a.cos(), a.sin())
                })
                .collect();
            painter.add(egui::Shape::line(pts, Stroke::new(1.2, c)));
            ui.ctx().request_repaint_after(Duration::from_millis(60));
        }
    } else if let Some(r) = cols.size {
        let (s_text, c) = if e.is_dir() {
            (
                "—".to_string(),
                if on_accent {
                    sub_color
                } else {
                    p.text_tertiary
                },
            )
        } else {
            (util::format_size(e.size), sub_color)
        };
        painter.text(
            egui::pos2(r.max, cy),
            egui::Align2::RIGHT_CENTER,
            s_text,
            small.clone(),
            c,
        );
    }
    if let Some(r) = cols.modified {
        let mut job = egui::text::LayoutJob::simple_singleline(
            util::format_time_relative(e.modified, now),
            small.clone(),
            sub_color,
        );
        job.wrap = egui::text::TextWrapping::truncate_at_width(r.span());
        let g = painter.layout_job(job);
        painter.galley(egui::pos2(r.min, cy - g.size().y / 2.0), g, sub_color);
    }
    if let Some(r) = cols.perms {
        let c = if on_accent {
            sub_color
        } else {
            p.text_secondary
        };
        painter.text(
            egui::pos2(r.min, cy),
            egui::Align2::LEFT_CENTER,
            util::format_permissions(e.permissions, e.kind),
            theme::mono(text::CAPTION),
            c,
        );
    }
    if let Some(r) = cols.path {
        // The folder the result is in, cut from the start ("…/src/app").
        let g = widgets::truncate_start(
            ui,
            &crate::search::where_label(&e.name, &cols.root),
            small,
            sub_color,
            r.span(),
        );
        painter.galley(egui::pos2(r.min, cy - g.size().y / 2.0), g, sub_color);
    }
    (badge_rect, sync_rect)
}

/// Turn a backend error into a title and a plain-language explanation.
pub fn friendly_listing_error(err: &str) -> (&'static str, &'static str) {
    let e = err.to_lowercase();
    if e.starts_with("not found") || e.contains("no such file") {
        (t!("pane.err.not_found"), t!("pane.err.not_found.body"))
    } else if e.contains("permission denied") {
        (t!("pane.err.permission"), t!("pane.err.permission.body"))
    } else if e.starts_with("not a directory") {
        (t!("pane.err.not_dir"), t!("pane.err.not_dir.body"))
    } else if e.contains("connection")
        || e.contains("broken pipe")
        || e.contains("timed out")
        || e.contains("reset")
    {
        (t!("pane.err.connection"), t!("pane.err.connection.body"))
    } else {
        (t!("pane.err.generic"), "")
    }
}

fn menu_item(
    ui: &mut egui::Ui,
    cmd: Cmd,
    enabled: bool,
    actions: &mut Vec<Action>,
    action: Action,
) {
    let btn = egui::Button::new(format!("{}   {}", cmd.icon(), cmd.label()))
        .shortcut_text(cmd.shortcut());
    if ui.add_enabled(enabled, btn).clicked() {
        actions.push(action);
        ui.close();
    }
}

/// What the context menu needs to know about the pane and its selection.
pub(crate) struct MenuCtx {
    /// The pane shows an archive (read-only).
    read_only: bool,
    /// The folder that contains the archive can be written to.
    outer_writable: bool,
    /// Exactly one archive file is selected.
    archive_selected: bool,
    /// The pane shows an archive.
    archive: bool,
    /// The pane shows a running container: offer a shell inside it.
    container_shell: bool,
    /// The only pane: nothing to copy, move or extract to.
    single: bool,
    /// Items can go to the Trash (local, writable).
    trash: bool,
    /// "Download Now" / "Remove Download" (cloud folders).
    cloud: crate::cloud::MenuOffer,
    /// The Tags submenu (items on this Mac).
    tags: Option<crate::tags::TagMenu>,
    /// The pane shows search results ("Show in Enclosing Folder").
    search: bool,
    /// The one folder right-clicked: "New Folder / New File in" it.
    into: Option<(VfsPath, String)>,
}

impl MenuCtx {
    /// What the cloud part of the menu offers (tests).
    #[cfg(test)]
    pub(crate) fn cloud_offer(&self) -> crate::cloud::MenuOffer {
        self.cloud
    }

    pub(crate) fn of(ctx: &egui::Context, tab: &Tab, single: bool) -> MenuCtx {
        // Built every frame: references only (no clones of the selection).
        let sum = tab.selection_summary();
        let sel: Vec<&Entry> = if sum.view_idx.is_empty() {
            tab.cursor_entry().into_iter().collect()
        } else if sum.view_idx.len() == 1 || tab.cloud.is_some() {
            sum.view_idx
                .iter()
                .filter_map(|&vi| tab.entry_at(vi))
                .collect()
        } else {
            Vec::new() // only the count matters below
        };
        MenuCtx {
            read_only: tab.loc.read_only(),
            outer_writable: tab
                .loc
                .archive
                .as_ref()
                .is_some_and(|m| !m.outer.read_only()),
            archive_selected: matches!(sel.as_slice(), [e] if !e.is_dir()
                && manta_vfs::ArchiveFormat::detect(&e.name).is_some()),
            archive: tab.loc.archive.is_some(),
            container_shell: tab.loc.fs.protocol().is_container()
                && !tab.loc.read_only()
                && tab.loc.archive.is_none(),
            single,
            // An archive is read-only anyway: name the usual command.
            // In the Trash itself: "Delete Immediately…".
            trash: (crate::app::can_trash(&tab.loc.fs) || tab.loc.archive.is_some())
                && !crate::app::shows_trash(ctx, &tab.loc),
            cloud: match tab.cloud.as_deref() {
                Some(states) => crate::cloud::menu_offer(
                    Some(states),
                    &sel.iter()
                        .map(|e| (e.name.clone(), e.is_dir()))
                        .collect::<Vec<_>>(),
                ),
                None => Default::default(),
            },
            tags: tab.tag_menu(ctx),
            search: tab.in_search(),
            into: match sel.as_slice() {
                [e] if e.is_dir() && !crate::thumbs::is_app_bundle(e) && !tab.in_search() => {
                    Some((e.path.clone(), e.name.clone()))
                }
                _ => None,
            },
        }
    }

    /// The Tags submenu's state (tests).
    #[cfg(test)]
    pub(crate) fn tag_menu(&self) -> Option<&crate::tags::TagMenu> {
        self.tags.as_ref()
    }
}

fn menu_item_ro(
    ui: &mut egui::Ui,
    cmd: Cmd,
    read_only: bool,
    actions: &mut Vec<Action>,
    action: Action,
) {
    let btn = egui::Button::new(format!("{}   {}", cmd.icon(), cmd.label()))
        .shortcut_text(cmd.shortcut());
    let resp = ui
        .add_enabled(!read_only, btn)
        .on_disabled_hover_text(t!("pane.read_only_tip"));
    if resp.clicked() {
        actions.push(action);
        ui.close();
    }
}

fn plain_item(
    ui: &mut egui::Ui,
    glyph: &str,
    label: &str,
    shortcut: &str,
    enabled: bool,
    actions: &mut Vec<Action>,
    action: Action,
) {
    let btn = egui::Button::new(format!("{glyph}   {label}")).shortcut_text(shortcut);
    if ui.add_enabled(enabled, btn).clicked() {
        actions.push(action);
        ui.close();
    }
}

/// A menu item that needs a second pane: greyed out (with the reason) when
/// there is only one.
fn two_pane_item(
    ui: &mut egui::Ui,
    glyph: &str,
    label: &str,
    shortcut: &str,
    single: bool,
    actions: &mut Vec<Action>,
    action: Action,
) {
    let btn = egui::Button::new(format!("{glyph}   {label}")).shortcut_text(shortcut);
    let resp = ui
        .add_enabled(!single, btn)
        .on_disabled_hover_text(t!("panes.needs_two"));
    if resp.clicked() {
        actions.push(action);
        ui.close();
    }
}

pub(crate) fn context_menu(
    ui: &mut egui::Ui,
    side: Side,
    actions: &mut Vec<Action>,
    plugin_cmds: &[ContextCmd],
    m: &MenuCtx,
) {
    ui.set_min_width(220.0);
    menu_item(ui, Cmd::Open, true, actions, Action::OpenSelected(side));
    if m.archive_selected {
        plain_item(
            ui,
            icons::FILE_ARCHIVE,
            t!("pane.menu.browse_archive"),
            "",
            true,
            actions,
            Action::BrowseArchive(side),
        );
        menu_item(
            ui,
            Cmd::OpenWithApp,
            true,
            actions,
            Action::OpenWithApp(side),
        );
    }
    menu_item(ui, Cmd::QuickLook, true, actions, Action::Preview(side));
    menu_item(ui, Cmd::GetInfo, true, actions, Action::GetInfo(side));
    if m.search {
        plain_item(
            ui,
            icons::FOLDER_OPEN,
            t!("search.reveal"),
            Cmd::EnclosingFolder.shortcut(),
            true,
            actions,
            Action::RevealResult(side),
        );
    }
    if let Some(menu) = &m.tags {
        let mut edit = None;
        ui.menu_button(format!("{}   {}", icons::TAG, t!("tags.menu")), |ui| {
            edit = crate::tags::tags_menu(ui, menu)
        });
        if let Some(e) = edit {
            actions.push(Action::SetTag(side, e));
        }
    }
    if m.cloud.download || m.cloud.evict {
        ui.separator();
        if m.cloud.download {
            plain_item(
                ui,
                icons::CLOUD_ARROW_DOWN,
                t!("pane.menu.cloud_download"),
                "",
                true,
                actions,
                Action::CloudDownload(side),
            );
        }
        if m.cloud.evict {
            plain_item(
                ui,
                icons::CLOUD_SLASH,
                t!("pane.menu.cloud_evict"),
                "",
                true,
                actions,
                Action::CloudEvict(side),
            );
        }
    }
    ui.separator();
    if m.archive {
        two_pane_item(
            ui,
            Cmd::CopyToOther.icon(),
            t!("pane.menu.extract_to_other_ellipsis"),
            Cmd::CopyToOther.shortcut(),
            m.single,
            actions,
            Action::CopyToOther(side, false),
        );
    } else {
        two_pane_item(
            ui,
            Cmd::CopyToOther.icon(),
            Cmd::CopyToOther.label(),
            Cmd::CopyToOther.shortcut(),
            m.single,
            actions,
            Action::CopyToOther(side, false),
        );
    }
    if m.single && !m.read_only {
        two_pane_item(
            ui,
            Cmd::MoveToOther.icon(),
            Cmd::MoveToOther.label(),
            Cmd::MoveToOther.shortcut(),
            true,
            actions,
            Action::CopyToOther(side, true),
        );
    } else {
        menu_item_ro(
            ui,
            Cmd::MoveToOther,
            m.read_only,
            actions,
            Action::CopyToOther(side, true),
        );
    }
    menu_item_ro(ui, Cmd::Rename, m.read_only, actions, Action::Rename(side));
    ui.separator();
    if m.archive {
        let btn = egui::Button::new(format!(
            "{}   {}",
            Cmd::ExtractHere.icon(),
            Cmd::ExtractHere.label()
        ))
        .shortcut_text(Cmd::ExtractHere.shortcut());
        let resp = ui
            .add_enabled(m.outer_writable, btn)
            .on_hover_text(t!("pane.menu.extract_here_tip"))
            .on_disabled_hover_text(t!("pane.menu.extract_nested_tip"));
        if resp.clicked() {
            actions.push(Action::ExtractHere(side));
            ui.close();
        }
    } else if m.archive_selected {
        menu_item(
            ui,
            Cmd::ExtractHere,
            true,
            actions,
            Action::ExtractHere(side),
        );
        two_pane_item(
            ui,
            icons::ARCHIVE,
            t!("pane.menu.extract_to_other"),
            "",
            m.single,
            actions,
            Action::ExtractToOther(side),
        );
    }
    if !m.read_only {
        menu_item(ui, Cmd::Compress, true, actions, Action::Compress(side));
    }
    match &m.into {
        // A folder: create in it (the shortcuts keep creating next to it).
        Some((dir, name)) => {
            for (cmd, label, file) in [
                (
                    Cmd::NewFolder,
                    t!("pane.menu.new_folder_in", name = name),
                    false,
                ),
                (Cmd::NewFile, t!("pane.menu.new_file_in", name = name), true),
            ] {
                let btn = egui::Button::new(format!("{}   {}", cmd.icon(), label));
                let resp = ui
                    .add_enabled(!m.read_only, btn)
                    .on_disabled_hover_text(t!("pane.read_only_tip"));
                if resp.clicked() {
                    actions.push(Action::NewIn {
                        side,
                        dir: dir.clone(),
                        file,
                    });
                    ui.close();
                }
            }
        }
        None => {
            menu_item_ro(
                ui,
                Cmd::NewFolder,
                m.read_only,
                actions,
                Action::NewFolder(side),
            );
            menu_item_ro(
                ui,
                Cmd::NewFile,
                m.read_only,
                actions,
                Action::NewFile(side),
            );
        }
    }
    if m.container_shell {
        menu_item(
            ui,
            Cmd::OpenShellInContainer,
            true,
            actions,
            Action::OpenContainerShell(side),
        );
    }
    if !plugin_cmds.is_empty() {
        ui.separator();
        let item =
            |ui: &mut egui::Ui, c: &ContextCmd, glyph: Option<&str>, actions: &mut Vec<Action>| {
                let label = match glyph {
                    Some(g) => format!("{g}   {}", c.title),
                    None => c.title.clone(),
                };
                let btn = egui::Button::new(label).shortcut_text(c.shortcut.as_str());
                if ui.add(btn).clicked() {
                    actions.push(Action::RunPluginCommand(c.id.clone()));
                    ui.close();
                }
            };
        for (plugin, cmds) in group_context_cmds(plugin_cmds) {
            if cmds.len() > MAX_INLINE_PLUGIN_CMDS {
                ui.menu_button(format!("{}   {plugin}", icons::PUZZLE_PIECE), |ui| {
                    ui.set_min_width(220.0);
                    for c in cmds {
                        item(ui, c, None, actions);
                    }
                });
            } else {
                for c in cmds {
                    item(ui, c, Some(icons::PUZZLE_PIECE), actions);
                }
            }
        }
    }
    ui.separator();
    // Local items go to the Trash; servers have none (permanent, asks).
    if m.trash {
        menu_item_ro(
            ui,
            Cmd::MoveToTrash,
            m.read_only,
            actions,
            Action::Delete(side),
        );
    } else {
        menu_item_ro(
            ui,
            Cmd::DeletePermanently,
            m.read_only,
            actions,
            Action::DeletePermanently(side),
        );
    }
}

pub(crate) fn background_menu(
    ui: &mut egui::Ui,
    side: Side,
    actions: &mut Vec<Action>,
    m: &MenuCtx,
) {
    let read_only = m.read_only;
    ui.set_min_width(220.0);
    menu_item_ro(
        ui,
        Cmd::NewFolder,
        read_only,
        actions,
        Action::NewFolder(side),
    );
    menu_item_ro(ui, Cmd::NewFile, read_only, actions, Action::NewFile(side));
    menu_item(ui, Cmd::Refresh, true, actions, Action::Refresh(side));
    if m.archive {
        menu_item(
            ui,
            Cmd::ExtractHere,
            true,
            actions,
            Action::ExtractHere(side),
        );
    }
    if m.container_shell {
        menu_item(
            ui,
            Cmd::OpenShellInContainer,
            true,
            actions,
            Action::OpenContainerShell(side),
        );
    }
    ui.separator();
    menu_item(
        ui,
        Cmd::ToggleHidden,
        true,
        actions,
        Action::ToggleHidden(side),
    );
    menu_item_ro(
        ui,
        Cmd::AddToFavorites,
        m.archive,
        actions,
        Action::AddCurrentToFavorites,
    );
}

/// Copy or move for a drop of `payload` (copy unless ⌘ is held, see
/// [`crate::native_drag::drop_is_move`]). Archives are only extracted and
/// read-only sources only copied.
pub(crate) fn internal_drop_moves(ctx: &egui::Context, payload: &DragPayload) -> bool {
    if payload.fs.is_read_only() {
        return false;
    }
    let (alt, cmd) = crate::native_drag::drag_modifiers(ctx);
    crate::native_drag::drop_is_move(crate::native_drag::drops_move_by_default(ctx), alt, cmd)
}

#[derive(Clone, Copy, Default)]
struct BadgeDrawn(u64);

fn badge_drawn_id() -> egui::Id {
    egui::Id::unique("dnd-badge-drawn")
}

/// Accept files dragged from another pane.
fn drop_target(
    ui: &mut egui::Ui,
    side: Side,
    is_last: bool,
    rect: egui::Rect,
    dst: &Location,
    actions: &mut Vec<Action>,
) {
    let ctx = ui.ctx().clone();
    let Some(payload) = egui::DragAndDrop::payload::<DragPayload>(&ctx) else {
        return;
    };
    let read_only = dst.read_only();
    let p = pal(ui);
    let pointer = ctx.input(|i| i.pointer.hover_pos());
    let hovering = pointer.is_some_and(|pos| rect.contains(pos));
    let target_here = hovering && payload.from != side && !read_only;
    let extract = payload.fs.is_read_only();
    let mv = internal_drop_moves(&ctx, &payload);
    let n = payload.paths.len();
    let what = t!("pane.items", count = n);

    // Badge next to the cursor while dragging: drawn by the pane under the
    // pointer when it takes the drop (copy or move depends on it), else
    // once by the last pane.
    let frame = ctx.cumulative_frame_nr();
    let drawn = ctx
        .data(|d| d.get_temp::<BadgeDrawn>(badge_drawn_id()))
        .is_some_and(|b| b.0 == frame);
    if let Some(pos) = pointer.filter(|_| !drawn && (target_here || is_last)) {
        ctx.data_mut(|d| d.insert_temp(badge_drawn_id(), BadgeDrawn(frame)));
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::unique("dnd-badge"),
        ));
        let fav = crate::sidebar::favorites_drop_hover(&ctx);
        let trash = crate::sidebar::trash_drop_hover(&ctx);
        let text = if let Some(t) = trash {
            t.badge()
        } else if let Some(n) = fav {
            if n == 0 {
                t!("pane.drag.fav_folders_only").to_string()
            } else {
                t!("pane.drag.fav_add", count = n)
            }
        } else if extract {
            t!("pane.drag.extract", what = what)
        } else if mv {
            t!("pane.drag.move", what = what)
        } else {
            t!("pane.drag.copy", what = what)
        };
        // Files over the favorites: a neutral pill (they are refused).
        let refused = fav == Some(0) || trash.is_some_and(|t| t.refused());
        let (bg, fg) = if refused {
            (p.elevated, p.text_secondary)
        } else {
            (p.accent, p.text_on_accent)
        };
        let galley = painter.layout_no_wrap(text, theme::medium(text::SMALL), fg);
        let r = egui::Rect::from_min_size(pos + egui::vec2(14.0, 10.0), galley.size())
            .expand2(egui::vec2(8.0, 4.0));
        painter.rect_filled(r, CornerRadius::same(radius::PILL), bg);
        if refused {
            painter.rect_stroke(
                r,
                CornerRadius::same(radius::PILL),
                Stroke::new(1.0, p.separator),
                StrokeKind::Inside,
            );
        }
        painter.galley(r.min + egui::vec2(8.0, 4.0), galley, fg);
    }

    if payload.from == side || !hovering {
        return;
    }
    if read_only {
        // An archive pane: nothing can be dropped here.
        read_only_drop_hint(ui, rect);
        if ctx.input(|i| i.pointer.any_released()) {
            egui::DragAndDrop::clear_payload(&ctx);
            actions.push(Action::DropFiles {
                from: payload.from,
                to: side,
                fs: payload.fs.clone(),
                paths: payload.paths.clone(),
                mv,
            });
        }
        return;
    }
    let hint = if extract {
        t!("pane.drop.extract", what = what)
    } else if mv {
        t!("pane.drop.move", what = what)
    } else {
        t!("pane.drop.copy", what = what)
    };
    drop_highlight(ui, rect, None, &hint);
    if ctx.input(|i| i.pointer.any_released()) {
        if let Some(payload) = egui::DragAndDrop::take_payload::<DragPayload>(&ctx) {
            actions.push(Action::DropFiles {
                from: payload.from,
                to: side,
                fs: payload.fs.clone(),
                paths: payload.paths.clone(),
                mv,
            });
        }
    }
}

/// "Read-only archive — can't drop here" over a pane.
fn read_only_drop_hint(ui: &egui::Ui, rect: egui::Rect) {
    let p = pal(ui);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius::PANEL),
        Stroke::new(2.0, p.border),
        StrokeKind::Inside,
    );
    let hint = format!("{}  {}", icons::LOCK_SIMPLE, t!("pane.drop.read_only"));
    let galley = ui
        .painter()
        .layout_no_wrap(hint, theme::medium(text::BODY), p.text);
    let br = egui::Rect::from_center_size(
        rect.center_bottom() - egui::vec2(0.0, 40.0),
        galley.size() + egui::vec2(24.0, 12.0),
    );
    ui.painter()
        .rect_filled(br, CornerRadius::same(radius::PILL), p.elevated);
    ui.painter().rect_stroke(
        br,
        CornerRadius::same(radius::PILL),
        Stroke::new(1.0, p.separator),
        StrokeKind::Inside,
    );
    ui.painter()
        .galley(br.center() - galley.size() / 2.0, galley, p.text);
}

/// The drop highlight: the pane (`rect`) gets an accent ring, or, when the
/// drop goes into one of its folders, that folder's row (`folder`) does.
/// `hint` is shown as a pill at the bottom of the pane.
fn drop_highlight(ui: &egui::Ui, rect: egui::Rect, folder: Option<egui::Rect>, hint: &str) {
    let p = pal(ui);
    match folder {
        None => {
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(radius::PANEL),
                p.accent.gamma_multiply(0.07),
            );
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(radius::PANEL),
                Stroke::new(2.0, p.accent),
                StrokeKind::Inside,
            );
        }
        Some(f) => {
            let painter = ui.painter().with_clip_rect(rect);
            painter.rect_filled(
                f,
                CornerRadius::same(radius::ROW),
                p.accent.gamma_multiply(0.18),
            );
            painter.rect_stroke(
                f,
                CornerRadius::same(radius::ROW),
                Stroke::new(2.0, p.accent),
                StrokeKind::Inside,
            );
        }
    }
    let galley = ui.painter().layout_no_wrap(
        hint.to_string(),
        theme::medium(text::BODY),
        p.text_on_accent,
    );
    let br = egui::Rect::from_center_size(
        rect.center_bottom() - egui::vec2(0.0, 40.0),
        galley.size() + egui::vec2(24.0, 12.0),
    );
    ui.painter()
        .rect_filled(br, CornerRadius::same(radius::PILL), p.accent);
    ui.painter()
        .galley(br.center() - galley.size() / 2.0, galley, p.text_on_accent);
}

// ---------------------------------------------------------------- drag in

/// Folders the views painted this frame (for drops from other apps).
#[derive(Clone, Default)]
struct FolderTargets(u64, Vec<(egui::Rect, VfsPath)>);

fn folder_targets_id(side: Side) -> egui::Id {
    egui::Id::unique(("external-folder-targets", side))
}

/// Files from another app hover over the window or were just dropped.
pub(crate) fn external_drag_active(ctx: &egui::Context) -> bool {
    ctx.input(|i| !i.raw.hovered_files.is_empty() || !i.raw.dropped_files.is_empty())
}

/// A view painted the folder `path` at `rect`: files from another app
/// dropped onto it go into that folder (instead of the pane's folder).
/// Cheap no-op unless such a drag is in progress.
pub(crate) fn note_folder_target(ui: &egui::Ui, side: Side, rect: egui::Rect, path: &VfsPath) {
    let ctx = ui.ctx();
    if !external_drag_active(ctx) {
        return;
    }
    let r = rect.intersect(ui.clip_rect());
    if !r.is_positive() {
        return;
    }
    let frame = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| {
        let t = d.get_temp_mut_or_default::<FolderTargets>(folder_targets_id(side));
        if t.0 != frame {
            *t = FolderTargets(frame, Vec::new());
        }
        t.1.push((r, path.clone()));
    });
}

/// The folder under `pos` that a view registered this frame (the smallest
/// one: a folder row wins over a Columns column).
fn folder_target_at(
    ctx: &egui::Context,
    side: Side,
    pos: egui::Pos2,
) -> Option<(egui::Rect, VfsPath)> {
    let frame = ctx.cumulative_frame_nr();
    let t = ctx.data(|d| d.get_temp::<FolderTargets>(folder_targets_id(side)))?;
    if t.0 != frame {
        return None;
    }
    t.1.into_iter()
        .filter(|(r, _)| r.contains(pos))
        .min_by(|a, b| a.0.area().total_cmp(&b.0.area()))
}

/// Tests: the folders the views registered for `side` in the last frame
/// with an external drag.
#[cfg(test)]
pub(crate) fn folder_targets(ctx: &egui::Context, side: Side) -> Vec<(egui::Rect, VfsPath)> {
    ctx.data(|d| d.get_temp::<FolderTargets>(folder_targets_id(side)))
        .map(|t| t.1)
        .unwrap_or_default()
}

/// Where a drop from another app at `pos` goes: a folder the views
/// registered, else the pane's folder. Returns the highlight rect of the
/// folder (`None`: the whole pane) and the target directory.
fn external_target(
    ctx: &egui::Context,
    side: Side,
    pos: egui::Pos2,
    loc: &Location,
) -> (Option<egui::Rect>, VfsPath) {
    match folder_target_at(ctx, side, pos) {
        Some((r, path)) => (Some(r), path),
        None => (None, loc.path.clone()),
    }
}

/// Copy or move files from another app (copy unless ⌘ is held).
fn external_drop_moves(ctx: &egui::Context) -> bool {
    let (alt, cmd) = crate::native_drag::drag_modifiers(ctx);
    crate::native_drag::drop_is_move(crate::native_drag::drops_move_by_default(ctx), alt, cmd)
}

/// Accept files dragged in from another app (Finder, Mail, …): highlight
/// the pane or the folder under the pointer while they hover, and turn the
/// drop into [`Action::DropExternal`].
fn external_drop_target(
    ui: &mut egui::Ui,
    side: Side,
    rect: egui::Rect,
    loc: &Location,
    actions: &mut Vec<Action>,
) {
    let ctx = ui.ctx().clone();
    if let Some(h) = crate::native_drag::external_hover(&ctx) {
        if let Some(pos) = h.pos.filter(|p| rect.contains(*p)) {
            if loc.read_only() {
                read_only_drop_hint(ui, rect);
            } else {
                let (folder, dir) = external_target(&ctx, side, pos, loc);
                let mv = external_drop_moves(&ctx);
                let what = t!("pane.items", count = h.paths.len());
                let hint = match (folder.is_some(), mv) {
                    (false, false) => t!("pane.drop.copy", what = what),
                    (false, true) => t!("pane.drop.move", what = what),
                    (true, false) => {
                        t!("pane.drop.copy_into", what = what, folder = dir.file_name())
                    }
                    (true, true) => {
                        t!("pane.drop.move_into", what = what, folder = dir.file_name())
                    }
                };
                drop_highlight(ui, rect, folder, &hint);
            }
        }
    }
    if let Some(d) = crate::native_drag::take_external_drop_in(&ctx, rect) {
        let pos = d.pos.unwrap_or(rect.center());
        let (_, dir) = external_target(&ctx, side, pos, loc);
        let mv = !loc.read_only() && external_drop_moves(&ctx);
        actions.push(Action::DropExternal {
            to: side,
            dir,
            paths: d.paths,
            mv,
            from_self: d.from_self,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use manta_core::EntryKind;

    fn entry(name: &str) -> Entry {
        Entry {
            name: name.into(),
            path: VfsPath::new(&format!("/d/{name}")),
            kind: EntryKind::File,
            size: 1,
            modified: None,
            permissions: None,
            link_target: None,
            hidden: false,
        }
    }

    fn tab() -> Tab {
        let fs: SharedVfs = std::sync::Arc::new(manta_vfs::LocalFs::new());
        Tab::new(
            1,
            Location {
                fs,
                conn: None,
                path: VfsPath::new("/d"),
                archive: None,
            },
        )
    }

    #[test]
    fn reload_with_fewer_entries_does_not_panic() {
        let mut t = tab();
        let five: Vec<Entry> = ["a", "b", "c", "d", "e"].iter().map(|n| entry(n)).collect();
        t.apply_listing(VfsPath::new("/d"), five);
        t.move_cursor(1, false);
        t.move_cursor(10, false); // cursor on "e"
        t.move_cursor(-2, true); // anchor "e", range c..e
        assert_eq!(t.cursor_entry().map(|e| e.name.as_str()), Some("c"));
        t.apply_listing(VfsPath::new("/d"), vec![entry("a"), entry("c")]);
        assert!(t.cursor.is_none_or(|c| c < t.view.len()));
        assert_eq!(
            t.cursor_entry().map(|e| e.name.as_str()),
            Some("c"),
            "cursor follows its name"
        );
        assert!(t.anchor.is_none_or(|a| a < t.view.len()));
        assert!(t.selected.iter().all(|n| n == "c"));
        // Range selection and keyboard moves stay in bounds.
        t.move_cursor(5, true);
        t.click(1, egui::Modifiers::SHIFT);
        assert!(t.selected_entries().len() <= 2);

        t.apply_listing(VfsPath::new("/d"), Vec::new());
        assert!(t.cursor.is_none() && t.anchor.is_none() && t.selected.is_empty());
        t.move_cursor(1, true);
        assert!(t.entry_at(0).is_none());
    }

    #[test]
    fn stale_view_is_harmless() {
        let mut t = tab();
        t.apply_listing(VfsPath::new("/d"), vec![entry("a"), entry("b"), entry("c")]);
        // Simulate entries swapped without rebuilding the view.
        t.entries.truncate(1);
        assert!(t.entry_at(2).is_none());
        assert_eq!(t.view_index_of("c"), None);
        t.select_all();
        t.rebuild_view();
        assert_eq!(t.view.len(), 1);
    }
}
