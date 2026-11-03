//! Built-in Terminal plugin. PUBLIC API CONTRACT (used by the app): `TerminalPlugin::new()`.
//!
//! A terminal below each file pane that follows the pane live: when the pane
//! shows another local folder, the shell `cd`s there as soon as it is idle at
//! its prompt (keeping whatever the user had typed), and a `cd` typed in the
//! terminal navigates the pane (setting "Pane follows terminal cd").
//! When the pane shows a Docker container, a Kubernetes pod or an SFTP
//! server, the terminal is a shell *there* (`docker exec`, `kubectl exec`,
//! `ssh`; see [`remote`]) and follows in both directions the same way. Each
//! pane keeps up to [`MAX_SESSIONS`] sessions (one per place), so going
//! back to a folder brings its shell back.
//!
//! Modules:
//! - [`session`]: PTY + shell process + reader/writer/waiter threads.
//! - [`view`]: painting the cell grid, keyboard/mouse handling.
//! - [`follow`]: the follow state machine (pure).
//! - [`shim`]: generated zsh/bash integration (OSC 7 / OSC 133 hooks).
//! - [`actions`], [`echo`]: GUI file actions (copy, delete, …) shown in the
//!   terminal as the equivalent shell commands, printed through the shell.
//! - [`osc`], [`keys`], [`escape`], [`palette`], [`settings`]: helpers.

pub mod actions;
pub mod echo;
pub mod escape;
pub mod follow;
pub mod i18n;
pub mod keys;
pub mod osc;
pub mod palette;
pub mod remote;
pub mod rprompt;
pub mod session;
pub mod settings;
pub mod shim;
pub mod view;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Vec2};
use manta_core::VfsPath;
use manta_plugin::{
    CommandContext, CommandSpec, FileAction, FileActionKind, HostRequest, PaneId, PaneState,
    PanelPlacement, PanelSpec, Plugin, PluginEvent, PluginHandle, PluginInfo, ToastLevel,
};

use crate::actions::{Outcome, ShellCtx};
use crate::i18n::tr;
use crate::palette::Theme;
use crate::remote::{Probe, SshDest, Target};
use crate::session::{GridSize, RemoteSpawn, Session, SpawnOptions};
use crate::settings::Settings;
use crate::view::{InputContext, PaintOptions, ViewState};

pub const PANEL_ID: &str = "terminal";
/// Sessions kept per pane (the visible one plus recently used places).
pub const MAX_SESSIONS: usize = 4;
/// How long checking a container may take.
const PROBE_TIMEOUT: Duration = Duration::from_secs(6);
/// How long `ssh -G` may take (it only reads the local config).
const SSH_RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);
/// Font family of the bold terminal face we register (SF Mono, wght 700).
pub const BOLD_FAMILY: &str = "manta-terminal-bold";
const SF_MONO: &str = "/System/Library/Fonts/SFNSMono.ttf";

/// Register a bold SF Mono for the terminal (off the UI thread). The view
/// only uses it if its advance matches the app's monospace font.
fn install_bold_font(ctx: egui::Context) {
    std::thread::spawn(move || {
        let Ok(bytes) = std::fs::read(SF_MONO) else {
            return;
        };
        let mut data = egui::FontData::from_owned(bytes);
        data.tweak.coords = egui::epaint::text::VariationCoords::new([(b"wght", 700.0)]);
        ctx.add_font(egui::epaint::text::FontInsert {
            name: BOLD_FAMILY.into(),
            data,
            families: vec![egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Name(BOLD_FAMILY.into()),
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        });
        ctx.request_repaint();
    });
}
/// The note shown on remote panes, in the app's language.
pub fn remote_note() -> &'static str {
    tr!("term.remote_note")
}

/// One terminal panel instance (one per pane).
#[derive(Default)]
struct PaneTerm {
    session: Option<Session>,
    view: ViewState,
    /// egui pass number of the last frame this panel was drawn in.
    last_pass: Option<u64>,
    focus_request: bool,
    /// Spawn failure, shown instead of the terminal.
    error: Option<String>,
    /// The local folder last forwarded to the session.
    last_local: Option<Option<PathBuf>>,
    /// Latest file action, shown as an overlay when the shell cannot print
    /// it (no shell integration).
    notice: Option<Notice>,
    /// Commands to run (`PluginEvent::RunInTerminal`) that arrived before
    /// the session started; handed to it once it exists.
    queued_runs: Vec<(String, Option<PathBuf>)>,
    /// Which place `session` belongs to ([`Target::key`]).
    key: Option<String>,
    /// Sessions of other places of this pane, most recently used first.
    parked: Vec<(String, Session)>,
    /// Checking a container before opening a shell in it.
    probe: Option<ProbeJob>,
    /// An `ssh` running in this (local) shell and the server it went to.
    ssh: Option<SshOffer>,
}

/// An interactive `ssh` typed in a local terminal: resolved in the
/// background with `ssh -G`, then offered as a button that opens the server
/// in the pane over SFTP.
struct SshOffer {
    /// The command line the shell reported.
    command: String,
    rx: Option<std::sync::mpsc::Receiver<Option<SshDest>>>,
    /// The server, once resolved (`None`: not an ssh shell, or unresolved).
    dest: Option<SshDest>,
    /// The button was clicked: don't offer it again for this command.
    used: bool,
}

/// Keep `offer` in step with the command running in the shell; returns
/// the server to offer, if any.
fn ssh_offer(
    offer: &mut Option<SshOffer>,
    running: Option<String>,
    handle: Option<&PluginHandle>,
) -> Option<SshDest> {
    let Some(command) = running else {
        *offer = None;
        return None;
    };
    if offer.as_ref().is_none_or(|o| o.command != command) {
        let rx = remote::ssh_shell_args(&command).map(|args| {
            let (tx, rx) = std::sync::mpsc::channel();
            let handle = handle.cloned();
            std::thread::spawn(move || {
                let _ = tx.send(remote::resolve_ssh(&args, SSH_RESOLVE_TIMEOUT));
                if let Some(h) = handle {
                    h.request_repaint();
                }
            });
            rx
        });
        *offer = Some(SshOffer {
            command,
            rx,
            dest: None,
            used: false,
        });
    }
    let o = offer.as_mut()?;
    if let Some(rx) = &o.rx {
        if let Ok(d) = rx.try_recv() {
            o.dest = d;
            o.rx = None;
        }
    }
    o.dest.clone().filter(|_| !o.used)
}

/// A background [`remote::probe`] of one target.
struct ProbeJob {
    key: String,
    rx: std::sync::mpsc::Receiver<Probe>,
    result: Option<Probe>,
}

/// Switch the visible item of a pane to `key`: the current one (if any) is
/// parked most-recent-first, an earlier one for `key` comes back, and items
/// beyond `max` in total are returned for disposal (least recently used).
fn lru_switch<T>(
    current: Option<(String, T)>,
    parked: &mut Vec<(String, T)>,
    key: &str,
    max: usize,
) -> (Option<T>, Vec<T>) {
    if let Some(cur) = current {
        parked.insert(0, cur);
    }
    let restored = parked
        .iter()
        .position(|(k, _)| k == key)
        .map(|i| parked.remove(i).1);
    let keep = max.saturating_sub(1);
    let evicted = if parked.len() > keep {
        parked.drain(keep..).map(|(_, v)| v).collect()
    } else {
        Vec::new()
    };
    (restored, evicted)
}

/// The folder the shell of `target` should be in for this pane.
fn shell_dir(pane: &PaneState, target: &Target) -> Option<PathBuf> {
    match target {
        Target::Local => pane.local_path.clone(),
        Target::NoShell => None,
        _ => Some(PathBuf::from(pane.path.as_str())),
    }
}

/// A file action shown on top of the terminal (shells without integration).
#[derive(Debug, Clone)]
struct Notice {
    id: u64,
    command: String,
    result: Option<(Outcome, String)>,
    until: Instant,
}

/// Which terminals show the file action `a`, given the running shells and
/// their (canonical) directories: the one below the pane the action started
/// in, plus any whose shell sits in the destination folder of a copy, move
/// or sync (so a drag from left to right shows in both terminals when the
/// right one is in the target).
pub fn route(a: &FileAction, shells: &[(PaneId, Option<PathBuf>)]) -> Vec<PaneId> {
    let dest_canon = match (a.kind, &a.destination) {
        (FileActionKind::Copy | FileActionKind::Move | FileActionKind::Sync, Some(d))
            if a.dst_local =>
        {
            let p = PathBuf::from(d.as_str());
            Some(std::fs::canonicalize(&p).unwrap_or(p))
        }
        _ => None,
    };
    shells
        .iter()
        .filter(|(pane, cwd)| {
            *pane == a.pane || (dest_canon.is_some() && cwd.as_ref() == dest_canon.as_ref())
        })
        .map(|(pane, _)| *pane)
        .collect()
}

/// How long an overlay notice stays.
const NOTICE_TIME: Duration = Duration::from_secs(6);
/// Unique spool file names within this process.
static SPOOL_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TerminalPlugin {
    handle: Option<PluginHandle>,
    settings: Settings,
    panes: HashMap<PaneId, PaneTerm>,
    /// Which terminals showed the start of a running file action (its
    /// result goes to the same ones).
    routes: HashMap<u64, Vec<PaneId>>,
    /// Renders and tests: how to start a container / server shell instead
    /// of `docker` / `kubectl` / `ssh` (see [`TerminalPlugin::with_remote_override`]).
    remote_override: Option<RemoteOverride>,
}

/// Starts something else for a container / server pane (renders and
/// tests: a local `sh` that prints a scripted session). No probe runs.
pub type RemoteOverride = Arc<dyn Fn(&Target, &PaneState) -> RemoteSpawn + Send + Sync>;

impl TerminalPlugin {
    pub fn new() -> Self {
        TerminalPlugin {
            handle: None,
            settings: Settings::default(),
            panes: HashMap::new(),
            routes: HashMap::new(),
            remote_override: None,
        }
    }

    /// Renders and tests: container / server panes run `f`'s command
    /// instead of the real CLI, without probing the target first.
    #[doc(hidden)]
    pub fn with_remote_override(mut self, f: RemoteOverride) -> Self {
        self.remote_override = Some(f);
        self
    }

    /// The terminals that show `a` (see [`route`]).
    fn route(&self, a: &FileAction) -> Vec<PaneId> {
        let mut shells: Vec<(PaneId, Option<PathBuf>)> = self
            .panes
            .iter()
            .filter_map(|(pane, pt)| {
                let s = pt.session.as_ref()?;
                Some((*pane, s.follow.shell_cwd.as_ref().map(|d| d.canon.clone())))
            })
            .collect();
        shells.sort_by_key(|(p, _)| *p);
        route(a, &shells)
    }

    fn file_action(&mut self, a: &FileAction) {
        if !self.settings.show_file_actions {
            return;
        }
        let targets = if a.is_finished() {
            self.routes.remove(&a.id).unwrap_or_else(|| self.route(a))
        } else {
            if self.routes.len() > 1000 {
                self.routes.clear();
            }
            let t = self.route(a);
            self.routes.insert(a.id, t.clone());
            t
        };
        for pane in targets {
            if let Some(s) = self.panes.get_mut(&pane).and_then(|p| p.session.as_mut()) {
                s.push_action(a.clone());
            }
        }
    }

    /// Type `command` into the terminal below `pane` once its shell is idle
    /// (the host opened the panel; a session that doesn't exist yet gets it
    /// when it starts).
    fn run_in_terminal(&mut self, pane: PaneId, command: &str, cwd: Option<PathBuf>) {
        let exited = self
            .panes
            .get(&pane)
            .and_then(|p| p.session.as_ref())
            .is_some_and(|s| s.has_exited());
        if exited {
            // "Press Enter to restart": start a fresh shell for it.
            self.restart(pane);
        }
        let pt = self.panes.entry(pane).or_default();
        match pt.session.as_mut() {
            Some(s) => s.push_run(command, cwd.as_deref()),
            None => pt.queued_runs.push((command.to_string(), cwd)),
        }
        if let Some(h) = &self.handle {
            h.request_repaint();
        }
    }

    fn save_settings(&self) {
        if let Some(h) = &self.handle {
            if let Err(e) = self.settings.save(h.data_dir()) {
                h.send(HostRequest::Toast {
                    level: ToastLevel::Error,
                    text: tr!("term.toast.settings_not_saved", error = e),
                });
            }
        }
    }

    fn shell_path(&self) -> PathBuf {
        let s = self.settings.shell.trim();
        if s.is_empty() {
            session::default_shell()
        } else {
            PathBuf::from(s)
        }
    }

    fn spawn(
        &self,
        pane: PaneId,
        cwd: PathBuf,
        size: GridSize,
        cell_px: (u16, u16),
        theme: Theme,
        remote: Option<RemoteSpawn>,
    ) -> std::io::Result<Session> {
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| std::io::Error::other("plugin disabled"))?;
        let h = handle.clone();
        let shell = self.shell_path();
        if remote.is_none() && !shell.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("shell {} not found", shell.display()),
            ));
        }
        Session::spawn(SpawnOptions {
            shell,
            cwd,
            size,
            cell_px,
            scrollback: self.settings.scrollback_lines,
            shim_root: handle.data_dir().join("shell-integration"),
            theme,
            waker: Arc::new(move || h.request_repaint()),
            spool: Some(handle.data_dir().join("spool").join(format!(
                "{}-{}-{}.txt",
                std::process::id(),
                pane.key(),
                SPOOL_COUNTER.fetch_add(1, Ordering::Relaxed)
            ))),
            zdotdir: None,
            remote,
        })
    }

    /// Show the session of `target` in `pane`: park the current one, bring
    /// back an earlier one for this place, and drop the least recently used
    /// beyond [`MAX_SESSIONS`] (they are hung up cleanly).
    fn switch_target(&mut self, pane: PaneId, target: &Target) {
        let key = target.key();
        let pt = self.panes.entry(pane).or_default();
        if pt.key.as_deref() == Some(key.as_str()) {
            return;
        }
        let current = match (pt.session.take(), pt.key.take()) {
            (Some(old), Some(old_key)) if !old.has_exited() => Some((old_key, old)),
            _ => None,
        };
        // Evicted sessions are dropped here: their Drop hangs them up.
        let (restored, _evicted) = lru_switch(current, &mut pt.parked, &key, MAX_SESSIONS);
        pt.session = restored;
        pt.key = Some(key);
        // Forward the pane's folder to the (restored) session next frame.
        pt.last_local = None;
        pt.error = None;
        pt.notice = None;
    }

    /// The result of checking `target` (started in the background on first
    /// call); `None` while it runs.
    fn poll_probe(&mut self, pane: PaneId, target: &Target) -> Option<Probe> {
        let key = target.key();
        let handle = self.handle.clone();
        let pt = self.panes.entry(pane).or_default();
        match &mut pt.probe {
            Some(job) if job.key == key => {
                if job.result.is_none() {
                    if let Ok(r) = job.rx.try_recv() {
                        job.result = Some(r);
                    }
                }
                job.result.clone()
            }
            _ => {
                let (tx, rx) = std::sync::mpsc::channel();
                let t = target.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(remote::probe(&t, PROBE_TIMEOUT));
                    if let Some(h) = handle {
                        h.request_repaint();
                    }
                });
                pt.probe = Some(ProbeJob {
                    key,
                    rx,
                    result: None,
                });
                None
            }
        }
    }

    fn remote_spawn(target: &Target, pane: &PaneState) -> Option<RemoteSpawn> {
        let program = remote::find_program(target.program()?)?;
        Some(RemoteSpawn {
            program,
            args: target.args(pane.path.as_str()),
            osc_host: target.osc_host(),
            start_dir: pane.path.as_str().to_string(),
        })
    }

    /// Drop a pane's session (it is hung up in the background) so the next
    /// frame starts a fresh one.
    fn restart(&mut self, pane: PaneId) {
        if let Some(pt) = self.panes.get_mut(&pane) {
            pt.session = None;
            pt.error = None;
            pt.probe = None;
            pt.focus_request = true;
        }
    }

    fn clear(&mut self, pane: PaneId) {
        if let Some(s) = self.panes.get_mut(&pane).and_then(|p| p.session.as_mut()) {
            let idle = s.is_idle();
            {
                let mut sh = s.lock();
                sh.term.grid_mut().clear_history();
                sh.term.selection = None;
                sh.term
                    .scroll_display(alacritty_terminal::grid::Scroll::Bottom);
            }
            if idle {
                // Ctrl-L: the shell clears the screen and redraws its prompt.
                s.write(b"\x0c");
            }
        }
    }

    /// The pane is gone: hang up its shell and the parked ones (dropping a
    /// session hangs it up without blocking) and forget its state.
    fn close_pane(&mut self, pane: PaneId) {
        if let Some(mut pt) = self.panes.remove(&pane) {
            let sessions: Vec<Session> = pt
                .session
                .take()
                .into_iter()
                .chain(pt.parked.drain(..).map(|(_, s)| s))
                .collect();
            log::info!(
                "terminal: pane {pane} closed, hanging up {} session(s)",
                sessions.len()
            );
            for s in &sessions {
                s.hangup();
            }
        }
        for targets in self.routes.values_mut() {
            targets.retain(|p| *p != pane);
        }
    }

    fn shutdown_all(&mut self) {
        let mut sessions: Vec<Session> = self
            .panes
            .values_mut()
            .flat_map(|p| {
                p.session
                    .take()
                    .into_iter()
                    .chain(p.parked.drain(..).map(|(_, s)| s))
            })
            .collect();
        for s in &sessions {
            s.hangup();
        }
        for s in &mut sessions {
            let detached = s.shutdown(Duration::from_millis(300));
            if detached > 0 {
                log::debug!(
                    "terminal: {detached} reader thread(s) detached (PTY held by a background job)"
                );
            }
        }
    }

    fn draw(&mut self, pane: &PaneState, ui: &mut egui::Ui) {
        let Some(handle) = self.handle.clone() else {
            return;
        };
        let settings = self.settings.clone();
        let theme = Theme::from_visuals(ui.visuals());
        let font = FontId::monospace(settings.font_size);
        let bold_font = FontId::new(
            settings.font_size,
            egui::FontFamily::Name(BOLD_FAMILY.into()),
        );
        let full = ui.available_rect_before_wrap();
        if full.width() < 20.0 || full.height() < 10.0 {
            return;
        }
        let id = egui::Id::unique(("manta-terminal", pane.pane));
        ui.advance_cursor_after_rect(full);
        let target = Target::from_pane(pane);
        // No shell possible here (S3, FTP, archives): the last session stays,
        // dimmed, below a calm note.
        let remote = target == Target::NoShell;
        if !remote {
            self.switch_target(pane.pane, &target);
        }
        let rect = if remote {
            Rect::from_min_max(full.min + Vec2::new(0.0, BANNER_H), full.max)
        } else {
            full
        };
        let response = ui.interact(rect, id, Sense::click_and_drag());

        let needs_spawn = {
            let pt = self.panes.entry(pane.pane).or_default();
            pt.last_pass = Some(ui.ctx().cumulative_pass_nr());
            pt.session.is_none() && pt.error.is_none() && !remote
        };
        let metrics = self
            .panes
            .get_mut(&pane.pane)
            .map(|pt| pt.view.metrics(ui.ctx(), &font))
            .unwrap_or(view::Metrics {
                cell_w: 7.0,
                advance: 7.0,
                cell_h: 15.0,
                row_h: 14.0,
                ppp: 1.0,
            });
        let size = metrics.grid_size(rect);
        if needs_spawn {
            let result = if target == Target::Local {
                let cwd = pane
                    .local_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("/"));
                Some(self.spawn(pane.pane, cwd, size, metrics.cell_px(), theme, None))
            } else if let Some(f) = self.remote_override.clone() {
                let r = f(&target, pane);
                Some(self.spawn(
                    pane.pane,
                    PathBuf::from("/"),
                    size,
                    metrics.cell_px(),
                    theme,
                    Some(r),
                ))
            } else {
                match self.poll_probe(pane.pane, &target) {
                    Some(Probe::Ready) => Some(match Self::remote_spawn(&target, pane) {
                        Some(r) => self.spawn(
                            pane.pane,
                            PathBuf::from("/"),
                            size,
                            metrics.cell_px(),
                            theme,
                            Some(r),
                        ),
                        None => Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "CLI not found",
                        )),
                    }),
                    _ => None,
                }
            };
            if let Some(result) = result {
                let pt = self.panes.entry(pane.pane).or_default();
                match result {
                    Ok(s) => {
                        pt.session = Some(s);
                        pt.last_local = Some(shell_dir(pane, &target));
                        pt.view.blink_epoch = ui.input(|i| i.time);
                    }
                    Err(e) => pt.error = Some(tr!("term.error.start", error = e)),
                }
            }
        }
        let pt = self.panes.entry(pane.pane).or_default();
        let painter = ui.painter_at(full);

        if let Some(err) = &pt.error {
            painter.rect_filled(full, 0.0, theme.bg);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                err,
                FontId::proportional(12.0),
                theme.ansi[1],
            );
            if response.clicked() {
                pt.error = None; // retry
            }
            return;
        }
        let Some(session) = pt.session.as_mut() else {
            painter.rect_filled(full, 0.0, theme.bg);
            if remote {
                // No shell here and none from before to show.
                message(&painter, full, remote_note(), None, &theme);
                return;
            }
            // Waiting for (or stopped by) the container check.
            let probe = pt.probe.as_ref().and_then(|j| j.result.clone());
            let label = target.label();
            let (title, hint) = match &probe {
                None | Some(Probe::Ready) => (tr!("term.remote.connecting", target = label), None),
                Some(Probe::Stopped) => (
                    tr!("term.remote.stopped", name = label),
                    Some(tr!("term.remote.stopped_hint").to_string()),
                ),
                Some(Probe::NoCli(p)) => (
                    tr!("term.remote.no_cli", program = p),
                    Some(tr!("term.remote.no_cli_hint", program = p)),
                ),
                Some(Probe::Failed(e)) => {
                    let e = e.trim();
                    let hint = if e.is_empty() {
                        tr!("term.remote.retry_hint").to_string()
                    } else {
                        format!("{e}\n{}", tr!("term.remote.retry_hint"))
                    };
                    (tr!("term.remote.failed", target = label), Some(hint))
                }
            };
            message(&painter, full, &title, hint.as_deref(), &theme);
            if probe.is_some() && response.clicked() {
                pt.probe = None; // check again
                ui.ctx().request_repaint();
            }
            return;
        };

        // Keep the follow logic in sync with the pane (covers sessions
        // started after the DirectoryChanged event).
        let dir = shell_dir(pane, &target);
        if pt.last_local.as_ref() != Some(&dir) {
            session.pane_changed(dir.as_deref());
            pt.last_local = Some(dir);
        }
        for (command, cwd) in pt.queued_runs.drain(..) {
            session.push_run(&command, cwd.as_deref());
        }
        session.set_theme(theme);
        // Debounced: a splitter/panel drag must not SIGWINCH the shell on
        // every frame (each one makes zsh redraw its prompt).
        if let Some(wait) = session.request_resize(size, metrics.cell_px()) {
            ui.ctx().request_repaint_after(wait);
        }
        // Paint the grid the PTY actually has, flush with the right edge.
        let metrics = metrics.fit(rect, session.size().cols as usize);
        if let Some(path) = session.tick(settings.pane_follows_terminal && !remote) {
            if let Some(s) = path.to_str() {
                let path = VfsPath::new(s);
                if session.is_remote() {
                    // Only while the pane still shows this container/server.
                    handle.send(HostRequest::NavigateIn {
                        pane: pane.pane,
                        remote: pane.remote.clone(),
                        path,
                    });
                } else {
                    handle.send(HostRequest::Navigate {
                        pane: pane.pane,
                        path,
                    });
                }
            }
        }
        // An `ssh` typed in the local shell: offer its server in the pane.
        let running = (target == Target::Local)
            .then(|| session.running_command())
            .flatten();
        let ssh_dest = ssh_offer(&mut pt.ssh, running, Some(&handle));
        for a in session.take_fallback() {
            // Without integration the shell's cwd is a guess: the pane's.
            let cwd = session
                .follow
                .shell_cwd
                .as_ref()
                .or(session.follow.pane_dir.as_ref());
            let ctx = ShellCtx::new(cwd, session.kind);
            let until = Instant::now() + NOTICE_TIME;
            match &mut pt.notice {
                Some(n) if n.id == a.id && a.is_finished() => {
                    n.result = actions::result_text(&a, None);
                    n.until = until;
                }
                _ => {
                    let result = actions::result_text(&a, None);
                    pt.notice = Some(Notice {
                        id: a.id,
                        command: actions::command(&a, ctx),
                        result,
                        until,
                    });
                }
            }
        }

        // Focus.
        let mut focused = ui.memory(|m| m.has_focus(id));
        if response.clicked() || response.drag_started() || std::mem::take(&mut pt.focus_request) {
            ui.memory_mut(|m| m.request_focus(id));
            focused = true;
        } else if focused && ui.input(|i| i.pointer.any_pressed()) && !response.hovered() {
            ui.memory_mut(|m| m.surrender_focus(id));
            focused = false;
        }
        if focused {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                )
            });
        }
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }

        // Input.
        let exited = session.has_exited();
        let out = {
            let mut sh = session.lock();
            view::handle_input(
                ui,
                &mut sh.term,
                &mut pt.view,
                InputContext {
                    focused,
                    response: &response,
                    rect,
                    metrics,
                    option_as_meta: settings.option_as_meta,
                    exited,
                },
            )
        };
        if let Some(text) = out.copy {
            ui.ctx().copy_text(text);
        }
        if let Some(text) = session.take_clipboard() {
            ui.ctx().copy_text(text);
        }
        session.write(&out.bytes);
        if out.submitted {
            session.user_submitted();
        }
        let pending_hint = session.follow.has_pending() && session.is_busy();
        let echo_waiting = if session.is_busy() {
            session.echo_waiting()
        } else {
            0
        };
        let run_waiting = if session.is_busy() {
            session.next_run().map(short_command)
        } else {
            None
        };
        let pending_name = session
            .follow
            .pending
            .as_ref()
            .and_then(|d| d.raw.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "/".into());

        // Paint.
        let now = ui.input(|i| i.time);
        let seq = session.lock().output_seq;
        if seq != pt.view.last_output_seq {
            pt.view.last_output_seq = seq;
            pt.view.blink_epoch = now;
        }
        let blink_on = if focused {
            let phase = (now - pt.view.blink_epoch).rem_euclid(1.06);
            let next = if phase < 0.53 {
                0.53 - phase
            } else {
                1.06 - phase
            };
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(next.max(0.01)));
            phase < 0.53
        } else {
            true
        };
        let bold_font = pt.view.usable_bold(ui.ctx(), &font, &bold_font);
        {
            let sh = session.lock();
            view::paint(
                ui,
                &painter,
                rect,
                &sh.term,
                &sh.floats,
                &mut pt.view,
                &metrics,
                &PaintOptions {
                    theme,
                    font: font.clone(),
                    bold_font,
                    focused,
                    blink_on,
                    dimmed: remote,
                },
            );
        }

        let restart = out.restart;
        let clear = out.clear;
        if remote {
            banner(
                &painter,
                Rect::from_min_max(full.min, egui::pos2(full.max.x, rect.min.y)),
                remote_note(),
                &theme,
            );
        } else if let Some(cmd) = &run_waiting {
            pill(&painter, rect, &tr!("term.pill.run", command = cmd), &theme);
        } else if pending_hint || echo_waiting > 0 {
            let actions = match echo_waiting {
                0 => String::new(),
                n => tr!("term.pill.actions", count = n),
            };
            let text = match (pending_hint, echo_waiting) {
                (true, 0) => tr!("term.pill.cd", name = pending_name),
                (true, _) => tr!(
                    "term.pill.cd_and_show",
                    name = pending_name,
                    actions = actions
                ),
                _ => tr!("term.pill.show", actions = actions),
            };
            pill(&painter, rect, &text, &theme);
        }
        if let Some(d) = &ssh_dest {
            let below_pill = run_waiting.is_some() || pending_hint || echo_waiting > 0;
            let text = tr!("term.ssh.open", target = d.label());
            if ssh_button(ui, rect, id.with("ssh"), &text, &theme, below_pill).clicked() {
                handle.send(HostRequest::OpenSftp {
                    pane: pane.pane,
                    user: d.user.clone(),
                    host: d.host.clone(),
                    port: d.port,
                });
                if let Some(o) = pt.ssh.as_mut() {
                    o.used = true;
                }
            }
        }
        if let Some(n) = &pt.notice {
            let now = Instant::now();
            if now >= n.until {
                pt.notice = None;
            } else {
                notice(
                    &painter,
                    rect,
                    n,
                    &theme,
                    pending_hint || echo_waiting > 0 || run_waiting.is_some(),
                );
                ui.ctx().request_repaint_after(n.until - now);
            }
        }
        if restart {
            self.restart(pane.pane);
            ui.ctx().request_repaint();
        }
        if clear {
            self.clear(pane.pane);
        }
    }
}

/// A command shortened for the status pill.
fn short_command(cmd: &str) -> String {
    let line = cmd.lines().next().unwrap_or("").trim();
    if line.chars().count() > 40 {
        line.chars().take(39).collect::<String>() + "…"
    } else {
        line.to_string()
    }
}

/// A calm, centred message (title and an optional smaller hint).
fn message(painter: &egui::Painter, rect: Rect, title: &str, hint: Option<&str>, theme: &Theme) {
    let title_g = painter.layout(
        title.to_owned(),
        FontId::proportional(13.0),
        palette::mix(theme.fg, theme.bg, 0.25),
        rect.width() - 48.0,
    );
    let hint_g = hint.map(|h| {
        painter.layout(
            h.to_owned(),
            FontId::proportional(11.5),
            muted(theme),
            rect.width() - 48.0,
        )
    });
    let gap = 6.0;
    let h = title_g.size().y + hint_g.as_ref().map_or(0.0, |g| g.size().y + gap);
    let top = rect.center().y - h / 2.0;
    painter.galley(
        egui::pos2(rect.center().x - title_g.size().x / 2.0, top),
        title_g.clone(),
        theme.fg,
    );
    if let Some(g) = hint_g {
        let y = top + title_g.size().y + gap;
        painter.galley(
            egui::pos2(rect.center().x - g.size().x / 2.0, y),
            g,
            theme.fg,
        );
    }
}

fn muted(theme: &Theme) -> Color32 {
    palette::mix(theme.fg, theme.bg, 0.45)
}

/// A calm full-width note across the top of the panel.
const BANNER_H: f32 = 24.0;

fn banner(painter: &egui::Painter, r: Rect, text: &str, theme: &Theme) {
    let font = FontId::proportional(12.0);
    let galley = painter.layout_no_wrap(text.to_owned(), font, muted(theme));
    painter.rect_filled(r, 0.0, palette::mix(theme.bg, theme.fg, 0.06));
    painter.line_segment(
        [r.left_bottom(), r.right_bottom()],
        egui::Stroke::new(1.0, palette::mix(theme.bg, theme.fg, 0.12)),
    );
    painter.galley(r.center() - galley.size() / 2.0, galley, muted(theme));
}

/// A small status pill in the top-right corner.
fn pill(painter: &egui::Painter, rect: Rect, text: &str, theme: &Theme) {
    let font = FontId::proportional(11.0);
    let color = palette::mix(theme.fg, theme.bg, 0.25);
    let galley = painter.layout_no_wrap(text.to_owned(), font, color);
    let pad = Vec2::new(8.0, 3.0);
    let size = galley.size() + pad * 2.0;
    let r = Rect::from_min_size(rect.right_top() + Vec2::new(-size.x - 8.0, 6.0), size);
    painter.rect_filled(
        r,
        CornerRadius::same(99),
        palette::mix(theme.bg, theme.fg, 0.10),
    );
    painter.rect_stroke(
        r,
        CornerRadius::same(99),
        egui::Stroke::new(1.0, palette::mix(theme.bg, theme.fg, 0.18)),
        egui::StrokeKind::Inside,
    );
    painter.galley(r.min + pad, galley, color);
}

/// The "Open user@host in the pane" button in the top-right corner (below
/// the status pill when one shows).
fn ssh_button(
    ui: &mut egui::Ui,
    rect: Rect,
    id: egui::Id,
    text: &str,
    theme: &Theme,
    below_pill: bool,
) -> egui::Response {
    let font = FontId::proportional(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(format!("{text}  ↗"), font, theme.bg);
    let pad = Vec2::new(10.0, 4.0);
    let size = galley.size() + pad * 2.0;
    let top = if below_pill { 34.0 } else { 6.0 };
    let r = Rect::from_min_size(rect.right_top() + Vec2::new(-size.x - 8.0, top), size);
    let resp = ui
        .interact(r, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let accent = theme.ansi[4];
    let fill = if resp.hovered() {
        palette::mix(accent, theme.fg, 0.15)
    } else {
        accent
    };
    let painter = ui.painter();
    painter.rect_filled(r, CornerRadius::same(99), fill);
    painter.galley(r.min + pad, galley, theme.bg);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, text));
    resp
}

/// The latest file action as a small card in the top-right corner (shells
/// that cannot print it): the command, and the result below once finished.
fn notice(painter: &egui::Painter, rect: Rect, n: &Notice, theme: &Theme, below_pill: bool) {
    let mono = FontId::monospace(11.0);
    let accent = theme.ansi[6];
    let max_w = (rect.width() * 0.6).max(240.0).min(rect.width() - 32.0);
    let row = |parts: Vec<(String, Color32)>| {
        let mut job = egui::text::LayoutJob::default();
        for (t, c) in parts {
            job.append(&t, 0.0, egui::TextFormat::simple(mono.clone(), c));
        }
        job.wrap.max_width = max_w;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        painter.layout_job(job)
    };
    let cmd = row(vec![("→ ".into(), accent), (n.command.clone(), accent)]);
    let result = n.result.as_ref().map(|(o, text)| {
        let c = match o {
            Outcome::Ok => theme.ansi[2],
            Outcome::Failed => theme.ansi[1],
            Outcome::Cancelled => theme.ansi[3],
        };
        row(vec![
            (format!("{} ", actions::glyph(*o)), c),
            (text.clone(), muted(theme)),
        ])
    });
    let pad = Vec2::new(10.0, 6.0);
    let gap = 3.0;
    let w = cmd
        .size()
        .x
        .max(result.as_ref().map_or(0.0, |r| r.size().x));
    let h = cmd.size().y + result.as_ref().map_or(0.0, |r| r.size().y + gap);
    let size = Vec2::new(w, h) + pad * 2.0;
    let y = if below_pill { 34.0 } else { 6.0 };
    let r = Rect::from_min_size(
        egui::pos2(rect.right() - size.x - 8.0, rect.top() + y),
        size,
    );
    painter.rect_filled(
        r,
        CornerRadius::same(6),
        palette::mix(theme.bg, theme.fg, 0.08),
    );
    painter.rect_stroke(
        r,
        CornerRadius::same(6),
        egui::Stroke::new(1.0, palette::mix(theme.bg, theme.fg, 0.18)),
        egui::StrokeKind::Inside,
    );
    let cmd_h = cmd.size().y;
    painter.galley(r.min + pad, cmd, theme.fg);
    if let Some(res) = result {
        painter.galley(r.min + pad + Vec2::new(0.0, cmd_h + gap), res, theme.fg);
    }
}

/// Test hooks (integration tests and render checks).
#[doc(hidden)]
impl TerminalPlugin {
    /// The panes the terminal keeps state for (tests).
    pub fn debug_panes(&self) -> Vec<PaneId> {
        let mut v: Vec<PaneId> = self.panes.keys().copied().collect();
        v.sort();
        v
    }

    pub fn debug_write(&self, pane: PaneId, bytes: &[u8]) {
        if let Some(s) = self.panes.get(&pane).and_then(|p| p.session.as_ref()) {
            s.write(bytes);
        }
    }

    pub fn debug_screen(&self, pane: PaneId) -> Option<String> {
        self.panes
            .get(&pane)?
            .session
            .as_ref()
            .map(|s| s.lock().screen_text())
    }

    /// Process ids of the pane's shells, the shown one first (tests).
    pub fn debug_pids(&self, pane: PaneId) -> Vec<i32> {
        let Some(pt) = self.panes.get(&pane) else {
            return Vec::new();
        };
        pt.session
            .iter()
            .chain(pt.parked.iter().map(|(_, s)| s))
            .filter_map(|s| s.pid())
            .collect()
    }

    pub fn debug_is_idle(&self, pane: PaneId) -> bool {
        self.panes
            .get(&pane)
            .and_then(|p| p.session.as_ref())
            .is_some_and(|s| s.is_idle())
    }

    pub fn debug_runs_waiting(&self, pane: PaneId) -> usize {
        self.panes.get(&pane).map_or(0, |p| {
            p.queued_runs.len() + p.session.as_ref().map_or(0, |s| s.runs_waiting())
        })
    }

    pub fn debug_has_pending(&self, pane: PaneId) -> bool {
        self.panes
            .get(&pane)
            .and_then(|p| p.session.as_ref())
            .is_some_and(|s| s.follow.has_pending())
    }

    /// Key of the visible session's place and the number of parked ones.
    pub fn debug_sessions(&self, pane: PaneId) -> (Option<String>, Vec<String>) {
        match self.panes.get(&pane) {
            Some(pt) => (
                pt.session.as_ref().and(pt.key.clone()),
                pt.parked.iter().map(|(k, _)| k.clone()).collect(),
            ),
            None => (None, Vec::new()),
        }
    }

    /// The finished container check of the pane, if any (`"Stopped"`, …).
    pub fn debug_probe(&self, pane: PaneId) -> Option<String> {
        self.panes
            .get(&pane)?
            .probe
            .as_ref()?
            .result
            .as_ref()
            .map(|p| format!("{p:?}"))
    }

    /// The visible session's shell directory (as the shell reported it).
    pub fn debug_shell_cwd(&self, pane: PaneId) -> Option<String> {
        let s = self.panes.get(&pane)?.session.as_ref()?;
        s.follow
            .shell_cwd
            .as_ref()
            .map(|d| d.raw.to_string_lossy().into_owned())
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
}

impl Default for TerminalPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TerminalPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "manta.terminal".into(),
            name: tr!("term.info.name").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: tr!("term.info.description").into(),
            author: "Manta".into(),
            builtin: true,
        }
    }

    fn enable(&mut self, handle: PluginHandle) {
        self.settings = Settings::load(handle.data_dir());
        install_bold_font(handle.egui_ctx().clone());
        self.handle = Some(handle);
    }

    fn disable(&mut self) {
        self.shutdown_all();
        self.panes.clear();
        self.handle = None;
    }

    fn on_event(&mut self, event: &PluginEvent) {
        if let PluginEvent::LocaleChanged(_) = event {
            // Banners and pills are built every frame; the host asks for
            // commands and panels again. Redraw now.
            if let Some(h) = &self.handle {
                h.request_repaint();
            }
            return;
        }
        if let PluginEvent::FileAction(a) = event {
            self.file_action(a);
            return;
        }
        if let PluginEvent::PaneClosed(pane) = event {
            self.close_pane(*pane);
            return;
        }
        if let PluginEvent::RunInTerminal { pane, command, cwd } = event {
            self.run_in_terminal(
                *pane,
                command,
                cwd.as_ref().map(|c| PathBuf::from(c.as_str())),
            );
            return;
        }
        if let PluginEvent::DirectoryChanged(state) = event {
            let target = Target::from_pane(state);
            if let Some(pt) = self.panes.get_mut(&state.pane) {
                // Another place: the next frame switches sessions and
                // forwards the folder.
                let same =
                    target == Target::NoShell || pt.key.as_deref() == Some(target.key().as_str());
                if let (true, Some(s)) = (same, pt.session.as_mut()) {
                    let dir = shell_dir(state, &target);
                    if pt.last_local.as_ref() != Some(&dir) {
                        s.pane_changed(dir.as_deref());
                        pt.last_local = Some(dir);
                    }
                }
            }
        }
    }

    fn commands(&self) -> Vec<CommandSpec> {
        vec![
            CommandSpec {
                id: "toggle".into(),
                title: tr!("term.cmd.toggle").into(),
                shortcut: Some("⌃`".into()),
                context: CommandContext::Palette,
            },
            CommandSpec {
                id: "new_session".into(),
                title: tr!("term.cmd.new_session").into(),
                shortcut: None,
                context: CommandContext::Palette,
            },
            CommandSpec {
                id: "clear".into(),
                title: tr!("term.cmd.clear").into(),
                shortcut: None,
                context: CommandContext::Palette,
            },
        ]
    }

    fn run_command(&mut self, id: &str, pane: &PaneState) {
        match id {
            "toggle" => {
                let Some(h) = &self.handle else { return };
                // Focus the terminal the next time it is drawn (i.e. when the
                // toggle opened it).
                self.panes.entry(pane.pane).or_default().focus_request = true;
                h.send(HostRequest::TogglePanel {
                    panel: PANEL_ID.into(),
                    pane: Some(pane.pane),
                });
            }
            "new_session" => self.restart(pane.pane),
            "clear" => self.clear(pane.pane),
            _ => {}
        }
    }

    fn panels(&self) -> Vec<PanelSpec> {
        vec![PanelSpec {
            id: PANEL_ID.into(),
            title: tr!("term.panel.title").into(),
            placement: PanelPlacement::PaneBottom,
            default_open: true,
        }]
    }

    fn panel_ui(&mut self, panel: &str, pane: &PaneState, ui: &mut egui::Ui) {
        if panel == PANEL_ID {
            self.draw(pane, ui);
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let before = self.settings.clone();
        egui::Grid::new("manta-terminal-settings")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                let label = ui.label(tr!("term.settings.shell"));
                let hint = session::default_shell().display().to_string();
                ui.add(
                    egui::TextEdit::singleline(&mut self.settings.shell)
                        .hint_text(hint)
                        .desired_width(220.0),
                )
                .labelled_by(label.id);
                ui.end_row();

                let label = ui.label(tr!("term.settings.font_size"));
                ui.add(
                    egui::Slider::new(&mut self.settings.font_size, 9.0..=24.0)
                        .step_by(0.5)
                        .suffix(" pt")
                        .custom_formatter(|v, _| manta_plugin::i18n::format_decimal(v, 1))
                        .custom_parser(|s| s.trim().replace(',', ".").parse().ok()),
                )
                .labelled_by(label.id);
                ui.end_row();

                let label = ui.label(tr!("term.settings.scrollback"));
                ui.add(
                    egui::DragValue::new(&mut self.settings.scrollback_lines)
                        .range(settings::MIN_SCROLLBACK..=settings::MAX_SCROLLBACK)
                        .speed(100)
                        .suffix(tr!("term.settings.lines_suffix"))
                        .custom_formatter(
                            |v, _| manta_plugin::i18n::format_count(v.max(0.0) as u64),
                        )
                        .custom_parser(|s| {
                            s.chars()
                                .filter(char::is_ascii_digit)
                                .collect::<String>()
                                .parse()
                                .ok()
                        }),
                )
                .labelled_by(label.id);
                ui.end_row();

                ui.label("");
                ui.checkbox(
                    &mut self.settings.option_as_meta,
                    tr!("term.settings.option_meta"),
                )
                .on_hover_text(tr!("term.settings.option_meta.tip"));
                ui.end_row();

                ui.label("");
                ui.checkbox(
                    &mut self.settings.pane_follows_terminal,
                    tr!("term.settings.follow"),
                )
                .on_hover_text(tr!("term.settings.follow.tip"));
                ui.end_row();

                ui.label("");
                ui.checkbox(
                    &mut self.settings.show_file_actions,
                    tr!("term.settings.show_actions"),
                )
                .on_hover_text(tr!("term.settings.show_actions.tip"));
                ui.end_row();
            });
        if self.settings != before {
            self.settings = self.settings.clone().sanitized_keep_shell();
            if self.settings.scrollback_lines != before.scrollback_lines {
                let lines = self.settings.scrollback_lines;
                for s in self.panes.values().filter_map(|p| p.session.as_ref()) {
                    let mut sh = s.lock();
                    let config = alacritty_terminal::term::Config {
                        scrolling_history: lines,
                        ..Default::default()
                    };
                    sh.term.set_options(config);
                }
            }
            self.save_settings();
        }
        true
    }
}

impl Settings {
    /// Clamp numbers but keep the shell text as typed (trimming would fight
    /// the text field's cursor).
    fn sanitized_keep_shell(self) -> Settings {
        let shell = self.shell.clone();
        Settings {
            shell,
            ..self.sanitized()
        }
    }
}

impl Drop for TerminalPlugin {
    fn drop(&mut self) {
        // Hang up every shell (the sessions' own Drop does not block).
        self.panes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_wait_for_a_session() {
        let mut t = TerminalPlugin::new();
        let ev = PluginEvent::RunInTerminal {
            pane: PaneId::Right,
            command: "ls".into(),
            cwd: Some(VfsPath::new("/tmp")),
        };
        t.on_event(&ev);
        assert_eq!(t.debug_runs_waiting(PaneId::Right), 1);
        assert_eq!(t.debug_runs_waiting(PaneId::Left), 0);
        assert_eq!(
            t.panes[&PaneId::Right].queued_runs[0],
            ("ls".to_string(), Some(PathBuf::from("/tmp")))
        );
        assert_eq!(short_command("a\nb"), "a");
        assert_eq!(short_command(&"x".repeat(50)).chars().count(), 40);
    }

    #[test]
    fn session_lru() {
        let mut parked: Vec<(String, u32)> = Vec::new();
        // local(1) → docker:a: nothing to restore, local parked.
        let (r, ev) = lru_switch(Some(("local".into(), 1)), &mut parked, "docker:a", 4);
        assert_eq!((r, ev), (None, vec![]));
        // docker:a(2) → local: local comes back, docker:a parked.
        let (r, ev) = lru_switch(Some(("docker:a".into(), 2)), &mut parked, "local", 4);
        assert_eq!((r, ev), (Some(1), vec![]));
        assert_eq!(parked, vec![("docker:a".to_string(), 2)]);
        // Visit b, c, d: at most 4 alive in total, the oldest (a) goes.
        let (_, ev) = lru_switch(Some(("local".into(), 1)), &mut parked, "docker:b", 4);
        assert!(ev.is_empty());
        let (_, ev) = lru_switch(Some(("docker:b".into(), 3)), &mut parked, "docker:c", 4);
        assert!(ev.is_empty());
        let (_, ev) = lru_switch(Some(("docker:c".into(), 4)), &mut parked, "docker:d", 4);
        assert_eq!(ev, vec![2], "least recently used: docker:a");
        assert_eq!(
            parked.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
            ["docker:c", "docker:b", "local"]
        );
        // Back to local: restored from the parked list.
        let (r, _) = lru_switch(Some(("docker:d".into(), 5)), &mut parked, "local", 4);
        assert_eq!(r, Some(1));
    }

    #[test]
    fn routing() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = std::fs::canonicalize(tmp.path()).unwrap();
        let mut a = FileAction::new(
            1,
            PaneId::Left,
            FileActionKind::Copy,
            vec![VfsPath::new("/x/a")],
        );
        // Spelled differently from the canonical cwd (macOS /var → /private/var).
        a.destination = Some(VfsPath::new(&tmp.path().to_string_lossy()));
        let both = [
            (PaneId::Left, Some(PathBuf::from("/x"))),
            (PaneId::Right, Some(dest.clone())),
        ];
        assert_eq!(route(&a, &both), vec![PaneId::Left, PaneId::Right]);
        let elsewhere = [
            (PaneId::Left, Some(PathBuf::from("/x"))),
            (PaneId::Right, Some(PathBuf::from("/y"))),
        ];
        assert_eq!(route(&a, &elsewhere), vec![PaneId::Left]);
        // Only the right terminal is open: it still shows the drop.
        assert_eq!(route(&a, &both[1..]), vec![PaneId::Right]);
        // Remote destinations and deletes go to the source pane only.
        a.dst_local = false;
        assert_eq!(route(&a, &both), vec![PaneId::Left]);
        let mut d = FileAction::new(
            2,
            PaneId::Right,
            FileActionKind::Delete,
            vec![VfsPath::new("/y/a")],
        );
        d.destination = None;
        assert_eq!(route(&d, &both), vec![PaneId::Right]);
    }
}
