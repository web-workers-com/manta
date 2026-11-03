//! The terminal plugin's strings (English, German); see
//! `manta_plugin::i18n`. Only the plugin's own UI is translated, never
//! shell output or the shell commands echoed for file actions.

use manta_plugin::i18n::{Catalog, Entries, Lang};

mod es;
mod fr;
mod it;
mod ru;
mod zh;

pub static CATALOG: Catalog = Catalog::with(
    ENTRIES,
    &[
        (Lang::Zh, zh::ENTRIES),
        (Lang::Es, es::ENTRIES),
        (Lang::Fr, fr::ENTRIES),
        (Lang::It, it::ENTRIES),
        (Lang::Ru, ru::ENTRIES),
    ],
);

const ENTRIES: Entries = &[
    // Plugin, commands, panel
    ("term.info.name", "Terminal", "Terminal"),
    ("term.info.description", "A terminal below each pane that follows the pane's folder live.", "Ein Terminal unter jedem Bereich, das dem Ordner des Bereichs live folgt."),
    ("term.panel.title", "Terminal", "Terminal"),
    ("term.cmd.toggle", "Toggle Terminal", "Terminal ein-/ausblenden"),
    ("term.cmd.new_session", "New Terminal Session", "Neue Terminal-Sitzung"),
    ("term.cmd.clear", "Clear Terminal", "Terminal leeren"),
    // Banners, pills, errors
    ("term.remote_note", "No terminal for this location (S3, FTP, archives)", "Für diesen Ort gibt es kein Terminal (S3, FTP, Archive)"),
    ("term.remote.connecting", "Opening a shell in {target} …", "Öffne eine Shell in {target} …"),
    ("term.remote.stopped", "Container “{name}” is stopped", "Container „{name}“ ist gestoppt"),
    ("term.remote.stopped_hint", "Start it, then click here to open a shell.", "Starte ihn und klicke dann hier, um eine Shell zu öffnen."),
    ("term.remote.no_cli", "{program} is not installed", "{program} ist nicht installiert"),
    ("term.remote.no_cli_hint", "The terminal needs the {program} command line tool.", "Das Terminal braucht das Kommandozeilenwerkzeug {program}."),
    ("term.remote.failed", "Could not open a shell in {target}", "In {target} konnte keine Shell geöffnet werden"),
    ("term.remote.retry_hint", "Click to try again.", "Zum erneuten Versuchen klicken."),
    ("term.error.start", "Could not start the shell: {error}", "Die Shell konnte nicht gestartet werden: {error}"),
    ("term.toast.settings_not_saved", "Terminal settings not saved: {error}", "Die Terminal-Einstellungen wurden nicht gesichert: {error}"),
    ("term.ssh.open", "Open {target} in the pane", "{target} im Bereich öffnen"),
    ("term.pill.run", "Will run “{command}” when idle", "Führt „{command}“ aus, sobald die Shell bereit ist"),
    ("term.pill.cd", "Will cd to “{name}” when idle", "Wechselt nach „{name}“, sobald die Shell bereit ist"),
    ("term.pill.cd_and_show", "Will cd to “{name}” and show {actions} when idle", "Wechselt nach „{name}“ und zeigt {actions}, sobald die Shell bereit ist"),
    ("term.pill.show", "Will show {actions} when idle", "Zeigt {actions}, sobald die Shell bereit ist"),
    ("term.pill.actions.one", "{count} file action", "{count} Dateiaktion"),
    ("term.pill.actions.other", "{count} file actions", "{count} Dateiaktionen"),
    ("term.exit_note", "[Process exited — press Enter to restart]", "[Prozess beendet – zum Neustarten den Zeilenschalter drücken]"),
    // Settings
    ("term.settings.shell", "Shell", "Shell"),
    ("term.settings.font_size", "Font size", "Schriftgröße"),
    ("term.settings.scrollback", "Scrollback", "Verlauf"),
    ("term.settings.lines_suffix", " lines", " Zeilen"),
    ("term.settings.option_meta", "Use Option as Meta key", "Wahltaste als Meta-Taste verwenden"),
    ("term.settings.option_meta.tip", "Option+key sends ESC+key (for Emacs-style shortcuts) instead of typing special characters.", "Wahltaste+Taste sendet ESC+Taste (für Kurzbefehle wie in Emacs), statt Sonderzeichen zu tippen."),
    ("term.settings.follow", "Pane follows terminal cd", "Bereich folgt „cd“ im Terminal"),
    ("term.settings.follow.tip", "When you cd in the terminal, the file pane above shows that folder too.", "Wenn du im Terminal mit „cd“ den Ordner wechselst, zeigt der Bereich darüber diesen Ordner ebenfalls."),
    ("term.settings.show_actions", "Show file actions in terminal", "Dateiaktionen im Terminal zeigen"),
    ("term.settings.show_actions.tip", "Copy, move, delete, rename and the like done in the file panes are shown in the terminal as the equivalent shell command, with the result once finished. Manta does the work itself; nothing is run in the shell.", "Kopieren, Bewegen, Löschen, Umbenennen und Ähnliches in den Dateibereichen erscheint im Terminal als entsprechender Shell-Befehl, mit dem Ergebnis, sobald es fertig ist. Manta erledigt die Arbeit selbst; in der Shell wird nichts ausgeführt."),
    // Echoed result lines
    ("term.result.cancelled", "cancelled", "abgebrochen"),
    ("term.result.failed", "failed", "fehlgeschlagen"),
    ("term.result.failed_error", "failed: {error}", "fehlgeschlagen: {error}"),
    ("term.result.failed_more", "failed: {error} (+{more} more)", "fehlgeschlagen: {error} (+{more} weitere)"),
    ("term.result.done", "done", "fertig"),
    ("term.result.files.one", "{count} file", "{count} Datei"),
    ("term.result.files.other", "{count} files", "{count} Dateien"),
    ("term.result.items.one", "{count} item", "{count} Objekt"),
    ("term.result.items.other", "{count} items", "{count} Objekte"),
    ("term.result.minutes", "{m}m {s}s", "{m} min {s} s"),
    ("term.result.label", "{verb} “{name}”", "{verb} „{name}“"),
    ("term.more", "… (+{n} more)", "… (+{n} weitere)"),
    ("term.dropped.one", "… {count} earlier file action line not shown", "… {count} frühere Zeile mit Dateiaktionen nicht angezeigt"),
    ("term.dropped.other", "… {count} earlier file action lines not shown", "… {count} frühere Zeilen mit Dateiaktionen nicht angezeigt"),
    ("term.verb.copy", "copy", "kopieren"),
    ("term.verb.move", "move", "bewegen"),
    ("term.verb.delete", "delete", "löschen"),
    ("term.verb.trash", "move to Trash", "in den Papierkorb legen"),
    ("term.verb.rename", "rename", "umbenennen"),
    ("term.verb.mkdir", "mkdir", "neuer Ordner"),
    ("term.verb.touch", "touch", "neue Datei"),
    ("term.verb.open", "open", "öffnen"),
    ("term.verb.connect", "connect", "verbinden"),
    ("term.verb.disconnect", "disconnect", "trennen"),
    ("term.verb.sync", "sync", "synchronisieren"),
    ("term.verb.compress", "compress", "komprimieren"),
    ("term.verb.extract", "extract", "entpacken"),
];

/// `tr!("key")`, `tr!("key", count = n)`, `tr!("key", name = v)`.
macro_rules! tr {
    ($($t:tt)*) => { manta_plugin::tr_with!(crate::i18n::CATALOG, $($t)*) };
}
pub(crate) use tr;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use manta_plugin::i18n::check;

    use super::*;

    #[test]
    fn catalog_is_complete() {
        let p = check::catalog_problems(&CATALOG, &[]);
        assert!(p.is_empty(), "{}", p.join("\n"));
    }

    #[test]
    fn every_used_key_exists() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let m = check::missing_keys(&CATALOG, &src, 30);
        assert!(m.is_empty(), "{}", m.join("\n"));
    }

    #[test]
    fn no_hard_coded_ui_strings() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let hits = check::hard_coded_strings(
            &src,
            &[
                "pill(&painter, rect, &format!(\"",
                "banner(&painter",
                "styled_note(&format!(\"",
                "pt.error = Some(format!(\"",
            ],
            &[],
        );
        assert!(hits.is_empty(), "{}", hits.join("\n"));
    }
}
