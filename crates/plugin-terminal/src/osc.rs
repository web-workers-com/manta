//! A tiny streaming scanner for the OSC sequences the emulator ignores but we
//! need: OSC 7 (current directory) and OSC 133 (prompt marks).
//!
//! The bytes are still fed to the emulator unchanged; this only observes.

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

const MAX_OSC: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OscEvent {
    /// OSC 7: the URL of the shell's working directory, e.g.
    /// `file://host/some%20path`. Which hosts to accept depends on where the
    /// shell runs: see [`cwd_from_osc7`].
    Cwd(String),
    /// OSC 133;A: the prompt is about to be drawn (shell idle). `push` is
    /// true if the Manta shim announced its line-editor key bindings,
    /// `echo` if it bound the key that prints spooled file actions.
    PromptStart {
        push: bool,
        echo: bool,
    },
    /// OSC 133;C: a command line was submitted and is about to run.
    CommandStart,
    /// Private OSC 7702: the zsh shim reports a command line that mentions
    /// `ssh` just before it runs (so the terminal can offer to open that
    /// server in the pane).
    Command(String),
    /// Private OSC 7701;s / 7701;e: the shim wraps zsh's RPROMPT in these
    /// zero-width markers so we know exactly which cells are the right
    /// prompt (see `session::Shared`).
    RpromptStart,
    RpromptEnd,
}

/// The markers the zsh shim puts around RPROMPT (inside `%{…%}`).
pub const RPROMPT_START: &str = "\x1b]7701;s\x07";
pub const RPROMPT_END: &str = "\x1b]7701;e\x07";

#[derive(Debug, Default)]
enum State {
    #[default]
    Ground,
    Esc,
    Osc,
    OscEsc,
}

#[derive(Debug, Default)]
pub struct OscScanner {
    state: State,
    buf: Vec<u8>,
    overflow: bool,
}

impl OscScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed raw PTY output; returns the events completed in this chunk.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<OscEvent> {
        self.feed_at(bytes).into_iter().map(|(_, e)| e).collect()
    }

    /// Like [`Self::feed`], with the offset just past each event's
    /// terminator, so the caller can advance the emulator up to it first.
    pub fn feed_at(&mut self, bytes: &[u8]) -> Vec<(usize, OscEvent)> {
        let mut out = Vec::new();
        for (i, &b) in bytes.iter().enumerate() {
            let before = out.len();
            match self.state {
                State::Ground => {
                    if b == 0x1b {
                        self.state = State::Esc;
                    }
                }
                State::Esc => {
                    self.state = match b {
                        b']' => {
                            self.buf.clear();
                            self.overflow = false;
                            State::Osc
                        }
                        0x1b => State::Esc,
                        _ => State::Ground,
                    }
                }
                State::Osc => match b {
                    0x07 => {
                        self.finish(&mut out);
                        self.state = State::Ground;
                    }
                    0x1b => self.state = State::OscEsc,
                    // CAN / SUB abort the sequence.
                    0x18 | 0x1a => self.state = State::Ground,
                    _ => self.push(b),
                },
                State::OscEsc => {
                    if b == b'\\' {
                        self.finish(&mut out);
                        self.state = State::Ground;
                    } else if b == b']' {
                        // Unterminated OSC followed by a new one.
                        self.buf.clear();
                        self.overflow = false;
                        self.state = State::Osc;
                    } else {
                        self.state = if b == 0x1b { State::Esc } else { State::Ground };
                    }
                }
            }
            for ev in out.iter_mut().skip(before) {
                ev.0 = i + 1;
            }
        }
        out
    }

    fn push(&mut self, b: u8) {
        if self.buf.len() < MAX_OSC {
            self.buf.push(b);
        } else {
            self.overflow = true;
        }
    }

    fn finish(&mut self, out: &mut Vec<(usize, OscEvent)>) {
        if self.overflow {
            return;
        }
        if let Some(ev) = parse_osc(&self.buf) {
            out.push((0, ev));
        }
    }
}

fn parse_osc(payload: &[u8]) -> Option<OscEvent> {
    match payload {
        b"7701;s" => return Some(OscEvent::RpromptStart),
        b"7701;e" => return Some(OscEvent::RpromptEnd),
        _ => {}
    }
    if let Some(rest) = payload.strip_prefix(b"7702;") {
        return Some(OscEvent::Command(
            String::from_utf8_lossy(rest).into_owned(),
        ));
    }
    if let Some(rest) = payload.strip_prefix(b"7;") {
        let s = std::str::from_utf8(rest).ok()?;
        return Some(OscEvent::Cwd(s.to_string()));
    }
    if let Some(rest) = payload.strip_prefix(b"133;") {
        let s = std::str::from_utf8(rest).ok()?;
        let mut parts = s.split(';');
        return match parts.next()? {
            "A" => {
                let parts: Vec<&str> = parts.collect();
                Some(OscEvent::PromptStart {
                    push: parts.contains(&"manta=push"),
                    echo: parts.contains(&"manta-echo"),
                })
            }
            "C" => Some(OscEvent::CommandStart),
            _ => None,
        };
    }
    None
}

/// Parse an OSC 7 payload like `file://host/some%20path` into a local path.
///
/// Returns `None` for a non-`file` URL, a relative path, or a host that is
/// not this machine (e.g. the user `ssh`ed somewhere whose shell also emits
/// OSC 7 — we must not follow a remote path locally).
pub fn parse_osc7(url: &str) -> Option<PathBuf> {
    cwd_from_osc7(url, None)
}

/// The directory of an OSC 7 URL from a shell running locally (`remote:
/// None`: the host must be this machine) or in a container / on a server
/// (`remote: Some(id)`: the host must be exactly the id our hooks put there,
/// so a nested `ssh` elsewhere is never followed). Remote shells without
/// percent-encoding (busybox ash) are accepted as they are.
pub fn cwd_from_osc7(url: &str, remote: Option<&str>) -> Option<PathBuf> {
    let rest = url
        .strip_prefix("file://")
        .or_else(|| url.strip_prefix("kitty-shell-cwd://"))?;
    let i = rest.find('/')?;
    let (host, path) = (&rest[..i], &rest[i..]);
    match remote {
        None if !is_local_host(host) => return None,
        Some(id) if host != id => return None,
        Some(_) => {
            let bytes = percent_decode(path.as_bytes()).unwrap_or_else(|| path.as_bytes().to_vec());
            return Some(PathBuf::from(OsString::from_vec(bytes)));
        }
        None => {}
    }
    // Drop a query/fragment if any (never produced by shells, but be safe).
    let bytes = percent_decode(path.as_bytes())?;
    if bytes.first() != Some(&b'/') {
        return None;
    }
    Some(PathBuf::from(OsString::from_vec(bytes)))
}

fn percent_decode(s: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            let hex = std::str::from_utf8(hex).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    Some(out)
}

fn is_local_host(host: &str) -> bool {
    if host.is_empty() || host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let local = local_hostname();
    let strip = |h: &str| -> String {
        let h = h.to_ascii_lowercase();
        h.strip_suffix(".local").map(str::to_owned).unwrap_or(h)
    };
    let (a, b) = (strip(host), strip(&local));
    a == b || a.split('.').next() == b.split('.').next() && !a.is_empty()
}

pub fn local_hostname() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: buf is valid for buf.len() bytes; gethostname NUL-terminates
    // (or truncates) within that length.
    let r = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if r != 0 {
        return String::new();
    }
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc7_basic() {
        assert_eq!(parse_osc7("file:///tmp/a"), Some(PathBuf::from("/tmp/a")));
        assert_eq!(
            parse_osc7("file://localhost/tmp/a%20b"),
            Some(PathBuf::from("/tmp/a b"))
        );
        assert_eq!(
            parse_osc7("file:///Users/x/%C3%9Cbergr%C3%B6%C3%9Fe"),
            Some(PathBuf::from("/Users/x/Übergröße"))
        );
        // Raw UTF-8 is accepted too.
        assert_eq!(
            parse_osc7("file:///tmp/Ü 🚀"),
            Some(PathBuf::from("/tmp/Ü 🚀"))
        );
        assert_eq!(
            parse_osc7("file:///tmp/100%25"),
            Some(PathBuf::from("/tmp/100%"))
        );
        let host = local_hostname();
        assert_eq!(
            parse_osc7(&format!("file://{host}/x")),
            Some(PathBuf::from("/x"))
        );
    }

    #[test]
    fn osc7_remote_hosts() {
        let id = "manta-docker_web";
        assert_eq!(
            cwd_from_osc7("file://manta-docker_web/etc/nginx", Some(id)),
            Some(PathBuf::from("/etc/nginx"))
        );
        assert_eq!(
            cwd_from_osc7("file://manta-docker_web/a%20b", Some(id)),
            Some(PathBuf::from("/a b"))
        );
        // ash prints $PWD raw: a literal % or space still works.
        assert_eq!(
            cwd_from_osc7("file://manta-docker_web/100% sure", Some(id)),
            Some(PathBuf::from("/100% sure"))
        );
        // Another target, the local machine, or a nested ssh: rejected.
        assert_eq!(cwd_from_osc7("file://manta-docker_db/etc", Some(id)), None);
        assert_eq!(cwd_from_osc7("file:///etc", Some(id)), None);
        assert_eq!(
            cwd_from_osc7(&format!("file://{}/etc", local_hostname()), Some(id)),
            None
        );
        assert_eq!(cwd_from_osc7("file://some-server/etc", Some(id)), None);
        // And a local shell never accepts the remote id.
        assert_eq!(cwd_from_osc7("file://manta-docker_web/etc", None), None);
    }

    #[test]
    fn osc7_rejects() {
        assert_eq!(parse_osc7("http://x/y"), None);
        assert_eq!(
            parse_osc7("file://some-remote-box.example.org/home/anna"),
            None
        );
        assert_eq!(parse_osc7("file://"), None);
        assert_eq!(parse_osc7("file:///bad%zz"), None);
        assert_eq!(parse_osc7("file:///trunc%2"), None);
    }

    #[test]
    fn scanner_streams_and_splits() {
        let mut s = OscScanner::new();
        let data =
            b"hello\x1b]7;file:///tmp/x%20y\x07\x1b[31mred\x1b]133;A;manta=push;manta-echo\x1b\\$ ";
        // Feed byte by byte to exercise chunk boundaries.
        let mut evs = Vec::new();
        for b in data {
            evs.extend(s.feed(std::slice::from_ref(b)));
        }
        assert_eq!(
            evs,
            vec![
                OscEvent::Cwd("file:///tmp/x%20y".into()),
                OscEvent::PromptStart {
                    push: true,
                    echo: true
                }
            ]
        );
        assert_eq!(
            s.feed(b"\x1b]133;C\x07\x1b]7702;ssh -p 2222 web\x07"),
            vec![
                OscEvent::CommandStart,
                OscEvent::Command("ssh -p 2222 web".into())
            ]
        );
        assert_eq!(
            s.feed(b"\x1b]133;C\x07\x1b]133;A\x07"),
            vec![
                OscEvent::CommandStart,
                OscEvent::PromptStart {
                    push: false,
                    echo: false
                }
            ]
        );
        assert_eq!(
            s.feed_at(b"ab\x1b]7701;s\x07xy\x1b]7701;e\x1b\\"),
            vec![(11, OscEvent::RpromptStart), (23, OscEvent::RpromptEnd)]
        );
        // Titles and unknown OSCs are ignored.
        assert!(s.feed(b"\x1b]0;title\x07\x1b]8;;http://x\x07").is_empty());
    }
}
