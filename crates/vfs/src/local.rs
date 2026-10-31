//! Local file system backend on top of `std::fs`.
//!
//! A [`VfsPath`] maps 1:1 to an absolute OS path.

use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use manta_core::{Entry, EntryKind, Protocol, Space, Vfs, VfsError, VfsPath, VfsResult};

/// The local file system.
pub struct LocalFs;

impl LocalFs {
    pub fn new() -> Self {
        LocalFs
    }
}

impl Default for LocalFs {
    fn default() -> Self {
        Self::new()
    }
}

fn os_path(path: &VfsPath) -> PathBuf {
    PathBuf::from(path.as_str())
}

/// Map an `io::Error` for `path` to the matching [`VfsError`] variant.
pub(crate) fn map_io(err: io::Error, path: &VfsPath) -> VfsError {
    let p = path.to_string();
    match err.kind() {
        io::ErrorKind::NotFound => VfsError::NotFound(p),
        io::ErrorKind::PermissionDenied => VfsError::PermissionDenied(p),
        io::ErrorKind::AlreadyExists => VfsError::AlreadyExists(p),
        io::ErrorKind::DirectoryNotEmpty => VfsError::DirectoryNotEmpty(p),
        io::ErrorKind::NotADirectory => VfsError::NotADirectory(p),
        _ => VfsError::Io(err),
    }
}

fn kind_of(meta: &Metadata) -> EntryKind {
    let ft = meta.file_type();
    if ft.is_dir() {
        EntryKind::Directory
    } else if ft.is_file() {
        EntryKind::File
    } else if ft.is_symlink() {
        EntryKind::Symlink
    } else {
        EntryKind::Other
    }
}

#[cfg(target_os = "macos")]
fn has_hidden_flag(meta: &Metadata) -> bool {
    use std::os::macos::fs::MetadataExt;
    const UF_HIDDEN: u32 = 0x8000;
    meta.st_flags() & UF_HIDDEN != 0
}

#[cfg(not(target_os = "macos"))]
fn has_hidden_flag(_meta: &Metadata) -> bool {
    false
}

/// Build an [`Entry`] for `path` from its `lstat` metadata. Symlinks are
/// resolved: a link to a directory gets `kind = Directory` (so it can be
/// navigated into); size/mtime/permissions come from the target if it exists.
fn entry_from_lstat(path: &VfsPath, name: String, lmeta: &Metadata) -> Entry {
    let os = os_path(path);
    let mut kind = kind_of(lmeta);
    let mut link_target = None;
    let mut meta_for_attrs: Option<Metadata> = None;

    if lmeta.file_type().is_symlink() {
        link_target = fs::read_link(&os)
            .ok()
            .map(|t| t.to_string_lossy().into_owned());
        if let Ok(target_meta) = fs::metadata(&os) {
            if target_meta.is_dir() {
                kind = EntryKind::Directory;
            }
            meta_for_attrs = Some(target_meta);
        }
    }
    let meta = meta_for_attrs.as_ref().unwrap_or(lmeta);

    let hidden = name.starts_with('.') || has_hidden_flag(lmeta);
    Entry {
        size: if kind == EntryKind::Directory {
            0
        } else {
            meta.len()
        },
        modified: meta.modified().ok().map(DateTime::<Utc>::from),
        permissions: Some(meta.permissions().mode() & 0o7777),
        link_target,
        hidden,
        kind,
        name,
        path: path.clone(),
    }
}

/// Entries per part of [`Vfs::list_parts`] on the portable path.
const PART: usize = 4096;

/// The entry `name` in `dir`, from `lstat` (a symlink also from its target).
fn stat_entry(dir: &VfsPath, name: String) -> Entry {
    let path = dir.join(&name);
    match fs::symlink_metadata(os_path(&path)) {
        Ok(m) => entry_from_lstat(&path, name, &m),
        Err(e) => {
            log::debug!("cannot stat {path}: {e}");
            unknown_entry(path, name)
        }
    }
}

/// Listing with `getattrlistbulk(2)`: name, type, date, mode, flags and
/// size of hundreds of entries per system call, read from the directory
/// itself. `readdir` + one `lstat` per file needs a kernel vnode per file:
/// a folder with more files than the vnode cache holds (`kern.maxvnodes`,
/// about 260,000 here) took 3.4 s for 300,000 files that way.
#[cfg(target_os = "macos")]
mod bulk {
    use std::ffi::CString;
    use std::io;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    use chrono::{DateTime, Utc};
    use manta_core::{Entry, EntryKind, VfsPath};

    /// `vtype` values of `ATTR_CMN_OBJTYPE` (sys/vnode.h).
    const VREG: u32 = 1;
    const VDIR: u32 = 2;
    const VLNK: u32 = 5;
    const UF_HIDDEN: u32 = 0x8000;
    /// sys/attr.h (not in the libc crate).
    const ATTR_CMN_ERROR: u32 = 0x2000_0000;

    struct Fd(libc::c_int);

    impl Drop for Fd {
        fn drop(&mut self) {
            // SAFETY: an fd this module opened and nobody else closes.
            unsafe { libc::close(self.0) };
        }
    }

    fn u32_at(b: &[u8], at: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(b.get(at..at + 4)?.try_into().ok()?))
    }

    fn i32_at(b: &[u8], at: usize) -> Option<i32> {
        Some(i32::from_ne_bytes(b.get(at..at + 4)?.try_into().ok()?))
    }

    fn i64_at(b: &[u8], at: usize) -> Option<i64> {
        Some(i64::from_ne_bytes(b.get(at..at + 8)?.try_into().ok()?))
    }

    /// Hands the entries of `os` (shown as `dir`) to `part`, one part per
    /// system call.
    pub fn list(os: &Path, dir: &VfsPath, part: &mut dyn FnMut(Vec<Entry>)) -> io::Result<()> {
        let c = CString::new(os.as_os_str().as_bytes())?;
        // SAFETY: a valid NUL-terminated path; the result is checked.
        let fd = unsafe {
            libc::open(
                c.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = Fd(fd);
        // SAFETY: attrlist is plain old data; zero is a valid start.
        let mut al: libc::attrlist = unsafe { std::mem::zeroed() };
        al.bitmapcount = libc::ATTR_BIT_MAP_COUNT;
        al.commonattr = libc::ATTR_CMN_RETURNED_ATTRS
            | ATTR_CMN_ERROR
            | libc::ATTR_CMN_NAME
            | libc::ATTR_CMN_OBJTYPE
            | libc::ATTR_CMN_MODTIME
            | libc::ATTR_CMN_ACCESSMASK
            | libc::ATTR_CMN_FLAGS;
        al.fileattr = libc::ATTR_FILE_DATALENGTH;
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            // SAFETY: `al` and `buf` outlive the call; the size is buf's.
            let n = unsafe {
                libc::getattrlistbulk(
                    fd.0,
                    (&mut al as *mut libc::attrlist).cast(),
                    buf.as_mut_ptr().cast(),
                    buf.len(),
                    0,
                )
            };
            if n < 0 {
                let e = io::Error::last_os_error();
                if e.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(e);
            }
            if n == 0 {
                return Ok(());
            }
            let mut out = Vec::with_capacity(n as usize);
            let mut at = 0usize;
            for _ in 0..n {
                let len = u32_at(&buf, at).ok_or_else(bad)? as usize;
                let rec = buf.get(at..at + len).ok_or_else(bad)?;
                if let Some(e) = parse(rec, dir) {
                    out.push(e);
                }
                at += len;
            }
            part(out);
        }
    }

    fn bad() -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "bad getattrlistbulk record")
    }

    /// One record; `None` for "." / ".." or a record without a name.
    fn parse(rec: &[u8], dir: &VfsPath) -> Option<Entry> {
        // length, then attribute_set_t (5 x u32) of what was returned.
        let common = u32_at(rec, 4)?;
        let file = u32_at(rec, 4 + 12)?;
        let mut f = 4 + 20;
        let mut error = 0;
        if common & ATTR_CMN_ERROR != 0 {
            error = u32_at(rec, f)?;
            f += 4;
        }
        if common & libc::ATTR_CMN_NAME == 0 {
            return None;
        }
        // attrreference_t: offset (from itself) and length (with the NUL).
        let off = i32_at(rec, f)?;
        let nlen = u32_at(rec, f + 4)? as usize;
        let start = (f as i64 + off as i64) as usize;
        let raw = rec.get(start..start + nlen.saturating_sub(1))?;
        let name = String::from_utf8_lossy(raw).into_owned();
        f += 8;
        if name == "." || name == ".." {
            return None;
        }
        let mut objtype = None;
        if common & libc::ATTR_CMN_OBJTYPE != 0 {
            objtype = Some(u32_at(rec, f)?);
            f += 4;
        }
        let mut modified = None;
        if common & libc::ATTR_CMN_MODTIME != 0 {
            let (secs, nanos) = (i64_at(rec, f)?, i64_at(rec, f + 8)?);
            modified = DateTime::<Utc>::from_timestamp(secs, nanos.clamp(0, 999_999_999) as u32);
            f += 16;
        }
        let mut mode = None;
        if common & libc::ATTR_CMN_ACCESSMASK != 0 {
            mode = Some(u32_at(rec, f)? & 0o7777);
            f += 4;
        }
        let mut flags = 0;
        if common & libc::ATTR_CMN_FLAGS != 0 {
            flags = u32_at(rec, f)?;
            f += 4;
        }
        let mut size = 0;
        if file & libc::ATTR_FILE_DATALENGTH != 0 {
            size = i64_at(rec, f)?.max(0) as u64;
        }
        // Symlinks (their target decides kind, size and date) and anything
        // the bulk call couldn't read: the per-file way.
        let kind = match objtype {
            _ if error != 0 => None,
            Some(VREG) => Some(EntryKind::File),
            Some(VDIR) => Some(EntryKind::Directory),
            Some(VLNK) | None => None,
            Some(_) => Some(EntryKind::Other),
        };
        let Some(kind) = kind else {
            return Some(super::stat_entry(dir, name));
        };
        Some(Entry {
            path: dir.join(&name),
            hidden: name.starts_with('.') || flags & UF_HIDDEN != 0,
            name,
            kind,
            size: if kind == EntryKind::Directory {
                0
            } else {
                size
            },
            modified,
            permissions: mode,
            link_target: None,
        })
    }
}

/// Best-effort entry for something we could not stat.
fn unknown_entry(path: VfsPath, name: String) -> Entry {
    Entry {
        hidden: name.starts_with('.'),
        name,
        path,
        kind: EntryKind::Other,
        size: 0,
        modified: None,
        permissions: None,
        link_target: None,
    }
}

/// `~/Library/Mobile Documents/com~apple~CloudDocs`.
fn is_icloud_top(p: &Path) -> bool {
    crate::cloud_local::cloud_home()
        .is_some_and(|h| p == crate::icloud::mobile_documents(&h).join(crate::icloud::CLOUD_DOCS))
}

/// An app's folder in iCloud Drive can't be renamed, moved or deleted.
fn refuse_app_library(path: &VfsPath) -> VfsResult<()> {
    if crate::icloud::is_app_library(Path::new(path.as_str())) {
        return Err(VfsError::PermissionDenied(format!(
            "{path}: an app's folder in iCloud Drive can't be renamed, moved or deleted"
        )));
    }
    Ok(())
}

/// Finder shows Apple's apps (`/System/Applications`, on the read-only
/// system volume) inside Applications, and their Utilities inside
/// Utilities: the folder whose items join `dir`'s listing.
fn system_apps_of(dir: &Path) -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    match dir.to_str()? {
        "/Applications" => Some(PathBuf::from("/System/Applications")),
        "/Applications/Utilities" => Some(PathBuf::from("/System/Applications/Utilities")),
        _ => None,
    }
}

/// `dir`'s entries plus the ones in `system` it doesn't have itself (by
/// name, ignoring case). Links to Apple's apps (`Safari.app` points into
/// `/System/Cryptexes`) are flagged hidden, but Finder shows them.
fn list_with_system_apps(
    dir: &VfsPath,
    system: &Path,
    part: &mut dyn FnMut(Vec<Entry>),
) -> VfsResult<()> {
    let mut names = std::collections::HashSet::new();
    list_dir(dir, &mut |mut p| {
        for e in &mut p {
            names.insert(e.name.to_lowercase());
            let to_system = e
                .link_target
                .as_deref()
                .is_some_and(|t| t.starts_with("/System/") || t.starts_with("../System/"));
            if to_system && !e.name.starts_with('.') {
                e.hidden = false;
            }
        }
        part(p)
    })?;
    let system_dir = VfsPath::new(&system.to_string_lossy());
    let mut extra = Vec::new();
    // Not there (other systems, tests): Applications as it is.
    let _ = list_dir(&system_dir, &mut |p| {
        extra.extend(
            p.into_iter()
                .filter(|e| !e.hidden && !names.contains(&e.name.to_lowercase())),
        )
    });
    if !extra.is_empty() {
        part(extra);
    }
    Ok(())
}

/// The folder's entries, in parts (macOS: `getattrlistbulk`).
fn list_dir(dir: &VfsPath, part: &mut dyn FnMut(Vec<Entry>)) -> VfsResult<()> {
    let os = os_path(dir);
    let meta = fs::metadata(&os).map_err(|e| map_io(e, dir))?;
    if !meta.is_dir() {
        return Err(VfsError::NotADirectory(dir.to_string()));
    }
    #[cfg(target_os = "macos")]
    {
        let mut delivered = false;
        match bulk::list(&os, dir, &mut |p| {
            delivered = true;
            part(p)
        }) {
            Ok(()) => return Ok(()),
            // Nothing handed over yet: the portable way below.
            Err(e) if !delivered => {
                log::debug!("getattrlistbulk failed in {dir}: {e}; using readdir")
            }
            Err(e) => return Err(map_io(e, dir)),
        }
    }
    let rd = fs::read_dir(&os).map_err(|e| map_io(e, dir))?;
    let mut out = Vec::new();
    for item in rd {
        let item = match item {
            Ok(i) => i,
            Err(e) => {
                log::warn!("skipping unreadable entry in {dir}: {e}");
                continue;
            }
        };
        let name = item.file_name().to_string_lossy().into_owned();
        out.push(stat_entry(dir, name));
        if out.len() == PART {
            part(std::mem::take(&mut out));
        }
    }
    if !out.is_empty() {
        part(out);
    }
    Ok(())
}

impl Vfs for LocalFs {
    fn label(&self) -> String {
        "Local".into()
    }

    fn protocol(&self) -> Protocol {
        Protocol::Local
    }

    fn home(&self) -> VfsResult<VfsPath> {
        let home = dirs::home_dir()
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .ok_or_else(|| VfsError::Other("cannot determine home directory".into()))?;
        Ok(VfsPath::new(&home.to_string_lossy()))
    }

    fn list(&self, dir: &VfsPath) -> VfsResult<Vec<Entry>> {
        let mut out = Vec::new();
        self.list_parts(dir, &mut |part| out.extend(part))?;
        Ok(out)
    }

    /// iCloud Drive's top folder also lists the apps' folders, and
    /// Applications Apple's own apps, like Finder ([`crate::icloud`],
    /// [`system_apps_of`]).
    fn list_parts(&self, dir: &VfsPath, part: &mut dyn FnMut(Vec<Entry>)) -> VfsResult<()> {
        let os = os_path(dir);
        if let Some(system) = system_apps_of(&os) {
            return list_with_system_apps(dir, &system, part);
        }
        if !is_icloud_top(&os) {
            return list_dir(dir, part);
        }
        let mut names = std::collections::HashSet::new();
        list_dir(dir, &mut |p| {
            names.extend(p.iter().map(|e| e.name.to_lowercase()));
            part(p)
        })?;
        let apps: Vec<Entry> = crate::icloud::app_libraries()
            .into_iter()
            .filter(|l| !names.contains(&l.name.to_lowercase()))
            .filter_map(|l| {
                let m = fs::symlink_metadata(&l.docs).ok()?;
                let path = VfsPath::new(&l.docs.to_string_lossy());
                Some(entry_from_lstat(&path, l.name, &m))
            })
            .collect();
        if !apps.is_empty() {
            part(apps);
        }
        Ok(())
    }

    fn stat(&self, path: &VfsPath) -> VfsResult<Entry> {
        let m = fs::symlink_metadata(os_path(path)).map_err(|e| map_io(e, path))?;
        let name = if path.is_root() {
            "/".to_string()
        } else {
            path.file_name().to_string()
        };
        Ok(entry_from_lstat(path, name, &m))
    }

    fn create_dir(&self, path: &VfsPath) -> VfsResult<()> {
        fs::create_dir(os_path(path)).map_err(|e| map_io(e, path))
    }

    fn remove_file(&self, path: &VfsPath) -> VfsResult<()> {
        let os = os_path(path);
        let m = fs::symlink_metadata(&os).map_err(|e| map_io(e, path))?;
        if m.is_dir() {
            return Err(VfsError::Other(format!("is a directory: {path}")));
        }
        fs::remove_file(&os).map_err(|e| map_io(e, path))
    }

    fn remove_dir(&self, path: &VfsPath) -> VfsResult<()> {
        refuse_app_library(path)?;
        let os = os_path(path);
        let m = fs::symlink_metadata(&os).map_err(|e| map_io(e, path))?;
        if !m.is_dir() {
            return Err(VfsError::NotADirectory(path.to_string()));
        }
        fs::remove_dir(&os).map_err(|e| map_io(e, path))
    }

    fn rename(&self, from: &VfsPath, to: &VfsPath) -> VfsResult<()> {
        refuse_app_library(from)?;
        fs::rename(os_path(from), os_path(to)).map_err(|e| {
            // Attribute "not found" to the source, everything else to the target.
            if e.kind() == io::ErrorKind::NotFound && !Path::new(from.as_str()).exists() {
                map_io(e, from)
            } else {
                map_io(e, to)
            }
        })
    }

    fn open_read(&self, path: &VfsPath) -> VfsResult<Box<dyn Read + Send>> {
        let f = File::open(os_path(path)).map_err(|e| map_io(e, path))?;
        let m = f.metadata().map_err(|e| map_io(e, path))?;
        if m.is_dir() {
            return Err(VfsError::Other(format!("is a directory: {path}")));
        }
        Ok(Box::new(f))
    }

    fn open_write(&self, path: &VfsPath) -> VfsResult<Box<dyn Write + Send>> {
        let f = File::create(os_path(path)).map_err(|e| map_io(e, path))?;
        Ok(Box::new(f))
    }

    fn open_read_at(&self, path: &VfsPath, offset: u64) -> VfsResult<Box<dyn Read + Send>> {
        use std::io::{Seek, SeekFrom};
        let mut f = File::open(os_path(path)).map_err(|e| map_io(e, path))?;
        let m = f.metadata().map_err(|e| map_io(e, path))?;
        if m.is_dir() {
            return Err(VfsError::Other(format!("is a directory: {path}")));
        }
        if offset > m.len() {
            return Err(VfsError::Other(format!(
                "{path}: file is shorter ({} bytes) than the resume offset {offset}",
                m.len()
            )));
        }
        f.seek(SeekFrom::Start(offset))
            .map_err(|e| map_io(e, path))?;
        Ok(Box::new(f))
    }

    fn open_append(&self, path: &VfsPath) -> VfsResult<Box<dyn Write + Send>> {
        let f = File::options()
            .create(true)
            .append(true)
            .open(os_path(path))
            .map_err(|e| map_io(e, path))?;
        Ok(Box::new(f))
    }

    fn supports_resume(&self) -> manta_core::ResumeCaps {
        manta_core::ResumeCaps {
            read_at: true,
            append: true,
            upload_session: false,
        }
    }

    fn set_permissions(&self, path: &VfsPath, mode: u32) -> VfsResult<()> {
        fs::set_permissions(os_path(path), fs::Permissions::from_mode(mode & 0o7777))
            .map_err(|e| map_io(e, path))
    }

    fn set_modified(&self, path: &VfsPath, mtime: DateTime<Utc>) -> VfsResult<()> {
        let file = File::options()
            .write(true)
            .open(os_path(path))
            .or_else(|_| File::open(os_path(path)))
            .map_err(|e| map_io(e, path))?;
        file.set_modified(mtime.into()).map_err(|e| map_io(e, path))
    }

    fn local_path(&self, path: &VfsPath) -> Option<PathBuf> {
        Some(os_path(path))
    }

    /// iCloud Drive: what is left of the account; else the disk.
    fn space(&self, path: &VfsPath) -> VfsResult<Option<Space>> {
        let p = os_path(path);
        let in_icloud = crate::cloud_local::cloud_home()
            .is_some_and(|h| p.starts_with(h.join("Library/Mobile Documents")));
        if in_icloud {
            if let Some(s) = crate::space::icloud_quota() {
                return Ok(Some(s));
            }
        }
        Ok(crate::space::disk_space(&p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, VfsPath, LocalFs) {
        let td = tempfile::tempdir().expect("tempdir");
        // Canonicalize: on macOS the temp dir lives behind the /var -> /private/var symlink.
        let canon = fs::canonicalize(td.path()).expect("canonicalize");
        let root = VfsPath::new(&canon.to_string_lossy());
        (td, root, LocalFs::new())
    }

    /// The bulk listing (getattrlistbulk on macOS) gives exactly what
    /// readdir + lstat give, in parts that add up to the whole folder.
    #[test]
    fn list_parts_match_per_file_stat() {
        let (_td, root, l) = setup();
        for i in 0..5000 {
            write(
                &l,
                &root.join(&format!("file {i}.txt")),
                &vec![b'x'; i % 300],
            );
        }
        l.create_dir(&root.join("sub")).unwrap();
        write(&l, &root.join(".dotfile"), b"x");
        write(&l, &root.join("Ünïcødé ✓.md"), b"abc");
        write(&l, &root.join("flagged"), b"x");
        let flagged = root.join("flagged");
        assert!(std::process::Command::new("chflags")
            .args(["hidden", flagged.as_str()])
            .status()
            .unwrap()
            .success());
        fs::set_permissions(
            root.join("Ünïcødé ✓.md").as_str(),
            fs::Permissions::from_mode(0o640),
        )
        .unwrap();
        std::os::unix::fs::symlink("sub", root.join("link-to-dir").as_str()).unwrap();
        std::os::unix::fs::symlink("missing", root.join("broken").as_str()).unwrap();
        let mut parts = 0;
        let mut got = Vec::new();
        l.list_parts(&root, &mut |p| {
            parts += 1;
            got.extend(p)
        })
        .unwrap();
        let mut want: Vec<Entry> = fs::read_dir(root.as_str())
            .unwrap()
            .map(|e| stat_entry(&root, e.unwrap().file_name().to_string_lossy().into_owned()))
            .collect();
        got.sort_by(|a, b| a.name.cmp(&b.name));
        want.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(got.len(), 5006);
        assert!(parts > 1, "a big folder comes in several parts");
        for (g, w) in got.iter().zip(&want) {
            assert_eq!(
                (
                    &g.name,
                    &g.path,
                    g.kind,
                    g.size,
                    g.permissions,
                    &g.link_target,
                    g.hidden
                ),
                (
                    &w.name,
                    &w.path,
                    w.kind,
                    w.size,
                    w.permissions,
                    &w.link_target,
                    w.hidden
                )
            );
            assert_eq!(g.modified, w.modified, "{}", g.name);
        }
        let by = |n: &str| got.iter().find(|e| e.name == n).unwrap();
        assert!(by("flagged").hidden);
        assert!(by(".dotfile").hidden);
        assert_eq!(by("link-to-dir").kind, EntryKind::Directory);
        assert_eq!(by("Ünïcødé ✓.md").permissions, Some(0o640));
        assert_eq!(by("file 299.txt").size, 299);
        assert_eq!(l.list(&root).unwrap().len(), 5006);
    }

    fn write(fs_: &LocalFs, p: &VfsPath, data: &[u8]) {
        let mut w = fs_.open_write(p).expect("open_write");
        w.write_all(data).expect("write");
        w.flush().expect("flush");
    }

    fn read(fs_: &LocalFs, p: &VfsPath) -> Vec<u8> {
        let mut r = fs_.open_read(p).expect("open_read");
        let mut buf = Vec::new();
        r.read_to_end(&mut buf).expect("read");
        buf
    }

    #[test]
    fn home_and_label() {
        let l = LocalFs::new();
        assert_eq!(l.label(), "Local");
        assert_eq!(l.protocol(), Protocol::Local);
        assert!(!l.home().expect("home").is_root() || std::env::var("HOME").as_deref() == Ok("/"));
    }

    #[test]
    fn read_write_roundtrip_and_stat() {
        let (_td, root, l) = setup();
        let p = root.join("hello.txt");
        write(&l, &p, b"hello world");
        assert_eq!(read(&l, &p), b"hello world");
        let e = l.stat(&p).expect("stat");
        assert_eq!(e.name, "hello.txt");
        assert_eq!(e.kind, EntryKind::File);
        assert_eq!(e.size, 11);
        assert!(!e.hidden);
        assert!(e.modified.is_some());
        assert!(e.permissions.is_some());
        assert_eq!(e.path, p);
        assert_eq!(l.local_path(&p), Some(PathBuf::from(p.as_str())));
        // Truncate on rewrite.
        write(&l, &p, b"x");
        assert_eq!(read(&l, &p), b"x");
    }

    #[test]
    fn list_entries() {
        let (_td, root, l) = setup();
        write(&l, &root.join("a.txt"), b"abc");
        write(&l, &root.join(".hidden"), b"");
        l.create_dir(&root.join("sub")).expect("mkdir");
        let mut entries = l.list(&root).expect("list");
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec![".hidden", "a.txt", "sub"]);
        assert!(entries[0].hidden);
        assert_eq!(entries[1].size, 3);
        assert!(entries[2].is_dir());
        assert_eq!(entries[2].path, root.join("sub"));
    }

    /// Applications also lists Apple's apps, like Finder; its own items
    /// win, and a link to a system app isn't hidden.
    #[test]
    fn applications_include_system_apps() {
        let (_td, root, l) = setup();
        let apps = root.join("Applications");
        let system = root.join("System");
        for d in [
            &apps,
            &system,
            &apps.join("Pages.app"),
            &system.join("Mail.app"),
        ] {
            l.create_dir(d).expect("mkdir");
        }
        l.create_dir(&system.join("pages.app")).expect("mkdir");
        l.create_dir(&system.join(".hidden.app")).expect("mkdir");
        std::os::unix::fs::symlink("../System/Mail.app", apps.join("Safari.app").as_str())
            .expect("symlink");
        #[cfg(target_os = "macos")]
        {
            let c = std::ffi::CString::new(apps.join("Safari.app").as_str()).unwrap();
            assert_eq!(
                unsafe { libc::lchflags(c.as_ptr(), 0x8000) },
                0,
                "UF_HIDDEN"
            );
            assert!(l.stat(&apps.join("Safari.app")).expect("stat").hidden);
        }

        let mut out = Vec::new();
        list_with_system_apps(&apps, std::path::Path::new(system.as_str()), &mut |p| {
            out.extend(p)
        })
        .expect("list");
        let mut names: Vec<(&str, &str, bool)> = out
            .iter()
            .map(|e| (e.name.as_str(), e.path.as_str(), e.hidden))
            .collect();
        names.sort();
        let (a, s) = (apps.as_str().to_string(), system.as_str().to_string());
        assert_eq!(
            names,
            [
                ("Mail.app", format!("{s}/Mail.app").as_str(), false),
                ("Pages.app", format!("{a}/Pages.app").as_str(), false),
                ("Safari.app", format!("{a}/Safari.app").as_str(), false),
            ]
        );
        assert_eq!(system_apps_of(std::path::Path::new("/Users")), None);

        // The real one, where there is one (macOS).
        if std::path::Path::new("/System/Applications/Calculator.app").exists() {
            let real = l.list(&VfsPath::new("/Applications")).expect("list");
            let calc = real
                .iter()
                .find(|e| e.name == "Calculator.app")
                .expect("Calculator");
            assert_eq!(calc.path.as_str(), "/System/Applications/Calculator.app");
            if let Some(safari) = real.iter().find(|e| e.name == "Safari.app") {
                assert!(!safari.hidden, "Safari shows like in Finder");
            }
        }
    }

    #[test]
    fn symlinks() {
        let (_td, root, l) = setup();
        l.create_dir(&root.join("dir")).expect("mkdir");
        write(&l, &root.join("file"), b"12345");
        std::os::unix::fs::symlink(root.join("dir").as_str(), root.join("ldir").as_str())
            .expect("symlink");
        std::os::unix::fs::symlink("file", root.join("lfile").as_str()).expect("symlink");
        std::os::unix::fs::symlink("missing", root.join("broken").as_str()).expect("symlink");

        let ldir = l.stat(&root.join("ldir")).expect("stat");
        assert_eq!(ldir.kind, EntryKind::Directory);
        assert_eq!(ldir.link_target.as_deref(), Some(root.join("dir").as_str()));

        let lfile = l.stat(&root.join("lfile")).expect("stat");
        assert_eq!(lfile.kind, EntryKind::Symlink);
        assert_eq!(lfile.link_target.as_deref(), Some("file"));
        assert_eq!(lfile.size, 5);

        let broken = l.stat(&root.join("broken")).expect("stat");
        assert_eq!(broken.kind, EntryKind::Symlink);

        // Listing through a symlinked dir works; removing the link keeps the target.
        assert!(l.list(&root.join("ldir")).expect("list").is_empty());
        l.remove_file(&root.join("ldir")).expect("rm link");
        assert!(l.stat(&root.join("dir")).is_ok());
        assert_eq!(l.list(&root).expect("list").len(), 4);
    }

    #[test]
    fn create_remove_dir() {
        let (_td, root, l) = setup();
        let d = root.join("d");
        l.create_dir(&d).expect("mkdir");
        assert!(matches!(l.create_dir(&d), Err(VfsError::AlreadyExists(_))));
        write(&l, &d.join("f"), b"x");
        assert!(matches!(
            l.remove_dir(&d),
            Err(VfsError::DirectoryNotEmpty(_))
        ));
        assert!(matches!(
            l.remove_dir(&d.join("f")),
            Err(VfsError::NotADirectory(_))
        ));
        assert!(l.remove_file(&d).is_err());
        l.remove_file(&d.join("f")).expect("rm");
        l.remove_dir(&d).expect("rmdir");
        assert!(matches!(l.stat(&d), Err(VfsError::NotFound(_))));
    }

    #[test]
    fn rename_entries() {
        let (_td, root, l) = setup();
        let a = root.join("a");
        let b = root.join("b");
        write(&l, &a, b"data");
        l.rename(&a, &b).expect("rename");
        assert!(matches!(l.stat(&a), Err(VfsError::NotFound(_))));
        assert_eq!(read(&l, &b), b"data");
        match l.rename(&a, &root.join("c")) {
            Err(VfsError::NotFound(p)) => assert_eq!(p, a.to_string()),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn resume_read_at_and_append() {
        let (_td, root, l) = setup();
        let a = root.join("a");
        write(&l, &a, b"0123456789");
        let mut out = Vec::new();
        l.open_read_at(&a, 4)
            .expect("read_at")
            .read_to_end(&mut out)
            .expect("read");
        assert_eq!(out, b"456789");
        assert!(l.open_read_at(&a, 11).is_err());
        {
            let mut w = l.open_append(&a).expect("append");
            w.write_all(b"abc").expect("write");
        }
        assert_eq!(read(&l, &a), b"0123456789abc");
        let caps = l.supports_resume();
        assert!(caps.read_at && caps.append && !caps.upload_session);
    }

    #[test]
    fn error_mapping() {
        let (_td, root, l) = setup();
        let missing = root.join("nope");
        assert!(matches!(l.stat(&missing), Err(VfsError::NotFound(_))));
        assert!(matches!(l.list(&missing), Err(VfsError::NotFound(_))));
        assert!(matches!(l.open_read(&missing), Err(VfsError::NotFound(_))));
        assert!(matches!(
            l.remove_file(&missing),
            Err(VfsError::NotFound(_))
        ));
        assert!(matches!(
            l.open_write(&root.join("nope/x")),
            Err(VfsError::NotFound(_))
        ));
        let f = root.join("f");
        write(&l, &f, b"");
        assert!(matches!(l.list(&f), Err(VfsError::NotADirectory(_))));
        assert!(l.open_read(&root).is_err());
    }

    #[test]
    fn permissions() {
        let (_td, root, l) = setup();
        let f = root.join("f");
        write(&l, &f, b"secret");
        l.set_permissions(&f, 0o600).expect("chmod");
        assert_eq!(l.stat(&f).expect("stat").permissions, Some(0o600));

        // Unreadable directory -> PermissionDenied (skip when running as root).
        let d = root.join("locked");
        l.create_dir(&d).expect("mkdir");
        l.set_permissions(&d, 0o000).expect("chmod");
        let res = l.list(&d);
        l.set_permissions(&d, 0o755).expect("chmod back");
        let is_root = std::env::var("USER").as_deref() == Ok("root");
        if !is_root {
            assert!(matches!(res, Err(VfsError::PermissionDenied(_))), "{res:?}");
        }
    }
}
