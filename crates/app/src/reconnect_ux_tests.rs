//! A server session that died while Manta sat idle (Wi-Fi gone, laptop
//! asleep): the next listing logs in again instead of showing "The
//! connection to the server was lost" in that tab and every new one.

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use eframe::egui::Theme;
use manta_core::{
    Auth, ConnectionConfig, Entry, EntryKind, Protocol, SharedVfs, Vfs, VfsError, VfsPath,
    VfsResult,
};

use super::*;
use crate::app::Action;
use crate::worker::ConnectRequest;

/// An SFTP server whose session is dead until `reconnect` logs in again.
#[derive(Default)]
struct Flaky {
    dead: AtomicBool,
    /// The server can't be reached either (logging in again fails).
    down: AtomicBool,
    reconnects: AtomicUsize,
}

impl Flaky {
    fn alive(&self) -> VfsResult<()> {
        if self.dead.load(Ordering::SeqCst) {
            Err(VfsError::Connection(
                "/srv: Unable to send FXP_OPEN*".into(),
            ))
        } else {
            Ok(())
        }
    }
}

impl Vfs for Flaky {
    fn label(&self) -> String {
        "anna@files.example.org (SFTP)".into()
    }
    fn protocol(&self) -> Protocol {
        Protocol::Sftp
    }
    fn home(&self) -> VfsResult<VfsPath> {
        self.alive()?;
        Ok(VfsPath::new("/srv"))
    }
    fn list(&self, dir: &VfsPath) -> VfsResult<Vec<Entry>> {
        self.alive()?;
        Ok(vec![Entry {
            name: "index.html".into(),
            path: dir.join("index.html"),
            kind: EntryKind::File,
            size: 512,
            modified: None,
            permissions: Some(0o644),
            link_target: None,
            hidden: false,
        }])
    }
    fn stat(&self, path: &VfsPath) -> VfsResult<Entry> {
        self.alive()?;
        Err(VfsError::NotFound(path.to_string()))
    }
    fn create_dir(&self, _: &VfsPath) -> VfsResult<()> {
        Err(VfsError::Unsupported("mock".into()))
    }
    fn remove_file(&self, _: &VfsPath) -> VfsResult<()> {
        Err(VfsError::Unsupported("mock".into()))
    }
    fn remove_dir(&self, _: &VfsPath) -> VfsResult<()> {
        Err(VfsError::Unsupported("mock".into()))
    }
    fn rename(&self, _: &VfsPath, _: &VfsPath) -> VfsResult<()> {
        Err(VfsError::Unsupported("mock".into()))
    }
    fn open_read(&self, _: &VfsPath) -> VfsResult<Box<dyn Read + Send>> {
        Err(VfsError::Unsupported("mock".into()))
    }
    fn open_write(&self, _: &VfsPath) -> VfsResult<Box<dyn Write + Send>> {
        Err(VfsError::Unsupported("mock".into()))
    }
    fn reconnect(&self) -> VfsResult<()> {
        self.reconnects.fetch_add(1, Ordering::SeqCst);
        if self.down.load(Ordering::SeqCst) {
            return Err(VfsError::Connection(
                "cannot connect to files.example.org:22: timed out".into(),
            ));
        }
        self.dead.store(false, Ordering::SeqCst);
        Ok(())
    }
}

fn connect(h: &mut H, server: &Arc<Flaky>) {
    let fs: SharedVfs = server.clone();
    let action = app(h).actions.next_id();
    app(h).on_connected(
        ConnectRequest {
            config: ConnectionConfig {
                name: String::new(),
                protocol: Protocol::Sftp,
                host: "files.example.org".into(),
                port: 22,
                username: "anna".into(),
                auth: Auth::Password("pw".into()),
                initial_path: None,
                s3: None,
                container: None,
                ftp: None,
                webdav: None,
                smb: None,
            },
            favorite: None,
            side: Side::Left,
            action,
            keychain: false,
        },
        Ok((fs, VfsPath::new("/srv"))),
    );
    settle(h);
}

fn act(h: &mut H, action: Action) {
    let ctx = h.ctx.clone();
    app(h).handle_action(&ctx, action);
    settle(h);
}

fn shown(h: &mut H) -> (Vec<String>, Option<String>) {
    let tab = app(h).pane(Side::Left).tab();
    (
        tab.entries.iter().map(|e| e.name.clone()).collect(),
        tab.error.clone(),
    )
}

#[test]
fn dead_session_logs_in_again() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    let server = Arc::new(Flaky::default());
    connect(&mut h, &server);
    assert_eq!(shown(&mut h), (vec!["index.html".into()], None));

    // The session dies; reloading logs in again.
    server.dead.store(true, Ordering::SeqCst);
    act(&mut h, Action::Refresh(Side::Left));
    assert_eq!(shown(&mut h), (vec!["index.html".into()], None));
    assert_eq!(server.reconnects.load(Ordering::SeqCst), 1);

    // Died again: a new tab on the connection does the same.
    server.dead.store(true, Ordering::SeqCst);
    act(&mut h, Action::NewTab(Side::Left));
    assert_eq!(shown(&mut h), (vec!["index.html".into()], None));
    assert_eq!(server.reconnects.load(Ordering::SeqCst), 2);
}

#[test]
fn unreachable_server_still_says_so() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    settle(&mut h);
    let server = Arc::new(Flaky::default());
    connect(&mut h, &server);

    server.dead.store(true, Ordering::SeqCst);
    server.down.store(true, Ordering::SeqCst);
    act(&mut h, Action::Refresh(Side::Left));
    let (_, error) = shown(&mut h);
    let error = error.expect("listing error");
    assert_eq!(
        crate::pane::friendly_listing_error(&error).0,
        crate::i18n::tr("pane.err.connection")
    );

    // Back on the network: the next reload works.
    server.down.store(false, Ordering::SeqCst);
    act(&mut h, Action::Refresh(Side::Left));
    assert_eq!(shown(&mut h), (vec!["index.html".into()], None));
}
