//! "Open user@host in the pane" from the Terminal (an `ssh` typed in a
//! local shell): an open connection to that server is reused, else a
//! favorite for it, else the Connect dialog opens filled in.

use std::sync::Arc;

use manta_core::{Auth, ConnectionConfig, Protocol, SharedVfs, VfsPath};
use manta_vfs::LocalFs;

use super::*;
use crate::dialogs::AuthKind;
use crate::favorites::Favorite;
use crate::worker::ConnectRequest;

fn sftp(host: &str, user: &str, port: u16) -> ConnectionConfig {
    ConnectionConfig {
        name: String::new(),
        protocol: Protocol::Sftp,
        host: host.into(),
        port,
        username: user.into(),
        auth: Auth::Password(String::new()),
        initial_path: None,
        s3: None,
        container: None,
        ftp: None,
        webdav: None,
        smb: None,
    }
}

/// What the open Connect dialog shows.
#[derive(Debug)]
struct Form {
    side: Side,
    protocol: Protocol,
    name: String,
    host: String,
    user: String,
    port: String,
    auth_kind: AuthKind,
    editing_favorite: Option<usize>,
    error: Option<String>,
}

fn form(h: &mut H) -> Option<Form> {
    match &app(h).dialog {
        Some(Dialog::Connect(f)) => Some(Form {
            side: f.side,
            protocol: f.protocol,
            name: f.name.clone(),
            host: f.host.clone(),
            user: f.user.clone(),
            port: f.port.clone(),
            auth_kind: f.auth_kind,
            editing_favorite: f.editing_favorite,
            error: f.error.clone(),
        }),
        _ => None,
    }
}

#[test]
fn unknown_server_opens_the_connect_dialog_filled_in() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    app(&mut h).activate(Side::Left);
    app(&mut h).open_sftp(Side::Right, "anna".into(), "files.example.org".into(), 2222);
    settle(&mut h);
    let f = form(&mut h).expect("connect dialog");
    assert_eq!(f.side, Side::Right);
    assert_eq!(f.protocol, Protocol::Sftp);
    assert_eq!(
        (f.host.as_str(), f.user.as_str(), f.port.as_str()),
        ("files.example.org", "anna", "2222")
    );
    assert_eq!(f.auth_kind, AuthKind::Password);
    assert_eq!(
        app(&mut h).active,
        Side::Right,
        "the pane with the terminal"
    );
}

#[test]
fn open_connection_to_the_server_is_reused() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    let cfg = sftp("files.example.org", "anna", 22);
    let fs: SharedVfs = Arc::new(LocalFs::new());
    let action = app(&mut h).actions.next_id();
    app(&mut h).on_connected(
        ConnectRequest {
            config: cfg,
            favorite: None,
            side: Side::Right,
            action,
            keychain: false,
        },
        Ok((fs, VfsPath::new(&fx.root.to_string_lossy()))),
    );
    settle(&mut h);
    let conn = app(&mut h).pane(Side::Right).tab().loc.conn;
    assert!(conn.is_some());
    assert_eq!(app(&mut h).pane(Side::Left).tab().loc.conn, None);

    // Host names compare without case, like DNS.
    app(&mut h).open_sftp(Side::Left, "anna".into(), "Files.Example.org".into(), 22);
    settle(&mut h);
    assert!(form(&mut h).is_none(), "no dialog");
    assert_eq!(app(&mut h).pane(Side::Left).tab().loc.conn, conn);
    assert_eq!(app(&mut h).connections.len(), 1, "no second connection");
}

#[test]
fn favorite_for_the_server_is_used() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    let mut cfg = sftp("files.example.org", "anna", 22);
    cfg.name = "Files".into();
    app(&mut h).favorites.add(Favorite::Connection {
        config: cfg,
        keychain: false,
    });
    let i = app(&mut h).favorites.items.len() - 1;
    // Another user on the same server is a different favorite.
    app(&mut h).open_sftp(Side::Left, "ben".into(), "files.example.org".into(), 22);
    settle(&mut h);
    assert_eq!(form(&mut h).and_then(|f| f.editing_favorite), None);
    app(&mut h).dialog = None;

    // The favorite has no password saved: its form asks for it.
    app(&mut h).open_sftp(Side::Left, "anna".into(), "files.example.org".into(), 22);
    settle(&mut h);
    let f = form(&mut h).expect("password asked");
    assert_eq!(f.editing_favorite, Some(i));
    assert_eq!(f.name, "Files");
    assert!(f.error.is_some(), "asks for the password");
}
