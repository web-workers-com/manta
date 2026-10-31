//! Settings → "New tabs open in": ⌘T and the Dock menu's "New Tab" open
//! the chosen folder (like Finder's "New Finder windows show"); "Same
//! folder" copies the tab as before.

use super::*;
use crate::commands::Cmd;
use crate::prefs::NewTabFolder;
use egui::{Key, Modifiers};

fn local_path(fx: &Fixture, name: &str) -> VfsPath {
    VfsPath::new(&fx.root.join(name).to_string_lossy())
}

fn tabs(h: &mut H) -> (usize, VfsPath) {
    let side = app(h).active;
    let pane = app(h).pane(side);
    (pane.tabs.len(), pane.tab().loc.path.clone())
}

#[test]
fn new_tabs_open_in_the_folder_from_settings() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    app(&mut h).activate(Side::Left);
    app(&mut h).navigate_path(Side::Left, local_path(&fx, "src"));
    settle(&mut h);
    let (before, _) = tabs(&mut h);

    // Same folder (the default): ⌘T copies the tab.
    h.key_press_modifiers(Modifiers::COMMAND, Key::T);
    h.run_steps(2);
    settle(&mut h);
    assert_eq!(tabs(&mut h), (before + 1, local_path(&fx, "src")));

    // A chosen folder: ⌘T opens it.
    app(&mut h).prefs.new_tabs = NewTabFolder::Other(fx.root.join("Design"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::T);
    h.run_steps(2);
    settle(&mut h);
    assert_eq!(tabs(&mut h), (before + 2, local_path(&fx, "Design")));
    assert!(
        !app(&mut h).pane(Side::Left).tab().entries.is_empty(),
        "listed"
    );

    // The Dock menu's "New Tab" goes the same way as the menu bar.
    app(&mut h).navigate_path(Side::Left, local_path(&fx, "src"));
    settle(&mut h);
    let mut actions = Vec::new();
    app(&mut h).menu_commands(vec![Cmd::NewTab], &mut actions);
    let ctx = h.ctx.clone();
    for a in actions {
        app(&mut h).handle_action(&ctx, a);
    }
    h.run_steps(2);
    settle(&mut h);
    assert_eq!(tabs(&mut h), (before + 3, local_path(&fx, "Design")));

    // A folder that is gone: the tab is copied again.
    app(&mut h).prefs.new_tabs = NewTabFolder::Other(fx.root.join("gone"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::T);
    h.run_steps(2);
    settle(&mut h);
    assert_eq!(tabs(&mut h), (before + 4, local_path(&fx, "Design")));
}
