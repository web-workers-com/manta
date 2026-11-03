//! One shell in a PTY: spawning, the reader/writer/waiter threads, the
//! emulator state they share with the UI, and the live-follow logic.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Processor, Rgb};
use portable_pty::{CommandBuilder, MasterPty, PtySize};

use manta_plugin::FileAction;

use crate::actions::ShellCtx;
use crate::echo::EchoQueue;
use crate::escape::ShellKind;
use crate::follow::{Dir, Follow};
use crate::osc::{OscEvent, OscScanner};
use crate::palette::Theme;
use crate::rprompt::{self, Float, Floats, Live};
use crate::shim::{self, Integration};

/// Wakes the UI (egui `request_repaint`, or a no-op in tests).
pub type Waker = Arc<dyn Fn() + Send + Sync>;

/// Terminal grid size in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    pub cols: u16,
    pub rows: u16,
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }
    fn screen_lines(&self) -> usize {
        self.rows as usize
    }
    fn columns(&self) -> usize {
        self.cols as usize
    }
}

/// Receives the emulator's side-channel events (answers to queries, OSC 52).
pub struct Listener {
    tx: Sender<Vec<u8>>,
    clipboard: Arc<Mutex<Option<String>>>,
    theme: Arc<Mutex<Theme>>,
    window: Arc<Mutex<WindowSize>>,
}

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        match event {
            Event::PtyWrite(s) => {
                let _ = self.tx.send(s.into_bytes());
            }
            Event::ClipboardStore(_, text) => {
                if let Ok(mut c) = self.clipboard.lock() {
                    *c = Some(text);
                }
            }
            Event::ColorRequest(index, fmt) => {
                let t = *self.theme.lock().unwrap_or_else(|e| e.into_inner());
                let c = match index {
                    0..=255 => t.indexed(index as u8),
                    256 => t.fg,
                    257 => t.bg,
                    _ => t.cursor,
                };
                let _ = self.tx.send(
                    fmt(Rgb {
                        r: c.r(),
                        g: c.g(),
                        b: c.b(),
                    })
                    .into_bytes(),
                );
            }
            Event::TextAreaSizeRequest(fmt) => {
                let w = *self.window.lock().unwrap_or_else(|e| e.into_inner());
                let _ = self.tx.send(fmt(w).into_bytes());
            }
            _ => {}
        }
    }
}

/// State shared between the reader thread and the UI.
pub struct Shared {
    pub term: Term<Listener>,
    processor: Processor,
    osc: OscScanner,
    /// Cwd reports (OSC 7) not yet consumed by the follow logic.
    cwd_reports: Vec<Dir>,
    /// The shell drew a prompt and has not started a command since.
    pub at_prompt: bool,
    /// The command line the shim reported (OSC 7702, lines mentioning
    /// `ssh`) while it runs; cleared at the next prompt.
    pub running: Option<String>,
    /// `None` until the shell announced (or failed to announce) integration.
    pub integration: Option<Integration>,
    /// The shell announced the key that prints the spool file.
    pub echo: bool,
    /// Prompts drawn so far.
    pub prompt_count: u64,
    /// `Some(osc host id)` for a shell in a container / on a server: only
    /// OSC 7 with that host is accepted, and paths are not canonicalized.
    pub remote_host: Option<String>,
    /// Exit description once the shell is gone.
    pub exited: Option<String>,
    /// The shell's exit code (`None` while it runs or after a signal).
    pub exit_code: Option<u32>,
    exit_note_shown: bool,
    /// Bumped on every chunk of output.
    pub output_seq: u64,
    /// Right prompts of history lines (see [`crate::rprompt`]).
    pub floats: Floats,
    /// The current prompt line's right prompt (still in the grid).
    live_rprompt: Option<Live>,
    /// Cursor at the RPROMPT start marker.
    rprompt_start: Option<(Line, usize)>,
    /// Absolute line (history + viewport row) where the current prompt
    /// starts (OSC 133;A), while the shell sits at it.
    prompt_start_abs: Option<usize>,
}

impl Shared {
    /// Feed bytes as if they came from the PTY (also used by tests).
    pub fn feed(&mut self, bytes: &[u8]) {
        // Advance the emulator up to each event so cursor positions are
        // exact when we handle it.
        let mut pos = 0;
        for (end, ev) in self.osc.feed_at(bytes) {
            if end > pos {
                self.processor.advance(&mut self.term, &bytes[pos..end]);
                pos = end;
            }
            match ev {
                OscEvent::Cwd(url) => match &self.remote_host {
                    None => {
                        if let Some(p) = crate::osc::cwd_from_osc7(&url, None) {
                            let canon = std::fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
                            self.cwd_reports.push(Dir::new(p, canon));
                        }
                    }
                    // Paths inside a container/server: never canonicalize
                    // them on this Mac (/etc would become /private/etc).
                    Some(id) => {
                        if let Some(p) = crate::osc::cwd_from_osc7(&url, Some(id)) {
                            self.cwd_reports.push(remote_dir(&p));
                        }
                    }
                },
                OscEvent::PromptStart { push, echo } => {
                    self.finalize_rprompt();
                    self.prompt_start_abs = Some(self.cursor_abs_line());
                    self.at_prompt = true;
                    self.running = None;
                    self.echo = echo;
                    self.prompt_count = self.prompt_count.wrapping_add(1);
                    self.integration = Some(if push {
                        Integration::Push
                    } else {
                        Integration::Basic
                    });
                }
                OscEvent::CommandStart => {
                    self.finalize_rprompt();
                    self.prompt_start_abs = None;
                    self.at_prompt = false;
                    self.running = None;
                }
                // Only right after the shim's 133;C (a program can't fake
                // one mid-run without also faking the prompt).
                OscEvent::Command(line) if !self.at_prompt && self.running.is_none() => {
                    self.running = Some(line);
                }
                OscEvent::Command(_) => {}
                OscEvent::RpromptStart => self.rprompt_start = Some(self.cursor_point()),
                OscEvent::RpromptEnd => self.record_rprompt(),
            }
        }
        if pos < bytes.len() {
            self.processor.advance(&mut self.term, &bytes[pos..]);
        }
        self.output_seq = self.output_seq.wrapping_add(1);
    }

    /// Cursor line and the column the next character goes to.
    fn cursor_point(&self) -> (Line, usize) {
        let c = &self.term.grid().cursor;
        (
            c.point.line,
            c.point.column.0 + usize::from(c.input_needs_wrap),
        )
    }

    fn cursor_abs_line(&self) -> usize {
        let g = self.term.grid();
        g.history_size() + g.cursor.point.line.0.max(0) as usize
    }

    /// RPROMPT end marker: remember its cells and tag the line.
    fn record_rprompt(&mut self) {
        let Some((line, start)) = self.rprompt_start.take() else {
            return;
        };
        let (end_line, end) = self.cursor_point();
        let cols = self.term.grid().columns();
        if end_line != line || end <= start || start == 0 || end > cols {
            return;
        }
        let cells: Vec<Cell> = (start..end)
            .map(|c| self.term.grid()[Point::new(line, Column(c))].clone())
            .collect();
        let first = &mut self.term.grid_mut()[Point::new(line, Column(0))];
        // zsh redrew only the right prompt: keep the line's id.
        let id = match rprompt::line_marker(first) {
            Some(id) => id,
            None => {
                let id = self.floats.alloc();
                first.push_zerowidth(rprompt::marker_char(id));
                id
            }
        };
        self.live_rprompt = Some(Live {
            id,
            start,
            end,
            cells,
            indent: cols - end,
        });
    }

    /// The prompt line became history: move its right prompt out of the
    /// grid into a float (if it is still exactly where zsh drew it).
    fn finalize_rprompt(&mut self) {
        let Some(live) = self.live_rprompt.take() else {
            return;
        };
        let grid = self.term.grid();
        let top = grid.topmost_line().0.max(grid.bottommost_line().0 - 300);
        let mut line = grid.bottommost_line();
        let found = loop {
            if line.0 < top {
                break None;
            }
            if rprompt::line_marker(&grid[Point::new(line, Column(0))]) == Some(live.id) {
                break Some(line);
            }
            line -= 1i32;
        };
        let Some(line) = found else { return };
        if live.end > grid.columns() {
            return;
        }
        let intact = (live.start..live.end)
            .zip(&live.cells)
            .all(|(c, cell)| grid[Point::new(line, Column(c))].c == cell.c);
        if !intact {
            return;
        }
        for c in live.start..live.end {
            self.term.grid_mut()[Point::new(line, Column(c))] = Cell::default();
        }
        // The gap zsh left between the prompt and RPROMPT (and the indent
        // after it) are spaces that may carry a colour; they look empty but
        // reflow would treat them as content. Make them truly empty.
        let cols = self.term.grid().columns();
        let mut c = cols;
        while c > 0 {
            let cell = &mut self.term.grid_mut()[Point::new(line, Column(c - 1))];
            let blank = (cell.c == ' ' || cell.c == '\0')
                && cell.bg == Cell::default().bg
                && !cell.flags.intersects(
                    Flags::INVERSE | Flags::ALL_UNDERLINES | Flags::STRIKEOUT | Flags::WRAPLINE,
                )
                && cell.zerowidth().is_none_or(|z| z.is_empty());
            if !blank {
                break;
            }
            *cell = Cell::default();
            c -= 1;
        }
        self.floats.insert(
            live.id,
            Float {
                cells: live.cells,
                indent: live.indent,
            },
        );
    }

    /// Before a width change while the shell sits at its prompt: blank the
    /// prompt region (zsh redraws it for the new width right after
    /// SIGWINCH) so reflow cannot wrap it into fragments, and keep the
    /// cursor on its row so zsh's "move up to the prompt start" still lands
    /// there.
    pub fn clear_prompt_for_resize(&mut self) {
        let Some(start_abs) = self.prompt_start_abs else {
            return;
        };
        if !self.at_prompt || self.term.mode().contains(TermMode::ALT_SCREEN) {
            return;
        }
        self.live_rprompt = None;
        let g = self.term.grid();
        let history = g.history_size();
        let rows = g.screen_lines() as i32;
        let cursor_line = g.cursor.point.line.0;
        let first = (start_abs as i64 - history as i64).clamp(0, cursor_line.max(0) as i64) as i32;
        for l in first..rows {
            let line = Line(l);
            let cols = self.term.grid().columns();
            for c in 0..cols {
                self.term.grid_mut()[Point::new(line, Column(c))] = Cell::default();
            }
        }
        let cursor = &mut self.term.grid_mut().cursor;
        cursor.point.column = Column(0);
        cursor.input_needs_wrap = false;
    }

    /// Finish a synchronized update whose timeout passed. Returns how long
    /// to wait before checking again, if one is still running.
    pub fn flush_sync(&mut self) -> Option<Duration> {
        let deadline = self.processor.sync_timeout().sync_timeout()?;
        let now = Instant::now();
        if now >= deadline {
            self.processor.stop_sync(&mut self.term);
            None
        } else {
            Some(deadline - now)
        }
    }
}

/// A directory inside a container or on a server: normalized, never
/// canonicalized locally.
pub fn remote_dir(p: &Path) -> Dir {
    let v = manta_core::VfsPath::new(&p.to_string_lossy());
    Dir::same(v.as_str())
}

/// Create an emulator that is not attached to a PTY (tests, previews).
pub fn detached_shared(size: GridSize, scrollback: usize) -> Shared {
    let (tx, _rx) = mpsc::channel();
    let listener = Listener {
        tx,
        clipboard: Arc::new(Mutex::new(None)),
        theme: Arc::new(Mutex::new(Theme::dark())),
        window: Arc::new(Mutex::new(WindowSize {
            num_lines: size.rows,
            num_cols: size.cols,
            cell_width: 8,
            cell_height: 16,
        })),
    };
    new_shared(size, scrollback, listener)
}

fn new_shared(size: GridSize, scrollback: usize, listener: Listener) -> Shared {
    let config = Config {
        scrolling_history: scrollback,
        ..Config::default()
    };
    Shared {
        term: Term::new(config, &size, listener),
        processor: Processor::new(),
        osc: OscScanner::new(),
        cwd_reports: Vec::new(),
        at_prompt: false,
        running: None,
        integration: None,
        echo: false,
        prompt_count: 0,
        remote_host: None,
        exited: None,
        exit_code: None,
        exit_note_shown: false,
        output_seq: 0,
        floats: Floats::new(scrollback),
        live_rprompt: None,
        rprompt_start: None,
        prompt_start_abs: None,
    }
}

pub struct SpawnOptions {
    pub shell: PathBuf,
    pub cwd: PathBuf,
    pub size: GridSize,
    pub cell_px: (u16, u16),
    pub scrollback: usize,
    /// Where the generated shell integration lives.
    pub shim_root: PathBuf,
    pub theme: Theme,
    pub waker: Waker,
    /// Spool file for printing file actions through the shell (zsh/bash
    /// only). `None` disables it.
    pub spool: Option<PathBuf>,
    /// zsh only: use this as the user's ZDOTDIR instead of the inherited
    /// one (hermetic tests). `None`: inherit.
    pub zdotdir: Option<PathBuf>,
    /// Run a shell in a container / on a server instead of a local one
    /// (`shell`, `cwd`, `spool` and `zdotdir` are then ignored).
    pub remote: Option<RemoteSpawn>,
}

/// How to start a shell in a container or on a server (see
/// [`crate::remote::Target::args`]).
#[derive(Debug, Clone)]
pub struct RemoteSpawn {
    /// `docker`, `kubectl` or `ssh` (full path).
    pub program: PathBuf,
    pub args: Vec<String>,
    /// Host id the shell's OSC 7 reports carry.
    pub osc_host: String,
    /// The folder the shell starts in (inside the target).
    pub start_dir: String,
}

/// The user's login shell: `$SHELL`, else `/bin/zsh`.
pub fn default_shell() -> PathBuf {
    std::env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute() && p.exists())
        .unwrap_or_else(|| PathBuf::from("/bin/zsh"))
}

/// How long we wait for the first prompt marker before treating the shell
/// as one without integration.
const INTEGRATION_GRACE: Duration = Duration::from_secs(4);
/// Give up on the echo of a typed cd after this long.
const ECHO_TIMEOUT: Duration = Duration::from_secs(5);
/// Send the print key again if the spool file was not consumed after this.
const ECHO_RESEND: Duration = Duration::from_secs(3);
/// How long after the shell exits we wait for its last output before
/// printing the exit note.
const EXIT_NOTE_DELAY: Duration = Duration::from_millis(150);
/// A new terminal size must be stable this long before the PTY follows.
pub const RESIZE_DEBOUNCE: Duration = Duration::from_millis(150);

/// The note printed when the shell exits, in the app's language.
pub fn exit_note() -> &'static str {
    crate::i18n::tr!("term.exit_note")
}

pub struct Session {
    shared: Arc<Mutex<Shared>>,
    master: Option<Box<dyn MasterPty + Send>>,
    tx: Option<Sender<Vec<u8>>>,
    pid: Option<i32>,
    pub kind: ShellKind,
    pub shell: PathBuf,
    /// The shell runs in a container / on a server: paths are not local,
    /// and idleness comes from the prompt marks alone.
    remote: bool,
    size: GridSize,
    started: Instant,
    exited_flag: Arc<AtomicBool>,
    exited_at: Option<Instant>,
    /// [`Self::take_left`] answered already.
    left_reported: bool,
    threads: Vec<JoinHandle<()>>,
    theme: Arc<Mutex<Theme>>,
    window: Arc<Mutex<WindowSize>>,
    clipboard: Arc<Mutex<Option<String>>>,
    pub follow: Follow,
    cd_sent_at: Option<Instant>,
    waker: Waker,
    /// A size waiting to become stable (see `request_resize`).
    pending_resize: Option<(GridSize, Instant)>,
    /// Where the shell reads the lines to print (see `shim::ECHO_KEY`).
    spool: Option<PathBuf>,
    /// File-action lines waiting for an idle prompt.
    echo: EchoQueue,
    /// When we last sent the print key (until the shell removed the file).
    echo_sent_at: Option<Instant>,
    /// Actions this shell cannot print (no integration); the panel shows
    /// them instead.
    fallback: Vec<FileAction>,
    /// Order of cd follows and echoes: a shared counter, and the stamp of
    /// the pending cd.
    seq: u64,
    cd_seq: u64,
    /// Commands to type once the shell is idle ([`Self::push_run`]).
    runs: std::collections::VecDeque<Run>,
}

/// A command waiting to be typed into the shell.
#[derive(Debug, Clone)]
struct Run {
    command: String,
    cwd: Option<Dir>,
    /// We already asked the follow logic to `cd` there (once: a folder that
    /// can't be entered must not loop).
    cd_asked: bool,
}

impl Session {
    pub fn spawn(opts: SpawnOptions) -> io::Result<Session> {
        let remote = opts.remote.clone();
        // Remote shells announce themselves (bash with push keys, or a
        // plain sh); POSIX quoting and Ctrl-E Ctrl-U fit both.
        let kind = if remote.is_some() {
            ShellKind::Bash
        } else {
            ShellKind::from_path(&opts.shell)
        };
        let pty = portable_pty::native_pty_system();
        let pair = pty
            .openpty(PtySize {
                rows: opts.size.rows,
                cols: opts.size.cols,
                pixel_width: opts.size.cols * opts.cell_px.0,
                pixel_height: opts.size.rows * opts.cell_px.1,
            })
            .map_err(io::Error::other)?;

        let mut cmd = match &remote {
            Some(r) => {
                let mut c = CommandBuilder::new(&r.program);
                c.args(&r.args);
                c.cwd(
                    std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("/")),
                );
                // Credential helpers and kubectl auth plugins live next to
                // the CLIs; apps started from Finder lack them in PATH.
                if let Ok(path) = std::env::join_paths(crate::remote::search_path()) {
                    c.env("PATH", path);
                }
                c
            }
            None => {
                let mut c = CommandBuilder::new(&opts.shell);
                c.cwd(&opts.cwd);
                c
            }
        };
        for var in [
            "TERM_SESSION_ID",
            "ITERM_SESSION_ID",
            "ITERM_PROFILE",
            "LC_TERMINAL",
            "LC_TERMINAL_VERSION",
            "TERM_PROGRAM_VERSION",
            "VSCODE_INJECTION",
            "_MANTA_UZD",
            "MANTA_SHIM_DIR",
            shim::SPOOL_ENV,
        ] {
            cmd.env_remove(var);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("TERM_PROGRAM", "Manta");
        cmd.env("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION"));
        if remote.is_none() {
            cmd.env("PWD", &opts.cwd);
        }
        // Apps started from Finder get no locale; a C locale breaks UTF-8
        // input and file names in the shell.
        if ["LANG", "LC_ALL", "LC_CTYPE"]
            .iter()
            .all(|v| std::env::var_os(v).is_none())
        {
            cmd.env("LANG", "en_US.UTF-8");
        }
        let shims = if remote.is_none() {
            shim::install(&opts.shim_root)
        } else {
            Err(io::Error::other("remote shell"))
        };
        let spool = match (kind, &shims, &opts.spool) {
            (ShellKind::Zsh | ShellKind::Bash, Ok(_), Some(p)) => {
                let _ = std::fs::remove_file(p);
                if let Some(dir) = p.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                cmd.env(shim::SPOOL_ENV, p);
                Some(p.clone())
            }
            _ => None,
        };
        match (kind, &shims) {
            (ShellKind::Zsh, Ok(paths)) => {
                cmd.arg("-l");
                if let Some(orig) = opts
                    .zdotdir
                    .clone()
                    .map(Into::into)
                    .or_else(|| std::env::var_os("ZDOTDIR"))
                {
                    cmd.env("_MANTA_UZD", orig);
                }
                cmd.env("ZDOTDIR", &paths.zsh_dir);
                cmd.env("MANTA_SHIM_DIR", &paths.zsh_dir);
            }
            (ShellKind::Bash, Ok(paths)) => {
                cmd.arg("--rcfile");
                cmd.arg(&paths.bash_rc);
                cmd.arg("-i");
            }
            _ if remote.is_some() => {}
            _ => {
                if let Err(e) = &shims {
                    log::warn!("terminal: shell integration unavailable: {e}");
                }
                cmd.arg("-l");
            }
        }

        let child = pair.slave.spawn_command(cmd).map_err(io::Error::other)?;
        drop(pair.slave);
        let pid = child.process_id().map(|p| p as i32);

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let clipboard = Arc::new(Mutex::new(None));
        let theme = Arc::new(Mutex::new(opts.theme));
        let window = Arc::new(Mutex::new(WindowSize {
            num_lines: opts.size.rows,
            num_cols: opts.size.cols,
            cell_width: opts.cell_px.0,
            cell_height: opts.cell_px.1,
        }));
        let listener = Listener {
            tx: tx.clone(),
            clipboard: clipboard.clone(),
            theme: theme.clone(),
            window: window.clone(),
        };
        let mut sh = new_shared(opts.size, opts.scrollback, listener);
        sh.remote_host = remote.as_ref().map(|r| r.osc_host.clone());
        let shared = Arc::new(Mutex::new(sh));

        let mut reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        let mut writer = pair.master.take_writer().map_err(io::Error::other)?;
        let mut threads = Vec::new();

        // Reader: PTY output → OSC scanner + emulator.
        {
            let shared = shared.clone();
            let waker = opts.waker.clone();
            threads.push(
                std::thread::Builder::new()
                    .name("terminal-reader".into())
                    .spawn(move || {
                        let mut buf = vec![0u8; 64 * 1024];
                        loop {
                            match reader.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Ok(mut s) = shared.lock() {
                                        s.feed(&buf[..n]);
                                    }
                                    waker();
                                }
                                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                                Err(_) => break,
                            }
                        }
                        waker();
                    })?,
            );
        }
        let exited_flag = Arc::new(AtomicBool::new(false));
        // Writer: keeps the UI thread from blocking on a full PTY. The
        // emulator holds a Sender too, so also stop once the shell is gone.
        {
            let flag = exited_flag.clone();
            threads.push(
                std::thread::Builder::new()
                    .name("terminal-writer".into())
                    .spawn(move || loop {
                        match rx.recv_timeout(Duration::from_millis(50)) {
                            Ok(bytes) => {
                                if writer
                                    .write_all(&bytes)
                                    .and_then(|_| writer.flush())
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                if flag.load(Ordering::SeqCst) {
                                    break;
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    })?,
            );
        }
        // Waiter: reaps the shell (no zombies) and records the exit.
        {
            let shared = shared.clone();
            let waker = opts.waker.clone();
            let flag = exited_flag.clone();
            let mut child = child;
            threads.push(
                std::thread::Builder::new()
                    .name("terminal-waiter".into())
                    .spawn(move || {
                        let status = child.wait();
                        let code = status
                            .as_ref()
                            .ok()
                            .filter(|st| st.signal().is_none())
                            .map(|st| st.exit_code());
                        let text = match status {
                            Ok(st) => match st.signal() {
                                Some(sig) => format!("signal {sig}"),
                                None => format!("exit code {}", st.exit_code()),
                            },
                            Err(e) => e.to_string(),
                        };
                        if let Ok(mut s) = shared.lock() {
                            s.exited = Some(text);
                            s.exit_code = code;
                            s.at_prompt = false;
                        }
                        flag.store(true, Ordering::SeqCst);
                        waker();
                    })?,
            );
        }

        let mut follow = Follow::new();
        let start = match &remote {
            Some(r) => remote_dir(Path::new(&r.start_dir)),
            None => Dir::new(
                opts.cwd.clone(),
                std::fs::canonicalize(&opts.cwd).unwrap_or_else(|_| opts.cwd.clone()),
            ),
        };
        follow.pane_changed(Some(start));
        // The shell starts there; nothing to type.
        follow.pending = None;

        Ok(Session {
            shared,
            master: Some(pair.master),
            tx: Some(tx),
            pid,
            kind,
            shell: remote
                .as_ref()
                .map(|r| r.program.clone())
                .unwrap_or(opts.shell),
            remote: remote.is_some(),
            size: opts.size,
            started: Instant::now(),
            exited_flag,
            exited_at: None,
            left_reported: false,
            threads,
            theme,
            window,
            clipboard,
            follow,
            cd_sent_at: None,
            waker: opts.waker,
            pending_resize: None,
            spool,
            echo: EchoQueue::new(),
            echo_sent_at: None,
            fallback: Vec::new(),
            seq: 0,
            cd_seq: 0,
            runs: std::collections::VecDeque::new(),
        })
    }

    pub fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn pid(&self) -> Option<i32> {
        self.pid
    }

    pub fn size(&self) -> GridSize {
        self.size
    }

    pub fn has_exited(&self) -> bool {
        self.exited_flag.load(Ordering::SeqCst)
    }

    /// True once, after the shell was left on purpose (`exit`, Ctrl-D):
    /// it had drawn a prompt and ended with an exit code below 128. A
    /// shell that never started, a dropped `ssh` (255) or a killed
    /// container (128 + signal) doesn't count.
    pub fn take_left(&mut self) -> bool {
        if self.left_reported || !self.has_exited() {
            return false;
        }
        self.left_reported = true;
        let s = self.lock();
        s.prompt_count > 0 && s.exit_code.is_some_and(|c| c < 128)
    }

    /// Send input to the shell.
    pub fn write(&self, bytes: &[u8]) {
        if bytes.is_empty() || self.has_exited() {
            return;
        }
        if let Some(tx) = &self.tx {
            let _ = tx.send(bytes.to_vec());
        }
    }

    /// The user pressed Enter: a command (or continuation line) starts.
    pub fn user_submitted(&self) {
        self.lock().at_prompt = false;
    }

    pub fn set_theme(&self, theme: Theme) {
        if let Ok(mut t) = self.theme.lock() {
            *t = theme;
        }
    }

    /// Text a program asked to copy (OSC 52).
    pub fn take_clipboard(&self) -> Option<String> {
        self.clipboard.lock().ok()?.take()
    }

    /// Resize once `size` has been stable for [`RESIZE_DEBOUNCE`]. Call
    /// every frame with the current size; returns how long to wait before
    /// calling again while a change is pending.
    pub fn request_resize(&mut self, size: GridSize, cell_px: (u16, u16)) -> Option<Duration> {
        if size == self.size || size.cols == 0 || size.rows == 0 {
            self.pending_resize = None;
            return None;
        }
        match self.pending_resize {
            Some((pending, since)) if pending == size => {
                let waited = since.elapsed();
                if waited >= RESIZE_DEBOUNCE {
                    self.pending_resize = None;
                    self.resize(size, cell_px);
                    None
                } else {
                    Some(RESIZE_DEBOUNCE - waited)
                }
            }
            _ => {
                self.pending_resize = Some((size, Instant::now()));
                Some(RESIZE_DEBOUNCE)
            }
        }
    }

    /// Resize right away (see [`Self::request_resize`] for the UI path).
    pub fn resize(&mut self, size: GridSize, cell_px: (u16, u16)) {
        if size == self.size || size.cols == 0 || size.rows == 0 {
            return;
        }
        let cols_changed = size.cols != self.size.cols;
        self.size = size;
        // Emulator first, so the shell's redraw after SIGWINCH lands on the
        // new grid.
        {
            let mut sh = self.lock();
            if cols_changed {
                sh.clear_prompt_for_resize();
            }
            sh.term.resize(size);
        }
        if let Some(m) = &self.master {
            let _ = m.resize(PtySize {
                rows: size.rows,
                cols: size.cols,
                pixel_width: size.cols * cell_px.0,
                pixel_height: size.rows * cell_px.1,
            });
        }
        if let Ok(mut w) = self.window.lock() {
            *w = WindowSize {
                num_lines: size.rows,
                num_cols: size.cols,
                cell_width: cell_px.0,
                cell_height: cell_px.1,
            };
        }
    }

    /// The foreground process group of the PTY.
    pub fn foreground_pgid(&self) -> Option<i32> {
        self.master.as_ref()?.process_group_leader()
    }

    /// True if the shell itself is in the foreground and sitting at its
    /// prompt (no program running).
    pub fn is_idle(&self) -> bool {
        if self.has_exited() {
            return false;
        }
        let Some(pid) = self.pid else { return false };
        // Remote: the foreground process is always the docker/kubectl/ssh
        // client; only the prompt marks tell whether the shell is idle.
        if !self.remote && self.foreground_pgid() != Some(pid) {
            return false;
        }
        let s = self.lock();
        match s.integration {
            Some(Integration::Push | Integration::Basic) => s.at_prompt,
            // Without marks we can't tell: never type into a remote shell.
            Some(Integration::None) => !self.remote,
            None => false,
        }
    }

    pub fn integration(&self) -> Option<Integration> {
        self.lock().integration
    }

    /// The command line the shell reported for the command running now
    /// (only lines mentioning `ssh`, local zsh), `None` at the prompt.
    pub fn running_command(&self) -> Option<String> {
        self.lock().running.clone()
    }

    /// A program other than the shell is in the foreground.
    pub fn is_busy(&self) -> bool {
        !self.has_exited() && !self.is_idle()
    }

    /// The pane above now shows `dir` (`None` for remote folders).
    pub fn pane_changed(&mut self, dir: Option<&Path>) {
        let dir = dir.map(|p| self.dir(p));
        let before = self.follow.pending.clone();
        self.follow.pane_changed(dir);
        self.stamp_cd(before);
    }

    /// A directory of this shell's file system (canonical when local).
    fn dir(&self, p: &Path) -> Dir {
        if self.remote {
            remote_dir(p)
        } else {
            Dir::new(
                p,
                std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()),
            )
        }
    }

    /// The shell runs in a container or on a server.
    pub fn is_remote(&self) -> bool {
        self.remote
    }

    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// A new cd target is pending: remember when (relative to echoes).
    fn stamp_cd(&mut self, before: Option<Dir>) {
        if self.follow.pending.is_some() && self.follow.pending != before {
            self.cd_seq = self.next_seq();
        }
    }

    /// Whether this shell can print file actions: `None` while unknown
    /// (startup).
    pub fn can_echo(&self) -> Option<bool> {
        let s = self.lock();
        match s.integration {
            None => None,
            Some(Integration::None) => Some(false),
            Some(_) => Some(s.echo && self.spool.is_some()),
        }
    }

    /// Show a GUI file action in this shell (printed above the prompt once
    /// the shell is idle). Shells without integration hand it to
    /// [`Self::take_fallback`] instead.
    pub fn push_action(&mut self, action: FileAction) {
        if self.can_echo() == Some(false) {
            self.fallback.push(action);
            return;
        }
        let seq = self.next_seq();
        let now = Instant::now();
        self.echo.push(seq, action, now);
        if let Some(at) = self.echo.next_ready_at(now) {
            let waker = self.waker.clone();
            let wait = at.saturating_duration_since(now) + Duration::from_millis(5);
            std::thread::spawn(move || {
                std::thread::sleep(wait);
                waker();
            });
        }
        (self.waker)();
    }

    /// Type `command` into the shell and press Enter once it is idle at its
    /// prompt (after any waiting file-action lines and follow `cd`s), first
    /// `cd`-ing to `cwd` if the shell is elsewhere. The user's half-typed
    /// line is parked and comes back afterwards (see [`shim::run_input`]).
    pub fn push_run(&mut self, command: &str, cwd: Option<&Path>) {
        let cwd = cwd.map(|p| self.dir(p));
        self.runs.push_back(Run {
            command: command.to_string(),
            cwd,
            cd_asked: false,
        });
        (self.waker)();
    }

    /// Commands waiting for the shell to become idle.
    pub fn runs_waiting(&self) -> usize {
        self.runs.len()
    }

    /// The first waiting command (for the status pill).
    pub fn next_run(&self) -> Option<&str> {
        self.runs.front().map(|r| r.command.as_str())
    }

    /// Type the next waiting command if nothing else has to happen first.
    fn flush_runs(&mut self, integration: Option<Integration>) {
        if self.runs.is_empty()
            || !self.echo.is_empty()
            || self.follow.has_pending()
            || self.follow.in_flight.is_some()
            || !self.is_idle()
        {
            return;
        }
        let shell_cwd = self.follow.shell_cwd.as_ref().map(|d| d.canon.clone());
        let Some(run) = self.runs.front_mut() else {
            return;
        };
        if let Some(cwd) = run.cwd.clone() {
            if shell_cwd.as_ref() != Some(&cwd.canon) && !run.cd_asked {
                // Let the follow logic type the cd (next tick); its echo
                // never navigates the pane.
                run.cd_asked = true;
                self.follow.pending = Some(cwd);
                self.cd_seq = self.next_seq();
                (self.waker)();
                return;
            }
        }
        let Some(run) = self.runs.pop_front() else {
            return;
        };
        let integ = integration.unwrap_or(Integration::None);
        self.write(&shim::run_input(self.kind, integ, &run.command));
        if integ != Integration::None {
            self.lock().at_prompt = false;
        }
        if !self.runs.is_empty() {
            (self.waker)();
        }
    }

    /// Actions this shell could not print (see [`Self::push_action`]).
    pub fn take_fallback(&mut self) -> Vec<FileAction> {
        std::mem::take(&mut self.fallback)
    }

    /// File actions waiting for the shell to become idle.
    pub fn echo_waiting(&self) -> usize {
        self.echo.actions_waiting()
    }

    /// The spool file (tests).
    pub fn spool_path(&self) -> Option<&Path> {
        self.spool.as_deref()
    }

    /// Print the waiting lines older than `before` through the shell.
    /// Returns true if something was handed to the shell.
    fn flush_echo(&mut self, before: Option<u64>) -> bool {
        let Some(spool) = self.spool.clone() else {
            return false;
        };
        let now = Instant::now();
        if spool.exists() {
            // The last batch was not printed yet (the key got lost?).
            if self
                .echo_sent_at
                .is_none_or(|t| now.duration_since(t) > ECHO_RESEND)
            {
                self.write(shim::ECHO_KEY);
                self.echo_sent_at = Some(now);
            }
            return false;
        }
        let items = self.echo.take_ready(before, now);
        if items.is_empty() {
            if let Some(at) = self.echo.next_ready_at(now) {
                let waker = self.waker.clone();
                let wait = at.saturating_duration_since(now) + Duration::from_millis(5);
                std::thread::spawn(move || {
                    std::thread::sleep(wait);
                    waker();
                });
            }
            return false;
        }
        let prompt_count = self.lock().prompt_count;
        let ctx = ShellCtx::new(self.follow.shell_cwd.as_ref(), self.kind);
        let text = self.echo.render(&items, ctx, prompt_count);
        if let Err(e) = write_atomic(&spool, text.as_bytes()) {
            log::warn!("terminal: cannot write {}: {e}", spool.display());
            return false;
        }
        self.write(shim::ECHO_KEY);
        self.echo_sent_at = Some(now);
        true
    }

    /// Advance follow logic and housekeeping. Call every frame (cheap).
    /// Returns a directory the pane should navigate to (reverse follow).
    pub fn tick(&mut self, reverse: bool) -> Option<PathBuf> {
        let (reports, integration) = {
            let mut s = self.lock();
            if s.integration.is_none() && self.started.elapsed() > INTEGRATION_GRACE {
                s.integration = Some(Integration::None);
            }
            if let Some(wait) = s.flush_sync() {
                let waker = self.waker.clone();
                // Rare (synchronized updates); a short sleeping thread keeps
                // this free of egui types.
                std::thread::spawn(move || {
                    std::thread::sleep(wait);
                    waker();
                });
            }
            (std::mem::take(&mut s.cwd_reports), s.integration)
        };

        let mut navigate = None;
        for dir in reports {
            let before = self.follow.pending.clone();
            if let Some(d) = self.follow.cwd_reported(dir, reverse) {
                navigate = Some(d.raw);
            }
            self.stamp_cd(before);
        }
        if self.echo_sent_at.is_some() && self.spool.as_ref().is_some_and(|p| !p.exists()) {
            self.echo_sent_at = None;
        }
        if self.can_echo() == Some(false) && !self.echo.is_empty() {
            let drained = self.echo.drain_all();
            self.fallback.extend(drained);
        }
        if self.follow.in_flight.is_some() {
            if self.cd_sent_at.is_some_and(|t| t.elapsed() > ECHO_TIMEOUT) {
                self.follow.clear_in_flight();
            }
        } else {
            self.cd_sent_at = None;
        }

        // Echoes and cd follows go out in the order they happened, only
        // while the shell is idle at its prompt.
        while !self.echo.is_empty() && self.follow.in_flight.is_none() && self.is_idle() {
            let cd = self.follow.has_pending().then_some(self.cd_seq);
            let echo_first = match (self.echo.front_seq(), cd) {
                (Some(e), Some(c)) => e < c,
                (Some(_), None) => true,
                _ => false,
            };
            if !echo_first || !self.flush_echo(cd) {
                break;
            }
        }
        let echo_before_cd = matches!((self.echo.front_seq(), self.follow.has_pending()), (Some(e), true) if e < self.cd_seq);
        if self.follow.has_pending() && !echo_before_cd && self.is_idle() {
            if let Some(dir) = self.follow.take_cd(true) {
                let integ = integration.unwrap_or(Integration::None);
                let text = dir.raw.to_string_lossy().into_owned();
                self.write(&shim::cd_input(self.kind, integ, &text));
                self.cd_sent_at = Some(Instant::now());
                if integ == Integration::None {
                    self.follow.assume_arrived();
                } else {
                    self.lock().at_prompt = false;
                }
            }
        }
        self.flush_runs(integration);

        if self.has_exited() {
            let at = *self.exited_at.get_or_insert_with(Instant::now);
            let mut s = self.lock();
            if !s.exit_note_shown && at.elapsed() >= EXIT_NOTE_DELAY {
                s.exit_note_shown = true;
                let note = format!("\x1b[0m\r\n\x1b[2m{}\x1b[0m\r\n", exit_note());
                s.feed(note.as_bytes());
                s.term
                    .scroll_display(alacritty_terminal::grid::Scroll::Bottom);
            } else if !s.exit_note_shown {
                drop(s);
                let waker = self.waker.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(EXIT_NOTE_DELAY);
                    waker();
                });
            }
        }
        navigate
    }

    /// Ask the shell to exit (SIGHUP to it and its foreground job). Does
    /// not wait.
    pub fn hangup(&self) {
        if self.has_exited() {
            return;
        }
        if let Some(pid) = self.pid {
            let fg = self.foreground_pgid();
            // SAFETY: plain signal syscalls on pids we spawned; the waiter
            // thread has not reaped `pid` yet (has_exited is false).
            unsafe {
                if let Some(pg) = fg.filter(|&pg| pg > 0 && pg != pid) {
                    libc::kill(-pg, libc::SIGHUP);
                }
                libc::kill(pid, libc::SIGHUP);
            }
        }
    }

    fn force_kill(&self) {
        if self.has_exited() {
            return;
        }
        if let Some(pid) = self.pid {
            // SAFETY: see `hangup`.
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
                libc::kill(pid, libc::SIGKILL);
            }
        }
    }

    /// Stop the shell and all threads. Waits at most `timeout` (plus a
    /// short grace for SIGKILL and joins). Returns how many threads were
    /// still running and had to be detached (0 normally; a background job
    /// that keeps the PTY open can keep the reader alive).
    pub fn shutdown(&mut self, timeout: Duration) -> usize {
        self.hangup();
        let deadline = Instant::now() + timeout;
        while !self.has_exited() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if !self.has_exited() {
            self.force_kill();
            let hard = Instant::now() + Duration::from_millis(200);
            while !self.has_exited() && Instant::now() < hard {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        // Closing our ends lets the writer exit and the reader see EOF.
        self.tx = None;
        self.master = None;
        let join_deadline = Instant::now() + Duration::from_millis(300);
        let mut detached = 0;
        for t in self.threads.drain(..) {
            while !t.is_finished() && Instant::now() < join_deadline {
                std::thread::sleep(Duration::from_millis(2));
            }
            if t.is_finished() {
                let _ = t.join();
            } else {
                // Detach: it only holds Arcs and exits on EOF.
                detached += 1;
            }
        }
        detached
    }
}

/// Write `bytes` to `path` so a reader never sees a partial file.
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(p) = &self.spool {
            let _ = std::fs::remove_file(p);
        }
        if self.threads.is_empty() {
            return;
        }
        // Non-blocking: hang up now, make sure it dies shortly after.
        self.hangup();
        let flag = self.exited_flag.clone();
        let pid = self.pid;
        std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_millis(800);
            while !flag.load(Ordering::SeqCst) && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if let (false, Some(pid)) = (flag.load(Ordering::SeqCst), pid) {
                // SAFETY: the waiter thread has not reaped pid yet.
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                }
            }
        });
    }
}

impl Shared {
    /// The visible screen as the user sees it: grid text plus the floating
    /// right prompts drawn at the current width. For tests.
    pub fn visible_text(&self) -> String {
        use alacritty_terminal::index::{Column, Line, Point};
        let grid = self.term.grid();
        let overlays = rprompt::overlays(&self.term, &self.floats);
        let mut out = String::new();
        for (row, overlay) in overlays.iter().enumerate() {
            let line = Line(row as i32 - grid.display_offset() as i32);
            let mut chars: Vec<char> = (0..grid.columns())
                .map(|c| {
                    let cell = &grid[Point::new(line, Column(c))];
                    if cell
                        .flags
                        .contains(alacritty_terminal::term::cell::Flags::WIDE_CHAR_SPACER)
                    {
                        '\0'
                    } else {
                        cell.c
                    }
                })
                .collect();
            if let Some(o) = overlay {
                for (i, cell) in o.float.cells.iter().enumerate() {
                    chars[o.start + i] = cell.c;
                }
            }
            let s: String = chars.into_iter().filter(|&c| c != '\0').collect();
            out.push_str(s.trim_end());
            out.push('\n');
        }
        out
    }

    /// The visible screen as plain text, one line per row (trailing blanks
    /// trimmed). For tests and debugging.
    pub fn screen_text(&self) -> String {
        use alacritty_terminal::index::{Column, Line, Point};
        let grid = self.term.grid();
        let mut out = String::new();
        for row in 0..grid.screen_lines() {
            let line = Line(row as i32 - grid.display_offset() as i32);
            let mut s = String::new();
            for col in 0..grid.columns() {
                let cell = &grid[Point::new(line, Column(col))];
                if !cell
                    .flags
                    .contains(alacritty_terminal::term::cell::Flags::WIDE_CHAR_SPACER)
                {
                    s.push(cell.c);
                }
            }
            out.push_str(s.trim_end());
            out.push('\n');
        }
        out
    }
}
