//! Finder-like search across folders.
//!
//! ⌘F filters the current folder ("This Folder"). The scope bar under the
//! path bar switches to
//! - **Subfolders**: everything below the folder. On this Mac a parallel
//!   walker reads the tree (hidden items only when shown, no package
//!   contents); on servers, containers and in archives the folders are
//!   listed one by one, down to [`REMOTE_MAX_DEPTH`] levels and at most
//!   [`REMOTE_MAX_DIRS`] folders.
//! - **This Mac**: Spotlight (`mdfind`), names and contents
//!   (`kMDItemTextContent`), only for local folders.
//!
//! Names match as a substring (case-insensitive) or, with `*` / `?`, as a
//! wildcard pattern over the whole name. `tag:Name` terms (see
//! [`crate::tags::split_filter`]) find tagged items; the sidebar's tags
//! search This Mac for one tag.
//!
//! Results arrive while the search runs and fill the pane as a virtual
//! location ("Search: …", with a Path column); the search stops when the
//! query is cleared, the scope goes back to This Folder, the pane
//! navigates elsewhere, or with the stop button. Each result is named by
//! its path relative to the folder searched (unique in the list, see
//! [`crate::util::display_name`]).
//!
//! Spotlight goes through [`Spotlight`], so tests use a fake and never run
//! `mdfind`.

use std::collections::{HashMap, VecDeque};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use eframe::egui::{self, CornerRadius, Sense};
use manta_core::{Entry, SharedVfs, Vfs, VfsPath};

use crate::app::{Action, Side};
use crate::pane::Tab;
use crate::tags::{self, Tag};
use crate::theme::{self, pal, radius, size, space, text};
use crate::widgets;

/// Where a search looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scope {
    /// Filter the folder shown (⌘F, no search).
    #[default]
    Folder,
    /// The folder and everything below it.
    Subfolders,
    /// Spotlight: the whole Mac, names and contents.
    Mac,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Scope::Folder => t!("search.scope.folder"),
            Scope::Subfolders => t!("search.scope.subfolders"),
            Scope::Mac => t!("search.scope.mac"),
        }
    }
}

/// What a search looks for.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Query {
    /// The filter text: a name (or pattern) and `tag:` terms.
    Text(String),
    /// Everything with this tag (the sidebar's Tags).
    Tag(String),
}

impl Query {
    /// The words shown in the breadcrumb ("“report”", "Tag „Rot“").
    pub fn title(&self) -> String {
        match self {
            Query::Text(q) => t!("search.title", query = q.trim()),
            Query::Tag(name) => t!("search.title_tag", name = name),
        }
    }
}

/// Most results a search collects.
pub const MAX_RESULTS: usize = 10_000;
/// Servers: how deep a Subfolders search goes.
pub const REMOTE_MAX_DEPTH: usize = 12;
/// Servers: how many folders a Subfolders search lists at most.
pub const REMOTE_MAX_DIRS: usize = 5_000;
/// Typing pauses this long before the search (re)starts.
pub const DEBOUNCE: Duration = Duration::from_millis(250);
/// Results are handed over in batches of this size (or per folder).
const BATCH: usize = 64;

// ------------------------------------------------------------------ matching

/// Name matching: substring, or a `*` / `?` pattern over the whole name.
/// Case-insensitive.
#[derive(Debug, Clone)]
pub enum NameMatcher {
    Any,
    Substring(String),
    Pattern(Vec<char>),
}

impl NameMatcher {
    pub fn new(text: &str) -> NameMatcher {
        let t = text.trim().to_lowercase();
        if t.is_empty() {
            NameMatcher::Any
        } else if t.contains(['*', '?']) {
            NameMatcher::Pattern(t.chars().collect())
        } else {
            NameMatcher::Substring(t)
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        match self {
            NameMatcher::Any => true,
            NameMatcher::Substring(s) => name.to_lowercase().contains(s.as_str()),
            NameMatcher::Pattern(p) => {
                let n: Vec<char> = name.to_lowercase().chars().collect();
                glob(p, &n)
            }
        }
    }
}

/// `*` any run, `?` one character (iterative, no exponential backtracking).
fn glob(p: &[char], s: &[char]) -> bool {
    let (mut pi, mut si) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = si;
            pi += 1;
        } else if let Some(st) = star {
            pi = st + 1;
            mark += 1;
            si = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// The name part and the tag terms of a query.
struct Terms {
    name: NameMatcher,
    raw_name: String,
    tags: Vec<String>,
}

impl Terms {
    fn of(q: &Query) -> Terms {
        match q {
            Query::Text(t) => {
                let (tags, rest) = tags::split_filter(t);
                Terms {
                    name: NameMatcher::new(&rest),
                    raw_name: rest,
                    tags,
                }
            }
            Query::Tag(name) => Terms {
                name: NameMatcher::Any,
                raw_name: String::new(),
                tags: vec![name.to_lowercase()],
            },
        }
    }
}

// ------------------------------------------------------------------ Spotlight

/// Spotlight queries (a seam: tests use a fake).
pub trait Spotlight: Send + Sync {
    /// Run `query` (Spotlight's query language), `only_in` a folder or the
    /// whole Mac; hand each path to `found` until it returns `false`, the
    /// query is done or `cancel` is set.
    fn search(
        &self,
        query: &str,
        only_in: Option<&Path>,
        cancel: Arc<AtomicBool>,
        found: &mut dyn FnMut(PathBuf) -> bool,
    ) -> Result<(), String>;
}

/// `/usr/bin/mdfind -0`, killed when cancelled.
#[cfg_attr(test, allow(dead_code))]
pub struct MdFind;

impl Spotlight for MdFind {
    fn search(
        &self,
        query: &str,
        only_in: Option<&Path>,
        cancel: Arc<AtomicBool>,
        found: &mut dyn FnMut(PathBuf) -> bool,
    ) -> Result<(), String> {
        use std::os::unix::ffi::OsStrExt;
        let mut cmd = std::process::Command::new("/usr/bin/mdfind");
        cmd.arg("-0");
        if let Some(d) = only_in {
            cmd.arg("-onlyin").arg(d);
        }
        cmd.arg(query)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        let mut child = cmd.spawn().map_err(|e| format!("mdfind: {e}"))?;
        let stdout = child.stdout.take().ok_or("mdfind: no output")?;
        let child = Arc::new(Mutex::new(child));
        let done = Arc::new(AtomicBool::new(false));
        // mdfind may sit on a slow index: kill it as soon as the search is
        // cancelled, which also ends the read below.
        {
            let child = child.clone();
            let done = done.clone();
            let cancel = cancel.clone();
            let _ = std::thread::Builder::new()
                .name("manta-mdfind-watch".into())
                .spawn(move || {
                    while !done.load(Ordering::Relaxed) {
                        if cancel.load(Ordering::Relaxed) {
                            if let Ok(mut c) = child.lock() {
                                let _ = c.kill();
                            }
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(40));
                    }
                });
        }
        let mut r = std::io::BufReader::new(stdout);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match r.read_until(0, &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if buf.last() == Some(&0) {
                buf.pop();
            }
            if buf.is_empty() {
                continue;
            }
            let p = PathBuf::from(std::ffi::OsStr::from_bytes(&buf));
            if cancel.load(Ordering::Relaxed) || !found(p) {
                if let Ok(mut c) = child.lock() {
                    let _ = c.kill();
                }
                break;
            }
        }
        done.store(true, Ordering::Relaxed);
        if let Ok(mut c) = child.lock() {
            let _ = c.wait();
        }
        Ok(())
    }
}

/// Test Spotlight: answers every query with `results` (and records it).
#[cfg(test)]
#[derive(Default)]
pub struct FakeSpotlight {
    pub results: Mutex<Vec<PathBuf>>,
    pub queries: Mutex<Vec<(String, Option<PathBuf>)>>,
}

#[cfg(test)]
impl Spotlight for FakeSpotlight {
    fn search(
        &self,
        query: &str,
        only_in: Option<&Path>,
        cancel: Arc<AtomicBool>,
        found: &mut dyn FnMut(PathBuf) -> bool,
    ) -> Result<(), String> {
        self.queries
            .lock()
            .unwrap()
            .push((query.to_string(), only_in.map(Path::to_path_buf)));
        let results = self.results.lock().unwrap().clone();
        for p in results {
            if cancel.load(Ordering::Relaxed) || !found(p) {
                break;
            }
        }
        Ok(())
    }
}

/// A string in a Spotlight query.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        if matches!(c, '"' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// The Spotlight query for `q`: names (and, for plain words, contents),
/// tags.
pub fn spotlight_query(q: &Query) -> String {
    let terms = Terms::of(q);
    let mut parts = Vec::new();
    let name = terms.raw_name.trim();
    if !name.is_empty() {
        if name.contains(['*', '?']) {
            parts.push(format!("kMDItemFSName == {}c", quote(name)));
        } else {
            parts.push(format!(
                "(kMDItemDisplayName == {}cd || kMDItemTextContent == {}cdw)",
                quote(&format!("*{name}*")),
                quote(&format!("{name}*"))
            ));
        }
    }
    let original: Vec<String> = match q {
        Query::Tag(n) => vec![n.clone()],
        Query::Text(t) => {
            // Keep the case the user typed (Spotlight's `c` ignores it anyway).
            let (lower, _) = tags::split_filter(t);
            lower
        }
    };
    for t in original {
        parts.push(format!("kMDItemUserTags == {}c", quote(&t)));
    }
    parts.join(" && ")
}

// ------------------------------------------------------------------ results

/// One result: the entry (named by its relative path) and its tags.
#[derive(Debug, Clone)]
pub struct Hit {
    pub entry: Entry,
    pub tags: Vec<Tag>,
}

pub enum Msg {
    Found(Vec<Hit>),
    Done {
        truncated: bool,
        error: Option<String>,
    },
}

/// What to search.
pub struct Spec {
    pub query: Query,
    pub scope: Scope,
    pub fs: SharedVfs,
    pub root: VfsPath,
    /// The root on this Mac (`None`: a server, container or archive).
    pub local_root: Option<PathBuf>,
    pub show_hidden: bool,
}

/// A running (or finished) search of a tab.
pub struct Search {
    pub query: Query,
    pub scope: Scope,
    pub running: bool,
    /// Results so far.
    pub found: usize,
    /// Stopped at a limit ([`MAX_RESULTS`], the server limits).
    pub truncated: bool,
    /// Stopped by the user.
    pub stopped: bool,
    pub error: Option<String>,
    pub cancel: Arc<AtomicBool>,
    rx: Receiver<Msg>,
    /// The results' tags, by name (for the dots and `tag:` filters).
    pub tags: Arc<tags::FolderTags>,
}

impl Search {
    /// Start searching in the background.
    pub fn start(spec: Spec, ctx: &egui::Context, spotlight: Arc<dyn Spotlight>) -> Search {
        let (tx, rx) = channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let s = Search {
            query: spec.query.clone(),
            scope: spec.scope,
            running: true,
            found: 0,
            truncated: false,
            stopped: false,
            error: None,
            cancel: cancel.clone(),
            rx,
            tags: Arc::new(HashMap::new()),
        };
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("manta-search".into())
            .spawn(move || {
                let out = Out {
                    tx: tx.clone(),
                    ctx: ctx.clone(),
                    count: AtomicUsize::new(0),
                    full: AtomicBool::new(false),
                };
                let error = run(&spec, &out, &cancel, &*spotlight).err();
                let truncated = out.full.load(Ordering::Relaxed);
                let _ = tx.send(Msg::Done { truncated, error });
                ctx.request_repaint();
            });
        if let Err(e) = spawned {
            log::warn!("search: can't start: {e}");
        }
        s
    }

    /// Stop the search (keeps the results found so far).
    pub fn stop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if self.running {
            self.stopped = true;
        }
    }

    /// Take the results that arrived. Returns them and whether the search
    /// has finished since the last call.
    pub fn poll(&mut self) -> (Vec<Hit>, bool) {
        let mut hits = Vec::new();
        let mut finished = false;
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Found(h) => hits.extend(h),
                Msg::Done { truncated, error } => {
                    self.running = false;
                    self.truncated = truncated;
                    self.error = error;
                    finished = true;
                }
            }
        }
        if !hits.is_empty() {
            self.found += hits.len();
            let tags = Arc::make_mut(&mut self.tags);
            for h in &hits {
                if !h.tags.is_empty() {
                    tags.insert(h.entry.name.clone(), h.tags.clone());
                }
            }
        }
        (hits, finished)
    }

    /// Wait until the search is done (tests).
    #[cfg(test)]
    pub fn wait(&mut self, timeout: Duration) -> Vec<Hit> {
        let start = Instant::now();
        let mut all = Vec::new();
        while self.running && start.elapsed() < timeout {
            let (h, _) = self.poll();
            all.extend(h);
            std::thread::sleep(Duration::from_millis(2));
        }
        all.extend(self.poll().0);
        all
    }
}

impl Drop for Search {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Where the search threads put their results.
struct Out {
    tx: Sender<Msg>,
    ctx: egui::Context,
    count: AtomicUsize,
    /// [`MAX_RESULTS`] reached.
    full: AtomicBool,
}

impl Out {
    /// Hand over a batch; `false` once the result limit is reached.
    fn send(&self, mut hits: Vec<Hit>) -> bool {
        if hits.is_empty() {
            return !self.full.load(Ordering::Relaxed);
        }
        let before = self.count.fetch_add(hits.len(), Ordering::Relaxed);
        if before >= MAX_RESULTS {
            self.full.store(true, Ordering::Relaxed);
            return false;
        }
        if before + hits.len() > MAX_RESULTS {
            hits.truncate(MAX_RESULTS - before);
            self.full.store(true, Ordering::Relaxed);
        }
        let _ = self.tx.send(Msg::Found(hits));
        self.ctx.request_repaint();
        !self.full.load(Ordering::Relaxed)
    }
}

fn run(
    spec: &Spec,
    out: &Out,
    cancel: &Arc<AtomicBool>,
    spotlight: &dyn Spotlight,
) -> Result<(), String> {
    let terms = Terms::of(&spec.query);
    let spotlight_scope = matches!(spec.query, Query::Tag(_)) || spec.scope == Scope::Mac;
    if spotlight_scope {
        return run_spotlight(spec, out, cancel, spotlight);
    }
    match &spec.local_root {
        Some(root) => {
            walk_local(root, &terms, spec.show_hidden, cancel, out);
            Ok(())
        }
        None => walk_remote(spec, &terms, cancel, out),
    }
}

/// An entry for a local result, named `name`.
fn local_hit(path: &Path, name: String) -> Option<Hit> {
    let vp = VfsPath::new(&path.to_string_lossy());
    let mut entry = manta_vfs::LocalFs::new().stat(&vp).ok()?;
    entry.name = name;
    let tags = tags::read_tags(path).unwrap_or_default();
    Some(Hit { entry, tags })
}

fn run_spotlight(
    spec: &Spec,
    out: &Out,
    cancel: &Arc<AtomicBool>,
    spotlight: &dyn Spotlight,
) -> Result<(), String> {
    let query = spotlight_query(&spec.query);
    if query.is_empty() {
        return Ok(());
    }
    let mut batch = Vec::new();
    let mut last = Instant::now();
    let mut more = true;
    spotlight.search(&query, None, cancel.clone(), &mut |p| {
        // Hidden items (Spotlight leaves hidden folders out by itself).
        let hidden = p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'));
        if hidden && !spec.show_hidden {
            return true;
        }
        let name = p.to_string_lossy().into_owned();
        if let Some(h) = local_hit(&p, name) {
            batch.push(h);
        }
        if batch.len() >= BATCH || last.elapsed() > Duration::from_millis(100) {
            more = out.send(std::mem::take(&mut batch));
            last = Instant::now();
        }
        more && !cancel.load(Ordering::Relaxed)
    })?;
    if more {
        out.send(batch);
    }
    Ok(())
}

/// The walker's shared queue: folders to read and how many are being read.
struct Queue {
    dirs: Vec<PathBuf>,
    busy: usize,
}

/// Parallel walk of a local tree (several threads share a queue of
/// folders). Symlinks aren't followed, packages aren't entered, hidden
/// items only with `show_hidden`.
fn walk_local(root: &Path, terms: &Terms, show_hidden: bool, cancel: &AtomicBool, out: &Out) {
    let queue = Mutex::new(Queue {
        dirs: vec![root.to_path_buf()],
        busy: 0,
    });
    let wake = Condvar::new();
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .clamp(2, 8);
    let stop = AtomicBool::new(false);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let dir = {
                    let mut q = queue.lock().unwrap();
                    loop {
                        if cancel.load(Ordering::Relaxed) || stop.load(Ordering::Relaxed) {
                            wake.notify_all();
                            return;
                        }
                        if let Some(d) = q.dirs.pop() {
                            q.busy += 1;
                            break d;
                        }
                        if q.busy == 0 {
                            wake.notify_all();
                            return;
                        }
                        q = wake.wait_timeout(q, Duration::from_millis(50)).unwrap().0;
                    }
                };
                let (subdirs, hits) = read_dir_matches(root, &dir, terms, show_hidden, cancel);
                if !out.send(hits) {
                    stop.store(true, Ordering::Relaxed);
                }
                let mut q = queue.lock().unwrap();
                q.dirs.extend(subdirs);
                q.busy -= 1;
                wake.notify_all();
            });
        }
    });
}

/// One folder of the walk: its sub folders (to walk) and its matches.
fn read_dir_matches(
    root: &Path,
    dir: &Path,
    terms: &Terms,
    show_hidden: bool,
    cancel: &AtomicBool,
) -> (Vec<PathBuf>, Vec<Hit>) {
    let mut subdirs = Vec::new();
    let mut hits = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return (subdirs, hits);
    };
    for e in rd.flatten() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let name = e.file_name();
        let name = name.to_string_lossy();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let path = e.path();
        let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
        if is_dir && !crate::util::is_package(&name) {
            subdirs.push(path.clone());
        }
        if !terms.name.matches(&name) {
            continue;
        }
        if !terms.tags.is_empty() {
            let t = tags::read_tags(&path).unwrap_or_default();
            if !tags::matches_tags(&t, &terms.tags) {
                continue;
            }
        }
        let rel = path
            .strip_prefix(root)
            .map(|r| r.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned());
        if let Some(h) = local_hit(&path, rel) {
            hits.push(h);
        }
    }
    (subdirs, hits)
}

/// Breadth-first search on a backend without a local path (servers,
/// containers, archives), within [`REMOTE_MAX_DEPTH`] / [`REMOTE_MAX_DIRS`].
fn walk_remote(spec: &Spec, terms: &Terms, cancel: &AtomicBool, out: &Out) -> Result<(), String> {
    let fs = &spec.fs;
    let root = &spec.root;
    let mut queue: VecDeque<(VfsPath, usize)> = VecDeque::from([(root.clone(), 0)]);
    let mut listed = 0;
    let mut first = true;
    while let Some((dir, depth)) = queue.pop_front() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        if listed >= REMOTE_MAX_DIRS {
            out.full.store(true, Ordering::Relaxed);
            break;
        }
        listed += 1;
        let entries = match fs.list(&dir) {
            Ok(e) => e,
            // The folder itself can't be read: say so; below it, skip.
            Err(e) if first => return Err(e.to_string()),
            Err(_) => continue,
        };
        first = false;
        let mut hits = Vec::new();
        for e in entries {
            if !spec.show_hidden && (e.hidden || e.name.starts_with('.')) {
                continue;
            }
            if e.is_dir() && e.link_target.is_none() {
                if depth + 1 < REMOTE_MAX_DEPTH {
                    queue.push_back((e.path.clone(), depth + 1));
                } else {
                    out.full.store(true, Ordering::Relaxed);
                }
            }
            // Servers have no tags.
            if !terms.tags.is_empty() || !terms.name.matches(&e.name) {
                continue;
            }
            let rel = e
                .path
                .as_str()
                .strip_prefix(root.as_str())
                .map(|r| r.trim_start_matches('/').to_string())
                .filter(|r| !r.is_empty())
                .unwrap_or_else(|| e.path.to_string());
            let mut entry = e;
            entry.name = rel;
            hits.push(Hit {
                entry,
                tags: Vec::new(),
            });
        }
        if !out.send(hits) {
            break;
        }
    }
    Ok(())
}

/// The search for `tab`'s folder (`None` when the query is empty).
pub fn spec_for(
    query: Query,
    scope: Scope,
    loc: &crate::pane::Location,
    show_hidden: bool,
) -> Option<Spec> {
    if let Query::Text(t) = &query {
        if t.trim().is_empty() {
            return None;
        }
    }
    let local_root = (loc.conn.is_none() && loc.archive.is_none())
        .then(|| loc.fs.local_path(&loc.path))
        .flatten();
    Some(Spec {
        query,
        scope,
        fs: loc.fs.clone(),
        root: loc.path.clone(),
        local_root,
        show_hidden,
    })
}

/// The name of the folder searched (results directly in it are "in" it).
pub fn root_label(loc: &crate::pane::Location) -> String {
    let n = loc.path.file_name();
    if n.is_empty() {
        "/".to_string()
    } else {
        n.to_string()
    }
}

/// The Path column of a result: the folder it is in, relative to the
/// folder searched (`src/app`), or absolute with `~` for the home folder
/// (This Mac); `root` for results directly in the folder searched.
pub fn where_label(name: &str, root: &str) -> String {
    let dir = crate::util::display_dir(name);
    if dir.is_empty() {
        return root.to_string();
    }
    if dir.starts_with('/') {
        if let Some(home) = dirs::home_dir() {
            let h = home.to_string_lossy();
            if let Some(rest) = dir.strip_prefix(h.as_ref()) {
                if rest.is_empty() || rest.starts_with('/') {
                    return format!("~{rest}");
                }
            }
        }
    }
    dir.to_string()
}

/// Whether Spotlight (This Mac) can be offered for `loc`: local folders.
pub fn mac_available(loc: &crate::pane::Location) -> bool {
    loc.conn.is_none() && loc.archive.is_none() && loc.fs.local_path(&loc.path).is_some()
}

// ------------------------------------------------------------------ UI

/// Height of the scope bar under the path bar.
pub const SCOPE_BAR_H: f32 = 32.0;

/// While the filter field was focused this recently, the scope bar stays
/// (a click on a scope takes the focus from the field first).
pub(crate) fn scope_bar_hold_id(side: Side) -> egui::Id {
    egui::Id::unique(("manta-scope-bar-hold", side))
}

fn scope_bar_rect_id(side: Side) -> egui::Id {
    egui::Id::unique(("manta-scope-bar-rect", side))
}

/// The scope bar shows while a search scope is chosen or a search is
/// shown, and while the filter field is being used.
pub(crate) fn scope_bar_visible(ctx: &egui::Context, side: Side, tab: &Tab) -> bool {
    if tab.scope != Scope::Folder || tab.in_search() || !tab.filter.is_empty() {
        return true;
    }
    if ctx.memory(|m| m.has_focus(crate::pane::filter_field_id(side))) {
        return true;
    }
    let held = ctx
        .data(|d| d.get_temp::<Instant>(scope_bar_hold_id(side)))
        .is_some_and(|t| t.elapsed() < Duration::from_millis(400));
    let hovered = ctx
        .data(|d| d.get_temp::<egui::Rect>(scope_bar_rect_id(side)))
        .zip(ctx.pointer_hover_pos())
        .is_some_and(|(r, p)| r.contains(p));
    held || hovered
}

/// The filter text changed: filter the folder, or (re)start the search
/// shortly (typing is debounced), or leave it when the text is gone.
pub(crate) fn filter_changed(side: Side, tab: &mut Tab, actions: &mut Vec<Action>) {
    if tab.scope == Scope::Folder && !tab.in_search() {
        tab.rebuild_view();
        return;
    }
    if tab.filter.trim().is_empty() {
        tab.search_due = None;
        if tab.in_search() {
            actions.push(Action::EndSearch(side));
        }
    } else if tab.scope == Scope::Folder {
        // A sidebar tag search, then typed into: filter the results.
        tab.rebuild_view();
    } else {
        tab.search_due = Some(Instant::now() + DEBOUNCE);
    }
}

/// The scope bar: where to search, and how the search is going.
pub(crate) fn scope_bar(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    side: Side,
    tab: &mut Tab,
    actions: &mut Vec<Action>,
) {
    let p = pal(ui);
    ui.ctx()
        .data_mut(|d| d.insert_temp(scope_bar_rect_id(side), rect));
    let mac = mac_available(&tab.loc);
    let mut options = vec![
        (Scope::Folder, Scope::Folder.label()),
        (Scope::Subfolders, Scope::Subfolders.label()),
    ];
    if mac {
        options.push((Scope::Mac, Scope::Mac.label()));
    }
    let mut scope = if tab.in_search() && tab.scope == Scope::Folder {
        // A sidebar tag search runs on This Mac.
        Scope::Mac
    } else {
        tab.scope
    };
    let inner = rect.shrink2(egui::vec2(space::MD, 0.0));
    let mut right = inner.right();
    // Stop button while searching.
    let running = tab.search.as_ref().is_some_and(|s| s.running);
    if running {
        let r = egui::Rect::from_center_size(
            egui::pos2(right - size::FIELD / 2.0, rect.center().y),
            egui::vec2(size::FIELD, size::FIELD),
        );
        let resp = ui.interact(r, egui::Id::unique(("search-stop", side)), Sense::click());
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        widgets::paint_icon(
            ui,
            r,
            crate::icons::X_CIRCLE,
            14.0,
            if resp.hovered() {
                p.text
            } else {
                p.text_secondary
            },
            false,
        );
        if widgets::tooltip(resp, t!("search.stop"), "").clicked() {
            actions.push(Action::StopSearch(side));
        }
        right = r.left() - space::XS;
    }
    // Status: "Searching… 12 found", "12 found", "Stopped".
    let status = tab.search.as_ref().map(status_text);
    let seg_left = inner.left();
    let mut seg_w = 0.0;
    let seg = egui::Rect::from_min_max(
        egui::pos2(seg_left, rect.top()),
        egui::pos2(right, rect.bottom()),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(seg)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            let before = ui.cursor().left();
            if widgets::segmented(ui, ("search-scope", side), &mut scope, &options) {
                actions.push(Action::SetSearchScope(side, scope));
            }
            seg_w = ui.cursor().left() - before;
        },
    );
    if let Some(text) = status {
        let left = seg_left + seg_w + space::MD;
        let mut x = left;
        if running {
            let sr = egui::Rect::from_center_size(
                egui::pos2(x + 7.0, rect.center().y),
                egui::vec2(12.0, 12.0),
            );
            ui.put(sr, egui::Spinner::new().size(11.0).color(p.text_secondary));
            x = sr.right() + space::SM;
        }
        if right - x > 30.0 {
            let mut job = egui::text::LayoutJob::simple_singleline(
                text.clone(),
                theme::regular(text::SMALL),
                p.text_secondary,
            );
            job.wrap = egui::text::TextWrapping::truncate_at_width(right - x);
            let g = ui.painter().layout_job(job);
            ui.painter().galley(
                egui::pos2(x, rect.center().y - g.size().y / 2.0),
                g,
                p.text_secondary,
            );
        }
        // VoiceOver hears how the search is going (politely).
        crate::a11y::live(
            ui.ctx(),
            egui::Id::unique(("search-status", side)),
            egui::accesskit::Role::Status,
            &text,
            false,
        );
    }
}

/// The scope bar's status line for a search.
pub fn status_text(s: &Search) -> String {
    if let Some(e) = &s.error {
        return t!("search.failed", error = crate::i18n::localize_error(e));
    }
    if s.running {
        t!("search.searching", count = s.found)
    } else if s.stopped {
        t!("search.stopped", count = s.found)
    } else if s.truncated {
        t!("search.truncated", count = s.found)
    } else {
        t!("search.found", count = s.found)
    }
}

/// The breadcrumb of a search: "Search: “report”" and where.
pub(crate) fn search_crumb(ui: &mut egui::Ui, area: egui::Rect, side: Side, tab: &Tab) {
    let Some(s) = &tab.search else {
        return;
    };
    let p = pal(ui);
    let cy = area.center().y;
    let ir =
        egui::Rect::from_center_size(egui::pos2(area.left() + 9.0, cy), egui::vec2(16.0, 16.0));
    widgets::paint_icon(
        ui,
        ir,
        crate::icons::MAGNIFYING_GLASS,
        13.0,
        p.text_secondary,
        false,
    );
    let place = match (&s.query, s.scope) {
        (Query::Tag(_), _) | (_, Scope::Mac) => t!("search.where_mac").to_string(),
        _ => t!("search.where_folder", folder = root_label(&tab.loc)),
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &s.query.title(),
        0.0,
        egui::TextFormat::simple(theme::medium(text::BODY), p.text),
    );
    job.append(
        &place,
        space::SM,
        egui::TextFormat::simple(theme::regular(text::BODY), p.text_secondary),
    );
    job.wrap =
        egui::text::TextWrapping::truncate_at_width((area.right() - ir.right() - 6.0).max(10.0));
    let g = ui.painter().layout_job(job);
    let full = format!("{} {}", s.query.title(), place);
    let r = egui::Rect::from_min_size(
        egui::pos2(ir.right() + 5.0, cy - g.size().y / 2.0),
        g.size(),
    );
    ui.painter().galley(r.min, g, p.text);
    let resp = ui.interact(r, egui::Id::unique(("search-crumb", side)), Sense::hover());
    crate::a11y::name(&resp, egui::accesskit::Role::Label, &full);
    resp.on_hover_text(match &tab.loc.fs.local_path(&tab.loc.path) {
        Some(p) if s.scope != Scope::Mac => p.display().to_string(),
        _ => full.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_matching() {
        let m = NameMatcher::new("Report");
        assert!(m.matches("q3-report.pdf"));
        assert!(!m.matches("notes.txt"));
        let p = NameMatcher::new("*.PDF");
        assert!(p.matches("a.pdf"));
        assert!(!p.matches("a.pdf.zip"));
        let q = NameMatcher::new("file?.txt");
        assert!(q.matches("file1.txt"));
        assert!(!q.matches("file10.txt"));
        assert!(NameMatcher::new("a*b*c").matches("aXXbYYc"));
        assert!(!NameMatcher::new("a*b*c").matches("aXXbYY"));
        assert!(NameMatcher::new("").matches("anything"));
    }

    #[test]
    fn spotlight_queries() {
        assert_eq!(
            spotlight_query(&Query::Text("report".into())),
            "(kMDItemDisplayName == \"*report*\"cd || kMDItemTextContent == \"report*\"cdw)"
        );
        assert_eq!(
            spotlight_query(&Query::Text("*.pdf".into())),
            "kMDItemFSName == \"*.pdf\"c"
        );
        assert_eq!(
            spotlight_query(&Query::Tag("Rot".into())),
            "kMDItemUserTags == \"Rot\"c"
        );
        assert_eq!(
            spotlight_query(&Query::Text("tag:Rot q\"x".into())),
            "(kMDItemDisplayName == \"*q\\\"x*\"cd || kMDItemTextContent == \"q\\\"x*\"cdw) && kMDItemUserTags == \"rot\"c"
        );
    }

    fn tree() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        for d in ["src/app", "docs/old", ".hidden", "Tool.app/Contents"] {
            std::fs::create_dir_all(r.join(d)).unwrap();
        }
        for f in [
            "README.md",
            "src/main.rs",
            "src/app/report.rs",
            "docs/report.pdf",
            "docs/old/Report-2024.pdf",
            ".hidden/report.txt",
            "Tool.app/Contents/report.plist",
        ] {
            std::fs::write(r.join(f), b"x").unwrap();
        }
        tmp
    }

    fn spec(root: &Path, q: &str, hidden: bool) -> Spec {
        Spec {
            query: Query::Text(q.into()),
            scope: Scope::Subfolders,
            fs: Arc::new(manta_vfs::LocalFs::new()),
            root: VfsPath::new(&root.to_string_lossy()),
            local_root: Some(root.to_path_buf()),
            show_hidden: hidden,
        }
    }

    fn names(hits: &[Hit]) -> Vec<String> {
        let mut v: Vec<String> = hits.iter().map(|h| h.entry.name.clone()).collect();
        v.sort();
        v
    }

    #[test]
    fn subfolders_on_this_mac() {
        let tmp = tree();
        let ctx = egui::Context::default();
        let spot: Arc<dyn Spotlight> = Arc::new(FakeSpotlight::default());
        let mut s = Search::start(spec(tmp.path(), "report", false), &ctx, spot.clone());
        let hits = s.wait(Duration::from_secs(10));
        assert!(!s.running);
        // Not in hidden folders, not inside the package.
        assert_eq!(
            names(&hits),
            vec![
                "docs/old/Report-2024.pdf",
                "docs/report.pdf",
                "src/app/report.rs"
            ]
        );
        let h = hits
            .iter()
            .find(|h| h.entry.name == "docs/report.pdf")
            .unwrap();
        assert_eq!(
            h.entry.path.as_str(),
            tmp.path().join("docs/report.pdf").to_string_lossy()
        );
        assert_eq!(crate::util::display_name(&h.entry.name), "report.pdf");
        assert_eq!(crate::util::display_dir(&h.entry.name), "docs");
        // Hidden items when they're shown; patterns.
        let mut s = Search::start(spec(tmp.path(), "*.txt", true), &ctx, spot.clone());
        assert_eq!(
            names(&s.wait(Duration::from_secs(10))),
            vec![".hidden/report.txt"]
        );
        // Folders match too.
        let mut s = Search::start(spec(tmp.path(), "old", false), &ctx, spot);
        assert_eq!(names(&s.wait(Duration::from_secs(10))), vec!["docs/old"]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn subfolders_by_tag() {
        let tmp = tree();
        let f = tmp.path().join("src/main.rs");
        tags::set_tag(&f, &Tag::new("Rot", tags::TagColor::Red), true).unwrap();
        let ctx = egui::Context::default();
        let spot: Arc<dyn Spotlight> = Arc::new(FakeSpotlight::default());
        let mut s = Search::start(spec(tmp.path(), "tag:rot", false), &ctx, spot);
        let hits = s.wait(Duration::from_secs(10));
        assert_eq!(names(&hits), vec!["src/main.rs"]);
        assert_eq!(hits[0].tags, vec![Tag::new("Rot", tags::TagColor::Red)]);
        assert_eq!(s.tags.get("src/main.rs").map(Vec::len), Some(1));
    }

    #[test]
    fn cancel_stops_the_walk() {
        // A wide tree; cancelled right away, the walk ends quickly and
        // reports far fewer results than there are.
        let tmp = tempfile::tempdir().unwrap();
        for i in 0..200 {
            let d = tmp.path().join(format!("d{i}"));
            std::fs::create_dir(&d).unwrap();
            for j in 0..20 {
                std::fs::write(d.join(format!("match{j}.txt")), b"").unwrap();
            }
        }
        let ctx = egui::Context::default();
        let spot: Arc<dyn Spotlight> = Arc::new(FakeSpotlight::default());
        let mut s = Search::start(spec(tmp.path(), "match", false), &ctx, spot);
        s.stop();
        let start = Instant::now();
        let hits = s.wait(Duration::from_secs(10));
        assert!(!s.running, "finished after cancel");
        assert!(s.stopped);
        assert!(start.elapsed() < Duration::from_secs(5));
        assert!(hits.len() < 4000, "{} results after cancel", hits.len());
    }

    #[test]
    fn this_mac_goes_through_spotlight() {
        let tmp = tree();
        let fake = Arc::new(FakeSpotlight::default());
        *fake.results.lock().unwrap() = vec![
            tmp.path().join("docs/report.pdf"),
            tmp.path().join("missing.pdf"),
            tmp.path().join("docs/.report.swp"),
        ];
        std::fs::write(tmp.path().join("docs/.report.swp"), b"x").unwrap();
        let ctx = egui::Context::default();
        let mut sp = spec(tmp.path(), "report", false);
        sp.scope = Scope::Mac;
        let mut s = Search::start(sp, &ctx, fake.clone());
        let hits = s.wait(Duration::from_secs(10));
        // Named by the full path; vanished files and hidden ones dropped.
        assert_eq!(
            names(&hits),
            vec![tmp
                .path()
                .join("docs/report.pdf")
                .to_string_lossy()
                .into_owned()]
        );
        let q = fake.queries.lock().unwrap();
        assert_eq!(q.len(), 1);
        assert!(q[0].0.contains("kMDItemTextContent"), "{}", q[0].0);
    }

    #[test]
    fn servers_are_walked_with_limits() {
        let mem = manta_ops::memfs::MemFs::new();
        mem.mkdir_p("/srv/www/assets");
        mem.put("/srv/www/index.html", b"x", chrono::Utc::now());
        mem.put("/srv/www/assets/index.css", b"x", chrono::Utc::now());
        mem.put("/srv/other.txt", b"x", chrono::Utc::now());
        let fs: SharedVfs = Arc::new(mem);
        let ctx = egui::Context::default();
        let spot: Arc<dyn Spotlight> = Arc::new(FakeSpotlight::default());
        let mut s = Search::start(
            Spec {
                query: Query::Text("index".into()),
                scope: Scope::Subfolders,
                fs,
                root: VfsPath::new("/srv"),
                local_root: None,
                show_hidden: false,
            },
            &ctx,
            spot,
        );
        let hits = s.wait(Duration::from_secs(10));
        assert_eq!(names(&hits), vec!["www/assets/index.css", "www/index.html"]);
        assert!(s.error.is_none());
    }
}
