//! Spawns real shells through the session code and checks live follow.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use manta_plugin_terminal::palette::Theme;
use manta_plugin_terminal::session::{GridSize, Session, SpawnOptions};
use manta_plugin_terminal::shim::Integration;

fn canon(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap()
}

fn wait_until(
    s: &mut Session,
    limit: Duration,
    mut cond: impl FnMut(&mut Session) -> bool,
) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        s.tick(true);
        if cond(s) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

/// How long to wait for the shell before failing. Waits end as soon as
/// their condition holds; the limit only has to be far above what a loaded
/// CI runner needs.
const WAIT: Duration = Duration::from_secs(30);

fn shell_cwd(s: &Session) -> Option<PathBuf> {
    s.follow.shell_cwd.as_ref().map(|d| d.canon.clone())
}

fn follow_roundtrip(shell: &str) {
    if !Path::new(shell).exists() {
        eprintln!("skipping: {shell} not installed");
        return;
    }
    let a = tempfile::tempdir().unwrap();
    let b_root = tempfile::tempdir().unwrap();
    let b = b_root.path().join("it's a dir ü");
    std::fs::create_dir(&b).unwrap();
    let data = tempfile::tempdir().unwrap();
    // zsh: an empty ZDOTDIR, not the developer's own startup files.
    let zd = tempfile::tempdir().unwrap();

    let mut s = Session::spawn(SpawnOptions {
        shell: PathBuf::from(shell),
        cwd: a.path().to_path_buf(),
        size: GridSize {
            cols: 100,
            rows: 30,
        },
        cell_px: (8, 16),
        scrollback: 1000,
        shim_root: data.path().to_path_buf(),
        theme: Theme::dark(),
        waker: Arc::new(|| {}),
        spool: None,
        zdotdir: Some(zd.path().to_path_buf()),
        remote: None,
    })
    .expect("spawn");

    // First prompt.
    let a_canon = canon(a.path());
    assert!(
        wait_until(&mut s, WAIT, |s| shell_cwd(s).as_ref() == Some(&a_canon)
            && s.is_idle()),
        "{shell}: no initial OSC 7 / idle prompt. Screen:\n{}",
        s.lock().screen_text()
    );

    // Half-typed input must survive the follow.
    s.write(b"echo kee");
    assert!(wait_until(&mut s, WAIT, |s| s
        .lock()
        .screen_text()
        .contains("echo kee")));

    s.pane_changed(Some(&b));
    let b_canon = canon(&b);
    assert!(
        wait_until(&mut s, WAIT, |s| shell_cwd(s).as_ref() == Some(&b_canon)),
        "{shell}: no OSC 7 for the followed dir. Screen:\n{}",
        s.lock().screen_text()
    );
    assert!(!s.follow.has_pending());

    // Our cd's echo must not navigate the pane.
    assert_eq!(s.tick(true), None);

    // Finish the parked line: it must still read "echo kee".
    assert!(wait_until(&mut s, WAIT, |s| s.is_idle()));
    if s.integration() == Some(Integration::Push) {
        // The parked line is back at the new prompt before typing on.
        assert!(wait_until(&mut s, WAIT, |s| {
            let t = s.lock().screen_text();
            t.lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .is_some_and(|l| l.contains("echo kee"))
        }));
    }
    s.write(b"p_me\r");
    if s.integration() != Some(Integration::Push) {
        // bash 3.2 has no READLINE_LINE: the line is cleared (documented).
        eprintln!("{shell}: no push/restore support, input is not preserved");
        s.write(b"\x15");
    }
    let preserve = s.integration() == Some(Integration::Push);
    let ok = !preserve
        || wait_until(&mut s, WAIT, |s| {
            s.lock()
                .screen_text()
                .lines()
                .any(|l| l.trim() == "keep_me")
        });
    assert!(
        ok,
        "{shell}: typed input was lost. Screen:\n{}",
        s.lock().screen_text()
    );

    // Reverse direction: a cd typed in the terminal navigates the pane.
    let c = tempfile::tempdir().unwrap();
    let c_canon = canon(c.path());
    s.write(format!("cd '{}'\r", c.path().display()).as_bytes());
    let start = Instant::now();
    let mut nav = None;
    while nav.is_none() && start.elapsed() < WAIT {
        nav = s.tick(true);
        std::thread::sleep(Duration::from_millis(20));
    }
    let nav =
        nav.unwrap_or_else(|| panic!("{shell}: no Navigate. Screen:\n{}", s.lock().screen_text()));
    assert_eq!(canon(&nav), c_canon);

    let pid = s.pid().unwrap();
    let detached = s.shutdown(Duration::from_millis(500));
    assert_eq!(
        detached, 0,
        "{shell}: session threads still running after shutdown"
    );
    assert!(s.has_exited(), "{shell} still running after shutdown");
    // SAFETY: probing with signal 0.
    let alive = unsafe { libc::kill(pid, 0) } == 0;
    assert!(!alive, "{shell} pid {pid} not reaped");
}

#[test]
fn zsh_follows_pane() {
    follow_roundtrip("/bin/zsh");
}

#[test]
fn bash_follows_pane() {
    // Prefer a modern bash (push/restore keys) if installed.
    let shell = ["/opt/homebrew/bin/bash", "/usr/local/bin/bash", "/bin/bash"]
        .into_iter()
        .find(|p| Path::new(p).exists())
        .unwrap_or("/bin/bash");
    follow_roundtrip(shell);
}

#[test]
fn bash3_follows_pane() {
    if Path::new("/opt/homebrew/bin/bash").exists() || Path::new("/usr/local/bin/bash").exists() {
        follow_roundtrip("/bin/bash");
    }
}

#[test]
fn zsh_defers_cd_while_program_runs() {
    if !Path::new("/bin/zsh").exists() {
        return;
    }
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let c = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let zd = tempfile::tempdir().unwrap();
    let mut s = Session::spawn(SpawnOptions {
        shell: PathBuf::from("/bin/zsh"),
        cwd: a.path().to_path_buf(),
        size: GridSize {
            cols: 100,
            rows: 30,
        },
        cell_px: (8, 16),
        scrollback: 1000,
        shim_root: data.path().to_path_buf(),
        theme: Theme::dark(),
        waker: Arc::new(|| {}),
        spool: None,
        zdotdir: Some(zd.path().to_path_buf()),
        remote: None,
    })
    .unwrap();
    assert!(wait_until(&mut s, WAIT, |s| s.is_idle()));
    // Busy until the test ends it (a `sleep` may end before a loaded
    // machine even notices it started).
    s.write(b"read -r _manta_line\r");
    s.user_submitted();
    assert!(wait_until(&mut s, WAIT, |s| s.is_busy()));
    s.pane_changed(Some(b.path()));
    s.pane_changed(Some(c.path()));
    s.tick(true);
    assert!(s.follow.has_pending(), "must wait while the program runs");
    assert_eq!(
        s.follow.pending.as_ref().unwrap().canon,
        canon(c.path()),
        "latest target only"
    );
    assert_eq!(shell_cwd(&s), Some(canon(a.path())));
    s.write(b"\r");
    let c_canon = canon(c.path());
    assert!(
        wait_until(&mut s, WAIT, |s| shell_cwd(s).as_ref() == Some(&c_canon)),
        "deferred cd not applied. Screen:\n{}",
        s.lock().screen_text()
    );
    s.shutdown(Duration::from_millis(500));
    assert!(s.has_exited());
}

/// The zsh shim reports a command line mentioning `ssh` while it runs
/// (aliases expanded), and nothing for other commands.
#[test]
fn zsh_reports_ssh_command_lines() {
    if !Path::new("/bin/zsh").exists() {
        return;
    }
    let a = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let zd = tempfile::tempdir().unwrap();
    std::fs::write(
        zd.path().join(".zshrc"),
        "alias prod='read -r _ssh_alias'\n",
    )
    .unwrap();
    let mut s = Session::spawn(SpawnOptions {
        shell: PathBuf::from("/bin/zsh"),
        cwd: a.path().to_path_buf(),
        size: GridSize {
            cols: 100,
            rows: 30,
        },
        cell_px: (8, 16),
        scrollback: 1000,
        shim_root: data.path().to_path_buf(),
        theme: Theme::dark(),
        waker: Arc::new(|| {}),
        spool: None,
        zdotdir: Some(zd.path().to_path_buf()),
        remote: None,
    })
    .unwrap();
    assert!(wait_until(&mut s, WAIT, |s| s.is_idle()));

    // Not about ssh: nothing reported.
    s.write(b"read -r _plain\r");
    s.user_submitted();
    assert!(wait_until(&mut s, WAIT, |s| s.is_busy()));
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(s.running_command(), None);
    s.write(b"x\r");
    assert!(wait_until(&mut s, WAIT, |s| s.is_idle()));

    // An alias that expands to a line mentioning ssh.
    s.write(b"prod\r");
    s.user_submitted();
    assert!(
        wait_until(&mut s, WAIT, |s| s.running_command().is_some()),
        "{}",
        s.lock().screen_text()
    );
    assert_eq!(s.running_command().as_deref(), Some("read -r _ssh_alias"));
    // Gone at the next prompt.
    s.write(b"x\r");
    assert!(wait_until(&mut s, WAIT, |s| s.is_idle()));
    assert_eq!(s.running_command(), None);
    s.shutdown(Duration::from_millis(500));
}
