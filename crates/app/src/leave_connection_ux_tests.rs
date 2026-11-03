//! `exit` in the Terminal below a server or share: the pane goes back to
//! where it was on this Mac, and the connection closes unless another tab
//! still shows it.

use std::sync::Arc;

use manta_core::{Auth, ConnectionConfig, Protocol, SharedVfs, VfsPath};
use manta_vfs::LocalFs;

use super::*;
use crate::app::Action;
use crate::worker::ConnectRequest;

fn connect(h: &mut H, fx: &Fixture, side: Side) {
    let fs: SharedVfs = Arc::new(LocalFs::new());
    let action = app(h).actions.next_id();
    app(h).on_connected(
        ConnectRequest {
            config: ConnectionConfig {
                name: String::new(),
                protocol: Protocol::Smb,
                host: "nas.local".into(),
                port: 445,
                username: "anna".into(),
                auth: Auth::Password(String::new()),
                initial_path: None,
                s3: None,
                container: None,
                ftp: None,
                webdav: None,
                smb: None,
            },
            favorite: None,
            side,
            action,
            keychain: false,
        },
        Ok((fs, VfsPath::new(&fx.root.join("src").to_string_lossy()))),
    );
    settle(h);
}

fn local_path(fx: &Fixture) -> VfsPath {
    VfsPath::new(&fx.root.to_string_lossy())
}

#[test]
fn pane_goes_back_and_the_connection_closes() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    let before = app(&mut h).pane(Side::Left).tab().loc.path.clone();
    assert_eq!(before, local_path(&fx));
    connect(&mut h, &fx, Side::Left);
    assert!(app(&mut h).pane(Side::Left).tab().loc.conn.is_some());

    app(&mut h).leave_connection(Side::Left);
    settle(&mut h);
    let tab = app(&mut h).pane(Side::Left).tab();
    assert_eq!((tab.loc.conn, &tab.loc.path), (None, &before));
    assert!(app(&mut h).connections.is_empty(), "disconnected");
}

#[test]
fn connection_shown_in_another_tab_stays() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    connect(&mut h, &fx, Side::Left);
    let conn = app(&mut h).pane(Side::Left).tab().loc.conn;
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, Action::NewTab(Side::Left));
    settle(&mut h);
    assert_eq!(app(&mut h).pane(Side::Left).tab().loc.conn, conn);

    app(&mut h).leave_connection(Side::Left);
    settle(&mut h);
    assert_eq!(app(&mut h).pane(Side::Left).tab().loc.conn, None);
    assert_eq!(app(&mut h).connections.len(), 1, "the first tab shows it");
}
