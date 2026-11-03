//! Background execution of blocking VFS calls. Every potentially slow call
//! (listing, connecting, comparing, reading previews, rename/mkdir) runs on a
//! short-lived worker thread and reports back through a channel. The UI drains
//! the channel once per frame. Panics inside backends are caught and turned
//! into ordinary error messages so a buggy backend can't take the UI down.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use eframe::egui;
use manta_core::{ConnectionConfig, Entry, SharedVfs, VfsPath, VfsResult};
use manta_ops::CompareItem;

use crate::app::Side;

/// Data needed to finish a connection attempt on the UI thread.
#[derive(Clone)]
pub struct ConnectRequest {
    pub config: ConnectionConfig,
    /// Config as it should be persisted (secrets possibly stripped).
    pub favorite: Option<ConnectionConfig>,
    pub side: Side,
    /// The `FileAction` reported for this connect.
    pub action: u64,
    /// The secret comes from the Keychain (a favorite saved with "Remember
    /// password"); the favorite to save keeps it there.
    pub keychain: bool,
}

pub enum PreviewContent {
    Text { text: String, truncated: bool },
    Image { bytes: Arc<[u8]> },
    Binary { size: u64 },
}

pub enum WorkResult {
    Listing {
        tab_id: u64,
        gen: u64,
        result: Result<(VfsPath, Vec<Entry>), String>,
    },
    /// What a listing has read so far (a very large folder shown fresh);
    /// the `Listing` with the same `gen` replaces it.
    ListingPart {
        tab_id: u64,
        gen: u64,
        entries: Vec<Entry>,
    },
    Connected {
        request: Box<ConnectRequest>,
        result: Result<(SharedVfs, VfsPath), String>,
    },
    Compare {
        gen: u64,
        result: Result<Vec<CompareItem>, String>,
    },
    Preview {
        id: u64,
        result: Result<PreviewContent, String>,
    },
    /// "Test Connection" in the connect dialog: Ok(home directory).
    TestConnection { result: Result<String, String> },
    /// An SFTP server's host key is unknown or changed: ask the user.
    HostKey {
        info: Box<manta_core::HostKeyInfo>,
        changed: bool,
        retry: crate::host_key::HostKeyRetry,
    },
    /// "Go to Path": the typed path was checked (exists, folder or file).
    GotoChecked {
        gen: u64,
        new_tab: bool,
        result: Result<crate::goto::GotoTarget, String>,
    },
    /// "Go to Path": folder names for Tab completion.
    GotoFolders {
        gen: u64,
        dir_part: String,
        result: Result<Vec<String>, String>,
    },
    /// A plugin's batch rename finished (`HostRequest::RenameMany`).
    RenameMany {
        plugin: String,
        pane: manta_plugin::PaneId,
        dir: VfsPath,
        fs: SharedVfs,
        /// The `FileAction` to finish.
        action: u64,
        outcome: manta_ops::rename::RenameOutcome,
    },
    /// A folder listed for a plugin (`HostRequest::ListDir`).
    PluginListing {
        plugin: String,
        pane: manta_plugin::PaneId,
        dir: VfsPath,
        result: Result<Vec<Entry>, String>,
    },
    /// A file read for a plugin (`HostRequest::ReadFile`).
    PluginFileRead {
        plugin: String,
        content: manta_plugin::FileContent,
    },
    /// An archive was opened (or failed to open) in the background.
    ArchiveOpened(Box<crate::app::archive_ops::ArchiveOpened>),
    /// The Trash was measured for the "Empty Trash" question.
    TrashMeasured(Option<crate::trash::TrashSummary>),
    /// "Empty Trash" finished.
    TrashEmptied(Result<crate::trash::EmptyOutcome, String>),
    /// Free space of a folder ([`crate::space`]).
    Space {
        key: crate::space::Key,
        result: Result<Option<manta_core::Space>, String>,
    },
    /// A volume was ejected (or refused to).
    Ejected {
        path: std::path::PathBuf,
        name: String,
        result: Result<(), String>,
    },
    /// `/Volumes` changed and the sidebar's places were read again
    /// (locations, devices).
    Places(Option<(Vec<crate::sidebar::Place>, Vec<crate::sidebar::Place>)>),
    /// The disk image file of a volume was looked up ("Show in Enclosing
    /// Folder"; `None`: not a disk image).
    VolumeRevealed {
        volume: std::path::PathBuf,
        image: Option<std::path::PathBuf>,
    },
    /// `docker start|restart|stop` finished (sidebar context menu).
    DockerControlled {
        id: String,
        name: String,
        op: manta_vfs::container::discover::DockerOp,
        result: Result<(), String>,
    },
    /// Local items were moved to the Trash (⌘⌫).
    Trashed {
        /// The `FileAction` to finish.
        action: u64,
        /// Directories whose listings should be refreshed afterwards.
        refresh: Vec<(SharedVfs, VfsPath)>,
        outcome: crate::trash::TrashOutcome,
    },
    /// A quick file operation (mkdir, rename, open remote file).
    FileOp {
        description: String,
        /// Title shown if the operation fails, e.g. "Couldn't rename …".
        failure: String,
        /// Directories whose listings should be refreshed afterwards.
        refresh: Vec<(SharedVfs, VfsPath)>,
        /// Name to select in the given tab after refreshing.
        select: Option<(u64, String)>,
        /// The `FileAction` to finish with the result.
        action: Option<u64>,
        result: Result<(), String>,
        /// Reverts the operation: recorded for Undo when it succeeded.
        undo: Option<crate::undo::Op>,
    },
    /// An Undo or Redo step ran (`redo`: from Redo).
    Undone {
        redo: bool,
        /// Directories whose listings should be refreshed afterwards.
        refresh: Vec<(SharedVfs, VfsPath)>,
        outcome: crate::undo::Outcome,
    },
}

pub struct Worker {
    tx: Sender<WorkResult>,
    rx: Receiver<WorkResult>,
    ctx: egui::Context,
}

impl Worker {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx, rx) = channel();
        Worker { tx, rx, ctx }
    }

    /// Run `job` on a background thread, wrap its result with `wrap` and send
    /// it back to the UI thread (which is woken up via `request_repaint`).
    pub fn spawn<T, J, W>(&self, name: &str, job: J, wrap: W)
    where
        T: Send + 'static,
        J: FnOnce() -> Result<T, String> + Send + 'static,
        W: FnOnce(Result<T, String>) -> WorkResult + Send + 'static,
    {
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("manta-{name}"))
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(job)).unwrap_or_else(|panic| {
                    let msg = panic
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| panic.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "unknown panic".into());
                    Err(t!("error.internal", detail = msg))
                });
                let _ = tx.send(wrap(result));
                ctx.request_repaint();
            });
        if let Err(e) = spawned {
            log::error!("failed to spawn worker thread: {e}");
        }
    }

    /// Send results from a job before it is done (e.g. the first part of
    /// a very large folder); wakes the UI.
    pub fn messenger(&self) -> impl Fn(WorkResult) + Send + 'static {
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        move |r| {
            let _ = tx.send(r);
            ctx.request_repaint();
        }
    }

    /// The UI context the worker wakes up.
    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    pub fn poll(&self) -> Vec<WorkResult> {
        self.rx.try_iter().collect()
    }
}

/// Blocking: `op` on `fs`; after a network error the backend logs in
/// again and `op` runs once more. A session that died while Manta sat
/// idle (Wi-Fi gone, laptop asleep) is otherwise reused by every reload
/// and every new tab on that connection.
pub fn with_reconnect<T>(fs: &SharedVfs, mut op: impl FnMut() -> VfsResult<T>) -> VfsResult<T> {
    match op() {
        Err(e) if e.is_transient() => {
            log::info!("{}: {e}; reconnecting", fs.label());
            fs.reconnect()?;
            op()
        }
        r => r,
    }
}

/// Maximum number of bytes read for text previews.
pub const TEXT_PREVIEW_LIMIT: u64 = 256 * 1024;
/// Images are read fully, up to this size.
pub const IMAGE_PREVIEW_LIMIT: u64 = 64 * 1024 * 1024;

/// Blocking: read the start of a file and classify it for the preview window.
pub fn load_preview(fs: &SharedVfs, entry: &Entry) -> Result<PreviewContent, String> {
    use std::io::Read;
    let image = crate::util::is_image_name(&entry.name);
    let limit = if image {
        IMAGE_PREVIEW_LIMIT
    } else {
        TEXT_PREVIEW_LIMIT
    };
    let reader = fs.open_read(&entry.path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    reader
        .take(limit)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    if image {
        return Ok(PreviewContent::Image { bytes: buf.into() });
    }
    let sniff = &buf[..buf.len().min(8192)];
    if sniff.contains(&0) {
        return Ok(PreviewContent::Binary { size: entry.size });
    }
    let truncated = buf.len() as u64 >= limit && entry.size > limit;
    Ok(PreviewContent::Text {
        text: String::from_utf8_lossy(&buf).into_owned(),
        truncated,
    })
}

/// Blocking: download a remote file into a temp directory and return its path.
pub fn download_to_temp(fs: &SharedVfs, path: &VfsPath) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir()
        .join("manta-open")
        .join(format!("{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join(path.file_name());
    let mut reader = fs.open_read(path).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&target).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    Ok(target)
}

/// Blocking: read up to `max_bytes` of `path` for a plugin
/// (`HostRequest::ReadFile`). Never fails: errors go into `error`.
pub fn read_for_plugin(
    fs: &SharedVfs,
    pane: manta_plugin::PaneId,
    path: &VfsPath,
    max_bytes: u64,
) -> manta_plugin::FileContent {
    use std::io::Read;
    let mut c = manta_plugin::FileContent {
        pane,
        path: path.clone(),
        bytes: Vec::new(),
        size: None,
        truncated: false,
        modified: None,
        error: None,
    };
    // (bytes, size, modified)
    type Loaded = (Vec<u8>, Option<u64>, Option<i64>);
    let read = || -> Result<Loaded, String> {
        let st = fs.stat(path).map_err(|e| e.to_string())?;
        if st.is_dir() {
            return Err(t!("error.is_folder", name = st.name));
        }
        let mut buf = Vec::new();
        let reader = fs.open_read(path).map_err(|e| e.to_string())?;
        // One byte more tells a file of exactly `max_bytes` from a longer one.
        reader
            .take(max_bytes.saturating_add(1))
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        Ok((buf, Some(st.size), st.modified.map(|t| t.timestamp())))
    };
    match read() {
        Ok((mut buf, size, modified)) => {
            if buf.len() as u64 > max_bytes {
                buf.truncate(max_bytes as usize);
                c.truncated = true;
            }
            c.size = size.or(Some(buf.len() as u64));
            c.modified = modified;
            c.bytes = buf;
        }
        Err(e) => c.error = Some(e),
    }
    c
}
