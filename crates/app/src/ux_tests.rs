//! Headless render tests: build the real `MantaApp` against a fixture
//! directory and render PNGs with `egui_kittest` + wgpu.
//!
//! The renders are written to `$MANTA_RENDER_DIR` (default:
//! `target/ux-renders`). They are not compared against reference images (so
//! they can't be flaky across GPUs / font versions); the tests make sure every
//! screen renders without panicking and give designers something to look at.
//! When no wgpu adapter is available the tests skip.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Theme};
use egui_kittest::Harness;
use manta_core::{Entry, EntryKind, VfsPath};
use manta_ops::{CompareItem, CompareStatus, JobKind, JobState, JobStatus};

use manta_plugin::external::ExternalPlugin;
use manta_plugin::host::PluginHost;
use manta_plugin::manifest::Manifest;
use manta_plugin::protocol::{Block, ButtonSpec, KeyValueRow, ListRow};
use manta_plugin::{
    Badge, Chip, CommandContext, CommandSpec, Decoration, HostRequest, PaneId, Plugin, PluginEvent,
    PluginHandle, PluginInfo, PluginStatus, Tone,
};

use super::{AppInit, MantaApp, Side};
use crate::dialogs::{ConnectForm, DeleteForm, Dialog, TransferForm};
use crate::favorites::{Favorite, Favorites};
use crate::icons;
use crate::preview::Preview;
use crate::sidebar::Place;
use crate::worker::PreviewContent;

const WIDE: [f32; 2] = [1280.0, 800.0];
const NARROW: [f32; 2] = [900.0, 600.0];

/// `None` if wgpu can't create a device here (CI without GPU etc.).
fn wgpu_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        let ok = std::panic::catch_unwind(|| {
            egui_kittest::wgpu::create_render_state(
                egui_kittest::wgpu::default_wgpu_setup(),
                egui_wgpu::RendererOptions::PREDICTABLE,
            )
        })
        .is_ok();
        if !ok {
            eprintln!("ux_tests: no wgpu adapter available, skipping renders");
        }
        ok
    })
}

fn out_dir() -> PathBuf {
    let dir = std::env::var_os("MANTA_RENDER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/ux-renders"));
    std::fs::create_dir_all(&dir).expect("create render dir");
    dir
}

// ------------------------------------------------------------------ fixture

struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

fn ago(secs: u64) -> SystemTime {
    // Anchor "today" at 09:41 local-ish so relative dates look natural.
    SystemTime::now() - Duration::from_secs(secs)
}

fn touch(path: &Path, len: u64, mtime: SystemTime) {
    let f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .unwrap();
    if len > 0 && f.metadata().unwrap().len() == 0 {
        f.set_len(len).unwrap();
    }
    f.set_modified(mtime).unwrap();
}

fn write(path: &Path, text: &str, mtime: SystemTime) {
    std::fs::write(path, text).unwrap();
    touch(path, 0, mtime);
}

fn mkdir(path: &Path, mtime: SystemTime) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::File::open(path)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
}

const README: &str = "# Projects\n\nA small fixture tree used by the Manta render tests.\n\n\
- `src/` — Rust sources\n- `Design/` — mockups and exports\n- `Archive 2025/` — intentionally empty\n\n\
## Building\n\n```sh\ncargo run --release\n```\n";

fn fixture() -> Fixture {
    fixture_at("Projects")
}

/// [`fixture`] at `sub` inside the temp dir (the manual's screenshots use
/// `Users/anna/Projects`, so long breadcrumbs read like a real home).
fn fixture_at(sub: &str) -> Fixture {
    let tmp = tempfile::Builder::new()
        .prefix("manta-ux")
        .tempdir()
        .unwrap();
    let root = tmp.path().join(sub);
    let h = 3600;
    let d = 24 * h;
    std::fs::create_dir_all(&root).unwrap();
    // Folders
    mkdir(&root.join("src"), ago(2 * h));
    write(
        &root.join("src/main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
        ago(2 * h),
    );
    write(&root.join("src/lib.rs"), "pub mod app;\n", ago(5 * d));
    mkdir(&root.join("Design"), ago(d + 3 * h));
    touch(
        &root.join("Design/hero-banner.png"),
        2_480_000,
        ago(d + 3 * h),
    );
    touch(&root.join("Design/app-icon.svg"), 18_400, ago(9 * d));
    touch(&root.join("Design/Brand Guide.pdf"), 6_200_000, ago(40 * d));
    mkdir(&root.join("Archive 2025"), ago(200 * d));
    mkdir(&root.join("photos"), ago(12 * d));
    mkdir(&root.join(".git"), ago(h));
    // Files
    write(&root.join("README.md"), README, ago(25 * 60));
    touch(&root.join("report-q3.pdf"), 1_840_000, ago(3 * d));
    touch(&root.join("backup-2026-10-01.zip"), 48_300_000, ago(19 * d));
    touch(
        &root.join("podcast-episode-12.mp3"),
        23_700_000,
        ago(30 * d),
    );
    touch(&root.join("screen-recording.mov"), 312_000_000, ago(2 * d));
    write(
        &root.join("data.json"),
        "{\"items\": [1, 2, 3]}\n",
        ago(400 * d),
    );
    write(&root.join("notes.txt"), "Buy milk\n", ago(d + 6 * h));
    write(&root.join(".env"), "TOKEN=secret\n", ago(6 * d));
    write(
        &root.join("install.sh"),
        "#!/bin/sh\necho hi\n",
        ago(70 * d),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            root.join("install.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        std::os::unix::fs::symlink("src", root.join("latest")).unwrap();
    }
    Fixture { _tmp: tmp, root }
}

fn favorites(root: &Path) -> Favorites {
    let mut cfg = manta_core::ConnectionConfig {
        name: "Production web".into(),
        protocol: manta_core::Protocol::Sftp,
        host: "web-01.example.com".into(),
        port: 22,
        username: "deploy".into(),
        auth: manta_core::Auth::Agent,
        initial_path: Some("/var/www".into()),
        s3: None,
        container: None,
        ftp: None,
        webdav: None,
        smb: None,
    };
    let mut items = vec![
        Favorite::Local {
            name: "Projects".into(),
            path: root.display().to_string(),
        },
        Favorite::Connection {
            config: cfg.clone(),
            keychain: false,
        },
    ];
    cfg.name = String::new();
    cfg.protocol = manta_core::Protocol::Ftp;
    cfg.host = "files.example.org".into();
    cfg.username = "anna".into();
    items.push(Favorite::Connection {
        config: cfg,
        keychain: false,
    });
    Favorites { items }
}

fn places(root: &Path) -> (Vec<Place>, Vec<Place>) {
    let p = |icon: &'static str, name: &str, path: PathBuf| Place {
        icon,
        name: name.into(),
        path,
        network: false,
        ejectable: false,
        image: None,
    };
    (
        vec![
            p(icons::HOUSE, "Home", root.to_path_buf()),
            p(icons::DESKTOP, "Desktop", root.join("Design")),
            p(icons::FILE_TEXT, "Documents", root.join("src")),
            p(icons::DOWNLOAD_SIMPLE, "Downloads", root.join("photos")),
        ],
        vec![
            p(icons::HARD_DRIVES, "Macintosh HD", PathBuf::from("/")),
            p(icons::HARD_DRIVE, "Backup Drive", root.join("Archive 2025")),
        ],
    )
}

// ------------------------------------------------------------------ harness

type H = Harness<'static, MantaApp>;

fn harness(fx: &Fixture, size: [f32; 2], theme: Theme) -> H {
    harness_with(fx, size, theme, true)
}

/// `gpu: false` builds a harness that can step but not render (for
/// behaviour tests that must run everywhere).
fn harness_with(fx: &Fixture, size: [f32; 2], theme: Theme, gpu: bool) -> H {
    harness_full(
        fx,
        size,
        theme,
        gpu,
        Box::new(|ctx: &egui::Context| PluginHost::new(ctx.clone(), Vec::new(), None)),
    )
}

#[allow(clippy::type_complexity)]
fn harness_full(
    fx: &Fixture,
    size: [f32; 2],
    theme: Theme,
    gpu: bool,
    plugins: Box<dyn FnOnce(&egui::Context) -> PluginHost>,
) -> H {
    harness_init(fx, size, theme, gpu, plugins, |_| {})
}

/// [`harness_full`] with a last change to the [`AppInit`].
#[allow(clippy::type_complexity)]
fn harness_init(
    fx: &Fixture,
    size: [f32; 2],
    theme: Theme,
    gpu: bool,
    plugins: Box<dyn FnOnce(&egui::Context) -> PluginHost>,
    tweak: impl FnOnce(&mut AppInit),
) -> H {
    harness_init_ppp(fx, size, 2.0, theme, gpu, plugins, tweak)
}

/// [`harness_init`] at `ppp` physical pixels per point.
#[allow(clippy::type_complexity)]
fn harness_init_ppp(
    fx: &Fixture,
    size: [f32; 2],
    ppp: f32,
    theme: Theme,
    gpu: bool,
    plugins: Box<dyn FnOnce(&egui::Context) -> PluginHost>,
    tweak: impl FnOnce(&mut AppInit),
) -> H {
    let mut init = AppInit {
        start_dir: Some(fx.root.clone()),
        favorites: Some(favorites(&fx.root)),
        places: Some(places(&fx.root)),
        plugins: Some(plugins),
        prefs: Some(crate::prefs::Prefs::default()),
        quicklook: None,
        containers: None,
        updates: None,
        session_path: None,
        // Never the user's real Trash: a folder next to the fixture.
        trasher: Some(crate::trash::FakeTrash::new(
            &fx.root.parent().unwrap().join("Trash"),
        )),
        existing_install: None,
        crash_dir: None,
        secrets: None,
        live_refresh: None,
        cloud: None,
        gdrive_client_file: None,
        ..Default::default()
    };
    tweak(&mut init);
    let builder = Harness::builder()
        .with_size(egui::vec2(size[0], size[1]))
        .with_pixels_per_point(ppp)
        .with_theme(theme)
        .with_max_steps(10_000)
        .allow_missing_glyphs()
        .with_context_setup(|ctx| {
            // Same as eframe's default `system_font_fallback`.
            ctx.add_font_provider(std::sync::Arc::new(
                egui_system_font_provider::SystemFontProvider::new(),
            ));
            crate::theme::install(ctx);
        });
    let builder = if gpu { builder.wgpu() } else { builder };
    let mut h = builder.build_eframe(move |cc| MantaApp::with_init(&cc.egui_ctx, init));
    settle(&mut h);
    // Right pane shows a sub folder so the two panes differ.
    let design = VfsPath::new(&fx.root.join("Design").to_string_lossy());
    app(&mut h).navigate_path(Side::Right, design);
    settle(&mut h);
    h
}

fn app(h: &mut H) -> &mut MantaApp {
    h.state_mut()
}

/// Step until all listings have arrived (worker threads), then a few more frames.
fn settle(h: &mut H) {
    let start = Instant::now();
    loop {
        h.step();
        let busy = h
            .state()
            .panes
            .iter()
            .any(|p| p.tabs.iter().any(|t| t.loading));
        if !busy || start.elapsed() > crate::test_support::wait() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    h.run_steps(4);
}

fn save(h: &mut H, name: &str) {
    h.run_steps(3);
    let img = h.render().expect("render");
    let path = out_dir().join(format!("{name}.png"));
    img.save(&path).expect("save png");
    eprintln!("ux_tests: wrote {}", path.display());
}

/// A harness in German (this thread only; see `i18n::set_locale`).
fn harness_de(fx: &Fixture, size: [f32; 2], theme: Theme) -> H {
    let mut h = harness(fx, size, theme);
    crate::i18n::set_locale(crate::i18n::Locale::De);
    h.run_steps(3);
    h
}

/// A sub folder of the render directory (one per feature).
fn render_dir(name: &str) -> PathBuf {
    let dir = out_dir().join(name);
    std::fs::create_dir_all(&dir).expect("create render dir");
    dir
}

fn save_to(h: &mut H, dir: &Path, name: &str) {
    h.run_steps(3);
    let img = h.render().expect("render");
    let path = dir.join(format!("{name}.png"));
    img.save(&path).expect("save png");
    eprintln!("ux_tests: wrote {}", path.display());
}

fn select(h: &mut H, side: Side, names: &[&str]) {
    let tab = app(h).pane_mut(side).tab_mut();
    tab.selected = names.iter().map(|s| s.to_string()).collect();
    tab.cursor = names.last().and_then(|n| tab.view_index_of(n));
    h.run_steps(2);
}

macro_rules! require_gpu {
    () => {
        // Screenshots are opt-in: they need a GPU and are slow, so a plain
        // `cargo test` skips them. Set MANTA_RENDER_DIR to produce them.
        if std::env::var_os("MANTA_RENDER_DIR").is_none() {
            return;
        }
        if !wgpu_available() {
            return;
        }
    };
}

// ------------------------------------------------------------------ renders

#[test]
fn render_main_window() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        for (size, sname) in [(WIDE, "1280"), (NARROW, "900")] {
            let mut h = harness(&fx, size, theme);
            select(&mut h, Side::Left, &["README.md", "report-q3.pdf"]);
            save(&mut h, &format!("main_{tname}_{sname}"));
        }
    }
}

#[test]
fn render_dialogs() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        app(&mut h).dialog = Some(Dialog::Connect(ConnectForm::new(Side::Left)));
        save(&mut h, &format!("connect_{tname}"));

        let mut form = ConnectForm::new(Side::Left);
        form.host = "web-01.example.com".into();
        form.user = "deploy".into();
        form.password = "hunter2".into();
        form.error = Some("authentication failed: Username/PublicKey combination invalid".into());
        app(&mut h).dialog = Some(Dialog::Connect(form));
        save(&mut h, &format!("connect_error_{tname}"));
    }

    let mut h = harness(&fx, WIDE, Theme::Light);
    select(&mut h, Side::Left, &["README.md", "report-q3.pdf", "src"]);
    let a = app(&mut h);
    let fs = a.local.clone();
    let paths: Vec<VfsPath> = a
        .pane(Side::Left)
        .tab()
        .selected_entries()
        .into_iter()
        .map(|e| e.path)
        .collect();
    let folders = vec![VfsPath::new(&fx.root.join("src").to_string_lossy())];
    a.dialog = Some(Dialog::Delete(DeleteForm {
        side: Side::Left,
        fs: fs.clone(),
        paths: paths.clone(),
        folders: folders.clone(),
        no_trash: false,
        in_trash: false,
    }));
    save(&mut h, "delete_confirm");

    let dst = app(&mut h).pane(Side::Right).tab().loc.path.clone();
    app(&mut h).dialog = Some(Dialog::Transfer(TransferForm {
        extract: false,
        archive: None,
        src_side: Side::Left,
        dst_side: Side::Right,
        targets: Vec::new(),
        mv: false,
        src_fs: fs.clone(),
        sources: paths,
        dst_fs: fs,
        dst_dir: dst,
        dst_label: "Local".into(),
        conflict: manta_ops::ConflictPolicy::Skip,
        folders,
        choose: false,
    }));
    save(&mut h, "copy_confirm");
}

fn job(
    id: u64,
    kind: JobKind,
    title: &str,
    state: JobState,
    done: u64,
    total: u64,
    files: (u64, u64),
) -> JobStatus {
    JobStatus {
        id,
        kind,
        title: title.into(),
        state,
        current_file: Some("Design/hero-banner.png".into()),
        bytes_done: done,
        bytes_total: total,
        files_done: files.0,
        files_total: files.1,
        speed: if done > 0 { 11_400_000.0 } else { 0.0 },
        errors: Vec::new(),
        resume: Default::default(),
    }
}

#[test]
fn render_transfers() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        let mut failed = job(
            4,
            JobKind::Copy,
            "Copy 2 items to /var/www",
            JobState::Failed("permission denied: /var/www".into()),
            0,
            0,
            (0, 2),
        );
        failed.errors = vec!["permission denied: /var/www/index.html".into()];
        let mut done_with_errors = job(
            5,
            JobKind::Move,
            "Move 12 items to /Users/anna/photos",
            JobState::Done,
            8_000_000,
            8_000_000,
            (12, 12),
        );
        done_with_errors.errors = vec![
            "skipped IMG_2041.jpg (exists)".into(),
            "skipped IMG_2042.jpg (exists)".into(),
        ];
        let a = app(&mut h);
        a.fake_jobs = Some(vec![
            job(
                1,
                JobKind::Copy,
                "Copy 3 items to deploy@web-01.example.com:/var/www",
                JobState::Running,
                31_200_000,
                52_000_000,
                (14, 22),
            ),
            job(
                2,
                JobKind::Copy,
                "Copy backup-2026-10-01.zip to /Volumes/Backup",
                JobState::Queued,
                0,
                48_300_000,
                (0, 1),
            ),
            job(
                3,
                JobKind::Delete,
                "Delete 4 items",
                JobState::Done,
                0,
                0,
                (4, 4),
            ),
            failed,
            done_with_errors,
            job(
                6,
                JobKind::Copy,
                "Copy notes.txt to /tmp",
                JobState::Cancelled,
                0,
                9,
                (0, 1),
            ),
        ]);
        a.show_transfers = true;
        save(&mut h, &format!("transfers_{tname}"));
    }
}

fn item(
    rel: &str,
    is_dir: bool,
    status: CompareStatus,
    l: Option<u64>,
    r: Option<u64>,
) -> CompareItem {
    CompareItem {
        rel_path: rel.into(),
        is_dir,
        status,
        left_size: l,
        right_size: r,
    }
}

#[test]
fn render_sync() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        let a = app(&mut h);
        a.sync.open = true;
        a.sync.left_label = format!("Local {}", fx.root.display());
        a.sync.right_label = "deploy@web-01.example.com /var/www".into();
        a.sync.items = Some(vec![
            item("src", true, CompareStatus::Identical, Some(0), Some(0)),
            item(
                "src/main.rs",
                false,
                CompareStatus::LeftNewer,
                Some(1_204),
                Some(1_180),
            ),
            item(
                "src/lib.rs",
                false,
                CompareStatus::Identical,
                Some(13),
                Some(13),
            ),
            item("Design", true, CompareStatus::OnlyLeft, Some(0), None),
            item(
                "Design/hero-banner.png",
                false,
                CompareStatus::OnlyLeft,
                Some(2_480_000),
                None,
            ),
            item(
                "README.md",
                false,
                CompareStatus::RightNewer,
                Some(220),
                Some(310),
            ),
            item(
                "report-q3.pdf",
                false,
                CompareStatus::Different,
                Some(1_840_000),
                Some(1_790_000),
            ),
            item("cache", true, CompareStatus::OnlyRight, None, Some(0)),
            item(
                "cache/sessions.db",
                false,
                CompareStatus::OnlyRight,
                None,
                Some(4_096_000),
            ),
            item(
                "notes.txt",
                false,
                CompareStatus::Identical,
                Some(9),
                Some(9),
            ),
        ]);
        save(&mut h, &format!("sync_{tname}"));
    }
}

#[test]
fn render_preview() {
    require_gpu!();
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    let path = VfsPath::new(&fx.root.join("README.md").to_string_lossy());
    let entry = Entry {
        name: "README.md".into(),
        path,
        kind: EntryKind::File,
        size: README.len() as u64,
        modified: None,
        permissions: Some(0o644),
        link_target: None,
        hidden: false,
    };
    let mut p = Preview::new(999, entry);
    p.loading = false;
    p.content = Some(PreviewContent::Text {
        text: README.into(),
        truncated: false,
    });
    app(&mut h).preview = Some(p);
    save(&mut h, "preview");
}

#[test]
fn render_pane_states() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        // Left: empty folder. Right: error.
        let empty = VfsPath::new(&fx.root.join("Archive 2025").to_string_lossy());
        app(&mut h).navigate_path(Side::Left, empty);
        settle(&mut h);
        let bad = VfsPath::new(&fx.root.join("does-not-exist").to_string_lossy());
        app(&mut h).navigate_path(Side::Right, bad);
        settle(&mut h);
        save(&mut h, &format!("empty_and_error_{tname}"));

        // Loading: a listing that never arrives.
        let tab = app(&mut h).pane_mut(Side::Right).tab_mut();
        tab.error = None;
        tab.entries.clear();
        tab.view.clear();
        tab.loading = true;
        tab.gen += 1; // ignore anything still in flight
        save(&mut h, &format!("loading_{tname}"));
    }
}

#[test]
fn render_palette_and_toasts() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        select(&mut h, Side::Left, &["README.md"]);
        let a = app(&mut h);
        a.set_success("Copied 3 items to “Design”");
        a.set_error_detail(
            "Couldn't rename “notes.txt”",
            "permission denied: /Users/anna/Projects/notes.txt",
        );
        save(&mut h, &format!("toasts_{tname}"));

        app(&mut h).palette.show();
        h.run_steps(2);
        h.event(egui::Event::Text("co".into()));
        save(&mut h, &format!("palette_{tname}"));
    }
}

#[test]
fn render_tabs_and_first_launch() {
    require_gpu!();
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    let ctx = h.ctx.clone();
    let a = app(&mut h);
    a.favorites = Favorites::default();
    a.handle_action(&ctx, super::Action::NewTab(Side::Left));
    settle(&mut h);
    let a = app(&mut h);
    let src = VfsPath::new(&fx.root.join("src").to_string_lossy());
    a.navigate_path(Side::Left, src);
    a.pane_mut(Side::Right).tab_mut().show_hidden = true;
    a.pane_mut(Side::Right).tab_mut().rebuild_view();
    a.active = Side::Right;
    settle(&mut h);
    let root = VfsPath::new(&fx.root.to_string_lossy());
    app(&mut h).navigate_path(Side::Right, root);
    settle(&mut h);
    app(&mut h).pane_mut(Side::Right).tab_mut().show_hidden = true;
    app(&mut h).pane_mut(Side::Right).tab_mut().rebuild_view();
    select(&mut h, Side::Right, &[".env", "data.json", "install.sh"]);
    save(&mut h, "tabs_hidden_inactive_selection");
}

/// Close-up of the left pane's path bar (2x pixels) to check that the
/// breadcrumb text, the filter placeholder / text and the path editor share
/// one vertical centre and the editor's text starts where the crumbs do.
#[test]
fn render_path_bar_alignment() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        // Use a short path so no segment is collapsed.
        app(&mut h).navigate_path(Side::Left, VfsPath::new("/usr/local"));
        settle(&mut h);
        let crop = |h: &mut H, name: &str| {
            h.run_steps(3);
            let img = h.render().expect("render");
            // Left pane path bar: y 40..100 pt, x 200..760 pt (2 px per pt).
            let sub = image::imageops::crop_imm(&img, 400, 80, 1120, 120).to_image();
            let path = out_dir().join(format!("{name}.png"));
            sub.save(&path).expect("save");
        };
        crop(&mut h, &format!("pathbar_empty_{tname}"));
        app(&mut h).pane_mut(Side::Left).tab_mut().filter = "rep".into();
        crop(&mut h, &format!("pathbar_filter_{tname}"));
        app(&mut h).pane_mut(Side::Left).tab_mut().filter.clear();
        let tab = app(&mut h).pane_mut(Side::Left).tab_mut();
        tab.path_edit = Some(tab.loc.path.to_string());
        crop(&mut h, &format!("pathbar_edit_{tname}"));
        // Loading: spinner between the breadcrumb and the filter.
        let tab = app(&mut h).pane_mut(Side::Left).tab_mut();
        tab.path_edit = None;
        tab.begin_loading();
        tab.gen += 1; // ignore anything still in flight
        crop(&mut h, &format!("pathbar_loading_{tname}"));
    }
}

// ------------------------------------------------------------------ go to path

/// Step until the "Go to Path" check (worker thread) has finished.
fn settle_goto(h: &mut H) {
    let start = Instant::now();
    loop {
        h.step();
        let busy = h
            .state()
            .goto
            .as_ref()
            .is_some_and(|g| g.checking.is_some());
        if !busy || start.elapsed() > crate::test_support::wait() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    settle(h);
}

fn type_text(h: &mut H, s: &str) {
    h.event(egui::Event::Text(s.into()));
    h.run_steps(2);
}

/// Enter opens a folder in a new tab of the active pane; a file opens its
/// folder with the file selected; ⌘Enter reuses the current tab; a missing
/// folder keeps the popup open with an inline error.
#[test]
fn goto_path_behaviour() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let ctx = h.ctx.clone();

    // New tab, relative path, trailing slash.
    app(&mut h).handle_action(&ctx, super::Action::GoToPath(Side::Left));
    h.run_steps(2);
    type_text(&mut h, "Design/");
    h.key_press(egui::Key::Enter);
    settle_goto(&mut h);
    assert!(app(&mut h).goto.is_none(), "popup closes on success");
    assert_eq!(
        app(&mut h).pane(Side::Left).tabs.len(),
        2,
        "opened in a new tab"
    );
    let design = VfsPath::new(&fx.root.join("Design").to_string_lossy());
    assert_eq!(app(&mut h).pane(Side::Left).tab().loc.path, design);

    // A file: its folder in the current tab (⌘Enter), file selected.
    app(&mut h).handle_action(&ctx, super::Action::GoToPath(Side::Left));
    h.run_steps(2);
    let readme = fx.root.join("README.md");
    type_text(&mut h, &format!("  '{}'  ", readme.display()));
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Enter);
    settle_goto(&mut h);
    assert_eq!(app(&mut h).pane(Side::Left).tabs.len(), 2, "no new tab");
    let tab = app(&mut h).pane(Side::Left).tab();
    assert_eq!(tab.loc.path, VfsPath::new(&fx.root.to_string_lossy()));
    assert!(tab.selected.contains("README.md"), "file selected");

    // Missing folder: inline error, popup stays open.
    app(&mut h).handle_action(&ctx, super::Action::GoToPath(Side::Left));
    h.run_steps(2);
    type_text(&mut h, "does-not-exist");
    h.key_press(egui::Key::Enter);
    settle_goto(&mut h);
    let g = app(&mut h).goto.as_ref().expect("popup stays open");
    assert_eq!(g.error.as_deref(), Some("Folder not found"));

    // Esc closes.
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(app(&mut h).goto.is_none());
}

/// Tab completes a unique folder name from a background listing.
#[test]
fn goto_path_tab_completion() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, super::Action::GoToPath(Side::Left));
    h.run_steps(2);
    type_text(&mut h, "pho");
    app(&mut h).goto.as_mut().unwrap().tab_pressed = true;
    let start = Instant::now();
    while app(&mut h).goto.as_ref().unwrap().input != "photos/"
        && start.elapsed() < crate::test_support::wait()
    {
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(app(&mut h).goto.as_ref().unwrap().input, "photos/");
}

#[test]
fn render_goto_path() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness(&fx, WIDE, theme);
        let ctx = h.ctx.clone();
        let a = app(&mut h);
        a.goto_recent = vec![
            (None, VfsPath::new(&fx.root.join("src").to_string_lossy())),
            (None, VfsPath::new("/usr/local/bin")),
            (
                None,
                VfsPath::new(&fx.root.join("photos").to_string_lossy()),
            ),
        ];
        a.handle_action(&ctx, super::Action::GoToPath(Side::Left));
        h.run_steps(2);
        save(&mut h, &format!("goto_empty_{tname}"));

        type_text(&mut h, &format!("{}/", fx.root.display()));
        app(&mut h).goto.as_mut().unwrap().tab_pressed = true;
        let start = Instant::now();
        while app(&mut h).goto.as_ref().unwrap().rows_len() < 2
            && start.elapsed() < crate::test_support::wait()
        {
            h.step();
            std::thread::sleep(Duration::from_millis(5));
        }
        h.key_press(egui::Key::ArrowDown);
        h.key_press(egui::Key::ArrowDown);
        save(&mut h, &format!("goto_filled_{tname}"));

        let g = app(&mut h).goto.as_mut().unwrap();
        g.input = "~/Projects/Desgin".into();
        g.clear_list();
        h.key_press(egui::Key::Enter);
        settle_goto(&mut h);
        save(&mut h, &format!("goto_error_{tname}"));
    }
}

// ------------------------------------------------------------------ plugins

/// (file name, badge text, badge tone, name tint, dimmed, tooltip)
type FakeDeco = (
    &'static str,
    &'static str,
    Tone,
    Option<Tone>,
    bool,
    &'static str,
);

/// A built-in style fake that decorates fixture files and adds a chip.
struct FakeDecorator {
    id: &'static str,
    name: &'static str,
    /// (file name, badge text, badge tone, name tint, dimmed, tooltip)
    decorations: Vec<FakeDeco>,
    chip: Option<Chip>,
    handle: Option<PluginHandle>,
}

impl Plugin for FakeDecorator {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: self.id.into(),
            name: self.name.into(),
            version: "1.0".into(),
            description: "Test plugin".into(),
            author: "Tests".into(),
            builtin: true,
        }
    }
    fn enable(&mut self, handle: PluginHandle) {
        self.handle = Some(handle);
    }
    fn disable(&mut self) {
        self.handle = None;
    }
    fn on_event(&mut self, ev: &PluginEvent) {
        let (PluginEvent::DirectoryChanged(p) | PluginEvent::ListingRefreshed(p)) = ev else {
            return;
        };
        let Some(h) = &self.handle else { return };
        let entries = self
            .decorations
            .iter()
            .map(|(name, badge, tone, tint, dim, tip)| {
                (
                    name.to_string(),
                    Decoration {
                        badge: (!badge.is_empty()).then(|| Badge {
                            text: badge.to_string(),
                            tone: *tone,
                        }),
                        name_tone: *tint,
                        dimmed: *dim,
                        tooltip: Some(tip.to_string()),
                    },
                )
            })
            .collect();
        h.send(HostRequest::SetDecorations {
            pane: p.pane,
            dir: p.path.clone(),
            entries,
        });
        if let Some(c) = &self.chip {
            h.send(HostRequest::SetChips {
                pane: p.pane,
                dir: p.path.clone(),
                chips: vec![c.clone()],
            });
        }
    }
    fn commands(&self) -> Vec<CommandSpec> {
        vec![CommandSpec {
            id: "status".into(),
            title: format!("{} Status", self.name),
            shortcut: None,
            context: CommandContext::FileContextMenu,
        }]
    }
}

fn decorators() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(FakeDecorator {
            id: "test.vcs",
            name: "VCS",
            decorations: vec![
                (
                    "README.md",
                    "M",
                    Tone::Warning,
                    Some(Tone::Warning),
                    false,
                    "Modified",
                ),
                (
                    "report-q3.pdf",
                    "A",
                    Tone::Success,
                    Some(Tone::Success),
                    false,
                    "Added",
                ),
                ("notes.txt", "?", Tone::Info, None, false, "Untracked"),
                (
                    "install.sh",
                    "R",
                    Tone::Accent,
                    Some(Tone::Accent),
                    false,
                    "Renamed",
                ),
                (
                    "data.json",
                    "",
                    Tone::Neutral,
                    None,
                    true,
                    "Ignored by .gitignore",
                ),
                (
                    "photos",
                    "",
                    Tone::Neutral,
                    None,
                    true,
                    "Ignored by .gitignore",
                ),
                ("src", "•", Tone::Warning, None, false, "Contains changes"),
                (
                    "screen-recording.mov",
                    "D",
                    Tone::Danger,
                    Some(Tone::Danger),
                    false,
                    "Deleted",
                ),
            ],
            chip: Some(Chip {
                text: "main".into(),
                tone: Tone::Accent,
                icon: Some("branch".into()),
                tooltip: Some("Branch main, 2 ahead".into()),
                command: Some("status".into()),
            }),
            handle: None,
        }),
        Box::new(FakeDecorator {
            id: "test.size",
            name: "Sizes",
            decorations: vec![
                (
                    "screen-recording.mov",
                    "L",
                    Tone::Warning,
                    None,
                    false,
                    "Large file: 312 MB",
                ),
                (
                    "backup-2026-10-01.zip",
                    "L",
                    Tone::Warning,
                    None,
                    false,
                    "Large file: 48.3 MB",
                ),
                ("notes.txt", "1", Tone::Neutral, None, false, "One line"),
            ],
            chip: Some(Chip {
                text: "14 items".into(),
                tone: Tone::Neutral,
                icon: Some("info".into()),
                tooltip: None,
                command: None,
            }),
            handle: None,
        }),
    ]
}

fn fake_external(toml: &str) -> ExternalPlugin {
    let m = Manifest::parse(toml, Path::new("plugin.toml")).expect("manifest");
    let dir = PathBuf::from("/Users/anna/Library/Application Support/Manta/plugins").join(&m.id);
    ExternalPlugin::new(m, dir)
}

const HELLO: &str = r#"
id = "hello-python"
name = "Hello Python"
version = "0.1.0"
description = "Flags large files, counts items and shows file info."
author = "Anna Berg"
api_version = 1
command = ["python3", "main.py"]

[[panels]]
id = "file-info"
title = "File Info"
placement = "right_sidebar"

[[panels]]
id = "outline"
title = "Outline"
placement = "pane_bottom"
"#;

const BROKEN: &str = r#"
id = "cloud-sync"
name = "Cloud Sync"
version = "2.3.1"
description = "Shows the sync state of files in your cloud folders."
author = "Example Corp"
api_version = 1
command = ["node", "index.js"]
"#;

fn file_info_blocks() -> Vec<Block> {
    vec![
        Block::Heading {
            text: "report-q3.pdf".into(),
        },
        Block::KeyValue {
            rows: vec![
                KeyValueRow {
                    key: "Kind".into(),
                    value: "PDF document".into(),
                    tone: Tone::Neutral,
                },
                KeyValueRow {
                    key: "Size".into(),
                    value: "1.8 MB".into(),
                    tone: Tone::Neutral,
                },
                KeyValueRow {
                    key: "Modified".into(),
                    value: "2026-10-17 14:02".into(),
                    tone: Tone::Neutral,
                },
                KeyValueRow {
                    key: "Permissions".into(),
                    value: "-rw-r--r--".into(),
                    tone: Tone::Neutral,
                },
                KeyValueRow {
                    key: "Path".into(),
                    value: "/Users/anna/Projects/report-q3.pdf".into(),
                    tone: Tone::Neutral,
                },
                KeyValueRow {
                    key: "Note".into(),
                    value: "Not backed up yet".into(),
                    tone: Tone::Warning,
                },
            ],
        },
        Block::Text {
            text: "Shared with 3 people.".into(),
            tone: Tone::Muted,
        },
        Block::Buttons {
            buttons: vec![
                ButtonSpec {
                    label: "Refresh".into(),
                    action: "refresh".into(),
                    primary: true,
                },
                ButtonSpec {
                    label: "Close".into(),
                    action: "close".into(),
                    primary: false,
                },
            ],
        },
    ]
}

fn outline_blocks() -> Vec<Block> {
    let row = |icon: &str, text: &str, detail: &str, tone: Tone| ListRow {
        icon: Some(icon.into()),
        text: text.into(),
        detail: Some(detail.into()),
        tone,
        action: Some(format!("open:{text}")),
        tooltip: None,
    };
    vec![
        Block::List {
            rows: vec![
                row("file", "README.md", "3 headings", Tone::Neutral),
                row("code", "src/main.rs", "fn main", Tone::Neutral),
                row(
                    "warning",
                    "install.sh",
                    "not executable on remote",
                    Tone::Warning,
                ),
                row("check", "report-q3.pdf", "signed", Tone::Success),
            ],
        },
        Block::Code {
            text: "$ make outline\nscanned 14 files in 0.03 s".into(),
        },
    ]
}

/// Host with the fake decorators and two external plugins: one running
/// with panels, one that crashed.
fn plugin_host(ctx: &egui::Context, builtins: Vec<Box<dyn Plugin>>) -> PluginHost {
    let mut host = PluginHost::new(ctx.clone(), builtins, None);
    let hello = host.add_external_for_test(fake_external(HELLO));
    hello.inject_state(
        PluginStatus::Running,
        &["hello-python started, data dir …/plugin-data/hello-python"],
        vec![
            (("file-info".into(), None), file_info_blocks()),
            (("outline".into(), None), outline_blocks()),
        ],
    );
    host.trust("hello-python");
    host.set_running_for_test("hello-python", true);
    let broken = host.add_external_for_test(fake_external(BROKEN));
    broken.inject_state(
        PluginStatus::Error(
            "exited with status 1; not restarted after 3 restarts within a minute".into(),
        ),
        &[
            "[manta] started (pid 48113)",
            "Error: Cannot find module 'chokidar'",
            "    at Module._resolveFilename (node:internal/modules/cjs/loader:1145:15)",
            "[manta] exited with status 1; restarting…",
            "[manta] started (pid 48120)",
            "Error: Cannot find module 'chokidar'",
            "[manta] exited with status 1; not restarted after 3 restarts within a minute",
        ],
        Vec::new(),
    );
    host
}

fn plugin_harness(fx: &Fixture, theme: Theme, real_builtins: bool) -> H {
    let mut h = harness_full(
        fx,
        WIDE,
        theme,
        true,
        Box::new(move |ctx: &egui::Context| {
            let builtins = if real_builtins {
                super::builtin_plugins()
            } else {
                decorators()
            };
            plugin_host(ctx, builtins)
        }),
    );
    // Built-in panels (e.g. the terminal) stay closed in these renders.
    let a = app(&mut h);
    for placement in [
        manta_plugin::PanelPlacement::PaneBottom,
        manta_plugin::PanelPlacement::RightSidebar,
    ] {
        for p in a.plugins.panels(placement) {
            if p.plugin_id != "hello-python" {
                a.plugins
                    .set_panel_open(&p.plugin_id, &p.spec.id, None, false);
            }
        }
    }
    settle(&mut h);
    h
}

#[test]
fn render_plugin_settings() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = plugin_harness(&fx, theme, true);
        let a = app(&mut h);
        a.settings.show(Some(crate::settings::SettingsTab::Plugins));
        a.settings.expand_log("cloud-sync");
        h.run_steps(4);
        save(&mut h, &format!("plugins_settings_{tname}"));
        if theme == Theme::Light {
            let a = app(&mut h);
            a.settings.tab = crate::settings::SettingsTab::General;
            save(&mut h, "plugins_settings_general_light");
            let a = app(&mut h);
            a.settings.tab = crate::settings::SettingsTab::Plugins;
            a.settings.confirm = Some(crate::settings::Confirm::Enable {
                id: "cloud-sync".into(),
                review: Box::new(sample_review(
                    "cloud-sync",
                    "Cloud Sync",
                    "node index.js",
                    false,
                )),
            });
            save(&mut h, "plugins_confirm_enable_light");
        }
    }
}

/// A plugin as the install / trust sheets show it (renders and manual).
pub(crate) fn sample_review(
    id: &str,
    name: &str,
    command: &str,
    archive: bool,
) -> manta_plugin::install::PluginReview {
    let folder = std::path::PathBuf::from(format!(
        "/Users/anna/Library/Application Support/Manta/plugins/{id}"
    ));
    manta_plugin::install::PluginReview {
        id: id.into(),
        name: name.into(),
        version: "1.2.0".into(),
        description: if crate::i18n::locale() == crate::i18n::Locale::De {
            "Zeigt den Sync-Status deiner Cloud-Ordner als Badges und ergänzt „Jetzt synchronisieren“ im Kontextmenü.".into()
        } else {
            "Shows the sync state of your cloud folders as badges and adds “Sync Now” to the context menu.".into()
        },
        author: "Anna Berger".into(),
        license: Some("MIT".into()),
        homepage: None,
        repository: Some(format!("https://github.com/anna-berger/manta-{id}")),
        permissions: vec!["read_files".into(), "network".into(), "run_programs".into()],
        command: command.into(),
        source: if archive {
            format!("/Users/anna/Downloads/{id}-1.2.0.mantaplugin").into()
        } else {
            folder.clone()
        },
        root: folder.clone(),
        staging: None,
        installed_version: None,
        quarantined: true,
        files: 14,
        bytes: 182_400,
        folder: (!archive).then_some(folder),
        verified_download: None,
    }
}

/// The install sheet (a downloaded .mantaplugin) and the trust sheet,
/// German and English, light: `MANTA_RENDER_DIR=… cargo test -p manta
/// render_plugin_install_sheets`.
#[test]
fn render_plugin_install_sheets() {
    require_gpu!();
    let fx = fixture();
    for (locale, lang) in [
        (crate::i18n::Locale::De, "de"),
        (crate::i18n::Locale::En, "en"),
    ] {
        crate::i18n::set_locale(locale);
        let mut h = plugin_harness(&fx, Theme::Light, true);
        crate::i18n::set_locale(locale);
        let a = app(&mut h);
        a.settings.show(Some(crate::settings::SettingsTab::Plugins));
        a.settings.confirm = Some(crate::settings::Confirm::Install(Box::new(sample_review(
            "cloud-sync",
            "Cloud Sync",
            "node index.js",
            true,
        ))));
        h.run_steps(4);
        save(&mut h, &format!("plugins_install_sheet_{lang}_light"));
        let a = app(&mut h);
        a.settings.confirm = Some(crate::settings::Confirm::Enable {
            id: "cloud-sync".into(),
            review: Box::new(sample_review(
                "cloud-sync",
                "Cloud Sync",
                "node index.js",
                false,
            )),
        });
        h.run_steps(4);
        save(&mut h, &format!("plugins_trust_sheet_{lang}_light"));
        // An update of an installed plugin, without quarantine.
        let mut r = sample_review("cloud-sync", "Cloud Sync", "node index.js", true);
        r.installed_version = Some("1.1.0".into());
        r.quarantined = false;
        app(&mut h).settings.confirm = Some(crate::settings::Confirm::Install(Box::new(r)));
        h.run_steps(4);
        save(&mut h, &format!("plugins_update_sheet_{lang}_light"));
    }
    crate::i18n::set_locale(crate::i18n::Locale::En);
}

#[test]
fn render_plugin_decorations_and_panels() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = plugin_harness(&fx, theme, false);
        // Show the project root in both panes (hidden files on the right).
        let root = VfsPath::new(&fx.root.to_string_lossy());
        app(&mut h).navigate_path(Side::Right, root);
        settle(&mut h);
        select(&mut h, Side::Left, &["report-q3.pdf"]);
        h.run_steps(4);
        save(&mut h, &format!("plugins_decorations_{tname}"));
        let crop = |h: &mut H, name: &str| {
            h.run_steps(3);
            let img = h.render().expect("render");
            // Left pane: path bar and the first rows (2 px per point).
            let sub = image::imageops::crop_imm(&img, 400, 80, 1120, 560).to_image();
            sub.save(out_dir().join(format!("{name}.png")))
                .expect("save");
        };
        crop(&mut h, &format!("plugins_decorations_crop_{tname}"));

        let a = app(&mut h);
        a.plugins
            .set_panel_open("hello-python", "file-info", None, true);
        a.plugins
            .set_panel_open("hello-python", "outline", Some(PaneId::Left), true);
        settle(&mut h);
        save(&mut h, &format!("plugins_panels_{tname}"));
    }
}

/// Regression: a resized right sidebar must keep its width while the mouse
/// moves over the app (it used to shrink by its margins on every repaint).
#[test]
fn sidebar_keeps_width_after_resize() {
    let fx = fixture();
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(|ctx: &egui::Context| plugin_host(ctx, decorators())),
    );
    app(&mut h)
        .plugins
        .set_panel_open("hello-python", "file-info", None, true);
    settle(&mut h);
    let before = app(&mut h).layout.sidebar_width;

    // Drag the sidebar's left edge 150 pt to the left.
    let y = WIDE[1] / 2.0;
    let edge = egui::pos2(WIDE[0] - before, y);
    let target = egui::pos2(edge.x - 150.0, y);
    let button = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    h.event(egui::Event::PointerMoved(edge));
    h.run_steps(2);
    h.event(button(edge, true));
    h.run_steps(2);
    for i in 1..=10 {
        h.event(egui::Event::PointerMoved(
            edge + (target - edge) * (i as f32 / 10.0),
        ));
        h.step();
    }
    h.event(button(target, false));
    h.run_steps(3);
    let resized = app(&mut h).layout.sidebar_width;
    assert!(
        (resized - (before + 150.0)).abs() < 4.0,
        "drag: {before} -> {resized}"
    );

    // Wiggle the mouse across the window; the width must not change.
    for i in 0..60 {
        let x = 100.0 + (i % 12) as f32 * 90.0;
        h.event(egui::Event::PointerMoved(egui::pos2(x, y)));
        h.step();
    }
    let after = app(&mut h).layout.sidebar_width;
    assert!(
        (after - resized).abs() < 0.5,
        "shrank while hovering: {resized} -> {after}"
    );
}

/// A plugin whose pane-bottom panel holds keyboard focus with a focus
/// lock, like the terminal.
struct FocusGrabber;

impl Plugin for FocusGrabber {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "test.focus".into(),
            name: "Focus".into(),
            version: "1".into(),
            description: String::new(),
            author: String::new(),
            builtin: true,
        }
    }
    fn enable(&mut self, _handle: PluginHandle) {}
    fn disable(&mut self) {}
    fn panels(&self) -> Vec<manta_plugin::PanelSpec> {
        vec![manta_plugin::PanelSpec {
            id: "term".into(),
            title: "Term".into(),
            placement: manta_plugin::PanelPlacement::PaneBottom,
            default_open: true,
        }]
    }
    fn panel_ui(&mut self, _panel: &str, pane: &manta_plugin::PaneState, ui: &mut egui::Ui) {
        if pane.pane != PaneId::Left {
            return;
        }
        let r = ui.add(egui::Button::new("terminal"));
        if !r.has_focus() {
            r.request_focus();
        }
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                r.id,
                egui::EventFilter {
                    tab: true,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: true,
                },
            )
        });
    }
}

/// Keyboard focus rules: while a plugin panel holds focus, plain keys
/// (Tab, arrows, Enter) and ⌘A go to it; other ⌘-shortcuts still work.
#[test]
fn plugin_focus_blocks_plain_shortcuts() {
    let fx = fixture();
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(|ctx: &egui::Context| {
            PluginHost::new(ctx.clone(), vec![Box::new(FocusGrabber)], None)
        }),
    );
    h.run_steps(4);
    assert!(app(&mut h).plugin_focus, "panel widget has focus");
    let active = app(&mut h).active;
    h.key_press(egui::Key::Tab);
    h.key_press(egui::Key::ArrowDown);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(2);
    let a = app(&mut h);
    assert_eq!(a.active, active, "Tab stays in the panel");
    assert!(
        a.pane(Side::Left).tab().cursor.is_none(),
        "arrows stay in the panel"
    );
    assert!(
        a.pane(Side::Left).tab().selected.is_empty(),
        "⌘A stays in the panel"
    );
    // ⌘-shortcuts still work.
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Comma);
    h.run_steps(2);
    assert!(app(&mut h).settings.open, "⌘, opens Settings");
}

/// Records the file actions reported to plugins.
struct ActionRecorder(std::sync::Arc<std::sync::Mutex<Vec<manta_plugin::FileAction>>>);

impl Plugin for ActionRecorder {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "test.actions".into(),
            name: "Actions".into(),
            version: "1".into(),
            description: String::new(),
            author: String::new(),
            builtin: true,
        }
    }
    fn enable(&mut self, _handle: PluginHandle) {}
    fn disable(&mut self) {}
    fn on_event(&mut self, ev: &PluginEvent) {
        if let PluginEvent::FileAction(a) = ev {
            self.0.lock().unwrap().push(a.clone());
        }
    }
}

#[test]
fn file_actions_are_reported_to_plugins() {
    use manta_plugin::{FileActionKind, FileActionPhase};
    let fx = fixture();
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let rec = log.clone();
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(move |ctx: &egui::Context| {
            PluginHost::new(ctx.clone(), vec![Box::new(ActionRecorder(rec))], None)
        }),
    );
    std::fs::write(fx.root.join("copy-me.txt"), vec![b'x'; 1234]).unwrap();
    std::fs::write(fx.root.join("and-me.txt"), vec![b'y'; 66]).unwrap();
    let src = |n: &str| VfsPath::new(&fx.root.join(n).to_string_lossy());
    let dest = VfsPath::new(&fx.root.join("Design").to_string_lossy());
    let wait = |h: &mut H, n: usize| {
        let start = Instant::now();
        while log.lock().unwrap().len() < n && start.elapsed() < crate::test_support::wait() {
            h.step();
            std::thread::sleep(Duration::from_millis(5));
        }
        std::mem::take(&mut *log.lock().unwrap())
    };

    // Drag & drop from the left pane onto the right pane, then confirm.
    let ctx = h.ctx.clone();
    let fs = app(&mut h).local.clone();
    app(&mut h).handle_action(
        &ctx,
        super::Action::DropFiles {
            from: Side::Left,
            to: Side::Right,
            fs,
            paths: vec![src("copy-me.txt"), src("and-me.txt")],
            mv: false,
        },
    );
    assert!(matches!(app(&mut h).dialog, Some(Dialog::Transfer(_))));
    app(&mut h).submit_dialog();
    let events = wait(&mut h, 2);
    assert_eq!(events.len(), 2, "{events:#?}");
    let (start, end) = (&events[0], &events[1]);
    assert_eq!(start.id, end.id);
    assert_eq!(start.phase, FileActionPhase::Started);
    assert_eq!(start.kind, FileActionKind::Copy);
    assert_eq!(start.pane, PaneId::Left, "the pane dragged from");
    assert_eq!(start.sources, vec![src("copy-me.txt"), src("and-me.txt")]);
    assert_eq!(start.destination.as_ref(), Some(&dest));
    assert!(start.src_local && start.dst_local);
    let FileActionPhase::Finished {
        ok,
        files,
        bytes,
        errors,
        cancelled,
        ..
    } = &end.phase
    else {
        panic!("{end:?}")
    };
    assert!(*ok && !*cancelled && errors.is_empty());
    assert_eq!((*files, *bytes), (2, 1300));
    assert!(fx.root.join("Design/copy-me.txt").exists());

    // F8 / context menu delete: the confirmed dialog reports a delete.
    settle(&mut h);
    app(&mut h).dialog = Some(Dialog::Delete(DeleteForm {
        side: Side::Right,
        fs: app(&mut h).local.clone(),
        paths: vec![dest.join("copy-me.txt")],
        folders: vec![],
        no_trash: false,
        in_trash: false,
    }));
    app(&mut h).submit_dialog();
    let events = wait(&mut h, 2);
    assert_eq!(events[0].kind, FileActionKind::Delete);
    assert_eq!(events[0].pane, PaneId::Right);
    assert!(
        matches!(
            events[1].phase,
            FileActionPhase::Finished {
                ok: true,
                files: 1,
                ..
            }
        ),
        "{events:?}"
    );

    // A failing copy (source vanished) reports the error.
    std::fs::remove_file(fx.root.join("and-me.txt")).unwrap();
    // F6 path (confirm_transfer → dialog → submit).
    let fs = app(&mut h).local.clone();
    app(&mut h).confirm_transfer(
        Side::Left,
        fs,
        vec![src("and-me.txt")],
        Side::Right,
        true,
        false,
    );
    app(&mut h).submit_dialog();
    let events = wait(&mut h, 2);
    assert_eq!(events[0].kind, FileActionKind::Move);
    let FileActionPhase::Finished { ok, errors, .. } = &events[1].phase else {
        panic!()
    };
    assert!(!ok && !errors.is_empty(), "{events:?}");
}

// ------------------------------------------------------------------ folder sizes

/// Sends fixed folder sizes for the fixture root (one still counting).
struct FakeSizes {
    handle: Option<PluginHandle>,
}

fn fake_sizes() -> Vec<(String, manta_plugin::SizeInfo)> {
    use manta_plugin::SizeInfo;
    let s = |bytes, complete, files, on_disk| SizeInfo {
        bytes,
        complete,
        files,
        on_disk: Some(on_disk),
        unreadable: 0,
    };
    vec![
        ("Design".into(), s(8_698_400, true, 3, 8_712_192)),
        ("src".into(), s(51, true, 2, 8_192)),
        (
            "photos".into(),
            s(1_240_000_000, false, 18_204, 1_251_000_000),
        ),
        ("Archive 2025".into(), s(0, true, 0, 0)),
    ]
}

impl Plugin for FakeSizes {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "test.sizes".into(),
            name: "Sizes".into(),
            version: "1".into(),
            description: String::new(),
            author: String::new(),
            builtin: true,
        }
    }
    fn enable(&mut self, handle: PluginHandle) {
        self.handle = Some(handle);
    }
    fn disable(&mut self) {}
    fn on_event(&mut self, ev: &PluginEvent) {
        let PluginEvent::DirectoryChanged(p) = ev else {
            return;
        };
        if p.path.file_name() == "Projects" {
            if let Some(h) = &self.handle {
                h.send(HostRequest::SetSizes {
                    pane: p.pane,
                    dir: p.path.clone(),
                    sizes: fake_sizes(),
                });
            }
        }
    }
}

fn sizes_harness(fx: &Fixture, theme: Theme, gpu: bool) -> H {
    let mut h = harness_full(
        fx,
        WIDE,
        theme,
        gpu,
        Box::new(|ctx: &egui::Context| {
            PluginHost::new(
                ctx.clone(),
                vec![Box::new(FakeSizes { handle: None })],
                None,
            )
        }),
    );
    settle(&mut h);
    h
}

fn view_names(h: &mut H, side: Side) -> Vec<String> {
    let tab = app(h).pane(side).tab();
    tab.view
        .iter()
        .map(|&i| tab.entries[i].name.clone())
        .collect()
}

#[test]
fn folder_sizes_show_and_sort() {
    let fx = fixture();
    let mut h = sizes_harness(&fx, Theme::Light, false);
    let tab = app(&mut h).pane(Side::Left).tab();
    assert_eq!(tab.dir_sizes.len(), 4, "SetSizes reached the tab");
    assert_eq!(tab.dir_sizes["Design"].bytes, 8_698_400);
    assert_eq!(super::pane::size_label(&tab.dir_sizes["Design"]), "8.7 MB");
    assert_eq!(
        super::pane::size_label(&tab.dir_sizes["photos"]),
        "≥ 1.2 GB…"
    );
    assert!(super::pane::size_tooltip(&tab.dir_sizes["photos"]).contains("18,204 files"));
    // The right pane (Design) got nothing: the sizes are for the root.
    assert!(app(&mut h).pane(Side::Right).tab().dir_sizes.is_empty());

    // Sort by size, largest first: folders by their measured size.
    let tab = app(&mut h).pane_mut(Side::Left).tab_mut();
    tab.sort = crate::util::SortSpec {
        column: crate::util::SortColumn::Size,
        ascending: false,
    };
    tab.rebuild_view();
    let names = view_names(&mut h, Side::Left);
    let dirs: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|n| ["photos", "Design", "src", "Archive 2025"].contains(n))
        .collect();
    assert_eq!(dirs, ["photos", "Design", "src", "Archive 2025"]);

    // Selection size includes known folder sizes.
    select(&mut h, Side::Left, &["Design", "notes.txt"]);
    assert_eq!(
        app(&mut h).pane(Side::Left).tab().selection_size(),
        8_698_400 + 9
    );

    // Another folder drops the sizes.
    let design = VfsPath::new(&fx.root.join("Design").to_string_lossy());
    app(&mut h).navigate_path(Side::Left, design);
    settle(&mut h);
    assert!(app(&mut h).pane(Side::Left).tab().dir_sizes.is_empty());
}

#[test]
fn real_folder_size_plugin_measures_fixture() {
    let fx = fixture();
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(|ctx: &egui::Context| {
            PluginHost::new(
                ctx.clone(),
                vec![Box::new(manta_plugin_foldersize::FolderSizePlugin::new())],
                None,
            )
        }),
    );
    let start = Instant::now();
    loop {
        h.step();
        let done = app(&mut h)
            .pane(Side::Left)
            .tab()
            .dir_sizes
            .get("Design")
            .is_some_and(|s| s.complete);
        if done || start.elapsed() > crate::test_support::wait() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let tab = app(&mut h).pane(Side::Left).tab();
    assert_eq!(
        tab.dir_sizes["Design"].bytes,
        2_480_000 + 18_400 + 6_200_000
    );
    assert_eq!(tab.dir_sizes["Design"].files, 3);
}

#[test]
fn render_folder_sizes() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = sizes_harness(&fx, theme, true);
        let tab = app(&mut h).pane_mut(Side::Left).tab_mut();
        tab.sort = crate::util::SortSpec {
            column: crate::util::SortColumn::Size,
            ascending: false,
        };
        tab.rebuild_view();
        select(&mut h, Side::Left, &["Design"]);
        h.run_steps(3);
        save(&mut h, &format!("foldersizes_list_{tname}"));
        let img = h.render().expect("render");
        let sub = image::imageops::crop_imm(&img, 400, 80, 1120, 560).to_image();
        sub.save(out_dir().join(format!("foldersizes_list_crop_{tname}.png")))
            .expect("save");
    }
}

// ------------------------------------------------------------------ batch rename

use manta_plugin_rename::rules::{ExtMode, Rule, RuleKind, RuleType, SpaceMode, Target};
use manta_plugin_rename::BatchRenamePlugin;

type SharedRename = std::sync::Arc<std::sync::Mutex<BatchRenamePlugin>>;

/// The real Batch Rename plugin, shared with the test so it can set rules.
struct RenameProxy(SharedRename);

impl Plugin for RenameProxy {
    fn info(&self) -> PluginInfo {
        self.0.lock().unwrap().info()
    }
    fn enable(&mut self, handle: PluginHandle) {
        self.0.lock().unwrap().enable(handle)
    }
    fn disable(&mut self) {
        self.0.lock().unwrap().disable()
    }
    fn on_event(&mut self, ev: &PluginEvent) {
        self.0.lock().unwrap().on_event(ev)
    }
    fn commands(&self) -> Vec<CommandSpec> {
        self.0.lock().unwrap().commands()
    }
    fn run_command(&mut self, id: &str, pane: &manta_plugin::PaneState) {
        self.0.lock().unwrap().run_command(id, pane)
    }
    fn panels(&self) -> Vec<manta_plugin::PanelSpec> {
        self.0.lock().unwrap().panels()
    }
    fn panel_size(&self, panel: &str) -> Option<[f32; 2]> {
        self.0.lock().unwrap().panel_size(panel)
    }
    fn panel_ui(&mut self, panel: &str, pane: &manta_plugin::PaneState, ui: &mut egui::Ui) {
        self.0.lock().unwrap().panel_ui(panel, pane, ui)
    }
}

const PHOTOS: &[&str] = &[
    "IMG_0041.JPG",
    "IMG_0042.JPG",
    "IMG_0043.JPG",
    "IMG_0044.JPEG",
    "IMG_0045.JPG",
    "notes.txt",
];

/// A "Photos" folder: camera files, notes and an existing "Iceland-42.jpg"
/// that the renames collide with.
fn photos_fixture() -> Fixture {
    let tmp = tempfile::Builder::new()
        .prefix("manta-rename")
        .tempdir()
        .unwrap();
    let root = tmp.path().canonicalize().unwrap().join("Photos");
    std::fs::create_dir_all(root.join("Design")).unwrap();
    for (i, n) in PHOTOS.iter().enumerate() {
        touch(
            &root.join(n),
            2_000_000 + i as u64 * 310_000,
            ago(3600 * (i as u64 + 1)),
        );
    }
    touch(&root.join("Iceland-42.jpg"), 1_900_000, ago(86_400 * 30));
    touch(
        &root.join("Screenshot 2026-10-01 at 09.41.png"),
        640_000,
        ago(86_400 * 19),
    );
    Fixture { _tmp: tmp, root }
}

fn rename_harness(
    fx: &Fixture,
    theme: Theme,
    gpu: bool,
) -> (
    H,
    SharedRename,
    std::sync::Arc<std::sync::Mutex<Vec<manta_plugin::FileAction>>>,
) {
    let plugin: SharedRename = std::sync::Arc::new(std::sync::Mutex::new(BatchRenamePlugin::new()));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (p2, rec) = (plugin.clone(), log.clone());
    let h = harness_full(
        fx,
        WIDE,
        theme,
        gpu,
        Box::new(move |ctx: &egui::Context| {
            PluginHost::new(
                ctx.clone(),
                vec![Box::new(RenameProxy(p2)), Box::new(ActionRecorder(rec))],
                None,
            )
        }),
    );
    (h, plugin, log)
}

/// Three rules: regex replace, lower-case extension, spaces → dashes.
fn photo_rules() -> Vec<Rule> {
    let mut clean = RuleType::Clean.new_rule();
    if let RuleKind::Clean { spaces, .. } = &mut clean.kind {
        *spaces = SpaceMode::Dash;
    }
    vec![
        Rule {
            enabled: true,
            kind: RuleKind::Replace {
                find: r"IMG_00(\d+)".into(),
                replace: "Iceland $1".into(),
                regex: true,
                case_sensitive: true,
                all: true,
                target: Target::Name,
            },
        },
        Rule {
            enabled: true,
            kind: RuleKind::Extension {
                mode: ExtMode::Lower,
                new_ext: String::new(),
            },
        },
        clean,
    ]
}

fn wait_until(h: &mut H, what: &str, mut f: impl FnMut(&mut H) -> bool) {
    let start = Instant::now();
    while !f(h) {
        assert!(
            start.elapsed() < crate::test_support::wait(),
            "timed out waiting for {what}"
        );
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn names_in(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// A plain click at `pos` (List view row).
fn click_row_at(h: &mut H, pos: egui::Pos2) {
    h.hover_at(pos);
    h.step();
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        h.step();
    }
}

/// Centre of the List view row showing `name` in the left pane (WIDE).
fn row_pos(h: &mut H, name: &str) -> egui::Pos2 {
    let tab = app(h).pane(Side::Left).tab();
    let vi = tab.view_index_of(name).expect("row") as f32;
    egui::pos2(
        420.0,
        120.0 + vi * crate::theme::size::ROW - tab.scroll_offset,
    )
}

/// Let more than the double-click time pass between two clicks.
fn pause(h: &mut H) {
    std::thread::sleep(crate::pane::RENAME_CLICK_DELAY);
    h.run_steps(30);
}

/// Finder's click-to-rename: a second, slow click on the selected item
/// makes its name editable in place (stem selected); ↩ renames, Esc
/// cancels. A double-click opens instead, and a first click only selects.
#[test]
fn click_on_selected_item_renames_in_place() {
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    app(&mut h).active = Side::Left;
    let editing = |h: &mut H| app(h).pane(Side::Left).tab().inline_rename.is_some();

    // First click selects only.
    let pos = row_pos(&mut h, "notes.txt");
    click_row_at(&mut h, pos);
    pause(&mut h);
    assert!(
        !editing(&mut h),
        "a click on an unselected item only selects"
    );
    assert!(app(&mut h)
        .pane(Side::Left)
        .tab()
        .selected
        .contains("notes.txt"));

    // A second click on it, after a moment: the name becomes editable.
    click_row_at(&mut h, pos);
    pause(&mut h);
    assert!(editing(&mut h), "second click starts renaming");
    if wgpu_available() {
        save(&mut h, "inline_rename");
    }
    // The stem is selected: typing replaces "notes", keeps ".txt".
    h.event(egui::Event::Text("todo".into()));
    h.run_steps(2);
    let name = app(&mut h)
        .pane(Side::Left)
        .tab()
        .inline_rename
        .as_ref()
        .map(|r| r.form.name.clone());
    assert_eq!(name.as_deref(), Some("todo.txt"));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(!editing(&mut h));
    wait_until(&mut h, "the rename", |_| fx.root.join("todo.txt").exists());
    assert!(!fx.root.join("notes.txt").exists());
    wait_until(&mut h, "the renamed row selected", |h| {
        app(h)
            .pane(Side::Left)
            .tab()
            .view_index_of("todo.txt")
            .is_some()
            && app(h).pane(Side::Left).tab().selected.contains("todo.txt")
    });

    // Esc cancels.
    let pos = row_pos(&mut h, "todo.txt");
    click_row_at(&mut h, pos);
    pause(&mut h);
    assert!(editing(&mut h));
    h.event(egui::Event::Text("other".into()));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(!editing(&mut h));
    pause(&mut h);
    assert!(
        fx.root.join("todo.txt").exists(),
        "{:?}",
        names_in(&fx.root)
    );

    // A double-click on the selected item opens it, no renaming.
    let pos = row_pos(&mut h, "Design");
    click_row_at(&mut h, pos);
    pause(&mut h);
    // Each test frame advances egui's clock by ~0.25 s: widen the
    // double-click time so two clicks in a row count as one.
    h.ctx
        .options_mut(|o| o.input_options.max_double_click_delay = 2.0);
    click_row_at(&mut h, pos);
    click_row_at(&mut h, pos);
    pause(&mut h);
    assert!(!editing(&mut h), "double-click must not rename");
    settle(&mut h);
    assert!(app(&mut h)
        .pane(Side::Left)
        .tab()
        .loc
        .path
        .as_str()
        .ends_with("/Design"));
}

/// ⌥⌘N (File menu and both context menus) asks for a name, creates an
/// empty file and selects it; a taken name is refused in the dialog.
#[test]
fn new_file_creates_an_empty_file_and_selects_it() {
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::ALT,
        egui::Key::N,
    );
    h.run_steps(2);
    let Some(Dialog::Name(f)) = &mut app(&mut h).dialog else {
        panic!("⌥⌘N opens the name dialog");
    };
    assert!(f.new_file && f.original.is_none());
    assert_eq!(f.name, "untitled.txt");
    f.name = "README.md".into();
    assert!(f.validated().is_err(), "an existing name is refused");
    f.name = "todo.md".into();
    h.key_press(egui::Key::Enter);
    wait_until(&mut h, "the new file selected", |h| {
        app(h)
            .pane(Side::Left)
            .tab()
            .cursor_entry()
            .is_some_and(|e| e.name == "todo.md")
    });
    assert!(app(&mut h).dialog.is_none());
    assert_eq!(std::fs::metadata(fx.root.join("todo.md")).unwrap().len(), 0);
}

/// With only one pane left (either one closed), renaming works (issue #3:
/// it crashed, the pane of the renamed item was looked up as `panes[1]`).
#[test]
fn rename_with_a_single_pane() {
    for closed in [Side::Right, Side::Left] {
        let fx = fixture();
        let mut h = harness(&fx, WIDE, Theme::Light);
        let ctx = h.ctx.clone();
        app(&mut h).handle_action(&ctx, super::Action::ClosePane(closed));
        h.run_steps(2);
        assert_eq!(app(&mut h).panes.len(), 1);
        let side = app(&mut h).panes[0].id;
        app(&mut h).navigate_path(side, VfsPath::new(&fx.root.to_string_lossy()));
        settle(&mut h);
        select(&mut h, side, &["README.md"]);
        h.key_press(egui::Key::F2);
        h.run_steps(2);
        let Some(Dialog::Name(f)) = &mut app(&mut h).dialog else {
            panic!("F2 opens the name dialog");
        };
        f.name = "LIESMICH.md".into();
        h.key_press(egui::Key::Enter);
        wait_until(&mut h, "the renamed file selected", |h| {
            app(h)
                .pane(side)
                .tab()
                .cursor_entry()
                .is_some_and(|e| e.name == "LIESMICH.md")
        });
        assert!(fx.root.join("LIESMICH.md").exists() && !fx.root.join("README.md").exists());
    }
}

/// Right-click a folder: New Folder / New File are made inside it (the
/// dialog says where); the pane stays where it is.
#[test]
fn context_menu_creates_inside_the_clicked_folder() {
    use egui_kittest::kittest::Queryable;
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    for (item, file, name) in [
        ("New Folder in “Design”", false, "Made Inside"),
        ("New File in “Design”", true, "made-inside.txt"),
    ] {
        select(&mut h, Side::Left, &["Design"]);
        h.run_steps(2);
        let pos = egui::pos2(330.0, row_y(&mut h, "Design"));
        h.hover_at(pos);
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: Default::default(),
            });
        }
        h.run_steps(3);
        h.get_by_label_contains(item).click();
        h.run_steps(2);
        let Some(Dialog::Name(f)) = &mut app(&mut h).dialog else {
            panic!("{item} opens the name dialog");
        };
        assert!(f.inside && f.new_file == file);
        assert!(f.dir.as_str().ends_with("/Design"));
        f.name = name.into();
        h.key_press(egui::Key::Enter);
        let path = fx.root.join("Design").join(name);
        wait_until(&mut h, "made inside Design", |_| path.exists());
        assert_eq!(path.is_dir(), !file);
        assert!(!fx.root.join(name).exists(), "not next to it");
        h.run_steps(2);
        assert!(app(&mut h).dialog.is_none());
        assert_eq!(
            app(&mut h).pane(Side::Left).tab().loc.path.as_str(),
            fx.root.to_str().unwrap(),
            "the pane stays"
        );
    }
}

/// "File icons from macOS": List rows show the type's icon once Quick
/// Look made it (the glyph meanwhile).
#[cfg(target_os = "macos")]
#[test]
fn list_rows_show_macos_type_icons() {
    use crate::thumbs::{type_key, Method, Source, Thumb};
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    app(&mut h).test_type_icons = true;
    let px = crate::thumbs::bucket_for(18.0 * h.ctx.pixels_per_point());
    wait_until(&mut h, "the .md type icon made", |h| {
        let key = type_key("md", px);
        matches!(
            app(h)
                .thumbs
                .get_any(&key, || Some((Source::Type("md".into()), Method::TypeIcon))),
            Thumb::Ready(_)
        )
    });
    if std::env::var_os("MANTA_RENDER_DIR").is_some() {
        // Every type in view, not just .md.
        for _ in 0..20 {
            h.run_steps(2);
            app(&mut h).thumbs.wait_idle(Duration::from_secs(5));
        }
        save(&mut h, "list_system_icons");
    }
}

/// The status bar says how much space is left where the active pane is,
/// like Finder ("… · 18 GB available").
#[test]
fn status_bar_shows_free_space() {
    use egui_kittest::kittest::Queryable;
    let fx = fixture();
    let mut h = harness(&fx, WIDE, Theme::Light);
    assert!(
        h.query_by_label_contains("available").is_none(),
        "off in tests unless asked for"
    );
    app(&mut h).test_space = true;
    wait_until(&mut h, "the free space shown", |h| {
        h.query_by_label_contains(" available").is_some()
    });
    assert!(
        h.query_by_label_contains("hidden  ·  ").is_some(),
        "after the item count"
    );
}

#[test]
fn batch_rename_renames_and_undoes_on_local_fs() {
    use manta_plugin::{FileActionKind, FileActionPhase};
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap().join("Cycle");
    std::fs::create_dir_all(root.join("Design")).unwrap();
    for (n, body) in [
        ("a.txt", "A"),
        ("b.txt", "B"),
        ("c.txt", "C"),
        ("keep.md", "K"),
    ] {
        std::fs::write(root.join(n), body).unwrap();
    }
    let fx = Fixture {
        _tmp: tmp,
        root: root.clone(),
    };
    let (mut h, plugin, log) = rename_harness(&fx, Theme::Light, false);
    let ctx = h.ctx.clone();

    // One item: F2 keeps the classic rename dialog.
    select(&mut h, Side::Left, &["keep.md"]);
    app(&mut h).handle_action(&ctx, super::Action::Rename(Side::Left));
    assert!(matches!(app(&mut h).dialog, Some(Dialog::Name(_))));
    app(&mut h).dialog = None;

    // Several items: F2 opens the batch rename sheet.
    select(&mut h, Side::Left, &["a.txt", "b.txt", "c.txt"]);
    h.key_press(egui::Key::F2);
    wait_until(&mut h, "F2 with a multi-selection opens the sheet", |h| {
        app(h).plugin_modal_open()
    });
    {
        let p = plugin.lock().unwrap();
        let s = p.session.as_ref().expect("session");
        let names: Vec<&str> = s.items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(
            names,
            ["a.txt", "b.txt", "c.txt"],
            "pane order; selected {:?}, all {:?}",
            s.selected,
            s.all_names
        );
    }
    // A 3-cycle: a → b, b → c, c → a.
    let rep = |find: &str, to: &str| Rule {
        enabled: true,
        kind: RuleKind::Replace {
            find: find.into(),
            replace: to.into(),
            regex: true,
            case_sensitive: true,
            all: true,
            target: Target::Name,
        },
    };
    plugin.lock().unwrap().rules = vec![
        rep("^a$", "tmp"),
        rep("^c$", "a"),
        rep("^b$", "c"),
        rep("^tmp$", "b"),
    ];
    h.run_steps(2);
    {
        let mut p = plugin.lock().unwrap();
        let pv = p.preview().clone();
        assert_eq!(pv.changed, 3, "{:?}", pv.rows);
        assert_eq!(pv.conflicts, 0);
    }
    log.lock().unwrap().clear();
    // Enter applies (the find field has focus).
    h.key_press(egui::Key::Enter);
    wait_until(&mut h, "the renames", |h| {
        h.state()
            .toasts
            .actions()
            .iter()
            .any(|(label, _)| label == "Undo")
    });
    assert_eq!(std::fs::read_to_string(root.join("b.txt")).unwrap(), "A");
    assert_eq!(std::fs::read_to_string(root.join("c.txt")).unwrap(), "B");
    assert_eq!(std::fs::read_to_string(root.join("a.txt")).unwrap(), "C");
    assert_eq!(
        names_in(&root),
        ["Design", "a.txt", "b.txt", "c.txt", "keep.md"],
        "no temp names left"
    );
    assert!(
        !app(&mut h).plugin_modal_open(),
        "the sheet closes after success"
    );
    // One file action with every pair.
    let events = std::mem::take(&mut *log.lock().unwrap());
    let renames: Vec<_> = events
        .iter()
        .filter(|a| a.kind == FileActionKind::Rename)
        .collect();
    assert_eq!(renames.len(), 2, "started + finished: {events:#?}");
    assert_eq!(renames[0].renames.len(), 3);
    assert_eq!(renames[0].sources.len(), 3);
    assert!(
        matches!(
            renames[1].phase,
            FileActionPhase::Finished {
                ok: true,
                files: 3,
                ..
            }
        ),
        "{:?}",
        renames[1]
    );
    // The pane shows the new listing; renamed items stay selected. (The
    // reload starts after the job: wait for it rather than for a time.)
    let selected = |h: &mut H| {
        let mut v: Vec<String> = app(h)
            .pane(Side::Left)
            .tab()
            .selected
            .iter()
            .cloned()
            .collect();
        v.sort();
        v
    };
    wait_until(&mut h, "the renamed items selected", |h| {
        !app(h).pane(Side::Left).tab().loading && selected(h) == ["a.txt", "b.txt", "c.txt"]
    });

    // Undo from the toast.
    let (_, cmd) = h
        .state()
        .toasts
        .actions()
        .into_iter()
        .find(|(l, _)| l == "Undo")
        .unwrap();
    assert_eq!(cmd, "manta.rename:undo-rename");
    assert!(app(&mut h)
        .palette_entries()
        .iter()
        .any(|e| e.label == "Undo Batch Rename"));
    app(&mut h).handle_action(&ctx, super::Action::RunPluginCommand(cmd));
    wait_until(&mut h, "the undo", |_| {
        std::fs::read_to_string(root.join("a.txt")).unwrap() == "A"
    });
    assert_eq!(std::fs::read_to_string(root.join("b.txt")).unwrap(), "B");
    assert_eq!(std::fs::read_to_string(root.join("c.txt")).unwrap(), "C");
    wait_until(&mut h, "undo done", |_| {
        plugin.lock().unwrap().undo.is_none()
    });
    wait_until(&mut h, "Undo gone from the palette", |h| {
        !app(h)
            .palette_entries()
            .iter()
            .any(|e| e.label == "Undo Batch Rename")
    });

    // A collision with an untouched file is refused by the app as a whole
    // (even if a plugin skipped the preview).
    let pane = app(&mut h).pane_state(Side::Left);
    app(&mut h).plugin_rename_many(
        "manta.rename".into(),
        pane.pane,
        pane.path.clone(),
        vec![
            ("a.txt".into(), "x.txt".into()),
            ("b.txt".into(), "keep.md".into()),
        ],
    );
    wait_until(&mut h, "the refusal", |_| {
        log.lock()
            .unwrap()
            .iter()
            .any(|a| a.kind == FileActionKind::Rename && a.is_finished())
    });
    assert_eq!(
        names_in(&root),
        ["Design", "a.txt", "b.txt", "c.txt", "keep.md"]
    );
    assert_eq!(std::fs::read_to_string(root.join("keep.md")).unwrap(), "K");

    // Esc closes the sheet.
    select(&mut h, Side::Left, &["a.txt", "b.txt"]);
    app(&mut h).handle_action(
        &ctx,
        super::Action::RunPluginCommand(super::BATCH_RENAME_CMD.into()),
    );
    wait_until(&mut h, "the sheet", |h| app(h).plugin_modal_open());
    h.key_press(egui::Key::Escape);
    wait_until(&mut h, "Esc closes the sheet", |h| {
        !app(h).plugin_modal_open()
    });
}

#[test]
fn batch_rename_without_selection_uses_all_files() {
    let fx = photos_fixture();
    let (mut h, plugin, _) = rename_harness(&fx, Theme::Light, false);
    let ctx = h.ctx.clone();
    app(&mut h).pane_mut(Side::Left).tab_mut().selected.clear();
    app(&mut h).pane_mut(Side::Left).tab_mut().cursor = None;
    app(&mut h).handle_action(
        &ctx,
        super::Action::RunPluginCommand(super::BATCH_RENAME_CMD.into()),
    );
    h.run_steps(3);
    let p = plugin.lock().unwrap();
    let s = p.session.as_ref().unwrap();
    assert!(s.selected.is_empty());
    assert_eq!(s.items.len(), PHOTOS.len() + 2, "every file, no folders");
    assert!(s.items.iter().all(|i| !i.is_dir));
}

fn rename_sheet(theme: Theme) -> (Fixture, H, SharedRename) {
    let fx = photos_fixture();
    let (mut h, plugin, _) = rename_harness(&fx, theme, true);
    let ctx = h.ctx.clone();
    let mut sel: Vec<&str> = PHOTOS.to_vec();
    sel.push("Screenshot 2026-10-01 at 09.41.png");
    select(&mut h, Side::Left, &sel);
    app(&mut h).handle_action(&ctx, super::Action::Rename(Side::Left));
    h.run_steps(3);
    plugin.lock().unwrap().rules = photo_rules();
    h.run_steps(3);
    (fx, h, plugin)
}

#[test]
fn render_batch_rename() {
    require_gpu!();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let (_fx, mut h, plugin) = rename_sheet(theme);
        {
            let mut p = plugin.lock().unwrap();
            let pv = p.preview().clone();
            assert_eq!(pv.conflicts, 1, "{:?}", pv.rows);
            assert_eq!(pv.changed, 5);
        }
        save(&mut h, &format!("batch_rename_sheet_{tname}"));

        // The presets menu.
        {
            let mut p = plugin.lock().unwrap();
            p.presets.items.clear();
            p.presets.upsert("Camera imports", photo_rules());
            p.presets.upsert(
                "Dated scans",
                vec![RuleType::Date.new_rule(), RuleType::Clean.new_rule()],
            );
            p.presets.upsert(
                "Web-safe names",
                vec![RuleType::Clean.new_rule(), RuleType::Case.new_rule()],
            );
            p.ui.open_presets = true;
            p.ui.preset_name = "Iceland trip".into();
        }
        h.run_steps(4);
        save(&mut h, &format!("batch_rename_presets_{tname}"));
    }
}

#[test]
fn render_batch_rename_done_toast() {
    require_gpu!();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let (fx, mut h, plugin) = rename_sheet(theme);
        // Resolve the conflict, then apply.
        std::fs::remove_file(fx.root.join("Iceland-42.jpg")).unwrap();
        app(&mut h).refresh(Side::Left);
        settle(&mut h);
        let ctx = h.ctx.clone();
        let _ = ctx;
        {
            let mut p = plugin.lock().unwrap();
            let s = p.session.as_mut().unwrap();
            s.all_names.retain(|n| n != "Iceland-42.jpg");
            let pane = s.pane;
            let dir = s.dir.clone();
            let entries = s
                .listing
                .clone()
                .unwrap()
                .unwrap()
                .into_iter()
                .filter(|e| e.name != "Iceland-42.jpg")
                .collect();
            p.on_listing(&manta_plugin::DirListing {
                pane,
                dir,
                entries,
                error: None,
            });
        }
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        wait_until(&mut h, "the toast", |h| {
            h.state().toasts.actions().iter().any(|(l, _)| l == "Undo")
        });
        settle(&mut h);
        assert!(fx.root.join("Iceland-41.jpg").exists());
        save(&mut h, &format!("batch_rename_done_{tname}"));
    }
}

// ------------------------------------------------------------------ quick actions

const QA: &str = manta_plugin_quickactions::PLUGIN_ID;

/// An installed copy of a retired external plugin (just its manifest), so
/// Settings shows it as replaced by its built-in successor.
fn install_retired(root: &Path, id: &str, name: &str) {
    let dir = manta_plugin::host::HostPaths::under(root)
        .plugins_dir
        .join(id);
    std::fs::create_dir_all(&dir).unwrap();
    let toml = format!(
        "id = \"{id}\"\nname = \"{name}\"\nversion = \"1.1.0\"\nauthor = \"Manta\"\napi_version = 1\n\
         description = \"The former Python version.\"\ncommand = [\"python3\", \"main.py\"]\n"
    );
    std::fs::write(dir.join("plugin.toml"), toml).unwrap();
}

/// The app with only the built-in Quick Actions plugin running (and the
/// old Python copy installed, switched off).
fn qa_harness(fx: &Fixture, theme: Theme, root: &Path) -> H {
    let paths = manta_plugin::host::HostPaths::under(root);
    install_retired(root, "quick-actions", "Quick Actions");
    let mut h = harness_full(
        fx,
        WIDE,
        theme,
        true,
        Box::new(move |ctx: &egui::Context| {
            let builtins: Vec<Box<dyn Plugin>> = vec![Box::new(
                manta_plugin_quickactions::QuickActionsPlugin::new(),
            )];
            let mut host = PluginHost::new(ctx.clone(), builtins, Some(paths));
            host.set_bundled(manta_plugin::bundled::embedded());
            host
        }),
    );
    select(&mut h, Side::Left, &["README.md"]);
    wait_until(&mut h, "quick actions registered", |h| {
        h.state()
            .plugins
            .commands()
            .iter()
            .any(|c| c.full_id == format!("{QA}:run.count-lines"))
    });
    // The "is now built in" toast was checked by the host tests.
    app(&mut h).toasts.clear();
    h
}

/// Add two demo actions to actions.json: one that fails, one that runs long.
fn add_demo_actions(root: &Path, h: &mut H) {
    let file = root.join("plugin-data").join(QA).join("actions.json");
    wait_until(h, "actions.json", |_| file.exists());
    let text = std::fs::read_to_string(&file).unwrap();
    let extra = r#""actions": [
    {"id": "lint", "title": "Lint Markdown", "icon": "check", "applies_to": ["md"], "command": "markdownlint {files}"},
    {"id": "backup", "title": "Back Up to NAS", "icon": "download", "command": "sleep 30"},"#;
    std::fs::write(&file, text.replacen("\"actions\": [", extra, 1)).unwrap();
    wait_until(h, "reloaded actions", |h| {
        h.state()
            .plugins
            .commands()
            .iter()
            .any(|c| c.full_id == format!("{QA}:run.backup"))
    });
}

fn qa_panel_has(h: &mut H, needle: &str) -> bool {
    match h.state().plugins.panel_content(QA, "actions", None) {
        manta_plugin::host::PanelContent::Blocks(b) => {
            serde_json::to_string(&*b).unwrap().contains(needle)
        }
        _ => false,
    }
}

#[test]
fn render_quick_actions() {
    use egui_kittest::kittest::Queryable;
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let root = tempfile::tempdir().unwrap();
        let mut h = qa_harness(&fx, theme, root.path());
        add_demo_actions(root.path(), &mut h);
        app(&mut h).toasts.clear();
        let readme = app(&mut h).pane_state(Side::Left);

        // Context menu on README.md with the Quick Actions submenu open.
        let pos = egui::pos2(330.0, readme_row_y(&mut h));
        h.hover_at(pos);
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed: true,
            modifiers: Default::default(),
        });
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed: false,
            modifiers: Default::default(),
        });
        h.run_steps(3);
        h.get_by_label(&format!("{}   Quick Actions", crate::icons::PUZZLE_PIECE))
            .hover();
        h.run_steps(4);
        save(&mut h, &format!("quick_actions_menu_{tname}"));
        h.key_press(egui::Key::Escape);
        h.run_steps(3);

        // Panel with runs: two done, one failed, one running.
        let a = app(&mut h);
        a.plugins.set_panel_open(QA, "actions", None, true);
        for id in ["count-lines", "sha256", "lint", "backup"] {
            app(&mut h)
                .plugins
                .run_command(&format!("{QA}:run.{id}"), &readme);
            std::thread::sleep(Duration::from_millis(150));
            h.step();
        }
        wait_until(&mut h, "runs in the panel", |h| {
            qa_panel_has(h, "\"Count Lines\"")
                && qa_panel_has(h, "failed")
                && qa_panel_has(h, "running")
                && qa_panel_has(h, "README.md")
        });
        // Show the checksum's output.
        let sha_job = 2;
        app(&mut h)
            .plugins
            .panel_action(QA, "actions", &format!("show:{sha_job}"), &readme);
        wait_until(&mut h, "sha output", |h| qa_panel_has(h, "$ shasum"));
        std::thread::sleep(Duration::from_millis(1200));
        h.run_steps(6);
        let toasts = std::mem::take(&mut app(&mut h).toasts);
        h.run_steps(2);
        save(&mut h, &format!("quick_actions_panel_{tname}"));
        app(&mut h).toasts = toasts;
        h.run_steps(2);
        save(&mut h, &format!("quick_actions_toasts_{tname}"));

        // The confirmation sheet of the rsync action.
        app(&mut h).toasts.clear();
        app(&mut h)
            .plugins
            .run_command(&format!("{QA}:run.rsync-other"), &readme);
        wait_until(&mut h, "confirm sheet", |h| {
            h.state().plugins.is_panel_open(QA, "confirm", None)
        });
        h.run_steps(6);
        save(&mut h, &format!("quick_actions_confirm_{tname}"));
        app(&mut h)
            .plugins
            .set_panel_open(QA, "confirm", None, false);

        // Settings: the built-in, and the old Python copy marked as replaced.
        let a = app(&mut h);
        a.plugins.set_panel_open(QA, "actions", None, false);
        a.settings.show(Some(crate::settings::SettingsTab::Plugins));
        a.settings.scroll_to = Some(1000.0);
        h.run_steps(6);
        save(&mut h, &format!("quick_actions_settings_{tname}"));
        app(&mut h).plugins.shutdown(Duration::from_secs(1));
    }
}

/// y (points) of the README.md row in the left pane (fixture layout: the
/// first row at 120 pt, rows 24 pt high).
fn readme_row_y(h: &mut H) -> f32 {
    row_y(h, "README.md")
}

/// The y of `name`'s row in the left pane (List view).
fn row_y(h: &mut H, name: &str) -> f32 {
    let i = h
        .state()
        .pane(Side::Left)
        .tab()
        .view_index_of(name)
        .unwrap_or_else(|| panic!("{name} listed"));
    120.0 + 24.0 * i as f32
}

// ------------------------------------------------------------------ project info

const PI: &str = manta_plugin_projectinfo::PLUGIN_ID;

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), &target).unwrap();
        }
    }
}

/// `~/Sites`-like folder with the plugin's Laravel and Node fixtures
/// (as Git repositories on `main`) and a `Design` folder for the harness.
fn projects_fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("Sites");
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugin-projectinfo/tests/fixtures");
    for name in ["acme-shop", "storefront-ui"] {
        copy_tree(&src.join(name), &root.join(name));
        std::fs::create_dir_all(root.join(name).join(".git")).unwrap();
        std::fs::write(root.join(name).join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    }
    std::fs::create_dir_all(root.join("Design")).unwrap();
    std::fs::write(root.join("Design/notes.md"), "# Notes\n").unwrap();
    Fixture { _tmp: tmp, root }
}

/// Two earlier runs for the Recent section, in the format the Python
/// plugin wrote (and the built-in reads) into its data folder.
fn write_recent(fx: &Fixture, config: &Path) {
    let paths = manta_plugin::host::HostPaths::under(config);
    let shop = fx.root.join("acme-shop").to_string_lossy().into_owned();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let recent = serde_json::json!({ shop.clone(): [
        {"title": "dev", "section": "pnpm", "command": "pnpm run dev", "cwd": shop, "danger": null, "ts": now - 240.0},
        {"title": "migrate", "section": "Laravel", "command": "php artisan migrate", "cwd": shop, "danger": null, "ts": now - 3.0 * 3600.0},
    ]});
    std::fs::create_dir_all(paths.data_root.join(PI)).unwrap();
    std::fs::write(
        paths.data_root.join(PI).join("recent.json"),
        recent.to_string(),
    )
    .unwrap();
}

fn pi_host(ctx: &egui::Context, config: &Path) -> PluginHost {
    let builtins: Vec<Box<dyn Plugin>> =
        vec![Box::new(manta_plugin_projectinfo::ProjectInfoPlugin::new())];
    PluginHost::new(
        ctx.clone(),
        builtins,
        Some(manta_plugin::host::HostPaths::under(config)),
    )
}

/// The app with only the built-in Project Info plugin, the left pane in
/// the Laravel project and the right pane in the Node project.
fn project_info_harness(fx: &Fixture, theme: Theme, config: &Path) -> H {
    write_recent(fx, config);
    let config = config.to_path_buf();
    let mut h = harness_full(
        fx,
        WIDE,
        theme,
        true,
        Box::new(move |ctx: &egui::Context| pi_host(ctx, &config)),
    );
    app(&mut h).navigate_path(
        Side::Left,
        VfsPath::new(&fx.root.join("acme-shop").to_string_lossy()),
    );
    app(&mut h).navigate_path(
        Side::Right,
        VfsPath::new(&fx.root.join("storefront-ui").to_string_lossy()),
    );
    settle(&mut h);
    wait_until(&mut h, "project chips", |h| {
        let chip = |side: PaneId| h.state().plugins.chips(side).iter().any(|(id, _)| id == PI);
        chip(PaneId::Left) && chip(PaneId::Right)
    });
    h
}

fn pi_panel_has(h: &mut H, needle: &str) -> bool {
    pi_panel_has_in(h, "project", needle)
}

fn pi_panel_has_in(h: &mut H, panel: &str, needle: &str) -> bool {
    match h.state().plugins.panel_content(PI, panel, None) {
        manta_plugin::host::PanelContent::Blocks(b) => {
            serde_json::to_string(&*b).unwrap().contains(needle)
        }
        _ => false,
    }
}

#[test]
fn render_project_info() {
    require_gpu!();
    let fx = projects_fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let config = tempfile::tempdir().unwrap();
        let mut h = project_info_harness(&fx, theme, config.path());
        let chip = h
            .state()
            .plugins
            .chips(PaneId::Left)
            .into_iter()
            .find(|(id, _)| id == PI)
            .unwrap()
            .1;
        assert_eq!(chip.text, "Laravel · 28 scripts");

        // The chips in both path bars.
        h.run_steps(3);
        let img = h.render().expect("render");
        let bar = image::imageops::crop_imm(&img, 0, 70, 2560, 110).to_image();
        bar.save(out_dir().join(format!("project_info_chips_{tname}.png")))
            .expect("save");

        // Laravel project with the panel open.
        app(&mut h)
            .plugins
            .set_panel_open(PI, "project", None, true);
        wait_until(&mut h, "laravel panel", |h| {
            pi_panel_has(h, "\"acme-shop\"") && pi_panel_has(h, "Recent")
        });
        settle(&mut h);
        save(&mut h, &format!("project_info_laravel_{tname}"));

        // The Node-only project (right pane active).
        app(&mut h).active = Side::Right;
        wait_until(&mut h, "node panel", |h| {
            pi_panel_has(h, "\"storefront-ui\"")
        });
        settle(&mut h);
        save(&mut h, &format!("project_info_node_{tname}"));

        // Confirmation sheet for a destructive script.
        let state = app(&mut h).pane_state(Side::Right);
        let action = format!(
            "run\t{}\tnode/clean",
            fx.root.join("storefront-ui").display()
        );
        app(&mut h)
            .plugins
            .panel_action(PI, "project", &action, &state);
        wait_until(&mut h, "confirm sheet", |h| {
            h.state().plugins.is_panel_open(PI, "confirm", None)
        });
        settle(&mut h);
        save(&mut h, &format!("project_info_confirm_{tname}"));
        app(&mut h)
            .plugins
            .set_panel_open(PI, "confirm", None, false);

        // The picker (Run Project Script…) for the Laravel project.
        app(&mut h).active = Side::Left;
        wait_until(&mut h, "laravel again", |h| {
            pi_panel_has(h, "\"acme-shop\"")
        });
        let state = app(&mut h).pane_state(Side::Left);
        app(&mut h)
            .plugins
            .run_command(&format!("{PI}:run"), &state);
        wait_until(&mut h, "picker", |h| {
            h.state().plugins.is_panel_open(PI, "picker", None)
                && pi_panel_has_in(h, "picker", "Run a script in acme-shop")
        });
        settle(&mut h);
        save(&mut h, &format!("project_info_picker_{tname}"));
        app(&mut h)
            .plugins
            .set_panel_open(PI, "picker", None, false);

        // Settings: the old Python copy, replaced by the built-in.
        install_retired(config.path(), "project-info", "Project Info");
        app(&mut h).plugins.rescan();
        app(&mut h).toasts.clear();
        let a = app(&mut h);
        a.settings.show(Some(crate::settings::SettingsTab::Plugins));
        a.settings.scroll_to = Some(1000.0);
        h.run_steps(6);
        save(&mut h, &format!("project_info_settings_{tname}"));
        app(&mut h).plugins.shutdown(Duration::from_secs(1));
    }
}

/// Path-bar chips degrade instead of vanishing as the pane narrows: full,
/// cut text, icon only. Git-like and Project-Info-like chips at three pane
/// widths (wide window; sidebar open; small window with sidebar open).
#[test]
fn render_path_bar_chips_narrow() {
    require_gpu!();
    let fx = fixture();
    let chip_plugins = || -> Vec<Box<dyn Plugin>> {
        vec![
            Box::new(FakeDecorator {
                id: "test.git",
                name: "Git",
                decorations: Vec::new(),
                chip: Some(Chip {
                    text: "feature/markdown-preview ↑2".into(),
                    tone: Tone::Warning,
                    icon: Some("branch".into()),
                    tooltip: Some("Branch feature/markdown-preview, 2 ahead".into()),
                    command: Some("status".into()),
                }),
                handle: None,
            }),
            Box::new(FakeDecorator {
                id: "test.project",
                name: "Project",
                decorations: Vec::new(),
                chip: Some(Chip {
                    text: "Laravel · 28 scripts".into(),
                    tone: Tone::Neutral,
                    icon: Some("cube".into()),
                    tooltip: Some("Click to show or hide the Project panel".into()),
                    command: Some("status".into()),
                }),
                handle: None,
            }),
        ]
    };
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        for (size, sidebar, wname) in [
            (WIDE, false, "wide"),
            (WIDE, true, "sidebar"),
            ([1100.0, 700.0], true, "narrow"),
        ] {
            let mut h = harness_full(
                &fx,
                size,
                theme,
                true,
                Box::new(move |ctx: &egui::Context| plugin_host(ctx, chip_plugins())),
            );
            let root = VfsPath::new(&fx.root.to_string_lossy());
            app(&mut h).navigate_path(Side::Right, root);
            settle(&mut h);
            wait_until(&mut h, "chips", |h| {
                h.state().plugins.chips(PaneId::Left).len() == 2
            });
            if sidebar {
                app(&mut h)
                    .plugins
                    .set_panel_open("hello-python", "file-info", None, true);
            }
            settle(&mut h);
            h.run_steps(3);
            let img = h.render().expect("render");
            let bar =
                image::imageops::crop_imm(&img, 0, 70, (size[0] * 2.0) as u32, 110).to_image();
            bar.save(out_dir().join(format!("path_bar_chips_{wname}_{tname}.png")))
                .expect("save");
            // Both chips stay visible in every width.
            assert_eq!(h.state().plugins.chips(PaneId::Left).len(), 2);
        }
    }
}

/// Asks for files with `HostRequest::ReadFile` and keeps the answers.
struct FileReader {
    handle: Option<PluginHandle>,
    asks: Vec<(String, u64)>,
    got: std::sync::Arc<std::sync::Mutex<Vec<manta_plugin::FileContent>>>,
}

impl Plugin for FileReader {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "test.reader".into(),
            name: "Reader".into(),
            version: "1.0".into(),
            description: "Test plugin".into(),
            author: "Tests".into(),
            builtin: true,
        }
    }
    fn enable(&mut self, handle: PluginHandle) {
        self.handle = Some(handle);
    }
    fn disable(&mut self) {
        self.handle = None;
    }
    fn on_event(&mut self, ev: &PluginEvent) {
        match ev {
            PluginEvent::DirectoryChanged(p) if p.pane == PaneId::Left => {
                let Some(h) = &self.handle else { return };
                for (name, max) in self.asks.drain(..) {
                    h.send(HostRequest::ReadFile {
                        pane: PaneId::Left,
                        path: p.path.join(&name),
                        max_bytes: max,
                    });
                }
            }
            PluginEvent::FileContent(c) => self.got.lock().unwrap().push(c.clone()),
            _ => {}
        }
    }
}

/// `HostRequest::ReadFile` goes through the pane's VFS (LocalFs here,
/// standing in for a remote backend) on a worker and comes back as
/// `PluginEvent::FileContent`, cut at `max_bytes`.
#[test]
fn plugin_read_file_through_the_pane_vfs() {
    let fx = fixture();
    let got = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let reader = FileReader {
        handle: None,
        asks: vec![
            ("README.md".into(), 1 << 20),
            ("README.md".into(), 10),
            ("missing.md".into(), 100),
            ("src".into(), 100),
        ],
        got: got.clone(),
    };
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(move |ctx: &egui::Context| {
            PluginHost::new(ctx.clone(), vec![Box::new(reader)], None)
        }),
    );
    wait_until(&mut h, "file contents", |_| got.lock().unwrap().len() == 4);
    let got = got.lock().unwrap().clone();
    let readme = std::fs::read(fx.root.join("README.md")).unwrap();
    let by = |name: &str, max: usize| {
        got.iter()
            .find(|c| c.path.file_name() == name && (c.error.is_some() || c.bytes.len() <= max))
            .unwrap_or_else(|| panic!("{name}: {got:?}"))
            .clone()
    };
    let full = got
        .iter()
        .find(|c| c.bytes.len() == readme.len())
        .expect("full read");
    assert_eq!(full.bytes, readme);
    assert!(!full.truncated && full.size == Some(readme.len() as u64) && full.modified.is_some());
    let cut = by("README.md", 10);
    assert_eq!(cut.bytes, readme[..10].to_vec());
    assert!(cut.truncated);
    assert!(by("missing.md", 0).error.is_some());
    assert!(by("src", 0)
        .error
        .as_deref()
        .is_some_and(|e| e.contains("folder")));
}

/// The real Markdown plugin in the app: the fixture's README in the sidebar.
#[test]
fn render_markdown_panel_in_app() {
    require_gpu!();
    let fx = fixture();
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut h = harness_full(
            &fx,
            WIDE,
            theme,
            true,
            Box::new(|ctx: &egui::Context| {
                PluginHost::new(
                    ctx.clone(),
                    vec![Box::new(manta_plugin_markdown::MarkdownPlugin::new())],
                    None,
                )
            }),
        );
        select(&mut h, Side::Left, &["README.md"]);
        app(&mut h).plugins.set_panel_open(
            manta_plugin_markdown::PLUGIN_ID,
            "markdown",
            None,
            true,
        );
        for _ in 0..60 {
            h.step();
            std::thread::sleep(Duration::from_millis(15));
        }
        save(&mut h, &format!("markdown_panel_{tname}"));
    }
}

/// Space on a JPEG whose name has spaces shows the image (repro for a
/// user report). `MANTA_PREVIEW_FILE` previews a real file instead.
#[test]
fn preview_image_with_spaces() {
    require_gpu!();
    let fx = fixture();
    let name = match std::env::var_os("MANTA_PREVIEW_FILE") {
        Some(p) => {
            let p = PathBuf::from(p);
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            std::fs::copy(&p, fx.root.join(&name)).expect("copy preview file");
            name
        }
        None => {
            let name = "2024-07-15 11.14.56.jpg".to_string();
            let img =
                image::RgbImage::from_fn(320, 200, |x, y| image::Rgb([x as u8, y as u8, 160]));
            img.save(fx.root.join(&name)).expect("write jpeg");
            name
        }
    };
    let mut h = harness(&fx, WIDE, Theme::Light);
    settle(&mut h);
    select(&mut h, Side::Left, &[name.as_str()]);
    for _ in 0..5 {
        if app(&mut h).preview.is_some() {
            break;
        }
        h.key_press(egui::Key::Space);
        h.run_steps(2);
    }
    let start = Instant::now();
    while app(&mut h).preview.as_ref().is_some_and(|p| p.loading)
        && start.elapsed() < crate::test_support::wait()
    {
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
    let p = app(&mut h).preview.as_ref().expect("preview opened");
    assert_eq!(p.error, None);
    // Decoding (e.g. progressive JPEG) takes a few frames.
    let uri = p.image_uri();
    let start = Instant::now();
    loop {
        h.step();
        match h.ctx.try_load_image(&uri, egui::SizeHint::default()) {
            Ok(egui::load::ImagePoll::Ready { .. }) => break,
            Err(e) => panic!("image failed to load: {e}"),
            Ok(egui::load::ImagePoll::Pending { .. }) => {
                assert!(
                    start.elapsed() < crate::test_support::wait(),
                    "image still loading"
                );
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
    save(&mut h, "preview_image_with_spaces");
}

#[path = "archive_ux_tests.rs"]
mod archive;

#[path = "quicklook_ux_tests.rs"]
mod quicklook_ux;

#[path = "s3_ux_tests.rs"]
mod s3_ux;

#[path = "containers_ux_tests.rs"]
mod containers_ux;

#[path = "default_viewer_ux_tests.rs"]
mod default_viewer_ux;

#[path = "i18n_ux_tests.rs"]
mod i18n_ux_tests;

#[path = "i18n_plugin_ux_tests/mod.rs"]
mod i18n_plugin_ux_tests;

#[path = "native_windows_tests.rs"]
mod native_windows;

#[path = "update_ux_tests.rs"]
mod update_ux;

#[path = "ftps_ux_tests.rs"]
mod ftps_ux;

#[path = "smb_webdav_ux_tests.rs"]
mod smb_webdav_ux;

/// Typing letters in the file list jumps to the first matching name, like
/// Finder; a pause starts a new word; Space only opens Quick Look when no
/// word is being typed.
#[test]
fn type_to_select_jumps_to_names() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    app(&mut h).active = Side::Left;
    let cursor_name = |h: &mut H| {
        app(h)
            .pane(Side::Left)
            .tab()
            .cursor_entry()
            .map(|e| e.name.clone())
    };
    let type_text = |h: &mut H, t: &str| {
        h.event(egui::Event::Text(t.into()));
        h.run_steps(2);
    };

    type_text(&mut h, "R");
    assert_eq!(
        cursor_name(&mut h).as_deref(),
        Some("README.md"),
        "case-insensitive prefix"
    );
    type_text(&mut h, "ep");
    assert_eq!(
        cursor_name(&mut h).as_deref(),
        Some("report-q3.pdf"),
        "letters extend the word"
    );
    assert!(app(&mut h).preview.is_none());

    // A pause starts a new word.
    app(&mut h).pane_mut(Side::Left).tab_mut().type_ahead_at =
        Some(Instant::now() - Duration::from_secs(5));
    type_text(&mut h, "da");
    assert_eq!(cursor_name(&mut h).as_deref(), Some("data.json"));

    // No prefix match: the nearest following name.
    app(&mut h).pane_mut(Side::Left).tab_mut().type_ahead_at = None;
    type_text(&mut h, "q");
    assert_eq!(
        cursor_name(&mut h).as_deref(),
        Some("README.md"),
        "q… → next name after it"
    );

    // Space while typing belongs to the word (no Quick Look).
    app(&mut h).pane_mut(Side::Left).tab_mut().type_ahead_at = None;
    type_text(&mut h, "screen");
    h.key_press(egui::Key::Space);
    h.event(egui::Event::Text(" ".into()));
    h.run_steps(2);
    assert!(
        app(&mut h).preview.is_none(),
        "Space inside a word does not open the preview"
    );
    assert_eq!(app(&mut h).pane(Side::Left).tab().type_ahead, "screen ");

    // After the pause, Space is Quick Look again.
    app(&mut h).pane_mut(Side::Left).tab_mut().type_ahead_at =
        Some(Instant::now() - Duration::from_secs(5));
    h.key_press(egui::Key::Space);
    h.event(egui::Event::Text(" ".into()));
    h.run_steps(3);
    assert!(
        app(&mut h).preview.is_some(),
        "Space without typing opens the preview"
    );
}

/// Type-to-select in a long folder scrolls the match into view right away
/// (without waiting for another input event).
#[test]
fn type_to_select_scrolls_into_view() {
    let fx = fixture();
    let dir = fx.root.join("many");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..200 {
        std::fs::write(dir.join(format!("a-file-{i:03}.txt")), b"x").unwrap();
    }
    std::fs::write(dir.join("zebra.txt"), b"z").unwrap();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    app(&mut h).active = Side::Left;
    app(&mut h).navigate_path(Side::Left, VfsPath::new(&dir.to_string_lossy()));
    settle(&mut h);
    assert_eq!(app(&mut h).pane(Side::Left).tab().scroll_offset, 0.0);
    h.event(egui::Event::Text("z".into()));
    // Only the frames egui itself asks for: no further input events.
    h.run();
    let t = app(&mut h).pane(Side::Left).tab();
    assert_eq!(t.cursor_entry().map(|e| e.name.as_str()), Some("zebra.txt"));
    let row_top = t.cursor.unwrap() as f32 * crate::theme::size::ROW;
    assert!(
        row_top >= t.scroll_offset
            && row_top + crate::theme::size::ROW <= t.scroll_offset + t.viewport_h + 1.0,
        "zebra.txt row at {row_top} is visible in [{}, {}]",
        t.scroll_offset,
        t.scroll_offset + t.viewport_h
    );

    // The scrollable height is exactly the rows (no empty strip below the
    // last row, which showed up as a dark flickering bar).
    h.key_press(egui::Key::End);
    h.run();
    let t = app(&mut h).pane(Side::Left).tab();
    let rows_h = t.view.len() as f32 * crate::theme::size::ROW;
    let bottom = t.scroll_offset + t.viewport_h;
    eprintln!(
        "END: offset={} viewport={} rows_h={rows_h} n={}",
        t.scroll_offset,
        t.viewport_h,
        t.view.len()
    );
    assert!(
        (bottom - rows_h).abs() <= crate::theme::size::ROW,
        "scrolled to the end, the view ends at {bottom} but the rows end at {rows_h}"
    );
}

/// Repro for "type-ahead doesn't scroll" / "dark flickering bar at the bottom"
/// with the real built-in plugins (terminal panel open under the pane).
#[test]
#[ignore = "manual repro, writes renders"]
fn repro_type_ahead_with_terminal() {
    require_gpu!();
    let fx = fixture();
    let dir = fx.root.join("many");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..200 {
        std::fs::write(dir.join(format!("a-file-{i:03}.txt")), b"x").unwrap();
    }
    std::fs::write(dir.join("zebra.txt"), b"z").unwrap();
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Dark,
        true,
        Box::new(|ctx: &egui::Context| plugin_host(ctx, super::builtin_plugins())),
    );
    app(&mut h).active = Side::Left;
    app(&mut h).navigate_path(Side::Left, VfsPath::new(&dir.to_string_lossy()));
    settle(&mut h);
    for _ in 0..40 {
        h.step();
        std::thread::sleep(Duration::from_millis(25));
    }
    save(&mut h, "repro_before");
    h.event(egui::Event::Text("z".into()));
    h.step();
    save(&mut h, "repro_after_one_frame");
    let t = app(&mut h).pane(Side::Left).tab();
    eprintln!(
        "cursor={:?} offset={} viewport={}",
        t.cursor_entry().map(|e| e.name.clone()),
        t.scroll_offset,
        t.viewport_h
    );
}

/// Regression: with the terminal panel open under a pane, the file list
/// used an inherited item spacing as row pitch (30 instead of 24 pt), so
/// type-to-select scrolled to the wrong rows and an empty strip showed
/// below the last row.
#[test]
fn list_rows_line_up_with_the_terminal_open() {
    let fx = fixture();
    let dir = fx.root.join("many");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..200 {
        std::fs::write(dir.join(format!("a-file-{i:03}.txt")), b"x").unwrap();
    }
    let mut h = harness_full(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(|ctx: &egui::Context| plugin_host(ctx, super::builtin_plugins())),
    );
    app(&mut h).active = Side::Left;
    app(&mut h).navigate_path(Side::Left, VfsPath::new(&dir.to_string_lossy()));
    settle(&mut h);
    h.run_steps(6);
    let t = app(&mut h).pane(Side::Left).tab();
    let rows_h = t.view.len() as f32 * crate::theme::size::ROW;
    assert!(
        (t.content_h - rows_h).abs() <= crate::theme::size::ROW,
        "egui laid the rows out {} high, they are {rows_h}",
        t.content_h
    );
}

#[path = "panes_ux_tests.rs"]
mod panes_ux_tests;

#[path = "favorites_ux_tests.rs"]
mod favorites_ux_tests;

#[path = "devices_ux_tests.rs"]
mod devices_ux_tests;

#[path = "mouse_nav_ux_tests.rs"]
mod mouse_nav_ux_tests;

#[path = "new_tabs_ux_tests.rs"]
mod new_tabs_ux_tests;

#[path = "ssh_offer_ux_tests.rs"]
mod ssh_offer_ux_tests;

#[path = "views_ux_tests.rs"]
mod views_ux_tests;

#[path = "native_dnd_ux_tests.rs"]
mod native_dnd_ux_tests;

#[path = "trash_ux_tests.rs"]
mod trash_ux_tests;

#[path = "undo_ux_tests.rs"]
mod undo_ux_tests;

#[path = "host_key_ux_tests.rs"]
mod host_key_ux_tests;

#[path = "conflict_ux_tests.rs"]
mod conflict_ux_tests;

#[path = "live_refresh_ux_tests.rs"]
mod live_refresh_ux_tests;

#[path = "bench_listing_tests.rs"]
mod bench_listing_tests;

#[path = "large_folder_ux_tests.rs"]
mod large_folder_ux_tests;

#[path = "cloud_ux_tests.rs"]
mod cloud_ux_tests;

#[path = "finder_ux_tests.rs"]
mod finder_ux_tests;

#[path = "place_ux_tests.rs"]
mod place_ux_tests;

#[path = "progress_ux_tests.rs"]
mod progress_ux_tests;

#[path = "help_ux_tests.rs"]
mod help_ux_tests;

#[path = "manual_images_tests.rs"]
mod manual_images_tests;

#[path = "manual_images_extra_tests.rs"]
mod manual_images_extra_tests;

#[path = "readme_images_tests.rs"]
mod readme_images_tests;

#[path = "settings_scroll_ux_tests.rs"]
mod settings_scroll_ux_tests;

#[path = "modal_ux_tests.rs"]
mod modal_ux_tests;

#[path = "a11y_ux_tests.rs"]
mod a11y_ux_tests;

#[path = "discover_ux_tests.rs"]
mod discover_ux_tests;

#[path = "crash_ux_tests.rs"]
mod crash_ux_tests;
