//! The Terminal plugin with a pane that moves between a local folder and a
//! Docker container (real Docker, throwaway container with label
//! `manta-test`; the user's containers are never touched). Skips without
//! Docker or the SF fonts.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use manta_core::{Protocol, VfsPath};
use manta_plugin::{
    HostRequest, PaneId, PaneState, Plugin, PluginEvent, PluginHandle, RemoteEndpoint,
};
use manta_plugin_terminal::remote::find_program;
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

struct Ctr {
    name: String,
    docker: PathBuf,
}

impl Ctr {
    fn start() -> Option<Ctr> {
        if !containers_opted_in() {
            return None;
        }
        let docker = find_program("docker")?;
        let ok = Command::new(&docker)
            .args(["info", "--format", "{{.ServerVersion}}"])
            .output()
            .ok()?
            .status
            .success();
        if !ok {
            return None;
        }
        let name = format!("manta-test-plugin-{}", std::process::id());
        let out = Command::new(&docker)
            .args([
                "run",
                "-d",
                "--rm",
                "--label",
                "manta-test",
                "--name",
                &name,
                "alpine:3",
                "sleep",
                "300",
            ])
            .output()
            .ok()?;
        out.status.success().then_some(Ctr { name, docker })
    }
}

impl Drop for Ctr {
    fn drop(&mut self) {
        let _ = Command::new(&self.docker)
            .args(["rm", "-f", &self.name])
            .output();
    }
}

fn local(dir: &Path) -> PaneState {
    PaneState {
        pane: PaneId::Left,
        path: VfsPath::new(&dir.to_string_lossy()),
        local_path: Some(dir.to_path_buf()),
        protocol: Protocol::Local,
        backend_label: "Local".into(),
        selection: vec![],
        active: true,
        remote: None,
    }
}

fn container(name: &str, path: &str) -> PaneState {
    PaneState {
        pane: PaneId::Left,
        path: VfsPath::new(path),
        local_path: None,
        protocol: Protocol::Docker,
        backend_label: format!("{name} (Docker)"),
        selection: vec![],
        active: true,
        remote: Some(RemoteEndpoint {
            protocol: Protocol::Docker,
            user: String::new(),
            host: name.into(),
            port: 0,
            context: String::new(),
        }),
    }
}

type Shared = Rc<RefCell<(TerminalPlugin, PaneState)>>;

fn pump(
    h: &mut Harness<'_>,
    st: &Shared,
    limit: Duration,
    cond: impl Fn(&TerminalPlugin) -> bool,
) -> bool {
    let t = Instant::now();
    while t.elapsed() < limit {
        h.step();
        if cond(&st.borrow().0) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

fn set_pane(st: &Shared, p: PaneState) {
    st.borrow_mut().1 = p.clone();
    st.borrow_mut()
        .0
        .on_event(&PluginEvent::DirectoryChanged(p));
}

#[test]
fn local_container_local() {
    if !Path::new(SF_PRO).exists()
        || !Path::new(SF_MONO).exists()
        || !Path::new("/bin/zsh").exists()
    {
        return;
    }
    let Some(ctr) = Ctr::start() else {
        eprintln!("skipping: no docker");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let a = std::fs::canonicalize(tmp.path()).unwrap().join("a");
    let b = a.parent().unwrap().join("b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    let data = tmp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<(String, HostRequest)>();
    let st: Shared = Rc::new(RefCell::new((TerminalPlugin::new(), local(&a))));
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
    let cwd = |p: &TerminalPlugin| p.debug_shell_cwd(left);

    // 1. Local shell in a/.
    let a_s = a.to_string_lossy().into_owned();
    assert!(pump(&mut h, &st, Duration::from_secs(20), |p| p
        .debug_is_idle(left)
        && cwd(p).is_some()));
    assert_eq!(
        st.borrow().0.debug_sessions(left).0.as_deref(),
        Some("local")
    );

    // 2. The pane opens the container at /etc: a shell inside it, there.
    set_pane(&st, container(&ctr.name, "/etc"));
    assert!(
        pump(&mut h, &st, Duration::from_secs(20), |p| cwd(p).as_deref()
            == Some("/etc")
            && p.debug_is_idle(left)),
        "container shell:\n{:?}",
        st.borrow().0.debug_screen(left)
    );
    let (key, parked) = st.borrow().0.debug_sessions(left);
    assert_eq!(key, Some(format!("docker:{}", ctr.name)));
    assert_eq!(
        parked,
        vec!["local".to_string()],
        "local shell parked, not killed"
    );

    // 3. Pane → terminal inside the container.
    set_pane(&st, container(&ctr.name, "/usr/share"));
    assert!(pump(&mut h, &st, Duration::from_secs(8), |p| cwd(p)
        .as_deref()
        == Some("/usr/share")));

    // 4. Terminal → pane: a cd in the container sends NavigateIn for it.
    assert!(pump(&mut h, &st, Duration::from_secs(5), |p| p
        .debug_is_idle(left)));
    while rx.try_recv().is_ok() {}
    st.borrow().0.debug_write(left, b"cd /tmp\r");
    let nav = wait_nav(&mut h, &st, &rx);
    match nav {
        Some(HostRequest::NavigateIn {
            pane,
            remote: Some(r),
            path,
        }) => {
            assert_eq!(pane, left);
            assert_eq!(r.host, ctr.name);
            assert_eq!(path, VfsPath::new("/tmp"));
        }
        other => panic!("expected NavigateIn, got {other:?}"),
    }

    // 5. Back to local (b/): the local shell comes back at once and follows.
    set_pane(&st, local(&b));
    h.step();
    let (key, parked) = st.borrow().0.debug_sessions(left);
    assert_eq!(
        key.as_deref(),
        Some("local"),
        "local session restored immediately"
    );
    assert_eq!(parked, vec![format!("docker:{}", ctr.name)]);
    let b_s = b.to_string_lossy().into_owned();
    assert!(
        pump(&mut h, &st, Duration::from_secs(8), |p| cwd(p).as_deref()
            == Some(b_s.as_str())),
        "local follow after the container: {:?} (was {a_s})",
        cwd(&st.borrow().0)
    );
    // And a local cd still navigates with a plain Navigate.
    assert!(pump(&mut h, &st, Duration::from_secs(5), |p| p
        .debug_is_idle(left)));
    while rx.try_recv().is_ok() {}
    st.borrow()
        .0
        .debug_write(left, format!("cd '{a_s}'\r").as_bytes());
    match wait_nav(&mut h, &st, &rx) {
        Some(HostRequest::Navigate { path, .. }) => assert_eq!(path, VfsPath::new(&a_s)),
        other => panic!("expected Navigate, got {other:?}"),
    }
    st.borrow_mut().0.disable();
}

fn wait_nav(
    h: &mut Harness<'_>,
    st: &Shared,
    rx: &Receiver<(String, HostRequest)>,
) -> Option<HostRequest> {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(8) {
        h.step();
        let _ = st;
        while let Ok((_, r)) = rx.try_recv() {
            if matches!(
                r,
                HostRequest::Navigate { .. } | HostRequest::NavigateIn { .. }
            ) {
                return Some(r);
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    None
}

/// Real `docker` / `kubectl` only with `MANTA_TEST_CONTAINERS=1`.
fn containers_opted_in() -> bool {
    let ok = std::env::var_os("MANTA_TEST_CONTAINERS").is_some_and(|v| v == "1");
    if !ok {
        eprintln!("skipped: set MANTA_TEST_CONTAINERS=1 to run tests against docker/kubectl");
    }
    ok
}

/// An `ssh` typed in the local shell: while it runs, a button offers its
/// server; a click asks the host to open it in the pane (no server needed:
/// the proxy command just waits).
#[test]
fn ssh_in_local_shell_offers_the_server() {
    use egui_kittest::kittest::Queryable;
    if !Path::new(SF_PRO).exists()
        || !Path::new(SF_MONO).exists()
        || !Path::new("/bin/zsh").exists()
        || find_program("ssh").is_none()
    {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let a = std::fs::canonicalize(tmp.path()).unwrap();
    let data = a.join("data");
    std::fs::create_dir_all(&data).unwrap();
    // An empty ZDOTDIR: not the developer's own zsh startup files (prompts
    // like oh-my-zsh's update question would eat the typed command).
    let zd = a.join("zdotdir");
    std::fs::create_dir_all(&zd).unwrap();
    std::env::set_var("ZDOTDIR", &zd);
    let (tx, rx) = std::sync::mpsc::channel::<(String, HostRequest)>();
    let st: Shared = Rc::new(RefCell::new((TerminalPlugin::new(), local(&a))));
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
    assert!(pump(&mut h, &st, Duration::from_secs(20), |p| p
        .debug_is_idle(left)));
    let label = "Open anna@203.0.113.9:2222 in the pane";
    assert!(h.query_by_label(label).is_none());

    st.borrow().0.debug_write(
        left,
        b"ssh -o ProxyCommand='sleep 60' -p 2222 anna@203.0.113.9\r",
    );
    let t = Instant::now();
    while h.query_by_label(label).is_none() && t.elapsed() < Duration::from_secs(20) {
        h.step();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        h.query_by_label(label).is_some(),
        "no button:\n{:?}",
        st.borrow().0.debug_screen(left)
    );
    h.get_by_label(label).click();
    h.run_steps(2);
    let sent: Vec<HostRequest> = rx.try_iter().map(|(_, r)| r).collect();
    assert!(
        sent.iter().any(|r| matches!(r,
            HostRequest::OpenSftp { pane, user, host, port }
                if *pane == left && user == "anna" && host == "203.0.113.9" && *port == 2222)),
        "{sent:?}"
    );
    // Offered once per ssh.
    h.run_steps(2);
    assert!(h.query_by_label(label).is_none());

    // ssh ends: back at the prompt, nothing offered.
    st.borrow().0.debug_write(left, b"\x03");
    assert!(pump(&mut h, &st, Duration::from_secs(20), |p| p
        .debug_is_idle(left)));
}
