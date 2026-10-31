//! Deleting like Finder: ⌘⌫ moves local items to the Trash right away,
//! ⌥⌘⌫ deletes permanently after asking, and servers (no Trash) always ask
//! and say that deleting there is permanent. Plus the Finder shortcuts
//! (⌘T new tab, ⌥⌘L Transfers, ⌘Y Quick Look) and the one-time "what's
//! new" notice.

use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Key, Modifiers, Theme};
use egui_kittest::kittest::Queryable;
use manta_core::{Entry, Protocol, SharedVfs, Vfs, VfsPath, VfsResult};
use manta_ops::memfs::MemFs;
use manta_plugin::host::PluginHost;
use manta_plugin::{FileActionKind, FileActionPhase};

use super::{
    app, fixture, harness_init, harness_with, save, select, settle, ActionRecorder, Fixture, H,
    WIDE,
};
use crate::app::{Location, Side};
use crate::dialogs::Dialog;

/// An SFTP server stand-in: an in-memory tree that is not local.
struct FakeSftp(MemFs);

impl Vfs for FakeSftp {
    fn label(&self) -> String {
        "deploy@web-01 (SFTP)".into()
    }
    fn protocol(&self) -> Protocol {
        Protocol::Sftp
    }
    fn home(&self) -> VfsResult<VfsPath> {
        Ok(VfsPath::new("/srv"))
    }
    fn list(&self, dir: &VfsPath) -> VfsResult<Vec<Entry>> {
        self.0.list(dir)
    }
    fn stat(&self, path: &VfsPath) -> VfsResult<Entry> {
        self.0.stat(path)
    }
    fn create_dir(&self, path: &VfsPath) -> VfsResult<()> {
        self.0.create_dir(path)
    }
    fn remove_file(&self, path: &VfsPath) -> VfsResult<()> {
        self.0.remove_file(path)
    }
    fn remove_dir(&self, path: &VfsPath) -> VfsResult<()> {
        self.0.remove_dir(path)
    }
    fn rename(&self, from: &VfsPath, to: &VfsPath) -> VfsResult<()> {
        self.0.rename(from, to)
    }
    fn open_read(&self, path: &VfsPath) -> VfsResult<Box<dyn Read + Send>> {
        self.0.open_read(path)
    }
    fn open_write(&self, path: &VfsPath) -> VfsResult<Box<dyn Write + Send>> {
        self.0.open_write(path)
    }
}

fn fake_trash(fx: &Fixture) -> std::path::PathBuf {
    fx.root.parent().unwrap().join("Trash")
}

fn press(h: &mut H, mods: Modifiers, k: Key) {
    h.key_press_modifiers(mods, k);
    h.run_steps(2);
}

/// Step until `f` holds (worker threads finish in the background).
fn wait_for(h: &mut H, what: &str, mut f: impl FnMut(&mut H) -> bool) {
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

/// Show the left pane on the fake server, with a file and a folder.
pub(super) fn open_server(h: &mut H) -> MemFs {
    let mem = MemFs::new();
    mem.mkdir_p("/srv/www");
    mem.put("/srv/old.log", b"x", chrono::Utc::now());
    mem.put("/srv/www/index.html", b"<h1>", chrono::Utc::now());
    let fs: SharedVfs = Arc::new(FakeSftp(mem.clone()));
    let loc = Location {
        fs,
        conn: None,
        path: VfsPath::new("/srv"),
        archive: None,
    };
    app(h).navigate(Side::Left, loc, false);
    settle(h);
    mem
}

fn delete_form(h: &mut H) -> Option<(bool, usize)> {
    match &app(h).dialog {
        Some(Dialog::Delete(f)) => Some((f.no_trash, f.paths.len())),
        _ => None,
    }
}

#[test]
fn cmd_backspace_moves_local_items_to_the_trash() {
    let fx = fixture();
    let log = Arc::new(std::sync::Mutex::new(Vec::new()));
    let rec = log.clone();
    let mut h = harness_init(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(move |ctx: &egui::Context| {
            PluginHost::new(ctx.clone(), vec![Box::new(ActionRecorder(rec))], None)
        }),
        |_| {},
    );
    select(&mut h, Side::Left, &["README.md", "src"]);
    press(&mut h, Modifiers::COMMAND, Key::Backspace);
    assert!(app(&mut h).dialog.is_none(), "no confirmation, like Finder");
    // Both, whichever is moved last.
    wait_for(&mut h, "items in the Trash", |_| {
        fake_trash(&fx).join("src").exists() && fake_trash(&fx).join("README.md").exists()
    });
    assert!(
        !fx.root.join("README.md").exists() && !fx.root.join("src").exists(),
        "left the folder"
    );
    assert!(fake_trash(&fx).join("README.md").exists());
    // The pane refreshes, and plugins (the terminal echo) see a Trash action.
    wait_for(&mut h, "finished action", |_| {
        log.lock().unwrap().len() >= 2
    });
    // The pane reloads after the job; under load that takes a few frames.
    wait_for(&mut h, "the pane refreshed", |h| {
        !app(h)
            .pane(Side::Left)
            .tab()
            .entries
            .iter()
            .any(|e| e.name == "README.md")
    });
    let events = log.lock().unwrap().clone();
    assert_eq!(events[0].kind, FileActionKind::Trash);
    assert!(events[0].src_local);
    assert!(
        matches!(
            events[1].phase,
            FileActionPhase::Finished {
                ok: true,
                files: 2,
                ..
            }
        ),
        "{events:?}"
    );

    // The forward-delete key does the same.
    select(&mut h, Side::Left, &["report-q3.pdf"]);
    press(&mut h, Modifiers::NONE, Key::Delete);
    assert!(app(&mut h).dialog.is_none());
    wait_for(&mut h, "pdf in the Trash", |_| {
        fake_trash(&fx).join("report-q3.pdf").exists()
    });
}

#[test]
fn opt_cmd_backspace_deletes_permanently_after_asking() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    select(&mut h, Side::Left, &["README.md"]);
    press(&mut h, Modifiers::COMMAND | Modifiers::ALT, Key::Backspace);
    assert_eq!(
        delete_form(&mut h),
        Some((false, 1)),
        "asks; local, so no 'no Trash' note"
    );
    assert!(
        fx.root.join("README.md").exists(),
        "nothing happens before confirming"
    );
    app(&mut h).submit_dialog();
    wait_for(&mut h, "delete", |_| !fx.root.join("README.md").exists());
    assert!(
        !fake_trash(&fx).join("README.md").exists(),
        "not in the Trash"
    );

    // Cancel keeps the item.
    select(&mut h, Side::Left, &["report-q3.pdf"]);
    press(&mut h, Modifiers::COMMAND | Modifiers::ALT, Key::Backspace);
    assert!(delete_form(&mut h).is_some());
    press(&mut h, Modifiers::NONE, Key::Escape);
    assert!(app(&mut h).dialog.is_none());
    assert!(fx.root.join("report-q3.pdf").exists());
}

#[test]
fn server_items_are_deleted_permanently_after_asking() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let mem = open_server(&mut h);
    select(&mut h, Side::Left, &["old.log", "www"]);
    // ⌘⌫ can't use a Trash here: it asks and says so.
    press(&mut h, Modifiers::COMMAND, Key::Backspace);
    assert_eq!(delete_form(&mut h), Some((true, 2)));
    assert!(mem.exists("/srv/old.log"));
    app(&mut h).submit_dialog();
    wait_for(&mut h, "remote delete", |_| {
        !mem.exists("/srv/old.log") && !mem.exists("/srv/www")
    });
    assert!(
        std::fs::read_dir(fake_trash(&fx)).unwrap().next().is_none(),
        "the local Trash is untouched"
    );

    // ⌥⌘⌫ on a server asks the same way.
    mem.put("/srv/new.txt", b"y", chrono::Utc::now());
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, crate::app::Action::Refresh(Side::Left));
    settle(&mut h);
    select(&mut h, Side::Left, &["new.txt"]);
    press(&mut h, Modifiers::COMMAND | Modifiers::ALT, Key::Backspace);
    assert_eq!(delete_form(&mut h), Some((true, 1)));
}

#[test]
fn finder_shortcuts_for_tabs_transfers_and_quick_look() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let tabs = app(&mut h).pane(Side::Left).tabs.len();
    press(&mut h, Modifiers::COMMAND, Key::T);
    settle(&mut h);
    assert_eq!(
        app(&mut h).pane(Side::Left).tabs.len(),
        tabs + 1,
        "⌘T opens a tab"
    );
    assert!(
        !app(&mut h).show_transfers,
        "⌘T no longer toggles Transfers"
    );
    press(&mut h, Modifiers::COMMAND | Modifiers::ALT, Key::L);
    assert!(app(&mut h).show_transfers, "⌥⌘L shows Transfers");
    assert!(
        app(&mut h).pane(Side::Left).tab().path_edit.is_none(),
        "⌥⌘L doesn't edit the path (⌘L)"
    );
    press(&mut h, Modifiers::COMMAND | Modifiers::ALT, Key::L);
    assert!(!app(&mut h).show_transfers);
    press(&mut h, Modifiers::COMMAND, Key::L);
    assert!(
        app(&mut h).pane(Side::Left).tab().path_edit.is_some(),
        "⌘L still edits the path"
    );
}

fn notice_harness(
    fx: &Fixture,
    existing: bool,
    prefs: crate::prefs::Prefs,
    gpu: bool,
    theme: Theme,
) -> H {
    harness_init(
        fx,
        WIDE,
        theme,
        gpu,
        Box::new(|ctx: &egui::Context| PluginHost::new(ctx.clone(), Vec::new(), None)),
        move |init| {
            init.existing_install = Some(existing);
            init.prefs = Some(prefs);
        },
    )
}

#[test]
fn whats_new_shows_once_after_an_update() {
    let fx = fixture();
    let mut h = notice_harness(&fx, true, Default::default(), false, Theme::Light);
    assert!(
        matches!(app(&mut h).dialog, Some(Dialog::WhatsNew(_))),
        "shown after updating"
    );
    // Shortcuts are off while it shows.
    let tabs = app(&mut h).pane(Side::Left).tabs.len();
    press(&mut h, Modifiers::COMMAND, Key::T);
    assert_eq!(app(&mut h).pane(Side::Left).tabs.len(), tabs);
    press(&mut h, Modifiers::NONE, Key::Enter);
    assert!(app(&mut h).dialog.is_none(), "Got It closes it");
    let prefs = app(&mut h).prefs.clone();
    assert_eq!(prefs.whats_new_seen, crate::app::WHATS_NEW);

    // Next launch: not again.
    let mut h = notice_harness(&fx, true, prefs, false, Theme::Light);
    assert!(app(&mut h).dialog.is_none());

    // Esc counts as seen too.
    let mut h = notice_harness(&fx, true, Default::default(), false, Theme::Light);
    press(&mut h, Modifiers::NONE, Key::Escape);
    assert!(app(&mut h).dialog.is_none());
    assert_eq!(app(&mut h).prefs.whats_new_seen, crate::app::WHATS_NEW);

    // A new install never sees it.
    let mut h = notice_harness(&fx, false, Default::default(), false, Theme::Light);
    assert!(app(&mut h).dialog.is_none());
    assert_eq!(app(&mut h).prefs.whats_new_seen, crate::app::WHATS_NEW);

    // Coming from 0.2.x (saw notice 1): the new notice shows once.
    assert_eq!(crate::app::WHATS_NEW, 2);
    let old = crate::prefs::Prefs {
        whats_new_seen: 1,
        ..Default::default()
    };
    let mut h = notice_harness(&fx, true, old, false, Theme::Light);
    assert!(
        matches!(app(&mut h).dialog, Some(Dialog::WhatsNew(_))),
        "shown after updating from 0.2.x"
    );
    assert!(h.query_by_label("Dragging now copies").is_some());
    press(&mut h, Modifiers::NONE, Key::Enter);
    let prefs = app(&mut h).prefs.clone();
    assert_eq!(prefs.whats_new_seen, 2);
    let mut h = notice_harness(&fx, true, prefs, false, Theme::Light);
    assert!(app(&mut h).dialog.is_none(), "only once");
}

/// German renders (light and dark) of both delete confirmations and the
/// notice, into `MANTA_RENDER_DIR`.
#[test]
fn render_delete_and_whats_new_de() {
    if !super::wgpu_available() {
        return;
    }
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let fx = fixture();
        let mut h = notice_harness(&fx, true, Default::default(), true, theme);
        crate::i18n::set_locale(crate::i18n::Locale::De);
        h.run_steps(3);
        save(&mut h, &format!("whats_new_de_{tname}"));
        press(&mut h, Modifiers::NONE, Key::Enter);

        select(&mut h, Side::Left, &["README.md", "src", "report-q3.pdf"]);
        press(&mut h, Modifiers::COMMAND | Modifiers::ALT, Key::Backspace);
        assert!(delete_form(&mut h).is_some());
        save(&mut h, &format!("delete_local_permanent_de_{tname}"));
        press(&mut h, Modifiers::NONE, Key::Escape);

        open_server(&mut h);
        select(&mut h, Side::Left, &["old.log"]);
        press(&mut h, Modifiers::COMMAND, Key::Backspace);
        assert_eq!(delete_form(&mut h).map(|f| f.0), Some(true));
        save(&mut h, &format!("delete_remote_de_{tname}"));
        press(&mut h, Modifiers::NONE, Key::Escape);
        crate::i18n::set_locale(crate::i18n::Locale::En);
    }
}

// ------------------------------------------------------------ sidebar Trash

/// A harness whose fake Trash the test keeps (to look at its calls).
fn trash_harness(fx: &Fixture, theme: Theme, gpu: bool) -> (H, Arc<crate::trash::FakeTrash>) {
    let fake = crate::trash::FakeTrash::new(&fake_trash(fx));
    let t = fake.clone();
    let h = harness_init(
        fx,
        WIDE,
        theme,
        gpu,
        Box::new(|ctx: &egui::Context| PluginHost::new(ctx.clone(), Vec::new(), None)),
        move |init| init.trasher = Some(t),
    );
    (h, fake)
}

fn trash_row(h: &mut H) -> egui::Rect {
    app(h).trash_rect.expect("the Trash row is drawn")
}

fn click_at(h: &mut H, pos: egui::Pos2) {
    h.hover_at(pos);
    h.step();
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
}

/// Where a drag starts: empty space in the left pane, below its rows.
fn drag_start(h: &mut H) -> egui::Pos2 {
    let rows = app(h).pane(Side::Left).tab().view.len();
    egui::pos2(330.0, (120.0 + 24.0 * (rows + 2) as f32).min(600.0))
}

/// Hover `pos` with `payload` in flight (set again each frame: egui clears
/// it on a press), then release there when `release`. Like a real drag the
/// button goes down elsewhere (in the pane), not on the target: a press
/// and release on the target would be a click (the Trash row opens the
/// Trash in the pane).
fn drag_to<P: std::any::Any + Send + Sync>(
    h: &mut H,
    pos: egui::Pos2,
    release: bool,
    make: impl Fn(&mut H) -> P,
) {
    let ctx = h.ctx.clone();
    if !ctx.input(|i| i.pointer.primary_down()) {
        let from = drag_start(h);
        h.hover_at(from);
        h.event(egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.step();
    }
    let p = make(h);
    egui::DragAndDrop::set_payload(&ctx, p);
    h.hover_at(pos);
    h.step();
    let p = make(h);
    egui::DragAndDrop::set_payload(&ctx, p);
    h.hover_at(pos);
    h.step();
    if release {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run_steps(3);
    }
}

fn local_payload(h: &mut H, fx: &Fixture, names: &[&str]) -> crate::pane::DragPayload {
    let fs = app(h).pane(Side::Left).tab().loc.fs.clone();
    let paths = names
        .iter()
        .map(|n| VfsPath::new(&fx.root.join(n).to_string_lossy()))
        .collect();
    crate::pane::DragPayload::files(Side::Left, fs, paths)
}

fn trash_items(h: &mut H) -> Option<usize> {
    app(h).trash.stale();
    h.step();
    app(h).trash.status.items
}

#[test]
fn the_sidebar_shows_the_trash_and_opens_it() {
    let fx = fixture();
    let (mut h, fake) = trash_harness(&fx, Theme::Light, false);
    assert_eq!(
        app(&mut h).trash.dir.as_deref(),
        Some(fake.dir.as_path()),
        "the fake Trash, never ~/.Trash"
    );
    assert_eq!(trash_items(&mut h), Some(0), "empty");
    std::fs::write(fake.dir.join("old.txt"), "x").unwrap();
    std::fs::write(fake.dir.join(".DS_Store"), "x").unwrap();
    assert_eq!(
        trash_items(&mut h),
        Some(1),
        "full (Finder's .DS_Store doesn't count)"
    );

    let pos = trash_row(&mut h).center();
    click_at(&mut h, pos);
    settle(&mut h);
    let tab = app(&mut h).pane(Side::Left).tab();
    assert_eq!(
        tab.loc.path.to_string(),
        fake.dir.to_string_lossy(),
        "opens in the active pane"
    );
    assert!(tab.entries.iter().any(|e| e.name == "old.txt"));

    // Inside the Trash, ⌘⌫ deletes immediately after asking (Finder).
    select(&mut h, Side::Left, &["old.txt"]);
    press(&mut h, Modifiers::COMMAND, Key::Backspace);
    match &app(&mut h).dialog {
        Some(Dialog::Delete(f)) => assert!(f.in_trash && !f.no_trash),
        _ => panic!("asks before deleting from the Trash"),
    }
    app(&mut h).submit_dialog();
    wait_for(&mut h, "deleted from the Trash", |_| {
        !fake.dir.join("old.txt").exists()
    });
    assert!(fake.calls.lock().unwrap().is_empty(), "not trashed again");
}

#[test]
fn dropping_local_items_on_the_trash_moves_them_there() {
    let fx = fixture();
    let log = Arc::new(std::sync::Mutex::new(Vec::new()));
    let rec = log.clone();
    let mut h = harness_init(
        &fx,
        WIDE,
        Theme::Light,
        false,
        Box::new(move |ctx: &egui::Context| {
            PluginHost::new(ctx.clone(), vec![Box::new(ActionRecorder(rec))], None)
        }),
        |_| {},
    );
    let pos = trash_row(&mut h).center();
    // While hovering: the badge says what happens.
    drag_to(&mut h, pos, false, |h| {
        local_payload(h, &fx, &["README.md", "notes.txt"])
    });
    assert_eq!(
        crate::sidebar::trash_drop_hover(&h.ctx),
        Some(crate::sidebar::TrashHover::Trash(2))
    );
    drag_to(&mut h, pos, true, |h| {
        local_payload(h, &fx, &["README.md", "notes.txt"])
    });
    assert!(app(&mut h).dialog.is_none(), "no question, like ⌘⌫");
    assert_eq!(
        app(&mut h).pane(Side::Left).tab().loc.path.as_str(),
        fx.root.to_string_lossy(),
        "a drop is not a click: the pane stays in its folder"
    );
    wait_for(&mut h, "items in the Trash", |_| {
        fake_trash(&fx).join("notes.txt").exists()
    });
    assert!(fake_trash(&fx).join("README.md").exists() && !fx.root.join("README.md").exists());
    wait_for(&mut h, "finished action", |_| {
        log.lock().unwrap().len() >= 2
    });
    settle(&mut h);
    assert!(
        app(&mut h)
            .toasts
            .titles()
            .iter()
            .any(|t| t.contains("Trash")),
        "{:?}",
        app(&mut h).toasts.titles()
    );
    assert_eq!(
        log.lock().unwrap()[0].kind,
        FileActionKind::Trash,
        "the terminal echo sees a Trash action"
    );
    // The pane reloads after the job; under load that can take a few frames.
    wait_for(&mut h, "the pane refreshed", |h| {
        !app(h)
            .pane(Side::Left)
            .tab()
            .entries
            .iter()
            .any(|e| e.name == "README.md")
    });
    assert_eq!(trash_items(&mut h), Some(2));
}

#[test]
fn dropping_server_items_on_the_trash_asks_to_delete_permanently() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let mem = open_server(&mut h);
    let fs = app(&mut h).pane(Side::Left).tab().loc.fs.clone();
    let payload = move |_: &mut H| {
        crate::pane::DragPayload::files(Side::Left, fs.clone(), vec![VfsPath::new("/srv/old.log")])
    };
    let pos = trash_row(&mut h).center();
    drag_to(&mut h, pos, false, &payload);
    assert_eq!(
        crate::sidebar::trash_drop_hover(&h.ctx),
        Some(crate::sidebar::TrashHover::Delete(1))
    );
    drag_to(&mut h, pos, true, &payload);
    assert_eq!(
        delete_form(&mut h),
        Some((true, 1)),
        "asks, and says there is no Trash"
    );
    assert!(mem.exists("/srv/old.log"));
    app(&mut h).submit_dialog();
    wait_for(&mut h, "remote delete", |_| !mem.exists("/srv/old.log"));
    assert!(
        std::fs::read_dir(fake_trash(&fx)).unwrap().next().is_none(),
        "the local Trash is untouched"
    );
}

#[test]
fn archive_items_are_refused_by_the_trash() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let pos = trash_row(&mut h).center();
    let make = |h: &mut H| {
        let mut p = local_payload(h, &fx, &["README.md"]);
        p.archive = true;
        p
    };
    drag_to(&mut h, pos, false, make);
    assert_eq!(
        crate::sidebar::trash_drop_hover(&h.ctx),
        Some(crate::sidebar::TrashHover::ReadOnly)
    );
    assert_eq!(
        h.output().platform_output.cursor_icon,
        egui::CursorIcon::NotAllowed
    );
    drag_to(&mut h, pos, true, make);
    h.run_steps(3);
    assert!(app(&mut h).dialog.is_none());
    assert!(fx.root.join("README.md").exists(), "nothing happened");
    assert!(std::fs::read_dir(fake_trash(&fx)).unwrap().next().is_none());
}

#[test]
fn files_from_other_apps_and_favorites_dropped_on_the_trash() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let outside = fx.root.parent().unwrap().join("Outside");
    std::fs::create_dir_all(&outside).unwrap();
    let f = outside.join("download.zip");
    std::fs::write(&f, "zip").unwrap();
    let pos = trash_row(&mut h).center();
    h.hover_at(pos);
    h.input_mut().hovered_files = vec![crate::native_drag::test_hooks::hovered(&f)];
    h.step();
    h.input_mut().hovered_files.clear();
    h.input_mut().dropped_files = vec![crate::native_drag::test_hooks::dropped(&f)];
    h.step();
    wait_for(&mut h, "external file in the Trash", |_| {
        fake_trash(&fx).join("download.zip").exists()
    });
    assert!(!f.exists());

    // A favorite dragged onto the Trash is removed (like Finder's sidebar).
    let before = app(&mut h).favorites.items.len();
    drag_to(&mut h, pos, true, |_| crate::sidebar::FavoriteDrag(0));
    assert_eq!(app(&mut h).favorites.items.len(), before - 1);
    assert!(fx.root.exists(), "only the favorite, not the folder");
}

#[test]
fn empty_trash_asks_with_count_and_size_then_deletes_everything() {
    let fx = fixture();
    let (mut h, fake) = trash_harness(&fx, Theme::Light, false);
    // Empty: nothing to ask.
    press(
        &mut h,
        Modifiers::COMMAND | Modifiers::SHIFT,
        Key::Backspace,
    );
    assert!(app(&mut h).dialog.is_none(), "an empty Trash isn't emptied");

    std::fs::create_dir_all(fake.dir.join("folder")).unwrap();
    std::fs::write(fake.dir.join("folder/a.bin"), vec![0u8; 3000]).unwrap();
    std::fs::write(fake.dir.join("b.txt"), "hello").unwrap();
    press(
        &mut h,
        Modifiers::COMMAND | Modifiers::SHIFT,
        Key::Backspace,
    );
    wait_for(
        &mut h,
        "size measured",
        |h| matches!(&app(h).dialog, Some(Dialog::EmptyTrash(f)) if !f.measuring),
    );
    match &app(&mut h).dialog {
        Some(Dialog::EmptyTrash(f)) => {
            assert_eq!((f.items, f.bytes, f.readable), (Some(2), Some(3005), true));
        }
        _ => unreachable!(),
    }
    assert!(
        fx.root.join("README.md").exists() && !fx.root.join("README.md").starts_with(&fake.dir)
    );
    // Cancel keeps everything.
    press(&mut h, Modifiers::NONE, Key::Escape);
    assert!(app(&mut h).dialog.is_none());
    assert!(fake.dir.join("b.txt").exists());

    // The palette has the command (⇧⌘⌫, like Finder).
    assert_eq!(crate::commands::Cmd::EmptyTrash.shortcut(), "⇧⌘⌫");
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, crate::app::Action::EmptyTrash);
    assert!(matches!(app(&mut h).dialog, Some(Dialog::EmptyTrash(_))));
    app(&mut h).submit_dialog();
    wait_for(&mut h, "emptied", |_| {
        !fake.dir.join("b.txt").exists() && !fake.dir.join("folder").exists()
    });
    h.run_steps(2);
    assert!(
        app(&mut h)
            .toasts
            .titles()
            .iter()
            .any(|t| t.contains("Emptied the Trash (2 items)")),
        "{:?}",
        app(&mut h).toasts.titles()
    );
    assert_eq!(trash_items(&mut h), Some(0));
    assert_eq!(
        fake.finder_empties
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

/// Without Full Disk Access (here: a folder Manta may not list) the Trash
/// shows what to do, and emptying goes through Finder.
#[cfg(unix)]
#[test]
fn trash_without_permission_explains_full_disk_access() {
    use std::os::unix::fs::PermissionsExt;
    let fx = fixture();
    let (mut h, fake) = trash_harness(&fx, Theme::Light, false);
    std::fs::write(fake.dir.join("x.txt"), "x").unwrap();
    std::fs::set_permissions(&fake.dir, std::fs::Permissions::from_mode(0o300)).unwrap();
    let restore =
        || std::fs::set_permissions(&fake.dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    let st = {
        app(&mut h).trash.stale();
        h.step();
        app(&mut h).trash.status
    };
    assert!(!st.readable);
    assert!(app(&mut h).trash_rect.is_some(), "the row stays");

    let pos = trash_row(&mut h).center();
    click_at(&mut h, pos);
    settle(&mut h);
    let ctx = h.ctx.clone();
    let tab = app(&mut h).pane(Side::Left).tab();
    let err = tab.error.clone().unwrap_or_default();
    assert!(crate::trash::is_permission_error(&err), "{err}");
    assert!(crate::app::shows_trash(&ctx, &tab.loc));

    app(&mut h).handle_action(&ctx, crate::app::Action::OpenFullDiskAccess);
    app(&mut h).handle_action(&ctx, crate::app::Action::ShowTrashInFinder);
    let launched = app(&mut h).launched.clone();
    assert_eq!(
        launched[0],
        vec![crate::app::trash_place::FULL_DISK_ACCESS_URL.to_string()]
    );
    assert_eq!(
        launched[1],
        vec![
            "-b".to_string(),
            "com.apple.finder".into(),
            fake.dir.to_string_lossy().into_owned()
        ],
        "Finder by id, never the default viewer"
    );

    // Emptying: Finder does it (Manta can't list the folder).
    press(
        &mut h,
        Modifiers::COMMAND | Modifiers::SHIFT,
        Key::Backspace,
    );
    let readable = match &app(&mut h).dialog {
        Some(Dialog::EmptyTrash(f)) => f.readable,
        _ => {
            restore();
            panic!("asks")
        }
    };
    assert!(!readable);
    app(&mut h).submit_dialog();
    wait_for(&mut h, "Finder asked", |_| {
        fake.finder_empties
            .load(std::sync::atomic::Ordering::SeqCst)
            == 1
    });
    restore();
    assert!(
        fake.dir.join("x.txt").exists(),
        "Manta itself deleted nothing"
    );
}

/// German renders (light and dark): the Trash row (full), the drop
/// highlight, "Empty Trash" and the Full Disk Access state, into
/// `$MANTA_RENDER_DIR/trash`.
#[test]
fn render_trash_de() {
    if !super::wgpu_available() {
        return;
    }
    let dir = super::render_dir("trash");
    for (theme, tname) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let fx = fixture();
        let (mut h, fake) = trash_harness(&fx, theme, true);
        crate::i18n::set_locale(crate::i18n::Locale::De);
        std::fs::write(fake.dir.join("Alte Notizen.txt"), vec![b'x'; 12_000]).unwrap();
        std::fs::write(fake.dir.join("Entwurf.pdf"), vec![b'x'; 1_200_000]).unwrap();
        trash_items(&mut h);
        h.run_steps(3);
        super::save_to(&mut h, &dir, &format!("trash_row_de_{tname}"));

        let pos = trash_row(&mut h).center();
        drag_to(&mut h, pos, false, |h| {
            local_payload(h, &fx, &["README.md", "notes.txt", "data.json"])
        });
        h.hover_at(pos);
        super::save_to(&mut h, &dir, &format!("trash_drop_de_{tname}"));
        let ctx = h.ctx.clone();
        egui::DragAndDrop::clear_payload(&ctx);
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run_steps(3);
        assert!(
            fx.root.join("data.json").exists(),
            "drag cancelled: nothing trashed"
        );

        press(
            &mut h,
            Modifiers::COMMAND | Modifiers::SHIFT,
            Key::Backspace,
        );
        wait_for(
            &mut h,
            "size",
            |h| matches!(&app(h).dialog, Some(Dialog::EmptyTrash(f)) if !f.measuring),
        );
        super::save_to(&mut h, &dir, &format!("trash_empty_dialog_de_{tname}"));
        press(&mut h, Modifiers::NONE, Key::Escape);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake.dir, std::fs::Permissions::from_mode(0o300)).unwrap();
            app(&mut h).trash.stale();
            let pos = trash_row(&mut h).center();
            click_at(&mut h, pos);
            settle(&mut h);
            super::save_to(&mut h, &dir, &format!("trash_full_disk_access_de_{tname}"));
            std::fs::set_permissions(&fake.dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        crate::i18n::set_locale(crate::i18n::Locale::En);
    }
}
