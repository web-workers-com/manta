//! Terminals inside containers and on servers.
//!
//! The terminal below a pane runs where the pane is: a local shell for local
//! folders, `docker exec` / `kubectl exec` for containers, `ssh` for SFTP
//! servers. S3, FTP and archives have no shell.
//!
//! **Shell integration without touching the target.** Nothing is written
//! into the container or onto the server. The exec'd command is
//! `sh -c <bootstrap>`, and the bootstrap script hands our prompt hooks to the
//! shell through a here-document on file descriptor 3:
//!
//! - bash: `exec bash --rcfile /dev/fd/3 -i 3<<…`. The rc file first loads
//!   the user's own `~/.bashrc` (SSH: the login files), then prepends a
//!   `PROMPT_COMMAND` function that prints OSC 7 (`file://<target-id><pwd>`)
//!   and OSC 133;A, adds `ignorespace` to `HISTCONTROL`, and (bash ≥ 4)
//!   binds the private keys that park and restore half-typed input.
//! - other `sh` (busybox ash, dash): `ENV=/dev/fd/3 exec sh -i 3<<…`. The ENV
//!   file wraps the existing `PS1` with the same invisible sequences inside
//!   `\[ \]`; ash expands `${PWD}` in `PS1` at every prompt.
//!
//! The user's prompt look stays: we add zero-width markers to their
//! `PS1`/`PROMPT_COMMAND` and replace nothing.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use manta_core::Protocol;
use manta_plugin::{PaneState, RemoteEndpoint};

/// Where the terminal of a pane runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A local folder.
    Local,
    Docker {
        container: String,
    },
    Kube {
        context: String,
        namespace: String,
        pod: String,
        container: String,
    },
    Ssh {
        user: String,
        host: String,
        port: u16,
    },
    /// A backend without a shell (S3, FTP, archives).
    NoShell,
}

impl Target {
    pub fn from_pane(pane: &PaneState) -> Target {
        if pane.local_path.is_some() {
            return Target::Local;
        }
        pane.remote
            .as_ref()
            .map(Target::from_endpoint)
            .unwrap_or(Target::NoShell)
    }

    pub fn from_endpoint(ep: &RemoteEndpoint) -> Target {
        match ep.protocol {
            Protocol::Docker if !ep.host.trim().is_empty() => Target::Docker {
                container: ep.host.trim().to_string(),
            },
            Protocol::Kubernetes if !ep.host.trim().is_empty() => {
                let (namespace, pod) = match ep.host.trim().split_once('/') {
                    Some((ns, pod)) => (ns.to_string(), pod.to_string()),
                    None => (String::new(), ep.host.trim().to_string()),
                };
                Target::Kube {
                    context: ep.context.trim().into(),
                    namespace,
                    pod,
                    container: ep.user.trim().into(),
                }
            }
            Protocol::Sftp if !ep.host.trim().is_empty() => Target::Ssh {
                user: ep.user.trim().into(),
                host: ep.host.trim().into(),
                port: ep.port,
            },
            _ => Target::NoShell,
        }
    }

    pub fn is_remote(&self) -> bool {
        matches!(
            self,
            Target::Docker { .. } | Target::Kube { .. } | Target::Ssh { .. }
        )
    }

    /// Stable key of the session for this target (one session per pane and
    /// target).
    pub fn key(&self) -> String {
        match self {
            Target::Local => "local".into(),
            Target::Docker { container } => format!("docker:{container}"),
            Target::Kube {
                context,
                namespace,
                pod,
                container,
            } => format!("kube:{context}:{namespace}/{pod}:{container}"),
            Target::Ssh { user, host, port } => format!("ssh:{user}@{host}:{port}"),
            Target::NoShell => "none".into(),
        }
    }

    /// The host part the remote shell puts into its OSC 7 URLs, so we only
    /// accept directory reports from this target (`[A-Za-z0-9._-]` only).
    pub fn osc_host(&self) -> String {
        let raw = format!("manta-{}", self.key());
        raw.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Short name for messages: the container, `namespace/pod`, `user@host`.
    pub fn label(&self) -> String {
        match self {
            Target::Docker { container } => container.clone(),
            Target::Kube { namespace, pod, .. } if !namespace.is_empty() => {
                format!("{namespace}/{pod}")
            }
            Target::Kube { pod, .. } => pod.clone(),
            Target::Ssh { user, host, .. } if !user.is_empty() => format!("{user}@{host}"),
            Target::Ssh { host, .. } => host.clone(),
            Target::Local => "Local".into(),
            Target::NoShell => String::new(),
        }
    }

    /// The CLI that opens the shell.
    pub fn program(&self) -> Option<&'static str> {
        match self {
            Target::Docker { .. } => Some("docker"),
            Target::Kube { .. } => Some("kubectl"),
            Target::Ssh { .. } => Some("ssh"),
            _ => None,
        }
    }

    /// Arguments for [`Self::program`] that open an interactive shell in
    /// `dir` with our integration.
    pub fn args(&self, dir: &str) -> Vec<String> {
        let id = self.osc_host();
        match self {
            Target::Docker { container } => {
                let script = bootstrap(&id, dir, false);
                vec![
                    "exec".into(),
                    "-it".into(),
                    "-e".into(),
                    "TERM=xterm-256color".into(),
                    "-e".into(),
                    "COLORTERM=truecolor".into(),
                    "-e".into(),
                    "TERM_PROGRAM=Manta".into(),
                    container.clone(),
                    "sh".into(),
                    "-c".into(),
                    script,
                ]
            }
            Target::Kube {
                context,
                namespace,
                pod,
                container,
            } => {
                let mut v = Vec::new();
                if !context.is_empty() {
                    v.extend(["--context".to_string(), context.clone()]);
                }
                if !namespace.is_empty() {
                    v.extend(["-n".to_string(), namespace.clone()]);
                }
                v.extend(["exec".to_string(), "-it".to_string(), pod.clone()]);
                if !container.is_empty() {
                    v.extend(["-c".to_string(), container.clone()]);
                }
                v.extend([
                    "--".to_string(),
                    "sh".to_string(),
                    "-c".to_string(),
                    bootstrap(&id, dir, false),
                ]);
                v
            }
            Target::Ssh { user, host, port } => {
                let mut v = vec![
                    "-t".to_string(),
                    "-o".into(),
                    "ServerAliveInterval=30".into(),
                ];
                if *port != 0 && *port != 22 {
                    v.extend(["-p".to_string(), port.to_string()]);
                }
                // `--` so a host can never be read as an option.
                let dest = if user.is_empty() {
                    host.clone()
                } else {
                    format!("{user}@{host}")
                };
                v.extend(["--".to_string(), dest]);
                // ssh hands one command string to the remote login shell.
                v.push(format!(
                    "exec sh -c {}",
                    sh_quote(&bootstrap(&id, dir, true))
                ));
                v
            }
            _ => Vec::new(),
        }
    }
}

/// Quote for a POSIX shell (also fine for fish and tcsh: `'\''`).
pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The rc file for a remote bash (read from fd 3 via `--rcfile`).
pub fn bash_rc(osc_host: &str) -> String {
    format!(
        r#"# Manta terminal integration (only for this shell; nothing is written to disk)
if [ -n "${{__rl_login-}}" ]; then
  [ -r /etc/profile ] && . /etc/profile
  if [ -r "$HOME/.bash_profile" ]; then . "$HOME/.bash_profile"
  elif [ -r "$HOME/.bash_login" ]; then . "$HOME/.bash_login"
  elif [ -r "$HOME/.profile" ]; then . "$HOME/.profile"
  elif [ -r "$HOME/.bashrc" ]; then . "$HOME/.bashrc"
  fi
else
  [ -r "$HOME/.bashrc" ] && . "$HOME/.bashrc"
fi
unset __rl_login
HISTCONTROL="${{HISTCONTROL:+$HISTCONTROL:}}ignorespace"
__manta_prompt() {{
  local ret=$? p=${{PWD//[%]/%25}}
  p=${{p// /%20}}
  printf '\033]7;file://{osc_host}%s\007\033]133;A;%s\007' "$p" "$__manta_cap"
  return $ret
}}
if [ "${{BASH_VERSINFO[0]:-0}}" -ge 4 ]; then
  __manta_cap='manta=push'
  __manta_save() {{
    __manta_line=$READLINE_LINE; __manta_point=$READLINE_POINT
    READLINE_LINE=; READLINE_POINT=0
  }}
  __manta_restore() {{
    READLINE_LINE=$__manta_line; READLINE_POINT=${{__manta_point:-0}}
    __manta_line=; __manta_point=0
  }}
  for __manta_km in emacs vi-insert vi-command; do
    bind -m "$__manta_km" -x '"\e[9001~": __manta_save' 2>/dev/null
    bind -m "$__manta_km" -x '"\e[9002~": __manta_restore' 2>/dev/null
  done
  unset __manta_km
else
  __manta_cap='manta=basic'
fi
case "$(declare -p PROMPT_COMMAND 2>/dev/null)" in
  "declare -a"*) PROMPT_COMMAND=(__manta_prompt "${{PROMPT_COMMAND[@]}}") ;;
  *) PROMPT_COMMAND="__manta_prompt${{PROMPT_COMMAND:+; $PROMPT_COMMAND}}" ;;
esac
"#
    )
}

/// The ENV file for a remote POSIX `sh` (busybox ash, dash): our markers
/// are added around the user's `PS1` (ash expands `${PWD}` per prompt).
pub fn sh_env(osc_host: &str) -> String {
    format!(
        r#"# Manta terminal integration (only for this shell; nothing is written to disk)
if [ -n "${{__rl_login-}}" ]; then
  [ -r /etc/profile ] && . /etc/profile
  [ -r "$HOME/.profile" ] && . "$HOME/.profile"
fi
unset __rl_login
__rl_pre="$(printf '\033]7;file://{osc_host}')"
__rl_post="$(printf '\007\033]133;A;manta=basic\007')"
PS1="\[$__rl_pre\]"'${{PWD}}'"\[$__rl_post\]${{PS1:-\$ }}"
unset __rl_pre __rl_post
"#
    )
}

/// The `sh -c` script that starts the best shell in `dir` with our hooks.
pub fn bootstrap(osc_host: &str, dir: &str, login: bool) -> String {
    format!(
        r#"__rl_login={login}
export __rl_login
export TERM=xterm-256color COLORTERM=truecolor TERM_PROGRAM=Manta
cd -- {dir} 2>/dev/null || cd 2>/dev/null
if command -v bash >/dev/null 2>&1; then
exec bash --rcfile /dev/fd/3 -i 3<<'__MANTA_BASH__'
{bash}__MANTA_BASH__
fi
ENV=/dev/fd/3 exec sh -i 3<<'__MANTA_SH__'
{sh}__MANTA_SH__
"#,
        login = if login { "1" } else { "" },
        dir = sh_quote(dir),
        bash = bash_rc(osc_host),
        sh = sh_env(osc_host),
    )
}

/// Directories where Docker Desktop, Homebrew, OrbStack, Rancher and
/// friends install their CLIs. Apps started from Finder get a minimal
/// `PATH`, so these are searched too.
pub fn search_path() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    let mut extra: Vec<PathBuf> = [
        "/usr/local/bin",
        "/opt/homebrew/bin",
        "/usr/bin",
        "/bin",
        "/Applications/Docker.app/Contents/Resources/bin",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    if let Some(h) = std::env::var_os("HOME").map(PathBuf::from) {
        for d in [
            ".docker/bin",
            ".rd/bin",
            ".orbstack/bin",
            ".local/bin",
            "bin",
        ] {
            extra.push(h.join(d));
        }
    }
    for d in extra {
        if !v.contains(&d) {
            v.push(d);
        }
    }
    v
}

/// Full path of an installed CLI.
pub fn find_program(program: &str) -> Option<PathBuf> {
    search_path()
        .into_iter()
        .map(|d| d.join(program))
        .find(|p| is_executable(p))
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// Can a shell be opened in the target right now?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Ready,
    /// The container exists but is not running.
    Stopped,
    /// The CLI (`docker`, `kubectl`, `ssh`) is not installed.
    NoCli(&'static str),
    /// Anything else (no such container, daemon down, …).
    Failed(String),
}

/// Set by [`block_cli_for_tests`].
static CLI_BLOCKED: AtomicBool = AtomicBool::new(false);

/// Tests and renders call this so no terminal ever runs `docker` /
/// `kubectl` against the developer's own containers: [`probe`] then fails
/// for container targets (unless `MANTA_TEST_CONTAINERS=1`).
pub fn block_cli_for_tests() {
    CLI_BLOCKED.store(true, Ordering::SeqCst);
}

fn cli_blocked() -> bool {
    (cfg!(test) || CLI_BLOCKED.load(Ordering::SeqCst))
        && std::env::var_os("MANTA_TEST_CONTAINERS").is_none_or(|v| v != "1")
}

/// Check the target (blocking; run on a background thread). Docker: is the
/// container running? Other targets are tried directly (the shell shows
/// any error itself).
pub fn probe(target: &Target, timeout: Duration) -> Probe {
    let Some(program) = target.program() else {
        return Probe::Failed("no shell".into());
    };
    if matches!(target, Target::Docker { .. } | Target::Kube { .. }) && cli_blocked() {
        return Probe::Failed(format!("{program} is disabled in tests"));
    }
    let Some(exe) = find_program(program) else {
        return Probe::NoCli(program);
    };
    match target {
        Target::Docker { container } => {
            let args = [
                "inspect",
                "--format",
                "{{.State.Running}}",
                container.as_str(),
            ];
            match run_with_timeout(&exe, &args, timeout) {
                Ok((true, out)) if out.trim() == "true" => Probe::Ready,
                Ok((true, out)) if out.trim() == "false" => Probe::Stopped,
                Ok((_, out)) => Probe::Failed(out.trim().lines().last().unwrap_or("").to_string()),
                Err(e) => Probe::Failed(e.to_string()),
            }
        }
        _ => Probe::Ready,
    }
}

/// Run `exe args`, returning (success, stdout+stderr) or an error on
/// timeout (the process is killed).
fn run_with_timeout(
    exe: &Path,
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<(bool, String)> {
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Ok(path) = std::env::join_paths(search_path()) {
        cmd.env("PATH", path);
    }
    let mut child = cmd.spawn()?;
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let out = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(o) = stdout.as_mut() {
            let _ = o.read_to_string(&mut s);
        }
        if let Some(e) = stderr.as_mut() {
            let _ = e.read_to_string(&mut s);
        }
        s
    });
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(st) = child.try_wait()? {
            let text = out.join().unwrap_or_default();
            return Ok((st.success(), text));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The server of an `ssh` typed in a local terminal, as `ssh -G` resolves
/// it (aliases and `~/.ssh/config` applied).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshDest {
    pub user: String,
    pub host: String,
    pub port: u16,
}

impl SshDest {
    /// `user@host`, with the port when it isn't 22.
    pub fn label(&self) -> String {
        let mut s = if self.user.is_empty() {
            self.host.clone()
        } else {
            format!("{}@{}", self.user, self.host)
        };
        if self.port != 22 {
            s.push_str(&format!(":{}", self.port));
        }
        s
    }
}

/// `ssh` options that take an argument (OpenSSH's `ssh(1)`).
const SSH_ARG_OPTS: &str = "BbcDEeFIiJLlmOoPpQRSWw";
/// Options that mean "no interactive shell": `-N` (no command), `-f`
/// (background), `-W` (stdio forward), `-O` (control), `-G` / `-V` / `-Q`
/// (print and exit).
const SSH_NO_SHELL: &str = "NfWOGVQ";

/// The `ssh` arguments up to the destination when `cmd` opens an
/// interactive shell on a server (`ssh [options] destination`, no remote
/// command), for `ssh -G`. `None` for anything else.
pub fn ssh_shell_args(cmd: &str) -> Option<Vec<String>> {
    let words = shell_words(cmd)?;
    let mut it = words.into_iter().peekable();
    // `command ssh`, `exec ssh`, `\ssh`, `/usr/bin/ssh`.
    while it.peek().is_some_and(|w| w == "command" || w == "exec") {
        it.next();
    }
    let prog = it.next()?;
    if prog.trim_start_matches('\\').rsplit('/').next() != Some("ssh") {
        return None;
    }
    let mut args = Vec::new();
    while let Some(w) = it.next() {
        if w == "--" {
            let dest = it.next()?;
            args.push(w);
            args.push(dest);
            break;
        }
        if let Some(flags) = w.strip_prefix('-').filter(|f| !f.is_empty()) {
            for (i, c) in flags.char_indices() {
                if SSH_NO_SHELL.contains(c) {
                    return None;
                }
                if SSH_ARG_OPTS.contains(c) {
                    args.push(w.clone());
                    // `-p2222` or `-p 2222`.
                    if i + c.len_utf8() == flags.len() {
                        args.push(it.next()?);
                    }
                    break;
                }
                if i + c.len_utf8() == flags.len() {
                    args.push(w.clone());
                }
            }
            continue;
        }
        args.push(w);
        break;
    }
    // A remote command after the destination: not an interactive shell.
    if it.next().is_some() || args.last().is_none_or(|d| d.starts_with('-')) {
        return None;
    }
    Some(args)
}

/// Split a command line into words like a POSIX shell (quotes and
/// backslashes; no expansion). `None` for anything with shell syntax we
/// don't follow (pipes, `;`, `&&`, `$(…)`, redirections).
fn shell_words(cmd: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut chars = cmd.trim().chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                loop {
                    match chars.next()? {
                        '\'' => break,
                        c => cur.push(c),
                    }
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match chars.next()? {
                        '"' => break,
                        '\\' => cur.push(chars.next()?),
                        '$' | '`' => return None,
                        c => cur.push(c),
                    }
                }
            }
            '\\' => {
                in_word = true;
                cur.push(chars.next()?);
            }
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            '|' | ';' | '&' | '<' | '>' | '$' | '`' | '(' | ')' => return None,
            c => {
                in_word = true;
                cur.push(c);
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    Some(words)
}

/// `user`, `hostname` and `port` from `ssh -G` output.
pub fn parse_ssh_config(out: &str) -> Option<SshDest> {
    let (mut user, mut host, mut port) = (None, None, 22);
    for line in out.lines() {
        let Some((k, v)) = line.trim().split_once(' ') else {
            continue;
        };
        match k {
            "user" => user = Some(v.trim().to_string()),
            "hostname" => host = Some(v.trim().to_string()),
            "port" => port = v.trim().parse().unwrap_or(22),
            _ => {}
        }
    }
    Some(SshDest {
        user: user.unwrap_or_default(),
        host: host.filter(|h| !h.is_empty())?,
        port,
    })
}

/// Resolve `ssh` arguments from [`ssh_shell_args`] with `ssh -G`
/// (blocking; run on a background thread). Reads only the local config.
pub fn resolve_ssh(args: &[String], timeout: Duration) -> Option<SshDest> {
    let exe = find_program("ssh")?;
    let mut all: Vec<&str> = vec!["-G"];
    all.extend(args.iter().map(String::as_str));
    match run_with_timeout(&exe, &all, timeout) {
        Ok((true, out)) => parse_ssh_config(&out),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use manta_core::VfsPath;
    use manta_plugin::PaneId;

    #[test]
    fn ssh_commands_that_open_a_shell() {
        let args = |c: &str| ssh_shell_args(c).map(|v| v.join(" "));
        assert_eq!(
            args("ssh anna@files.example.org").as_deref(),
            Some("anna@files.example.org")
        );
        assert_eq!(args("ssh -p 2222 web").as_deref(), Some("-p 2222 web"));
        assert_eq!(
            args("ssh -p2222 -l anna web").as_deref(),
            Some("-p2222 -l anna web")
        );
        assert_eq!(args("ssh -A -t prod").as_deref(), Some("-A -t prod"));
        assert_eq!(
            args("ssh -Ai ~/.ssh/k prod").as_deref(),
            Some("-Ai ~/.ssh/k prod")
        );
        assert_eq!(args("command ssh prod").as_deref(), Some("prod"));
        assert_eq!(args("/usr/bin/ssh 'my host'").as_deref(), Some("my host"));
        assert_eq!(args("ssh -- prod").as_deref(), Some("-- prod"));
        assert_eq!(
            args("ssh ssh://anna@web:2222").as_deref(),
            Some("ssh://anna@web:2222")
        );
        // A remote command, no shell, other programs, shell syntax.
        assert_eq!(args("ssh prod uptime"), None);
        assert_eq!(args("ssh -N -L 8080:localhost:80 prod"), None);
        assert_eq!(args("ssh -G prod"), None);
        assert_eq!(args("ssh"), None);
        assert_eq!(args("ssh -p"), None);
        assert_eq!(args("sshfs prod:/ /mnt"), None);
        assert_eq!(args("ssh-add -l"), None);
        assert_eq!(args("ssh prod | tee log"), None);
        assert_eq!(args("ssh $HOST"), None);
        assert_eq!(args("echo ssh prod"), None);
    }

    #[test]
    fn ssh_config_output() {
        let out =
            "user anna\nhostname files.example.org\nport 2222\nidentityfile ~/.ssh/id_ed25519\n";
        let d = parse_ssh_config(out).unwrap();
        assert_eq!(
            d,
            SshDest {
                user: "anna".into(),
                host: "files.example.org".into(),
                port: 2222
            }
        );
        assert_eq!(d.label(), "anna@files.example.org:2222");
        let d = parse_ssh_config("hostname web\n").unwrap();
        assert_eq!((d.port, d.label()), (22, "web".to_string()));
        assert_eq!(parse_ssh_config("user anna\n"), None);
    }

    /// The real `ssh -G` resolves without contacting the server.
    #[test]
    fn resolve_with_ssh_g() {
        if find_program("ssh").is_none() {
            return;
        }
        let args = ssh_shell_args("ssh -p 2222 -l anna 203.0.113.9").unwrap();
        let d = resolve_ssh(&args, Duration::from_secs(5)).expect("ssh -G");
        assert_eq!(d.user, "anna");
        assert_eq!(d.host, "203.0.113.9");
        assert_eq!(d.port, 2222);
    }

    fn pane(local: Option<&str>, remote: Option<RemoteEndpoint>) -> PaneState {
        PaneState {
            pane: PaneId::Left,
            path: VfsPath::new("/srv"),
            local_path: local.map(PathBuf::from),
            protocol: remote
                .as_ref()
                .map(|r| r.protocol)
                .unwrap_or(Protocol::Local),
            backend_label: String::new(),
            selection: vec![],
            active: true,
            remote,
        }
    }

    fn ep(protocol: Protocol, user: &str, host: &str, port: u16, context: &str) -> RemoteEndpoint {
        RemoteEndpoint {
            protocol,
            user: user.into(),
            host: host.into(),
            port,
            context: context.into(),
        }
    }

    #[test]
    fn targets_from_panes() {
        assert_eq!(Target::from_pane(&pane(Some("/tmp"), None)), Target::Local);
        assert_eq!(
            Target::from_pane(&pane(None, Some(ep(Protocol::Docker, "", "web", 0, "")))),
            Target::Docker {
                container: "web".into()
            }
        );
        assert_eq!(
            Target::from_pane(&pane(
                None,
                Some(ep(Protocol::Kubernetes, "app", "shop/web-0", 0, "prod"))
            )),
            Target::Kube {
                context: "prod".into(),
                namespace: "shop".into(),
                pod: "web-0".into(),
                container: "app".into()
            }
        );
        assert_eq!(
            Target::from_pane(&pane(
                None,
                Some(ep(Protocol::Kubernetes, "", "web-0", 0, ""))
            )),
            Target::Kube {
                context: "".into(),
                namespace: "".into(),
                pod: "web-0".into(),
                container: "".into()
            }
        );
        assert_eq!(
            Target::from_pane(&pane(
                None,
                Some(ep(Protocol::Sftp, "anna", "files.example.org", 2222, ""))
            )),
            Target::Ssh {
                user: "anna".into(),
                host: "files.example.org".into(),
                port: 2222
            }
        );
        for p in [Protocol::S3, Protocol::Ftp, Protocol::Ftps] {
            assert_eq!(
                Target::from_pane(&pane(None, Some(ep(p, "u", "h", 21, "")))),
                Target::NoShell
            );
        }
        // Archives: no local path and no remote.
        assert_eq!(Target::from_pane(&pane(None, None)), Target::NoShell);
    }

    #[test]
    fn keys_and_osc_hosts() {
        let k = Target::Kube {
            context: "prod".into(),
            namespace: "shop".into(),
            pod: "web-0".into(),
            container: "app".into(),
        };
        assert_eq!(k.key(), "kube:prod:shop/web-0:app");
        assert_eq!(k.osc_host(), "manta-kube_prod_shop_web-0_app");
        let d = Target::Docker {
            container: "my web".into(),
        };
        assert_eq!(d.osc_host(), "manta-docker_my_web");
        assert!(d
            .osc_host()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-._".contains(c)));
        assert_ne!(
            Target::Docker {
                container: "a".into()
            }
            .key(),
            Target::Docker {
                container: "b".into()
            }
            .key()
        );
    }

    #[test]
    fn docker_and_kubectl_commands() {
        let d = Target::Docker {
            container: "web".into(),
        };
        let a = d.args("/usr/share/nginx/html");
        assert_eq!(&a[..2], ["exec", "-it"]);
        assert_eq!(&a[8..11], ["web", "sh", "-c"]);
        assert!(a[11].contains("cd -- '/usr/share/nginx/html'"));
        assert!(a.contains(&"TERM=xterm-256color".to_string()));
        let k = Target::Kube {
            context: "prod".into(),
            namespace: "shop".into(),
            pod: "web-0".into(),
            container: "app".into(),
        };
        let a = k.args("/it's here");
        assert_eq!(
            &a[..9],
            [
                "--context",
                "prod",
                "-n",
                "shop",
                "exec",
                "-it",
                "web-0",
                "-c",
                "app"
            ]
        );
        assert_eq!(&a[9..12], ["--", "sh", "-c"]);
        assert!(a[12].contains("cd -- '/it'\\''s here'"));
        let k = Target::Kube {
            context: "".into(),
            namespace: "".into(),
            pod: "p".into(),
            container: "".into(),
        };
        assert_eq!(&k.args("/")[..4], ["exec", "-it", "p", "--"]);
    }

    #[test]
    fn ssh_command() {
        let s = Target::Ssh {
            user: "anna".into(),
            host: "files.example.org".into(),
            port: 2222,
        };
        let a = s.args("/var/www");
        assert_eq!(
            &a[..5],
            ["-t", "-o", "ServerAliveInterval=30", "-p", "2222"]
        );
        assert_eq!(&a[5..7], ["--", "anna@files.example.org"]);
        assert!(a[7].starts_with("exec sh -c '"));
        assert!(a[7].contains("__rl_login=1"));
        // Default port and no user: plain host.
        let s = Target::Ssh {
            user: "".into(),
            host: "h".into(),
            port: 22,
        };
        assert_eq!(&s.args("/")[3..5], ["--", "h"]);
        // A hostile host name can't become an option.
        let s = Target::Ssh {
            user: "".into(),
            host: "-oProxyCommand=x".into(),
            port: 22,
        };
        let a = s.args("/");
        assert_eq!(a[3], "--");
    }

    #[test]
    fn hooks() {
        let b = bash_rc("manta-docker_web");
        assert!(b.contains(". \"$HOME/.bashrc\""));
        assert!(b.contains("file://manta-docker_web%s"));
        assert!(b.contains("133;A;%s"));
        assert!(b.contains("ignorespace"));
        assert!(b.contains("PROMPT_COMMAND=\"__manta_prompt${PROMPT_COMMAND:+; $PROMPT_COMMAND}\""));
        assert!(b.contains("return $ret"), "keeps $? for the user's prompt");
        let s = sh_env("manta-docker_web");
        assert!(s.contains("'${PWD}'"));
        assert!(s.contains("${PS1:-\\$ }"), "keeps the user's PS1");
        assert!(
            s.contains("\\[$__rl_pre\\]"),
            "zero-width for the line editor"
        );
        let boot = bootstrap("x", "/a b", false);
        assert!(boot.contains("cd -- '/a b'"));
        assert!(boot.contains("exec bash --rcfile /dev/fd/3 -i 3<<'__MANTA_BASH__'"));
        assert!(boot.contains("ENV=/dev/fd/3 exec sh -i 3<<'__MANTA_SH__'"));
        assert!(boot.starts_with("__rl_login=\n"));
    }

    #[test]
    fn bootstrap_is_valid_sh() {
        for sh in ["/bin/sh", "/bin/bash"] {
            if !Path::new(sh).exists() {
                continue;
            }
            let script = bootstrap("manta-x", "/it's", true);
            let st = Command::new(sh)
                .arg("-n")
                .arg("-c")
                .arg(&script)
                .status()
                .unwrap();
            assert!(st.success(), "{sh} -n bootstrap");
            let st = Command::new("/bin/bash")
                .arg("-n")
                .arg("-c")
                .arg(bash_rc("manta-x"))
                .status()
                .unwrap();
            assert!(st.success(), "bash -n rc");
        }
    }
}
