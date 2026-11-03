//! `exit` in the Terminal below a server or share: the plugin asks the host
//! to leave the connection in the pane. A shell that ends otherwise (killed,
//! a dropped `ssh`) or a local pane doesn't. Real zsh; skips without it or
//! the SF fonts.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use manta_core::{Protocol, VfsPath};
use manta_plugin::{HostRequest, PaneId, PaneState, Plugin, PluginHandle, RemoteEndpoint};
use manta_plugin_terminal::TerminalPlugin;

const SF_PRO: &str = "/System/Library/Fonts/SFNS.ttf";
const SF_MONO: &str = "/System/Library/Fonts/SFNSMono.ttf";

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::empty();
    for (name, path, family) in [
        ("sf", SF_PRO, egui::FontFamily::Proportional),
        ("sf-mono", SF_MONO, egui::FontFamily::Monospace),
    ] {
        let bytes = std::fs::read(path).unwrap();
        fonts.font_data.insert(
            name.into(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts.families.entry(family).or_default().push(name.into());
    }
    ctx.set_fonts(fonts);
}

fn smb() -> RemoteEndpoint {
    RemoteEndpoint {
        protocol: Protocol::Smb,
        user: "anna".into(),
        host: "nas.local".into(),
        port: 445,
        context: String::new(),
    }
}

/// A pane in `dir` (an SMB share is mounted locally: it has both a local
/// path and the server).
fn pane(dir: &Path, remote: Option<RemoteEndpoint>) -> PaneState {
    PaneState {
        pane: PaneId::Left,
        path: VfsPath::new(&dir.to_string_lossy()),
        local_path: Some(dir.to_path_buf()),
        protocol: if remote.is_some() {
            Protocol::Smb
        } else {
            Protocol::Local
        },
        backend_label: "Local".into(),
        selection: vec![],
        active: true,
        remote,
    }
}

/// Start zsh below `pane`, type `line` at its prompt and collect what the
/// plugin asked the host for until the shell is gone (and a bit after).
fn run(dir: &Path, data: PathBuf, state: PaneState, line: &[u8]) -> Vec<HostRequest> {
    let (tx, rx): (_, Receiver<(String, HostRequest)>) = std::sync::mpsc::channel();
    type Shared = Rc<RefCell<(TerminalPlugin, PaneState)>>;
    let st: Shared = Rc::new(RefCell::new((TerminalPlugin::new(), state)));
    let s2 = st.clone();
    let mut tx = Some(tx);
    let mut h = Harness::builder()
        .with_size(egui::vec2(800.0, 300.0))
        .with_context_setup(install_fonts)
        .build_ui(move |ui| {
            let mut g = s2.borrow_mut();
            let (plugin, pane) = &mut *g;
            if let Some(tx) = tx.take() {
                plugin.enable(PluginHandle::new(
                    "manta.terminal".into(),
                    tx,
                    ui.ctx().clone(),
                    data.clone(),
                ));
                plugin.settings_mut().shell = "/bin/zsh".into();
            }
            let pane = pane.clone();
            plugin.panel_ui("terminal", &pane, ui);
        });
    let left = PaneId::Left;
    let t = Instant::now();
    while !st.borrow().0.debug_is_idle(left) {
        assert!(
            t.elapsed() < Duration::from_secs(20),
            "no prompt in {dir:?}"
        );
        h.step();
        std::thread::sleep(Duration::from_millis(20));
    }
    st.borrow().0.debug_write(left, line);
    let mut sent = Vec::new();
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(3) {
        h.step();
        sent.extend(rx.try_iter().map(|(_, r)| r));
        std::thread::sleep(Duration::from_millis(20));
    }
    sent
}

fn left(sent: &[HostRequest]) -> bool {
    sent.iter().any(|r| {
        matches!(r, HostRequest::LeaveConnection { pane, remote }
            if *pane == PaneId::Left && *remote == smb())
    })
}

#[test]
fn exit_leaves_the_share_in_the_pane() {
    if !Path::new(SF_PRO).exists()
        || !Path::new(SF_MONO).exists()
        || !Path::new("/bin/zsh").exists()
    {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let a = std::fs::canonicalize(tmp.path()).unwrap();
    let data = a.join("data");
    std::fs::create_dir_all(&data).unwrap();
    // Not the developer's own zsh startup files.
    let zd = a.join("zdotdir");
    std::fs::create_dir_all(&zd).unwrap();
    std::env::set_var("ZDOTDIR", &zd);

    let sent = run(&a, data.clone(), pane(&a, Some(smb())), b"exit\r");
    assert!(left(&sent), "exit: {sent:?}");
    // Also after a command that failed (exit keeps its status).
    let sent = run(&a, data.clone(), pane(&a, Some(smb())), b"false; exit\r");
    assert!(left(&sent), "false; exit: {sent:?}");

    // Ended otherwise: the pane stays.
    let sent = run(&a, data.clone(), pane(&a, Some(smb())), b"exit 255\r");
    assert!(!left(&sent), "exit 255 (a dropped ssh): {sent:?}");
    let sent = run(&a, data.clone(), pane(&a, Some(smb())), b"kill -9 $$\r");
    assert!(!left(&sent), "killed: {sent:?}");

    // A local pane has nothing to leave.
    let sent = run(&a, data, pane(&a, None), b"exit\r");
    assert!(
        !sent
            .iter()
            .any(|r| matches!(r, HostRequest::LeaveConnection { .. })),
        "{sent:?}"
    );
}
