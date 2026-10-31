//! Pure helpers (formatting, sorting, name collision handling) that are easy
//! to unit test and have no UI dependencies.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Local, Utc};
use manta_core::{Entry, EntryKind, SharedVfs};
use manta_plugin::SizeInfo;

/// Human readable size using decimal units like Finder ("1.2 MB", German
/// "1,2 MB").
pub fn format_size(bytes: u64) -> String {
    manta_plugin::i18n::format_size(bytes)
}

/// Thousands separators: "12,345" (German "12.345").
pub fn format_count(n: u64) -> String {
    crate::i18n::format_count(n)
}

/// `ls -l` style permission string, e.g. `drwxr-xr-x`.
pub fn format_permissions(mode: Option<u32>, kind: EntryKind) -> String {
    let Some(mode) = mode else {
        return "—".to_string();
    };
    let type_char = match kind {
        EntryKind::Directory => 'd',
        EntryKind::Symlink => 'l',
        EntryKind::File => '-',
        EntryKind::Other => '?',
    };
    let mut s = String::with_capacity(10);
    s.push(type_char);
    let triples = [(mode >> 6) & 7, (mode >> 3) & 7, mode & 7];
    for (i, t) in triples.iter().enumerate() {
        s.push(if t & 4 != 0 { 'r' } else { '-' });
        s.push(if t & 2 != 0 { 'w' } else { '-' });
        let exec = t & 1 != 0;
        // setuid / setgid / sticky bits
        let special = match i {
            0 => mode & 0o4000 != 0,
            1 => mode & 0o2000 != 0,
            _ => mode & 0o1000 != 0,
        };
        s.push(match (special, exec, i) {
            (true, true, 2) => 't',
            (true, false, 2) => 'T',
            (true, true, _) => 's',
            (true, false, _) => 'S',
            (false, true, _) => 'x',
            (false, false, _) => '-',
        });
    }
    s
}

/// Finder-style date: "Today 14:02", "Yesterday 09:15", "12 Oct 14:02" (this
/// year) or "12 Oct 2024"; German "Heute 14:02", "Gestern 09:15",
/// "12. Okt. 14:02", "12. Okt. 2024". `now` is passed in for testability.
pub fn format_time_relative(t: Option<DateTime<Utc>>, now: DateTime<Local>) -> String {
    let Some(t) = t else {
        return "—".to_string();
    };
    let t = t.with_timezone(&Local);
    let days = now
        .date_naive()
        .signed_duration_since(t.date_naive())
        .num_days();
    let time = t.format("%H:%M").to_string();
    match days {
        0 => t!("date.today", time = time),
        1 => t!("date.yesterday", time = time),
        _ if t.year() == now.year() && days > 0 => {
            crate::i18n::format_day(t.day(), t.month0(), None, Some(&time))
        }
        _ => crate::i18n::format_day(t.day(), t.month0(), Some(t.year()), None),
    }
}

/// "12 MB/s"; Russian "12 МБ/с", Chinese "12 MB/秒".
pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec <= 0.0 {
        return "—".to_string();
    }
    let per_sec = match crate::i18n::locale() {
        crate::i18n::Locale::Ru => "/с",
        crate::i18n::Locale::Zh => "/秒",
        _ => "/s",
    };
    format!("{}{per_sec}", format_size(bytes_per_sec as u64))
}

/// Hours, minutes and seconds abbreviations of the active language.
fn time_units() -> [&'static str; 3] {
    match crate::i18n::locale() {
        crate::i18n::Locale::Ru => ["\u{a0}ч", "\u{a0}мин", "\u{a0}с"],
        crate::i18n::Locale::Zh => ["小时", "分", "秒"],
        crate::i18n::Locale::Fr => ["\u{a0}h", "\u{a0}min", "\u{a0}s"],
        crate::i18n::Locale::Es | crate::i18n::Locale::It => [" h", " min", " s"],
        _ => ["h", "m", "s"],
    }
}

/// Time left: "10s", "1m 05s", "2h 03m" (French "1 min 05 s", Russian
/// "1 мин 05 с", Chinese "1分05秒").
pub fn format_eta(remaining_bytes: u64, bytes_per_sec: f64) -> String {
    if bytes_per_sec <= 1.0 || remaining_bytes == 0 {
        return "—".to_string();
    }
    let secs = (remaining_bytes as f64 / bytes_per_sec).round() as u64;
    let [h, m, s] = time_units();
    let gap = if crate::i18n::locale() == crate::i18n::Locale::Zh {
        ""
    } else {
        " "
    };
    if secs >= 3600 {
        format!("{}{h}{gap}{:02}{m}", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}{m}{gap}{:02}{s}", secs / 60, secs % 60)
    } else {
        format!("{secs}{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
    Permissions,
    /// The folder a search result is in (search results only).
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortSpec {
    pub column: SortColumn,
    pub ascending: bool,
}

impl Default for SortSpec {
    fn default() -> Self {
        SortSpec {
            column: SortColumn::Name,
            ascending: true,
        }
    }
}

impl SortSpec {
    /// Clicking a header: same column flips direction, new column sorts ascending.
    pub fn clicked(self, column: SortColumn) -> SortSpec {
        if self.column == column {
            SortSpec {
                column,
                ascending: !self.ascending,
            }
        } else {
            SortSpec {
                column,
                ascending: true,
            }
        }
    }
}

/// Case-insensitive "natural" comparison: `file2` < `file10`.
///
/// Allocation-free (sorting 300,000 names calls it millions of times):
/// ASCII is compared byte by byte, other characters by their first
/// lowercase character; digit runs by value; full ties by the bytes.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (ab, bb) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0, 0);
    loop {
        let (x, y) = match (ab.get(i), bb.get(j)) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(&x), Some(&y)) => (x, y),
        };
        if x.is_ascii_digit() && y.is_ascii_digit() {
            let (si, sj) = (i, j);
            while ab.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            while bb.get(j).is_some_and(u8::is_ascii_digit) {
                j += 1;
            }
            let trim = |d: &[u8]| -> usize { d.iter().take_while(|&&c| c == b'0').count() };
            let (na, nb) = (&ab[si..i], &bb[sj..j]);
            let (na, nb) = (&na[trim(na)..], &nb[trim(nb)..]);
            let ord = na.len().cmp(&nb.len()).then_with(|| na.cmp(nb));
            if ord != Ordering::Equal {
                return ord;
            }
        } else if x.is_ascii() && y.is_ascii() {
            let (lx, ly) = (x.to_ascii_lowercase(), y.to_ascii_lowercase());
            if lx != ly {
                return lx.cmp(&ly);
            }
            i += 1;
            j += 1;
        } else {
            // Both are at a char boundary: only whole ASCII bytes or whole
            // characters are ever skipped.
            let ca = a[i..].chars().next().unwrap_or_default();
            let cb = b[j..].chars().next().unwrap_or_default();
            let la = ca.to_lowercase().next().unwrap_or(ca);
            let lb = cb.to_lowercase().next().unwrap_or(cb);
            if la != lb {
                return la.cmp(&lb);
            }
            i += ca.len_utf8();
            j += cb.len_utf8();
        }
    }
}

/// `hay` contains `needle_lower` (already lowercase), ignoring case;
/// without allocating when both are ASCII.
pub fn contains_lowercase(hay: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if hay.is_ascii() && needle_lower.is_ascii() {
        let (h, n) = (hay.as_bytes(), needle_lower.as_bytes());
        return h.len() >= n.len()
            && h.windows(n.len())
                .any(|w| w.iter().zip(n).all(|(a, b)| a.to_ascii_lowercase() == *b));
    }
    hay.to_lowercase().contains(needle_lower)
}

/// `name` starts with `prefix_lower` (already lowercase), ignoring case.
pub fn starts_with_lowercase(name: &str, prefix_lower: &str) -> bool {
    if name.is_ascii() && prefix_lower.is_ascii() {
        let (h, n) = (name.as_bytes(), prefix_lower.as_bytes());
        return h.len() >= n.len() && h.iter().zip(n).all(|(a, b)| a.to_ascii_lowercase() == *b);
    }
    name.to_lowercase().starts_with(prefix_lower)
}

/// `name` lowercased compared with `other_lower` (already lowercase);
/// without allocating for ASCII names.
pub fn cmp_lowercase(name: &str, other_lower: &str) -> Ordering {
    if name.is_ascii() {
        name.bytes()
            .map(|b| b.to_ascii_lowercase())
            .cmp(other_lower.bytes())
    } else {
        name.to_lowercase().as_str().cmp(other_lower)
    }
}

/// Folder extensions that are packages (like Finder): sorted like files,
/// and a search doesn't look inside them.
const PACKAGES: &[&str] = &[
    "app",
    "bundle",
    "framework",
    "photoslibrary",
    "musiclibrary",
    "tvlibrary",
    "xcarchive",
    "plugin",
    "appex",
    "kext",
];

pub fn is_package(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && PACKAGES.contains(&ext.to_lowercase().as_str())
    })
}

/// The size used for sorting: plugin-measured folder sizes where known.
pub fn sort_size(e: &Entry, dir_sizes: &HashMap<String, SizeInfo>) -> u64 {
    if e.is_dir() {
        dir_sizes.get(&e.name).map_or(0, |s| s.bytes)
    } else {
        e.size
    }
}

/// Protected by macOS (System Integrity Protection, the `restricted`
/// flag): Apple's apps and the link to Safari can't be renamed, moved or
/// deleted, not even by an administrator.
pub fn sip_protected(path: &std::path::Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        use std::os::macos::fs::MetadataExt;
        const SF_RESTRICTED: u32 = 0x0008_0000;
        std::fs::symlink_metadata(path).is_ok_and(|m| m.st_flags() & SF_RESTRICTED != 0)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        false
    }
}

/// Files Finder never shows, not even with hidden files on: macOS
/// recreates `.DS_Store` in every folder Finder opens.
pub fn never_listed(name: &str) -> bool {
    display_name(name) == ".DS_Store"
}

/// How many of `entries` a pane can show (with hidden files on).
pub fn listable_count(entries: &[Entry]) -> usize {
    entries.iter().filter(|e| !never_listed(&e.name)).count()
}

/// Directories (not packages) always first, like Finder, then by the
/// chosen column. Ties fall back to the name. Folders use their
/// plugin-measured size (by name) when sorting by size.
pub fn compare_entries(
    a: &Entry,
    b: &Entry,
    sort: SortSpec,
    dir_sizes: &HashMap<String, SizeInfo>,
) -> Ordering {
    // Packages (`Safari.app`) sort like files, as in Finder.
    let dir_a = a.is_dir() && !is_package(&a.name);
    let dir_b = b.is_dir() && !is_package(&b.name);
    if dir_a != dir_b {
        return if dir_a {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    // Search results are named by their relative path (unique in the
    // list); they sort by the file name, then the path.
    let by_name = || {
        natural_cmp(display_name(&a.name), display_name(&b.name))
            .then_with(|| natural_cmp(&a.name, &b.name))
    };
    let ord = match sort.column {
        SortColumn::Name => by_name(),
        SortColumn::Size => sort_size(a, dir_sizes)
            .cmp(&sort_size(b, dir_sizes))
            .then_with(by_name),
        SortColumn::Modified => a.modified.cmp(&b.modified).then_with(by_name),
        SortColumn::Permissions => a.permissions.cmp(&b.permissions).then_with(by_name),
        SortColumn::Path => {
            natural_cmp(display_dir(&a.name), display_dir(&b.name)).then_with(by_name)
        }
    };
    if sort.ascending {
        ord
    } else {
        ord.reverse()
    }
}

/// Indices into `entries` after filtering (hidden + substring) and sorting.
/// Folders are sorted by `dir_sizes` (plugin-measured) when sorting by size.
pub fn visible_indices(
    entries: &[Entry],
    sort: SortSpec,
    show_hidden: bool,
    filter: &str,
    dir_sizes: &HashMap<String, SizeInfo>,
) -> Vec<usize> {
    visible_indices_tagged(entries, sort, show_hidden, filter, dir_sizes, None)
}

/// [`visible_indices`] where the filter may name tags (`tag:Rot`, see
/// [`crate::tags::split_filter`]); `tags` are the items' tags by name.
pub fn visible_indices_tagged(
    entries: &[Entry],
    sort: SortSpec,
    show_hidden: bool,
    filter: &str,
    dir_sizes: &HashMap<String, SizeInfo>,
    tags: Option<&crate::tags::FolderTags>,
) -> Vec<usize> {
    let (want_tags, rest) = crate::tags::split_filter(filter);
    let needle = rest.trim().to_lowercase();
    let mut idx: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| !never_listed(&e.name))
        .filter(|(_, e)| show_hidden || !(e.hidden || display_name(&e.name).starts_with('.')))
        .filter(|(_, e)| contains_lowercase(display_name(&e.name), &needle))
        .filter(|(_, e)| {
            want_tags.is_empty()
                || crate::tags::matches_tags(
                    tags.and_then(|t| t.get(&e.name))
                        .map_or(&[], |v| v.as_slice()),
                    &want_tags,
                )
        })
        .map(|(i, _)| i)
        .collect();
    // Names are unique in a folder and every column falls back to the
    // name, so the order is total: an unstable sort gives the same result,
    // faster.
    idx.sort_unstable_by(|&x, &y| compare_entries(&entries[x], &entries[y], sort, dir_sizes));
    idx
}

/// Pick a name not contained in `existing`: `base`, `base 2`, `base 3`, ...
/// The counter is inserted before the extension for files (`a.txt` -> `a 2.txt`).
pub fn unique_name<S: AsRef<str>>(existing: &[S], base: &str) -> String {
    let taken = |n: &str| existing.iter().any(|e| e.as_ref() == n);
    if !taken(base) {
        return base.to_string();
    }
    let (stem, ext) = match base.rfind('.') {
        Some(i) if i > 0 => (&base[..i], &base[i..]),
        _ => (base, ""),
    };
    (2..)
        .map(|n| format!("{stem} {n}{ext}"))
        .find(|candidate| !taken(candidate))
        .expect("infinite iterator")
}

/// What is wrong with a user-entered file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameProblem {
    Empty,
    /// "." or "..".
    Reserved,
    /// Contains "/" (or NUL).
    Slash,
}

/// The problem of a user-entered file name, if any.
pub fn file_name_problem(name: &str) -> Option<NameProblem> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Some(NameProblem::Empty)
    } else if trimmed == "." || trimmed == ".." {
        Some(NameProblem::Reserved)
    } else if trimmed.contains('/') || trimmed.contains('\0') {
        Some(NameProblem::Slash)
    } else {
        None
    }
}

/// Validate a user-entered file name. Returns an error message if invalid.
pub fn validate_file_name(name: &str) -> Result<(), String> {
    match file_name_problem(name) {
        None => Ok(()),
        Some(NameProblem::Empty) => Err(t!("error.name_empty").into()),
        Some(NameProblem::Reserved) => Err(t!("error.name_reserved").into()),
        Some(NameProblem::Slash) => Err(t!("error.name_slash").into()),
    }
}

/// The name to show for an entry: its file name. Search results are named
/// by their path relative to where the search started (`src/main.rs`,
/// unique in the list); file names never contain `/`.
pub fn display_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

/// The folder part of a search result's name (`src` for `src/main.rs`,
/// `/` for `/main.rs`, empty for a plain name).
pub fn display_dir(name: &str) -> &str {
    name.rsplit_once('/')
        .map_or("", |(d, _)| if d.is_empty() { "/" } else { d })
}

/// Identity of a backend instance (two tabs share a backend iff the Arcs match).
pub fn same_fs(a: &SharedVfs, b: &SharedVfs) -> bool {
    std::ptr::addr_eq(Arc::as_ptr(a), Arc::as_ptr(b))
}

/// Guess whether a file should be previewed as an image by its extension.
pub fn is_image_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".bmp", ".webp"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use manta_core::VfsPath;

    fn entry(name: &str, kind: EntryKind, size: u64) -> Entry {
        Entry {
            name: name.into(),
            path: VfsPath::new(&format!("/x/{name}")),
            kind,
            size,
            modified: None,
            permissions: None,
            link_target: None,
            hidden: name.starts_with('.'),
        }
    }

    #[test]
    fn sizes() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(1), "1 byte");
        assert_eq!(format_size(999), "999 bytes");
        assert_eq!(format_size(1_000), "1.0 KB");
        assert_eq!(format_size(1_234_567), "1.2 MB");
        assert_eq!(format_size(45_000_000), "45 MB");
        assert_eq!(format_size(3_000_000_000_000), "3.0 TB");
    }

    #[test]
    fn german_sizes_and_counts() {
        crate::i18n::with_locale(crate::i18n::Locale::De, || {
            assert_eq!(format_size(1), "1 Byte");
            assert_eq!(format_size(999), "999 Byte");
            assert_eq!(format_size(1_234_567), "1,2 MB");
            assert_eq!(format_size(45_000_000), "45 MB");
            assert_eq!(format_count(12_345), "12.345");
        });
    }

    #[test]
    fn permissions() {
        assert_eq!(
            format_permissions(Some(0o755), EntryKind::Directory),
            "drwxr-xr-x"
        );
        assert_eq!(
            format_permissions(Some(0o644), EntryKind::File),
            "-rw-r--r--"
        );
        assert_eq!(
            format_permissions(Some(0o1777), EntryKind::Directory),
            "drwxrwxrwt"
        );
        assert_eq!(
            format_permissions(Some(0o4755), EntryKind::File),
            "-rwsr-xr-x"
        );
        assert_eq!(
            format_permissions(Some(0o777), EntryKind::Symlink),
            "lrwxrwxrwx"
        );
        assert_eq!(format_permissions(None, EntryKind::File), "—");
    }

    #[test]
    fn relative_dates() {
        use chrono::TimeZone;
        let now = Local.with_ymd_and_hms(2026, 10, 20, 15, 0, 0).unwrap();
        let at = |y, mo, d, h, mi| {
            Some(
                Local
                    .with_ymd_and_hms(y, mo, d, h, mi, 0)
                    .unwrap()
                    .with_timezone(&Utc),
            )
        };
        assert_eq!(
            format_time_relative(at(2026, 10, 20, 9, 41), now),
            "Today 09:41"
        );
        assert_eq!(
            format_time_relative(at(2026, 10, 19, 23, 5), now),
            "Yesterday 23:05"
        );
        assert_eq!(
            format_time_relative(at(2026, 3, 2, 8, 0), now),
            "2 Mar 08:00"
        );
        assert_eq!(
            format_time_relative(at(2025, 12, 31, 8, 0), now),
            "31 Dec 2025"
        );
        assert_eq!(format_time_relative(None, now), "—");
        crate::i18n::with_locale(crate::i18n::Locale::De, || {
            assert_eq!(
                format_time_relative(at(2026, 10, 20, 14, 2), now),
                "Heute 14:02"
            );
            assert_eq!(
                format_time_relative(at(2026, 10, 19, 9, 41), now),
                "Gestern 09:41"
            );
            assert_eq!(
                format_time_relative(at(2026, 10, 12, 9, 41), now),
                "12. Okt. 09:41"
            );
            assert_eq!(
                format_time_relative(at(2025, 10, 12, 9, 41), now),
                "12. Okt. 2025"
            );
            assert_eq!(
                format_time_relative(at(2026, 3, 2, 8, 0), now),
                "2. März 08:00"
            );
        });
    }

    #[test]
    fn eta() {
        assert_eq!(format_eta(100, 10.0), "10s");
        assert_eq!(format_eta(6000, 10.0), "10m 00s");
        assert_eq!(format_eta(100, 0.0), "—");
        crate::i18n::with_locale(crate::i18n::Locale::Ru, || {
            assert_eq!(format_eta(6000, 10.0), "10\u{a0}мин 00\u{a0}с");
            assert_eq!(format_speed(12_000_000.0), "12\u{a0}МБ/с");
        });
        crate::i18n::with_locale(crate::i18n::Locale::Zh, || {
            assert_eq!(format_eta(6000, 10.0), "10分00秒")
        });
    }

    #[test]
    fn natural() {
        assert_eq!(natural_cmp("file2", "file10"), Ordering::Less);
        assert_eq!(natural_cmp("File", "file"), Ordering::Less); // tie broken by byte cmp
        assert_eq!(natural_cmp("abc", "ABD"), Ordering::Less);
        assert_eq!(natural_cmp("a", "a1"), Ordering::Less);
    }

    /// The previous (allocating) implementation, as the reference.
    fn natural_cmp_reference(a: &str, b: &str) -> Ordering {
        let mut ai = a.chars().peekable();
        let mut bi = b.chars().peekable();
        loop {
            match (ai.peek().copied(), bi.peek().copied()) {
                (None, None) => return a.cmp(b),
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(ca), Some(cb)) if ca.is_ascii_digit() && cb.is_ascii_digit() => {
                    let mut na = String::new();
                    while let Some(c) = ai.peek().copied().filter(char::is_ascii_digit) {
                        na.push(c);
                        ai.next();
                    }
                    let mut nb = String::new();
                    while let Some(c) = bi.peek().copied().filter(char::is_ascii_digit) {
                        nb.push(c);
                        bi.next();
                    }
                    let ta = na.trim_start_matches('0');
                    let tb = nb.trim_start_matches('0');
                    let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                (Some(ca), Some(cb)) => {
                    let la = ca.to_lowercase().next().unwrap_or(ca);
                    let lb = cb.to_lowercase().next().unwrap_or(cb);
                    if la != lb {
                        return la.cmp(&lb);
                    }
                    ai.next();
                    bi.next();
                }
            }
        }
    }

    #[test]
    fn natural_matches_the_reference() {
        let names = [
            "",
            "a",
            "A",
            "a1",
            "a01",
            "a001b",
            "a1b",
            "a10",
            "a9",
            "file2",
            "file10",
            "File10",
            "Ärger",
            "ärger",
            "arger",
            "Zebra",
            "zebra",
            "über",
            "Uber",
            "007",
            "7",
            "0",
            "00",
            "x0y",
            "x00y",
            "x1y",
            "IMG_000123.png",
            "IMG_123.png",
            "img_0123.PNG",
            "Résumé",
            "resume",
            "ÉCOLE",
            "école",
            "日本1",
            "日本10",
            "日本2",
            "\u{212a}elvin",
            "kelvin",
            "Ⅻ",
            "ⅻ",
            "notes 12 draft.txt",
            "notes 2 draft.txt",
            "a b",
            "a-b",
            "a_b",
            "a.b",
        ];
        for a in names {
            for b in names {
                assert_eq!(
                    natural_cmp(a, b),
                    natural_cmp_reference(a, b),
                    "{a:?} vs {b:?}"
                );
            }
        }
    }

    #[test]
    fn case_insensitive_helpers() {
        assert!(contains_lowercase("Hello World.TXT", "world"));
        assert!(contains_lowercase("anything", ""));
        assert!(!contains_lowercase("abc", "abcd"));
        assert!(contains_lowercase("Über.txt", "über"));
        assert!(contains_lowercase("Über.txt", "ber"));
        assert!(starts_with_lowercase("README.md", "read"));
        assert!(!starts_with_lowercase("RE", "read"));
        assert!(starts_with_lowercase("Ärger", "är"));
        assert_eq!(cmp_lowercase("ABC", "abc"), Ordering::Equal);
        assert_eq!(cmp_lowercase("ABD", "abc"), Ordering::Greater);
        assert_eq!(cmp_lowercase("Ä", "ä"), Ordering::Equal);
    }

    /// Apple's apps are protected by macOS; ordinary files aren't.
    #[test]
    fn sip_protected_items() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!sip_protected(dir.path()));
        let calc = std::path::Path::new("/System/Applications/Calculator.app");
        if calc.exists() {
            assert!(sip_protected(calc));
        }
    }

    #[test]
    fn sorting_dirs_first() {
        let entries = vec![
            entry("b.txt", EntryKind::File, 10),
            entry("zdir", EntryKind::Directory, 0),
            entry("a.txt", EntryKind::File, 50),
            entry(".hidden", EntryKind::File, 1),
            entry("adir", EntryKind::Directory, 0),
        ];
        let names = |idx: Vec<usize>| {
            idx.into_iter()
                .map(|i| entries[i].name.clone())
                .collect::<Vec<_>>()
        };

        let v = visible_indices(&entries, SortSpec::default(), false, "", &HashMap::new());
        assert_eq!(names(v), ["adir", "zdir", "a.txt", "b.txt"]);

        let desc = SortSpec {
            column: SortColumn::Size,
            ascending: false,
        };
        let v = visible_indices(&entries, desc, true, "", &HashMap::new());
        assert_eq!(names(v), ["zdir", "adir", "a.txt", "b.txt", ".hidden"]);

        let v = visible_indices(&entries, SortSpec::default(), false, "TXT", &HashMap::new());
        assert_eq!(names(v), ["a.txt", "b.txt"]);

        // .DS_Store never shows, like in Finder, not even with hidden files.
        let ds = vec![
            entry(".DS_Store", EntryKind::File, 6148),
            entry(".env", EntryKind::File, 1),
        ];
        let v = visible_indices(&ds, SortSpec::default(), true, "", &HashMap::new());
        assert_eq!(v, [1]);
        assert_eq!(listable_count(&ds), 1);

        // Packages sort among the files, like Finder ("Utilities" on top).
        let apps = vec![
            entry("Calculator.app", EntryKind::Directory, 0),
            entry("Utilities", EntryKind::Directory, 0),
            entry("Books.app", EntryKind::Directory, 0),
            entry("notes.txt", EntryKind::File, 1),
        ];
        let v = visible_indices(&apps, SortSpec::default(), false, "", &HashMap::new());
        let order: Vec<&str> = v.iter().map(|&i| apps[i].name.as_str()).collect();
        assert_eq!(
            order,
            ["Utilities", "Books.app", "Calculator.app", "notes.txt"]
        );

        // Plugin-measured folder sizes take part in size sorting.
        let sizes = HashMap::from([(
            "adir".to_string(),
            SizeInfo {
                bytes: 1_000,
                complete: true,
                ..Default::default()
            },
        )]);
        let v = visible_indices(&entries, desc, false, "", &sizes);
        assert_eq!(names(v), ["adir", "zdir", "a.txt", "b.txt"]);
        let asc = SortSpec {
            column: SortColumn::Size,
            ascending: true,
        };
        let v = visible_indices(&entries, asc, false, "", &sizes);
        assert_eq!(names(v), ["zdir", "adir", "b.txt", "a.txt"]);
    }

    #[test]
    fn sort_spec_toggle() {
        let s = SortSpec::default().clicked(SortColumn::Name);
        assert!(!s.ascending);
        let s = s.clicked(SortColumn::Size);
        assert_eq!(
            s,
            SortSpec {
                column: SortColumn::Size,
                ascending: true
            }
        );
    }

    #[test]
    fn unique_names() {
        let existing = ["untitled folder", "untitled folder 2", "a.txt", ".bashrc"];
        assert_eq!(
            unique_name(&existing, "untitled folder"),
            "untitled folder 3"
        );
        assert_eq!(unique_name(&existing, "a.txt"), "a 2.txt");
        assert_eq!(unique_name(&existing, ".bashrc"), ".bashrc 2");
        assert_eq!(unique_name(&existing, "new"), "new");
    }

    #[test]
    fn name_validation() {
        assert!(validate_file_name("ok.txt").is_ok());
        assert!(validate_file_name("  ").is_err());
        assert!(validate_file_name("a/b").is_err());
        assert!(validate_file_name("..").is_err());
    }

    #[test]
    fn image_detection() {
        assert!(is_image_name("Photo.JPG"));
        assert!(!is_image_name("notes.txt"));
    }
}
