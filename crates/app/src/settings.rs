//! Settings window (⌘,): General preferences and Plugins.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use eframe::egui::{self, CornerRadius, RichText, Sense, Stroke};
use manta_plugin::host::{BundledRow, PluginHost, PluginKind, PluginRow, RegistryRow};
use manta_plugin::install::PluginReview;
use manta_plugin::registry::Incompatible;
use manta_plugin::sandbox::SandboxState;
use manta_plugin::PluginStatus;

use crate::aux_window::{AuxKind, AuxWindow, Spec};
use crate::dialogs::{self, DialogResult};
use crate::i18n::Language;
use crate::icons;
use crate::plugin_ui;
use crate::prefs::{
    Accent, Appearance, ConflictPref, DropAction, MouseNav, NewTabFolder, Prefs, SpaceAction,
};
use crate::theme::{self, pal, radius, space, text, Palette};
use crate::widgets::{self, ButtonKind};

/// Inner size of the Settings window when first opened.
pub const SIZE: [f32; 2] = [620.0, 640.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    General,
    #[default]
    Plugins,
}

/// A confirmation shown on top of the Settings window.
#[derive(Debug, Clone, PartialEq)]
pub enum Confirm {
    /// First time an external plugin is switched on: the trust sheet.
    Enable {
        id: String,
        review: Box<PluginReview>,
    },
    /// A plugin folder or archive was checked: install it?
    Install(Box<PluginReview>),
    Replace {
        src: PathBuf,
        name: String,
        installed_version: String,
        new_version: String,
    },
    Uninstall {
        id: String,
        name: String,
    },
}

/// The two pages of Settings → Plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PluginsPage {
    /// Built-in, installed and bundled plugins.
    #[default]
    Installed,
    /// The community directory (loaded from GitHub while shown).
    Discover,
}

#[derive(Default)]
pub struct SettingsView {
    pub open: bool,
    pub tab: SettingsTab,
    pub plugins_page: PluginsPage,
    /// Files dropped on / dragged over the window while it is drawn inside
    /// the main window (render tests); a native Settings window gets them
    /// in its own input.
    pub(crate) dropped: Vec<PathBuf>,
    pub(crate) hovered: Vec<PathBuf>,
    pub confirm: Option<Confirm>,
    show_log: HashSet<String>,
    show_settings: HashSet<String>,
    /// Whether a plugin draws settings (probed while collapsed).
    has_settings: HashMap<String, bool>,
    /// File or folder picker running on a background thread.
    picking: Option<Receiver<Option<PathBuf>>>,
    /// The picker for "New tabs open in" → "Other…".
    picking_new_tabs: Option<Receiver<Option<PathBuf>>>,
    /// Scroll the page to this offset on the next frame (render tests).
    pub(crate) scroll_to: Option<f32>,
    /// Last frame's layout: the tab bar and the scroll area's visible rect
    /// (regression tests: the page never overlaps the header).
    pub(crate) last_layout: Option<(egui::Rect, egui::Rect)>,
    /// "Default file manager" state, set by the app each frame (`None`:
    /// not supported here, the section is hidden).
    pub viewer: Option<crate::default_viewer::ViewerStatus>,
    /// The Updates section, set by the app each frame (`None`: hidden).
    pub updates: Option<crate::updates::SettingsInfo>,
    /// Scroll the Updates section into view on the next frame.
    pub(crate) reveal_updates: bool,
    /// The native window (placement, focus).
    pub(crate) window: AuxWindow,
    /// Settings → General → Cloud.
    pub cloud: CloudSettings,
}

/// Settings → General → Cloud: Google Drive through Google Drive for
/// desktop, and (Expert) the user's own OAuth client for Manta's built-in
/// Google Drive connection.
#[derive(Default)]
pub struct CloudSettings {
    /// Accounts of the Google Drive for desktop folders (set by the app).
    pub drive_accounts: Vec<String>,
    /// Where the user's own OAuth client is kept (`gdrive-client.json`;
    /// `None`: nowhere, the Expert setting is hidden).
    pub client_file: Option<PathBuf>,
    expert_open: bool,
    /// Edit buffers, filled from the file when the Expert part opens.
    client_id: String,
    client_secret: String,
    loaded: bool,
    error: Option<String>,
}

impl CloudSettings {
    /// The user saved an OAuth client of their own (or set one in the
    /// environment, for development): Google Drive is offered in the
    /// connect dialog.
    pub fn own_client(&self) -> bool {
        (!cfg!(test) && std::env::var("MANTA_GDRIVE_CLIENT_ID").is_ok_and(|v| !v.trim().is_empty()))
            || self
                .client_file
                .as_deref()
                .and_then(manta_vfs::gdrive::OAuthClient::from_file)
                .is_some()
    }

    fn load(&mut self) {
        let c = self
            .client_file
            .as_deref()
            .and_then(manta_vfs::gdrive::OAuthClient::from_file);
        self.client_id = c.as_ref().map(|c| c.client_id.clone()).unwrap_or_default();
        self.client_secret = c.map(|c| c.client_secret).unwrap_or_default();
        self.loaded = true;
    }

    /// Save (or with an empty id: remove) the user's client.
    fn save(&mut self) {
        let Some(path) = self.client_file.clone() else {
            return;
        };
        self.error =
            manta_vfs::gdrive::OAuthClient::save_file(&path, &self.client_id, &self.client_secret)
                .err()
                .map(|e| e.to_string());
        self.load();
    }

    /// Tests: open the Expert part.
    #[cfg(test)]
    pub fn open_expert(&mut self) {
        self.expert_open = true;
    }
}

/// What the app has to do after the Settings window was drawn.
#[derive(Default)]
pub struct SettingsOutcome {
    pub prefs_changed: bool,
    /// The default-file-manager switch was flipped to this.
    pub viewer_switch: Option<bool>,
    /// A change in the Updates section.
    pub update_action: Option<crate::updates::SettingsAction>,
    /// A web link to open in the browser.
    pub open_link: Option<String>,
    pub error: Option<String>,
}

/// Run a future to completion on the current (background) thread.
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake};
    struct ThreadWaker(std::thread::Thread);
    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Arc::new(ThreadWaker(std::thread::current())).into();
    let mut cx = Context::from_waker(&waker);
    let mut f = std::pin::pin!(f);
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::park(),
        }
    }
}

impl SettingsView {
    /// Settings whose Expert cloud setting keeps the user's own OAuth
    /// client in `client_file` (`None`: hidden).
    pub fn with_client_file(client_file: Option<PathBuf>) -> SettingsView {
        let mut s = SettingsView::default();
        s.cloud.client_file = client_file;
        s
    }

    pub fn show(&mut self, tab: Option<SettingsTab>) {
        if !self.open {
            self.has_settings.clear();
        }
        self.open = true;
        // ⌘, while it is open brings it to the front.
        self.window.request_focus();
        if let Some(t) = tab {
            self.tab = t;
        }
    }

    /// Settings → Plugins → Discover ("Browse Plugins…").
    pub fn show_discover(&mut self) {
        self.plugins_page = PluginsPage::Discover;
        self.show(Some(SettingsTab::Plugins));
    }

    /// Bring the open window to the front (and make it key) next frame.
    pub fn bring_to_front(&mut self) {
        self.window.request_focus();
    }

    /// Test seam: a bring-to-front is pending.
    #[cfg(test)]
    pub fn focus_requested(&self) -> bool {
        self.window.focus_requested()
    }

    /// Test seam: expand a plugin's log.
    #[cfg(test)]
    pub fn expand_log(&mut self, id: &str) {
        self.show_log.insert(id.to_string());
    }

    /// A confirmation sheet is up (keyboard shortcuts stay off).
    pub fn modal_open(&self) -> bool {
        self.open && self.confirm.is_some()
    }

    /// Show the folder picker without blocking the UI: rfd's async dialog
    /// is a sheet on macOS; the future is driven on a background thread.
    /// `archive`: a `.mantaplugin` / `.zip` file, else a plugin folder.
    fn pick_plugin(&mut self, ctx: &egui::Context, archive: bool) {
        if self.picking.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        let start = if archive {
            dirs::download_dir().or_else(dirs::home_dir)
        } else {
            dirs::home_dir()
        };
        std::thread::spawn(move || {
            let title = if archive {
                t!("plugins.choose_file")
            } else {
                t!("plugins.choose_folder")
            };
            let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
            if let Some(dir) = start {
                dialog = dialog.set_directory(dir);
            }
            let picked = if archive {
                let dialog = dialog.add_filter(
                    t!("plugins.file_filter"),
                    manta_plugin::install::ARCHIVE_EXTENSIONS,
                );
                block_on(dialog.pick_file())
            } else {
                block_on(dialog.pick_folder())
            };
            let _ = tx.send(picked.map(|h| h.path().to_path_buf()));
            ctx.request_repaint();
        });
        self.picking = Some(rx);
    }

    /// "New tabs open in" → "Other…": a folder picker like the plugin one.
    fn pick_new_tab_folder(&mut self, ctx: &egui::Context, current: Option<PathBuf>) {
        if self.picking_new_tabs.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut dialog = rfd::AsyncFileDialog::new().set_title(t!("settings.new_tabs.choose"));
            if let Some(dir) = current.or_else(dirs::home_dir) {
                dialog = dialog.set_directory(dir);
            }
            let picked = block_on(dialog.pick_folder());
            let _ = tx.send(picked.map(|h| h.path().to_path_buf()));
            ctx.request_repaint();
        });
        self.picking_new_tabs = Some(rx);
    }

    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        host: &mut PluginHost,
        prefs: &mut Prefs,
    ) -> SettingsOutcome {
        let mut out = SettingsOutcome::default();
        if let Some(rx) = &self.picking_new_tabs {
            match rx.try_recv() {
                Ok(Some(path)) => {
                    self.picking_new_tabs = None;
                    prefs.new_tabs = NewTabFolder::Other(path);
                    out.prefs_changed = true;
                }
                Ok(None) | Err(mpsc::TryRecvError::Disconnected) => self.picking_new_tabs = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(rx) = &self.picking {
            match rx.try_recv() {
                Ok(Some(path)) => {
                    self.picking = None;
                    host.review_install(path);
                }
                Ok(None) | Err(mpsc::TryRecvError::Disconnected) => self.picking = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if !self.open {
            self.window.hidden(ctx, AuxKind::Settings);
            return out;
        }
        let spec = Spec {
            kind: AuxKind::Settings,
            title: t!("settings.title"),
            size: egui::vec2(SIZE[0], SIZE[1]),
            min_size: egui::vec2(480.0, 360.0),
            margin: egui::Margin::ZERO,
            activate: true,
            zoomable: false,
        };
        let mut window = std::mem::take(&mut self.window);
        let (_, closed) = window.show(ctx, &spec, |ui| {
            self.contents(ui, host, prefs, &mut out);
            // Confirmations are sheets of the Settings window.
            self.confirm_ui(&ui.ctx().clone(), host);
        });
        self.window = window;
        if closed.0 {
            self.open = false;
            self.window.hidden(ctx, AuxKind::Settings);
        }
        out
    }

    /// The window content: toolbar tabs and the scrolling page.
    pub(crate) fn contents(
        &mut self,
        ui: &mut egui::Ui,
        host: &mut PluginHost,
        prefs: &mut Prefs,
        out: &mut SettingsOutcome,
    ) {
        // The tabs are a fixed header: the page scrolls in its own rect
        // below the separator, so neither its content nor the scroll
        // area's top edge fade can reach into the tab bar.
        let bar = self.tab_strip(ui);
        let mut page = ui.max_rect();
        page.min.y = bar.bottom();
        let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]);
        if let Some(y) = self.scroll_to.take() {
            area = area.vertical_scroll_offset(y);
        }
        let mut page_ui = ui.new_child(egui::UiBuilder::new().max_rect(page).layout(*ui.layout()));
        let scrolled = area.show(&mut page_ui, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(20, space::LG as i8 + 2))
                .show(ui, |ui| match self.tab {
                    SettingsTab::General => {
                        let mut pick_folder = false;
                        out.prefs_changed |= general_ui(
                            ui,
                            prefs,
                            self.viewer.as_ref(),
                            &mut out.viewer_switch,
                            &mut pick_folder,
                        );
                        if pick_folder {
                            self.pick_new_tab_folder(&ui.ctx().clone(), prefs.new_tabs.dir());
                        }
                        ui.add_space(space::MD);
                        out.prefs_changed |=
                            cloud_section(ui, &mut self.cloud, prefs, &mut out.open_link);
                        if let Some(info) = &self.updates {
                            ui.add_space(space::MD);
                            let reveal = std::mem::take(&mut self.reveal_updates);
                            out.update_action =
                                updates_section(ui, info, reveal).or(out.update_action);
                        }
                        ui.add_space(space::MD);
                        about_section(ui, &mut out.open_link);
                    }
                    SettingsTab::Plugins => self.plugins_ui(ui, host, out),
                });
        });
        ui.advance_cursor_after_rect(page_ui.min_rect());
        self.last_layout = Some((bar, scrolled.inner_rect));
    }

    /// macOS-style toolbar tabs: icon above label, centred. Returns the
    /// bar's rect (its bottom edge is the separator).
    fn tab_strip(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        let p = pal(ui);
        let (bar, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 58.0), Sense::hover());
        ui.painter().hline(
            bar.x_range(),
            bar.bottom() - 0.5,
            Stroke::new(1.0, p.separator),
        );
        let tabs = [
            (
                SettingsTab::General,
                icons::GEAR_SIX,
                t!("settings.general"),
            ),
            (
                SettingsTab::Plugins,
                icons::PUZZLE_PIECE,
                t!("settings.plugins"),
            ),
        ];
        let w = 76.0;
        let total = w * tabs.len() as f32 + space::XS;
        let mut x = bar.center().x - total / 2.0;
        for (tab, glyph, label) in tabs {
            let r = egui::Rect::from_min_size(egui::pos2(x, bar.top() + 5.0), egui::vec2(w, 48.0));
            let resp = ui.interact(
                r,
                ui.scope_id().with(("settings-tab", tab as u8)),
                Sense::click(),
            );
            let on = self.tab == tab;
            if on {
                ui.painter()
                    .rect_filled(r, CornerRadius::same(radius::PANEL), p.hover);
            } else if resp.hovered() {
                ui.painter().rect_filled(
                    r,
                    CornerRadius::same(radius::PANEL),
                    p.hover.gamma_multiply(0.6),
                );
            }
            let color = if on { p.accent_text } else { p.text_secondary };
            let ir = egui::Rect::from_center_size(
                egui::pos2(r.center().x, r.top() + 17.0),
                egui::vec2(24.0, 24.0),
            );
            widgets::paint_icon(ui, ir, glyph, 20.0, color, on);
            ui.painter().text(
                egui::pos2(r.center().x, r.bottom() - 9.0),
                egui::Align2::CENTER_CENTER,
                label,
                if on {
                    theme::medium(text::SMALL)
                } else {
                    theme::regular(text::SMALL)
                },
                if on { p.text } else { p.text_secondary },
            );
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, on, label));
            if resp.clicked() {
                self.tab = tab;
            }
            x += w + space::XS;
        }
        let help = match self.tab {
            SettingsTab::General => "16-einstellungen.md#allgemein",
            SettingsTab::Plugins => "16-einstellungen.md#plugins",
        };
        let hr = egui::Rect::from_center_size(
            egui::pos2(bar.right() - space::XL - 10.0, bar.center().y),
            egui::vec2(20.0, 20.0),
        );
        // A child, not `scope_builder`: placing the button must not move
        // the parent's cursor back up into the bar.
        let mut help_ui = ui.new_child(egui::UiBuilder::new().max_rect(hr));
        crate::help::help_button(&mut help_ui, help);
        bar
    }

    fn plugins_ui(&mut self, ui: &mut egui::Ui, host: &mut PluginHost, out: &mut SettingsOutcome) {
        let p = pal(ui);
        ui.spacing_mut().item_spacing.y = space::SM;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = space::XXS;
                ui.label(
                    RichText::new(t!("settings.plugins"))
                        .font(theme::semibold(text::HEADLINE))
                        .color(p.text),
                );
                ui.label(RichText::new(t!("plugins.subtitle")).color(p.text_secondary));
            });
        });
        ui.add_space(space::XS);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::SM;
            let label = if self.picking.is_some() {
                t!("plugins.choosing")
            } else {
                t!("plugins.install_plugin")
            };
            let r = widgets::button(ui, label, ButtonKind::Primary, self.picking.is_none());
            let r = widgets::tooltip(r, t!("plugins.install_plugin.tip"), "");
            let mut pick = None;
            egui::Popup::menu(&r)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.set_min_width(240.0);
                    if ui.button(t!("plugins.install_from_file")).clicked() {
                        pick = Some(true);
                        ui.close();
                    }
                    if ui.button(t!("plugins.install_from_folder")).clicked() {
                        pick = Some(false);
                        ui.close();
                    }
                });
            if let Some(archive) = pick {
                self.pick_plugin(ui.ctx(), archive);
            }
            let has_dir = host.plugins_dir().is_some();
            let r = widgets::button(
                ui,
                t!("plugins.open_folder"),
                ButtonKind::Secondary,
                has_dir,
            );
            if r.clicked() {
                if let Some(dir) = host.plugins_dir() {
                    let _ = std::fs::create_dir_all(dir);
                    if let Err(e) = open::that_detached(dir) {
                        out.error = Some(t!("plugins.open_folder.error", error = e));
                    }
                }
            }
            let r = widgets::button(ui, t!("plugins.reload"), ButtonKind::Secondary, true);
            if widgets::tooltip(r, t!("plugins.reload.tip"), "").clicked() {
                host.rescan();
            }
        });
        ui.add_space(space::MD);
        let pages = [
            (PluginsPage::Installed, t!("plugins.page.installed")),
            (PluginsPage::Discover, t!("plugins.page.discover")),
        ];
        widgets::segmented(ui, "plugins-page", &mut self.plugins_page, &pages);
        ui.add_space(space::MD);
        self.plugin_drops(ui, host);
        match self.plugins_page {
            PluginsPage::Installed => self.installed_ui(ui, host),
            PluginsPage::Discover => discover_ui(ui, host, out),
        }
    }

    /// `.mantaplugin` files dragged onto the Plugins tab: a hint while they
    /// hover, the review sheet when dropped (one at a time).
    fn plugin_drops(&mut self, ui: &mut egui::Ui, host: &mut PluginHost) {
        let (mut hovered, mut dropped) = ui.ctx().input(|i| {
            (
                i.raw
                    .hovered_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect::<Vec<_>>(),
                i.raw
                    .dropped_files
                    .iter()
                    .map(|f| f.path().to_path_buf())
                    .collect::<Vec<_>>(),
            )
        });
        hovered.append(&mut self.hovered);
        dropped.append(&mut self.dropped);
        if self.confirm.is_some() {
            return;
        }
        if let Some(path) = dropped.into_iter().find(|p| is_plugin_drop(p)) {
            log::info!("plugin dropped on Settings: {}", path.display());
            host.review_install(path);
            return;
        }
        if hovered.iter().any(|p| is_plugin_drop(p)) {
            let p = pal(ui);
            ui.ctx().request_repaint();
            egui::Frame::new()
                .fill(p.accent_soft)
                .stroke(Stroke::new(1.5, p.accent))
                .corner_radius(CornerRadius::same(radius::PANEL))
                .inner_margin(egui::Margin::symmetric(space::LG as i8, space::MD as i8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        widgets::icon(ui, icons::DOWNLOAD_SIMPLE, 16.0, p.accent_text);
                        ui.label(
                            RichText::new(t!("plugins.drop"))
                                .font(theme::medium(text::BODY))
                                .color(p.accent_text),
                        );
                    });
                });
            ui.add_space(space::MD);
        }
    }

    /// Settings → Plugins → Installed: built-in, installed and bundled
    /// plugins.
    fn installed_ui(&mut self, ui: &mut egui::Ui, host: &mut PluginHost) {
        let p = pal(ui);
        let rows = host.rows();
        let bundled = host.bundled_rows();
        let (builtin, installed): (Vec<&PluginRow>, Vec<&PluginRow>) =
            rows.iter().partition(|r| r.kind == PluginKind::Builtin);
        if !builtin.is_empty() {
            section_header(ui, t!("plugins.section.builtin"), "14-plugins.md");
            for row in builtin {
                self.plugin_card(ui, host, row, None);
            }
            ui.add_space(space::MD);
        }
        section_header(ui, t!("plugins.section.installed"), "15-eigene-plugins.md");
        if installed.is_empty() {
            egui::Frame::new()
                .fill(p.stripe)
                .corner_radius(CornerRadius::same(radius::PANEL))
                .inner_margin(egui::Margin::symmetric(space::LG as i8, space::LG as i8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        widgets::icon(ui, icons::PUZZLE_PIECE, 22.0, p.text_tertiary);
                        ui.label(RichText::new(t!("plugins.none_installed")).color(p.text));
                        ui.label(
                            RichText::new(t!("plugins.none_installed.note"))
                                .font(theme::regular(text::SMALL))
                                .color(p.text_secondary),
                        );
                    });
                });
        }
        for row in installed {
            let update = bundled.iter().find(|b| {
                b.info.id == row.info.id && row.kind == PluginKind::External && b.update_available
            });
            self.plugin_card(ui, host, row, update);
        }
        let available: Vec<&BundledRow> = bundled
            .iter()
            .filter(|b| b.installed_version.is_none())
            .collect();
        if !available.is_empty() {
            ui.add_space(space::MD);
            section_header(
                ui,
                t!("plugins.section.available"),
                "16-einstellungen.md#plugins",
            );
            ui.label(
                RichText::new(t!("plugins.available.note"))
                    .font(theme::regular(text::SMALL))
                    .color(p.text_secondary),
            );
            for b in available {
                available_card(ui, host, b);
            }
        }
        ui.add_space(space::LG);
        ui.horizontal_top(|ui| {
            widgets::icon(ui, icons::SHIELD_WARNING, 15.0, p.text_secondary);
            ui.add(
                egui::Label::new(
                    RichText::new(t!("plugins.warning"))
                        .font(theme::regular(text::SMALL))
                        .color(p.text_secondary),
                )
                .wrap(),
            );
        });
    }

    fn plugin_card(
        &mut self,
        ui: &mut egui::Ui,
        host: &mut PluginHost,
        row: &PluginRow,
        update: Option<&BundledRow>,
    ) {
        let p = pal(ui);
        let id = row.info.id.clone();
        let error = match &row.status {
            PluginStatus::Error(e) => Some(e.clone()),
            _ => None,
        };
        let border = if error.is_some() {
            p.danger.gamma_multiply(0.5)
        } else {
            p.separator
        };
        egui::Frame::new()
            .fill(p.content)
            .stroke(Stroke::new(1.0, border))
            .corner_radius(CornerRadius::same(radius::PANEL + 2))
            .inner_margin(egui::Margin::symmetric(
                space::LG as i8,
                space::LG as i8 - 2,
            ))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.push_id(&id, |ui| {
                    ui.horizontal_top(|ui| {
                        // Icon tile.
                        let (ir, _) =
                            ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::hover());
                        let (fg, bg) = match row.kind {
                            PluginKind::Builtin => (p.accent_text, p.accent_soft),
                            PluginKind::External => (p.purple, p.purple_soft),
                            PluginKind::Invalid => (p.danger, p.danger_soft),
                        };
                        ui.painter()
                            .rect_filled(ir, CornerRadius::same(radius::PANEL + 1), bg);
                        let glyph = if row.kind == PluginKind::Invalid {
                            icons::WARNING
                        } else {
                            plugin_ui::plugin_glyph(&row.info.id, row.info.builtin)
                        };
                        widgets::paint_icon(ui, ir, glyph, 20.0, fg, false);
                        ui.add_space(space::XS);

                        let toggle_w = 44.0;
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - toggle_w);
                            ui.spacing_mut().item_spacing.y = space::XXS;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = space::SM;
                                ui.label(
                                    RichText::new(&row.info.name)
                                        .font(theme::semibold(text::BODY + 1.0))
                                        .color(p.text),
                                );
                                if !row.info.version.is_empty() {
                                    ui.label(
                                        RichText::new(&row.info.version)
                                            .font(theme::regular(text::SMALL))
                                            .color(p.text_secondary),
                                    );
                                }
                                let (tag, tfg, tbg) = match row.kind {
                                    PluginKind::Builtin => (
                                        t!("plugins.kind.builtin"),
                                        p.text_secondary,
                                        p.neutral_soft,
                                    ),
                                    PluginKind::External if row.dev => {
                                        (t!("plugins.kind.dev"), p.accent_text, p.accent_soft)
                                    }
                                    PluginKind::External if row.replaced_by.is_some() => {
                                        (t!("plugins.kind.replaced"), p.accent_text, p.accent_soft)
                                    }
                                    PluginKind::External => {
                                        (t!("plugins.kind.external"), p.purple, p.purple_soft)
                                    }
                                    PluginKind::Invalid => {
                                        (t!("plugins.kind.invalid"), p.danger, p.danger_soft)
                                    }
                                };
                                widgets::chip(ui, None, tag, tfg, tbg);
                                if let Some(sb) = &row.sandbox {
                                    sandbox_chip(ui, p, sb, &row.permissions);
                                }
                            });
                            if !row.info.description.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&row.info.description)
                                            .color(p.text_secondary),
                                    )
                                    .wrap(),
                                );
                            }
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = space::SM;
                                if !row.info.author.is_empty() {
                                    ui.label(
                                        RichText::new(t!("plugins.by", author = row.info.author))
                                            .font(theme::regular(text::SMALL))
                                            .color(p.text_secondary),
                                    );
                                    ui.label(RichText::new("·").color(p.text_tertiary));
                                }
                                status_label(ui, p, &row.status);
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.add_space(2.0);
                            let can_toggle =
                                row.kind != PluginKind::Invalid && row.replaced_by.is_none();
                            let r = widgets::toggle_switch(ui, row.enabled, can_toggle);
                            let tip = if row.enabled {
                                t!("plugins.turn_off")
                            } else {
                                t!("plugins.turn_on")
                            };
                            if widgets::tooltip(r, tip, "").clicked() {
                                let turn_on = !row.enabled;
                                if turn_on && !host.is_trusted(&id) {
                                    if let Some(review) = host.review(&id) {
                                        self.confirm = Some(Confirm::Enable {
                                            id: id.clone(),
                                            review: Box::new(review),
                                        });
                                    }
                                } else {
                                    host.set_enabled(&id, turn_on);
                                }
                            }
                        });
                    });

                    if let Some(e) = &error {
                        ui.add_space(space::SM);
                        error_banner(ui, p, e);
                    }
                    if let Some(by) = &row.replaced_by {
                        let name = host
                            .rows()
                            .into_iter()
                            .find(|r| &r.info.id == by)
                            .map(|r| r.info.name)
                            .unwrap_or_default();
                        ui.add_space(space::SM);
                        replaced_banner(ui, p, &t!("plugins.replaced.note", name = name));
                    }
                    if let Some(u) = update {
                        ui.add_space(space::SM);
                        update_banner(ui, host, p, &row.info.version, u);
                    } else if row.kind == PluginKind::External && !row.dev {
                        // A newer version in the community directory.
                        if let Some(u) = host.registry_update(&id) {
                            ui.add_space(space::SM);
                            directory_update_banner(ui, host, p, &row.info.version, &u);
                        }
                    }

                    // Disclosures and actions.
                    let has_log = row.kind != PluginKind::Builtin;
                    let probe = !self.show_settings.contains(&id)
                        && row.enabled
                        && row.status != PluginStatus::Stopped;
                    if probe && !self.has_settings.contains_key(&id) {
                        let mut child = ui.new_child(
                            egui::UiBuilder::new()
                                .invisible()
                                .sizing_pass()
                                .id_salt("probe"),
                        );
                        let has = host.settings_ui(&id, &mut child);
                        self.has_settings.insert(id.clone(), has);
                    }
                    let has_settings = self.has_settings.get(&id).copied().unwrap_or(false);
                    let external = row.kind != PluginKind::Builtin;
                    if has_log || has_settings || external {
                        ui.add_space(space::SM);
                        ui.horizontal(|ui| {
                            ui.add_space(36.0 + space::XS + space::MD);
                            ui.spacing_mut().item_spacing.x = space::MD;
                            if has_settings
                                && disclosure(
                                    ui,
                                    t!("plugins.settings"),
                                    self.show_settings.contains(&id),
                                )
                            {
                                toggle(&mut self.show_settings, &id);
                            }
                            if has_log {
                                let n = row.log.len();
                                let label = if n == 0 {
                                    t!("plugins.show_log").to_string()
                                } else {
                                    t!("plugins.show_log_n", n = n)
                                };
                                if disclosure(ui, &label, self.show_log.contains(&id)) {
                                    toggle(&mut self.show_log, &id);
                                }
                            }
                            if external {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.x = space::XS;
                                        if !row.dev
                                            && link_button(
                                                ui,
                                                icons::TRASH,
                                                t!("plugins.uninstall"),
                                                p.danger,
                                            )
                                            .clicked()
                                        {
                                            self.confirm = Some(Confirm::Uninstall {
                                                id: id.clone(),
                                                name: row.info.name.clone(),
                                            });
                                        }
                                        if let Some(dir) = &row.dir {
                                            if link_button(
                                                ui,
                                                icons::FOLDER_OPEN,
                                                t!("plugins.show_folder"),
                                                p.text_secondary,
                                            )
                                            .clicked()
                                            {
                                                let _ = open::that_detached(dir);
                                            }
                                        }
                                        if row.kind == PluginKind::External
                                            && link_button(
                                                ui,
                                                icons::ARROW_CLOCKWISE,
                                                t!("plugins.reload"),
                                                p.text_secondary,
                                            )
                                            .clicked()
                                        {
                                            host.reload(&id);
                                        }
                                    },
                                );
                            }
                        });
                    }
                    if has_settings && self.show_settings.contains(&id) {
                        ui.add_space(space::XS);
                        egui::Frame::new()
                            .fill(p.stripe)
                            .corner_radius(CornerRadius::same(radius::PANEL))
                            .inner_margin(egui::Margin::same(space::LG as i8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                if !host.settings_ui(&id, ui) {
                                    self.has_settings.insert(id.clone(), false);
                                }
                            });
                    }
                    if has_log && self.show_log.contains(&id) {
                        ui.add_space(space::XS);
                        log_view(ui, p, &row.log);
                    }
                });
            });
        ui.add_space(space::XS);
    }

    fn confirm_ui(&mut self, ctx: &egui::Context, host: &mut PluginHost) {
        let Some(c) = self.confirm.clone() else {
            return;
        };
        let p = theme::pal_ctx(ctx);
        let mut result = DialogResult::Open;
        let modal = dialogs::dialog_modal("plugin-confirm", ctx).show(ctx, |ui| {
            ui.set_width(460.0);
            match &c {
                Confirm::Enable { review, .. } => {
                    dialogs::dialog_header(
                        ui,
                        icons::SHIELD_WARNING,
                        p.warning,
                        p.warning_soft,
                        &t!("plugins.confirm_enable.title", name = review.name),
                        &review_subtitle(review),
                    );
                    ui.add_space(space::MD);
                    review_details(ui, p, review, false);
                    ui.add_space(space::MD);
                    warning_note(ui, p, t!("plugins.confirm_enable.body"));
                    if review.quarantined {
                        ui.add_space(space::SM);
                        info_note(ui, p, t!("plugins.review.quarantine_enable"));
                    }
                    ui.add_space(space::LG);
                    dialogs::buttons(
                        ui,
                        t!("plugins.turn_on"),
                        ButtonKind::Primary,
                        true,
                        &mut result,
                    );
                }
                Confirm::Install(review) => {
                    let title = if review.installed_version.is_some() {
                        t!("plugins.review.title_update", name = review.name)
                    } else {
                        t!("plugins.review.title", name = review.name)
                    };
                    dialogs::dialog_header(
                        ui,
                        icons::PUZZLE_PIECE,
                        p.purple,
                        p.purple_soft,
                        &title,
                        &review_subtitle(review),
                    );
                    ui.add_space(space::MD);
                    review_details(ui, p, review, true);
                    ui.add_space(space::MD);
                    warning_note(ui, p, t!("plugins.review.warning"));
                    if review.quarantined {
                        ui.add_space(space::SM);
                        info_note(ui, p, t!("plugins.review.quarantine_install"));
                    }
                    ui.add_space(space::LG);
                    let ok = if review.installed_version.is_some() {
                        t!("plugins.update")
                    } else {
                        t!("plugins.install")
                    };
                    dialogs::buttons(ui, ok, ButtonKind::Primary, true, &mut result);
                }
                Confirm::Replace {
                    name,
                    installed_version,
                    new_version,
                    ..
                } => {
                    dialogs::dialog_header(
                        ui,
                        icons::PUZZLE_PIECE,
                        p.accent_text,
                        p.accent_soft,
                        &t!("plugins.confirm_replace.title", name = name),
                        "",
                    );
                    ui.add_space(space::MD);
                    let what = if installed_version == new_version {
                        t!("plugins.confirm_replace.same", version = new_version)
                    } else {
                        t!(
                            "plugins.confirm_replace.different",
                            installed = installed_version,
                            version = new_version
                        )
                    };
                    ui.add(egui::Label::new(RichText::new(what).color(p.text)).wrap());
                    ui.add_space(space::LG);
                    dialogs::buttons(
                        ui,
                        t!("plugins.replace"),
                        ButtonKind::Primary,
                        true,
                        &mut result,
                    );
                }
                Confirm::Uninstall { name, .. } => {
                    dialogs::dialog_header(
                        ui,
                        icons::TRASH,
                        p.danger,
                        p.danger_soft,
                        &t!("plugins.confirm_uninstall.title", name = name),
                        "",
                    );
                    ui.add_space(space::MD);
                    ui.add(
                        egui::Label::new(
                            RichText::new(t!("plugins.confirm_uninstall.body")).color(p.text),
                        )
                        .wrap(),
                    );
                    ui.add_space(space::LG);
                    dialogs::buttons(
                        ui,
                        t!("plugins.uninstall"),
                        ButtonKind::Danger,
                        true,
                        &mut result,
                    );
                }
            }
        });
        match dialogs::finish(ctx, modal.should_close(), result) {
            DialogResult::Submit => {
                self.confirm = None;
                match c {
                    Confirm::Enable { id, review } => {
                        // Only now, with the user's OK: downloaded files
                        // lose macOS's quarantine flag so they can run.
                        if review.quarantined {
                            host.clear_quarantine(&id);
                        }
                        host.trust(&id);
                        host.set_enabled(&id, true);
                    }
                    Confirm::Install(review) => host.confirm_install(&review),
                    Confirm::Replace { src, .. } => host.install(src, true),
                    Confirm::Uninstall { id, .. } => host.uninstall(&id),
                }
            }
            DialogResult::Cancel => {
                if let Some(Confirm::Install(review)) = self.confirm.take() {
                    host.cancel_install(&review);
                }
            }
            _ => {}
        }
    }
}

/// "Version x is available" with an Update button, on an installed card.
fn update_banner(
    ui: &mut egui::Ui,
    host: &mut PluginHost,
    p: &Palette,
    installed: &str,
    u: &BundledRow,
) {
    egui::Frame::new()
        .fill(p.accent_soft)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::icon(ui, icons::DOWNLOAD_SIMPLE, 15.0, p.accent_text);
                ui.label(
                    RichText::new(t!("plugins.update.available", version = u.info.version))
                        .font(theme::medium(text::SMALL + 0.5))
                        .color(p.accent_text),
                );
                ui.label(
                    RichText::new(t!("plugins.update.installed", version = installed))
                        .font(theme::regular(text::SMALL))
                        .color(p.text_secondary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if u.installing {
                        t!("plugins.updating")
                    } else {
                        t!("plugins.update")
                    };
                    let r = widgets::button(ui, label, ButtonKind::Primary, !u.installing);
                    if widgets::tooltip(r, t!("plugins.update.tip"), "").clicked() {
                        host.install_bundled(&u.info.id);
                    }
                });
            });
        });
}

/// A bundled plugin that isn't installed yet, with an Install button.
fn available_card(ui: &mut egui::Ui, host: &mut PluginHost, b: &BundledRow) {
    let p = pal(ui);
    egui::Frame::new()
        .fill(p.content)
        .stroke(Stroke::new(1.0, p.separator))
        .corner_radius(CornerRadius::same(radius::PANEL + 2))
        .inner_margin(egui::Margin::symmetric(
            space::LG as i8,
            space::LG as i8 - 2,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.push_id(("bundled", &b.info.id), |ui| {
                ui.horizontal_top(|ui| {
                    let (ir, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::hover());
                    ui.painter().rect_filled(
                        ir,
                        CornerRadius::same(radius::PANEL + 1),
                        p.neutral_soft,
                    );
                    widgets::paint_icon(
                        ui,
                        ir,
                        plugin_ui::plugin_glyph(&b.info.id, false),
                        20.0,
                        p.text_secondary,
                        false,
                    );
                    ui.add_space(space::XS);
                    let button_w = 116.0;
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - button_w);
                        ui.spacing_mut().item_spacing.y = space::XXS;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = space::SM;
                            ui.label(
                                RichText::new(&b.info.name)
                                    .font(theme::semibold(text::BODY + 1.0))
                                    .color(p.text),
                            );
                            ui.label(
                                RichText::new(&b.info.version)
                                    .font(theme::regular(text::SMALL))
                                    .color(p.text_secondary),
                            );
                        });
                        if !b.info.description.is_empty() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&b.info.description).color(p.text_secondary),
                                )
                                .wrap(),
                            );
                        }
                        if !b.info.author.is_empty() {
                            ui.label(
                                RichText::new(t!("plugins.by", author = b.info.author))
                                    .font(theme::regular(text::SMALL))
                                    .color(p.text_secondary),
                            );
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let label = if b.installing {
                            t!("plugins.installing")
                        } else {
                            t!("plugins.install")
                        };
                        let r = widgets::button(ui, label, ButtonKind::Secondary, !b.installing);
                        if widgets::tooltip(r, t!("plugins.install.tip"), "").clicked() {
                            host.install_bundled(&b.info.id);
                        }
                    });
                });
            });
        });
    ui.add_space(space::XS);
}

/// "Version x is available in the plugin directory" on an installed card.
fn directory_update_banner(
    ui: &mut egui::Ui,
    host: &mut PluginHost,
    p: &Palette,
    installed: &str,
    u: &RegistryRow,
) {
    egui::Frame::new()
        .fill(p.accent_soft)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::icon(ui, icons::DOWNLOAD_SIMPLE, 15.0, p.accent_text);
                ui.label(
                    RichText::new(t!(
                        "plugins.discover.update_available",
                        version = u.entry.latest.version
                    ))
                    .font(theme::medium(text::SMALL + 0.5))
                    .color(p.accent_text),
                );
                ui.label(
                    RichText::new(t!("plugins.update.installed", version = installed))
                        .font(theme::regular(text::SMALL))
                        .color(p.text_secondary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if u.downloading {
                        t!("plugins.discover.downloading")
                    } else {
                        t!("plugins.update")
                    };
                    let r = widgets::button(ui, label, ButtonKind::Primary, !u.downloading);
                    if widgets::tooltip(r, t!("plugins.discover.install.tip"), "").clicked() {
                        host.install_from_registry(&u.entry.id);
                    }
                });
            });
        });
}

/// "Sandboxed" (the permissions are enforced) or "Not sandboxed" next to
/// an external plugin's name.
fn sandbox_chip(ui: &mut egui::Ui, p: &Palette, state: &SandboxState, perms: &[String]) {
    let (glyph, label, fg, bg, tip) = match state {
        SandboxState::Enforced => {
            let tip = if perms.is_empty() {
                t!("plugins.sandboxed.tip_none").to_string()
            } else {
                let list = perms
                    .iter()
                    .map(|x| permission_label(x).1)
                    .collect::<Vec<_>>()
                    .join(", ");
                t!("plugins.sandboxed.tip", permissions = list)
            };
            (
                icons::LOCK_SIMPLE,
                t!("plugins.sandboxed"),
                p.success,
                p.success_soft,
                tip,
            )
        }
        SandboxState::Unavailable(why) => (
            icons::SHIELD_WARNING,
            t!("plugins.not_sandboxed"),
            p.warning,
            p.warning_soft,
            t!("plugins.not_sandboxed.tip", reason = why),
        ),
    };
    let r = widgets::chip(ui, Some(glyph), label, fg, bg);
    widgets::tooltip(r, &tip, "");
}

/// A file or folder that can be dropped on the Plugins tab to install it.
fn is_plugin_drop(path: &std::path::Path) -> bool {
    manta_plugin::install::is_archive(path)
        || (path.is_dir() && path.join(manta_plugin::manifest::MANIFEST_FILE).is_file())
}

/// Settings → Plugins → Discover: the community directory. Opening it is
/// what loads the list from GitHub ([`PluginHost::open_discover`]).
fn discover_ui(ui: &mut egui::Ui, host: &mut PluginHost, out: &mut SettingsOutcome) {
    host.open_discover();
    let p = pal(ui);
    let d = host.discover().clone();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        let r = widgets::button(
            ui,
            t!("plugins.discover.refresh"),
            ButtonKind::Secondary,
            !d.loading,
        );
        if widgets::tooltip(r, t!("plugins.discover.refresh.tip"), "").clicked() {
            host.refresh_registry();
        }
        if d.loading {
            ui.add(egui::Spinner::new().size(14.0).color(p.text_secondary));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let r = link_button(ui, icons::GLOBE, t!("plugins.discover.web"), p.accent_text);
            if widgets::tooltip(r, t!("plugins.discover.web.tip"), "").clicked() {
                out.open_link = Some(directory_url());
            }
        });
    });
    ui.add_space(space::XS);
    ui.horizontal_top(|ui| {
        widgets::icon(ui, icons::INFO, 14.0, p.text_tertiary);
        ui.add(
            egui::Label::new(
                RichText::new(t!("plugins.discover.privacy"))
                    .font(theme::regular(text::SMALL))
                    .color(p.text_secondary),
            )
            .wrap(),
        );
    });
    ui.add_space(space::MD);
    let date = d.fetched_at.map(|t| {
        crate::util::format_time_relative(
            Some(chrono::DateTime::<chrono::Utc>::from(t)),
            chrono::Local::now(),
        )
    });
    if d.plugins.is_empty() {
        let (glyph, title, note) = if d.loading {
            (
                icons::CLOUD_ARROW_DOWN,
                t!("plugins.discover.loading").to_string(),
                String::new(),
            )
        } else if let Some(e) = &d.error {
            (
                icons::CLOUD_SLASH,
                t!("plugins.discover.failed").to_string(),
                e.clone(),
            )
        } else {
            (
                icons::PUZZLE_PIECE,
                t!("plugins.discover.empty").to_string(),
                t!("plugins.discover.empty.note").to_string(),
            )
        };
        egui::Frame::new()
            .fill(p.stripe)
            .corner_radius(CornerRadius::same(radius::PANEL))
            .inner_margin(egui::Margin::symmetric(space::LG as i8, space::LG as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    widgets::icon(ui, glyph, 22.0, p.text_tertiary);
                    ui.label(RichText::new(title).color(p.text));
                    if !note.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(note)
                                    .font(theme::regular(text::SMALL))
                                    .color(p.text_secondary),
                            )
                            .wrap(),
                        );
                    }
                });
            });
        return;
    }
    if let (Some(e), true) = (&d.error, d.from_cache) {
        let when = date.clone().unwrap_or_default();
        note(
            ui,
            icons::CLOUD_SLASH,
            p.warning,
            p.warning_soft,
            p.text,
            &t!("plugins.discover.offline", date = when, error = e),
        );
        ui.add_space(space::MD);
    }
    for row in host.registry_rows() {
        registry_card(ui, host, &row);
    }
}

/// One directory entry: name, description, author, licence, Manta version,
/// permissions, source link and Install / Update.
fn registry_card(ui: &mut egui::Ui, host: &mut PluginHost, row: &RegistryRow) {
    let p = pal(ui);
    let e = &row.entry;
    egui::Frame::new()
        .fill(p.content)
        .stroke(Stroke::new(1.0, p.separator))
        .corner_radius(CornerRadius::same(radius::PANEL + 2))
        .inner_margin(egui::Margin::symmetric(
            space::LG as i8,
            space::LG as i8 - 2,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.push_id(("registry", &e.id), |ui| {
                ui.horizontal_top(|ui| {
                    let (ir, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::hover());
                    ui.painter().rect_filled(
                        ir,
                        CornerRadius::same(radius::PANEL + 1),
                        p.purple_soft,
                    );
                    widgets::paint_icon(
                        ui,
                        ir,
                        plugin_ui::plugin_glyph(&e.id, false),
                        20.0,
                        p.purple,
                        false,
                    );
                    ui.add_space(space::XS);
                    let button_w = 116.0;
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - button_w);
                        ui.spacing_mut().item_spacing.y = space::XXS;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = space::SM;
                            ui.label(
                                RichText::new(&row.name)
                                    .font(theme::semibold(text::BODY + 1.0))
                                    .color(p.text),
                            );
                            ui.label(
                                RichText::new(&e.latest.version)
                                    .font(theme::regular(text::SMALL))
                                    .color(p.text_secondary),
                            );
                        });
                        if !row.description.is_empty() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&row.description).color(p.text_secondary),
                                )
                                .wrap(),
                            );
                        }
                        let facts = [
                            t!("plugins.by", author = e.author),
                            e.license.clone(),
                            t!(
                                "plugins.discover.min_version",
                                version = e.min_manta_version
                            ),
                        ]
                        .join(" · ");
                        ui.add(
                            egui::Label::new(
                                RichText::new(facts)
                                    .font(theme::regular(text::SMALL))
                                    .color(p.text_secondary),
                            )
                            .wrap(),
                        );
                        if !e.permissions.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = space::XS;
                                for perm in &e.permissions {
                                    let (glyph, label) = permission_label(perm);
                                    widgets::chip(
                                        ui,
                                        Some(glyph),
                                        &label,
                                        p.text_secondary,
                                        p.neutral_soft,
                                    );
                                }
                            });
                        }
                        let r = link_button(
                            ui,
                            icons::CODE,
                            e.repository.trim_start_matches("https://"),
                            p.accent_text,
                        );
                        if widgets::tooltip(r, t!("plugins.review.source_code"), "").clicked() {
                            let _ = open::that_detached(&e.repository);
                        }
                        if let Some(why) = &row.incompatible {
                            let msg = match why {
                                Incompatible::NeedsManta(v) => {
                                    t!("plugins.discover.needs_manta", version = v)
                                }
                                Incompatible::Platform(_) => {
                                    t!("plugins.discover.wrong_platform").to_string()
                                }
                            };
                            ui.label(
                                RichText::new(msg)
                                    .font(theme::regular(text::SMALL))
                                    .color(p.warning),
                            );
                        } else if let Some(v) = &row.installed_version {
                            ui.label(
                                RichText::new(t!(
                                    "plugins.discover.installed_version",
                                    version = v
                                ))
                                .font(theme::regular(text::SMALL))
                                .color(p.success),
                            );
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let installed = row.installed_version.is_some();
                        let (label, kind, enabled) = if row.downloading {
                            (
                                t!("plugins.discover.downloading"),
                                ButtonKind::Secondary,
                                false,
                            )
                        } else if row.update_available {
                            (
                                t!("plugins.update"),
                                ButtonKind::Primary,
                                row.incompatible.is_none(),
                            )
                        } else if installed {
                            (t!("plugins.review.installed"), ButtonKind::Secondary, false)
                        } else {
                            (
                                t!("plugins.install"),
                                ButtonKind::Secondary,
                                row.incompatible.is_none(),
                            )
                        };
                        let r = widgets::button(ui, label, kind, enabled);
                        if widgets::tooltip(r, t!("plugins.discover.install.tip"), "").clicked() {
                            host.install_from_registry(&e.id);
                        }
                    });
                });
            });
        });
    ui.add_space(space::XS);
}

fn toggle(set: &mut HashSet<String>, id: &str) {
    if !set.remove(id) {
        set.insert(id.to_string());
    }
}

/// The plugin directory on the website, in the app's language.
pub fn directory_url() -> String {
    use crate::i18n::Locale;
    let base = "https://manta-filemanager.com/plugins/";
    match crate::i18n::locale() {
        Locale::De => base.to_string(),
        Locale::En => format!("{base}en/"),
        Locale::Zh => format!("{base}zh/"),
        Locale::Es => format!("{base}es/"),
        Locale::Fr => format!("{base}fr/"),
        Locale::It => format!("{base}it/"),
        Locale::Ru => format!("{base}ru/"),
    }
}

/// "1.2.0 · by Anna" under the sheet's title.
fn review_subtitle(r: &PluginReview) -> String {
    if r.author.trim().is_empty() {
        r.version.clone()
    } else {
        format!("{} · {}", r.version, t!("plugins.by", author = r.author))
    }
}

/// Label and icon of a declared permission (unknown ones as written).
fn permission_label(perm: &str) -> (&'static str, String) {
    match perm {
        "read_files" => (icons::EYE, t!("plugins.perm.read_files").to_string()),
        "write_files" => (
            icons::PENCIL_SIMPLE,
            t!("plugins.perm.write_files").to_string(),
        ),
        "network" => (icons::GLOBE, t!("plugins.perm.network").to_string()),
        "run_programs" => (icons::PLAY, t!("plugins.perm.run_programs").to_string()),
        "terminal" => (icons::TERMINAL, t!("plugins.perm.terminal").to_string()),
        other => (icons::INFO, other.to_string()),
    }
}

/// What the install and trust sheets show about a plugin: description,
/// declared permissions and the facts (program, source, licence …).
fn review_details(ui: &mut egui::Ui, p: &Palette, r: &PluginReview, installing: bool) {
    if !r.description.trim().is_empty() {
        ui.add(egui::Label::new(RichText::new(&r.description).color(p.text)).wrap());
        ui.add_space(space::MD);
    }
    ui.label(
        RichText::new(t!("plugins.review.permissions"))
            .font(theme::semibold(text::SMALL + 0.5))
            .color(p.text),
    );
    ui.add_space(space::XXS);
    if r.permissions.is_empty() {
        ui.label(
            RichText::new(t!("plugins.review.no_permissions"))
                .font(theme::regular(text::SMALL + 0.5))
                .color(p.text_secondary),
        );
    }
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 1.0;
        ui.spacing_mut().interact_size.y = 16.0;
        for perm in &r.permissions {
            let (glyph, label) = permission_label(perm);
            let strong = matches!(
                perm.as_str(),
                "write_files" | "network" | "run_programs" | "terminal"
            );
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::SM;
                widgets::icon(
                    ui,
                    glyph,
                    14.0,
                    if strong { p.warning } else { p.text_secondary },
                );
                ui.label(
                    RichText::new(label)
                        .font(theme::regular(text::SMALL + 0.5))
                        .color(p.text),
                );
            });
        }
    });
    let (enforced_color, enforced) = match manta_plugin::sandbox::available() {
        Ok(()) => (p.text_tertiary, t!("plugins.review.enforced").to_string()),
        Err(_) => (p.warning, t!("plugins.review.not_enforced").to_string()),
    };
    ui.add(
        egui::Label::new(
            RichText::new(enforced)
                .font(theme::regular(text::SMALL))
                .color(enforced_color),
        )
        .wrap(),
    );
    ui.add_space(space::MD);
    egui::Frame::new()
        .fill(p.stripe)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8 + 1,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::XXS;
            kv_line(ui, p, t!("plugins.runs"), &r.command, true);
            if installing {
                let from = r
                    .source
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                kv_line(ui, p, t!("plugins.review.from"), &from, false);
            } else if let Some(dir) = &r.folder {
                kv_line(ui, p, t!("plugins.in"), &dir.display().to_string(), false);
            }
            if let Some(url) = &r.repository {
                kv_link(ui, p, t!("plugins.review.source_code"), url);
            }
            if let Some(url) = r
                .homepage
                .as_ref()
                .filter(|h| Some(*h) != r.repository.as_ref())
            {
                kv_link(ui, p, t!("plugins.review.website"), url);
            }
            let license = r
                .license
                .clone()
                .unwrap_or_else(|| t!("plugins.review.license_unknown").to_string());
            kv_line(ui, p, t!("plugins.review.license"), &license, false);
            let size = t!(
                "plugins.review.size",
                count = r.files,
                size = manta_plugin::i18n::format_size(r.bytes)
            );
            kv_line(ui, p, t!("plugins.review.contents"), &size, false);
            if r.verified_download.is_some() {
                kv_line(
                    ui,
                    p,
                    t!("plugins.review.checksum"),
                    t!("plugins.review.checksum_ok"),
                    false,
                );
            }
            if let Some(v) = &r.installed_version {
                kv_line(
                    ui,
                    p,
                    t!("plugins.review.installed"),
                    &t!("plugins.review.replaces", version = v),
                    false,
                );
            }
        });
}

fn warning_note(ui: &mut egui::Ui, p: &Palette, msg: &str) {
    note(
        ui,
        icons::SHIELD_WARNING,
        p.warning,
        p.warning_soft,
        p.text,
        msg,
    );
}

fn info_note(ui: &mut egui::Ui, p: &Palette, msg: &str) {
    note(ui, icons::INFO, p.accent_text, p.accent_soft, p.text, msg);
}

fn note(
    ui: &mut egui::Ui,
    glyph: &str,
    fg: egui::Color32,
    bg: egui::Color32,
    text_color: egui::Color32,
    msg: &str,
) {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8 + 1,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                widgets::icon(ui, glyph, 15.0, fg);
                ui.add(
                    egui::Label::new(
                        RichText::new(msg)
                            .font(theme::regular(text::SMALL + 0.5))
                            .color(text_color),
                    )
                    .wrap(),
                );
            });
        });
}

/// A key and a clickable https link (opened in the browser).
fn kv_link(ui: &mut egui::Ui, p: &Palette, key: &str, url: &str) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(KEY_W, 16.0), Sense::hover());
        ui.painter().text(
            r.left_center(),
            egui::Align2::LEFT_CENTER,
            key,
            theme::regular(text::SMALL),
            p.text_secondary,
        );
        let shown = url.trim_start_matches("https://");
        let resp = ui.add(
            egui::Label::new(
                RichText::new(shown)
                    .font(theme::regular(text::SMALL))
                    .color(p.accent_text)
                    .underline(),
            )
            .truncate()
            .sense(Sense::click()),
        );
        let resp = resp
            .on_hover_text(url)
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if resp.clicked() && url.starts_with("https://") {
            let _ = open::that_detached(url);
        }
    });
}

/// A section title with a "?" on the right that opens the manual at `help`.
fn section_header(ui: &mut egui::Ui, title: &str, help: &str) {
    ui.add_space(space::XXS);
    let title = widgets::section_title(ui, title);
    let r = egui::Rect::from_center_size(
        egui::pos2(ui.max_rect().right() - 10.0, title.rect.center().y),
        egui::vec2(20.0, 20.0),
    );
    crate::help::help_button_at(ui, r, help);
}

/// `pick_folder`: "New tabs open in" → "Other…" was chosen.
fn general_ui(
    ui: &mut egui::Ui,
    prefs: &mut Prefs,
    viewer: Option<&crate::default_viewer::ViewerStatus>,
    viewer_switch: &mut Option<bool>,
    pick_folder: &mut bool,
) -> bool {
    let p = pal(ui);
    let mut changed = false;
    ui.spacing_mut().item_spacing.y = space::SM;
    ui.label(
        RichText::new(t!("settings.general"))
            .font(theme::semibold(text::HEADLINE))
            .color(p.text),
    );
    ui.add_space(space::XS);
    section_header(
        ui,
        t!("settings.section.appearance"),
        "16-einstellungen.md#erscheinungsbild",
    );
    setting_card(ui, |ui| {
        setting_row_w(
            ui,
            t!("settings.appearance"),
            t!("settings.appearance.note"),
            350.0,
            |ui| {
                let options: Vec<(Appearance, &str)> = Appearance::ALL
                    .iter()
                    .map(|&a| (a, crate::i18n::tr(a.key())))
                    .collect();
                changed |= widgets::segmented(ui, "appearance", &mut prefs.appearance, &options);
            },
        );
        ui.add_space(space::SM);
        setting_row_w(
            ui,
            t!("settings.accent"),
            &t!(
                "settings.accent.note",
                name = crate::i18n::tr(prefs.accent.key())
            ),
            250.0,
            |ui| {
                changed |= accent_swatches(ui, &mut prefs.accent);
            },
        );
    });
    ui.add_space(space::MD);
    section_header(
        ui,
        t!("settings.section.language"),
        "16-einstellungen.md#sprache",
    );
    setting_card(ui, |ui| {
        let system = crate::i18n::system_locale().native_name();
        setting_row(
            ui,
            t!("settings.language"),
            &t!("settings.language.note", system = system),
            |ui| {
                egui::ComboBox::from_id_salt("language")
                    .width(170.0)
                    .selected_text(prefs.language.label())
                    .show_ui(ui, |ui| {
                        for l in Language::ALL {
                            if ui
                                .selectable_value(&mut prefs.language, l, l.label())
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            },
        );
    });
    if let Some(v) = viewer {
        ui.add_space(space::MD);
        section_header(
            ui,
            t!("viewer.section"),
            "16-einstellungen.md#standard-dateimanager",
        );
        setting_card(ui, |ui| default_viewer_row(ui, v, viewer_switch));
    }
    ui.add_space(space::MD);
    section_header(
        ui,
        t!("settings.section.files"),
        "16-einstellungen.md#dateien",
    );
    setting_card(ui, |ui| {
        setting_row(
            ui,
            t!("settings.show_hidden"),
            &t!(
                "settings.show_hidden.note",
                shortcut = crate::commands::Cmd::ToggleHidden.shortcut()
            ),
            |ui| {
                if widgets::toggle_switch(ui, prefs.show_hidden, true).clicked() {
                    prefs.show_hidden = !prefs.show_hidden;
                    changed = true;
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.open_archives"),
            &t!("settings.open_archives.note", ret = crate::icons::RETURN),
            |ui| {
                if widgets::toggle_switch(ui, prefs.open_archives, true).clicked() {
                    prefs.open_archives = !prefs.open_archives;
                    changed = true;
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.new_tabs"),
            t!("settings.new_tabs.note"),
            |ui| {
                egui::ComboBox::from_id_salt("new-tabs")
                    .width(170.0)
                    .selected_text(prefs.new_tabs.label())
                    .show_ui(ui, |ui| {
                        for f in NewTabFolder::PRESETS {
                            let label = f.label();
                            if ui.selectable_value(&mut prefs.new_tabs, f, label).changed() {
                                changed = true;
                            }
                        }
                        if let NewTabFolder::Other(_) = &prefs.new_tabs {
                            let label = prefs.new_tabs.label();
                            let _ = ui.selectable_label(true, label);
                        }
                        ui.separator();
                        if ui
                            .selectable_label(false, t!("settings.new_tabs.custom"))
                            .clicked()
                        {
                            *pick_folder = true;
                        }
                    });
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.view_per_folder"),
            t!("settings.view_per_folder.note"),
            |ui| {
                if widgets::toggle_switch(ui, prefs.remember_view_per_folder, true).clicked() {
                    prefs.remember_view_per_folder = !prefs.remember_view_per_folder;
                    changed = true;
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.mouse_nav"),
            &t!(
                "settings.mouse_nav.note",
                back = crate::commands::Cmd::Back.shortcut(),
                forward = crate::commands::Cmd::Forward.shortcut()
            ),
            |ui| {
                egui::ComboBox::from_id_salt("mouse-nav")
                    .width(170.0)
                    .selected_text(prefs.mouse_nav.label())
                    .show_ui(ui, |ui| {
                        for m in MouseNav::ALL {
                            if ui
                                .selectable_value(&mut prefs.mouse_nav, m, m.label())
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.remote_thumbnails"),
            t!("settings.remote_thumbnails.note"),
            |ui| {
                if widgets::toggle_switch(ui, prefs.remote_thumbnails, true).clicked() {
                    prefs.remote_thumbnails = !prefs.remote_thumbnails;
                    changed = true;
                }
            },
        );
        if cfg!(target_os = "macos") {
            ui.add_space(space::SM);
            setting_row(
                ui,
                t!("settings.system_icons"),
                t!("settings.system_icons.note"),
                |ui| {
                    if widgets::toggle_switch(ui, prefs.system_icons, true).clicked() {
                        prefs.system_icons = !prefs.system_icons;
                        changed = true;
                    }
                },
            );
        }
    });
    if cfg!(target_os = "macos") {
        ui.add_space(space::MD);
        section_header(
            ui,
            t!("settings.section.preview"),
            "16-einstellungen.md#vorschau",
        );
        setting_card(ui, |ui| {
            setting_row(
                ui,
                t!("settings.space_opens"),
                t!("settings.space_opens.note"),
                |ui| {
                    egui::ComboBox::from_id_salt("space-action")
                        .width(170.0)
                        .selected_text(prefs.space_action.label())
                        .show_ui(ui, |ui| {
                            for a in SpaceAction::ALL {
                                if ui
                                    .selectable_value(&mut prefs.space_action, a, a.label())
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                        });
                },
            );
        });
    }
    ui.add_space(space::MD);
    section_header(
        ui,
        t!("settings.section.copy_move"),
        "16-einstellungen.md#kopieren-bewegen",
    );
    setting_card(ui, |ui| {
        setting_row_w(
            ui,
            t!("settings.drop"),
            t!("settings.drop.note"),
            250.0,
            |ui| {
                egui::ComboBox::from_id_salt("drop-action")
                    .width(230.0)
                    .selected_text(prefs.drop_action.label())
                    .show_ui(ui, |ui| {
                        for a in DropAction::ALL {
                            if ui
                                .selectable_value(&mut prefs.drop_action, a, a.label())
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.conflict"),
            t!("settings.conflict.note"),
            |ui| {
                egui::ComboBox::from_id_salt("default-conflict")
                    .width(170.0)
                    .selected_text(dialogs::conflict_label(prefs.conflict.policy()))
                    .show_ui(ui, |ui| {
                        for c in ConflictPref::ALL {
                            if ui
                                .selectable_value(
                                    &mut prefs.conflict,
                                    c,
                                    dialogs::conflict_label(c.policy()),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            },
        );
    });
    changed
}

/// Settings → General → Cloud.
fn cloud_section(
    ui: &mut egui::Ui,
    c: &mut CloudSettings,
    prefs: &mut Prefs,
    open_link: &mut Option<String>,
) -> bool {
    let p = pal(ui);
    let mut changed = false;
    ui.spacing_mut().item_spacing.y = space::SM;
    section_header(
        ui,
        t!("settings.section.cloud"),
        "16-einstellungen.md#cloud",
    );
    setting_card(ui, |ui| {
        if c.drive_accounts.is_empty() {
            setting_row_w(
                ui,
                t!("settings.cloud.gdrive"),
                t!("settings.cloud.gdrive.missing"),
                230.0,
                |ui| {
                    if link_button(
                        ui,
                        icons::ARROW_SQUARE_OUT,
                        t!("cloud.hint.download"),
                        p.accent_text,
                    )
                    .on_hover_text(crate::cloud::GOOGLE_DRIVE_DOWNLOAD)
                    .clicked()
                    {
                        *open_link = Some(crate::cloud::GOOGLE_DRIVE_DOWNLOAD.to_string());
                    }
                },
            );
            ui.add_space(space::SM);
            setting_row(
                ui,
                t!("settings.cloud.hint"),
                t!("settings.cloud.hint.note"),
                |ui| {
                    if widgets::toggle_switch(ui, !prefs.gdrive_hint_dismissed, true).clicked() {
                        prefs.gdrive_hint_dismissed = !prefs.gdrive_hint_dismissed;
                        changed = true;
                    }
                },
            );
        } else {
            let note = t!(
                "settings.cloud.gdrive.connected",
                accounts = c.drive_accounts.join(", ")
            );
            setting_row(ui, t!("settings.cloud.gdrive"), &note, |ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::hover());
                widgets::paint_icon(ui, r, icons::CHECK_CIRCLE, 16.0, p.success, false);
            });
        }
        if c.client_file.is_none() {
            return;
        }
        ui.add_space(space::SM);
        if disclosure(ui, t!("settings.cloud.expert"), c.expert_open) {
            c.expert_open = !c.expert_open;
        }
        if !c.expert_open {
            return;
        }
        if !c.loaded {
            c.load();
        }
        ui.add(
            egui::Label::new(
                RichText::new(t!("settings.cloud.expert.note"))
                    .font(theme::regular(text::SMALL))
                    .color(p.text_secondary),
            )
            .wrap(),
        );
        let w = ui.available_width();
        widgets::show_input(
            ui,
            widgets::input(&mut c.client_id).desired_width(w),
            t!("settings.cloud.client_id.hint"),
        );
        widgets::show_input(
            ui,
            widgets::input(&mut c.client_secret)
                .password(true)
                .desired_width(w),
            t!("settings.cloud.client_secret.hint"),
        );
        let saved = c
            .client_file
            .as_deref()
            .and_then(manta_vfs::gdrive::OAuthClient::from_file);
        ui.horizontal(|ui| {
            let status = if saved.is_some() {
                t!("settings.cloud.expert.active")
            } else {
                t!("settings.cloud.expert.inactive")
            };
            ui.label(
                RichText::new(status)
                    .font(theme::regular(text::SMALL))
                    .color(if saved.is_some() {
                        p.success
                    } else {
                        p.text_tertiary
                    }),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let dirty = saved.as_ref().map_or(!c.client_id.trim().is_empty(), |s| {
                    s.client_id != c.client_id.trim() || s.client_secret != c.client_secret.trim()
                });
                if widgets::button(
                    ui,
                    t!("settings.cloud.expert.save"),
                    widgets::ButtonKind::Primary,
                    dirty && !c.client_id.trim().is_empty(),
                )
                .clicked()
                {
                    c.save();
                }
                if saved.is_some()
                    && widgets::button(
                        ui,
                        t!("settings.cloud.expert.remove"),
                        widgets::ButtonKind::Secondary,
                        true,
                    )
                    .clicked()
                {
                    c.client_id.clear();
                    c.client_secret.clear();
                    c.save();
                }
            });
        });
        if let Some(e) = &c.error {
            error_banner(ui, p, e);
        }
    });
    changed
}

/// Settings → General → About Manta: the website (help, news, contact),
/// the source code, feedback (an issue on GitHub), the licence and the
/// third-party licences.
fn about_section(ui: &mut egui::Ui, open_link: &mut Option<String>) {
    let p = pal(ui);
    ui.spacing_mut().item_spacing.y = space::SM;
    section_header(
        ui,
        t!("settings.section.about"),
        "16-einstellungen.md#ueber",
    );
    setting_card(ui, |ui| {
        setting_row(
            ui,
            t!("settings.about.website"),
            t!("settings.about.website.note"),
            |ui| {
                if link_button(ui, icons::GLOBE, crate::help::WEBSITE_LABEL, p.accent_text)
                    .on_hover_text(crate::help::WEBSITE_URL)
                    .clicked()
                {
                    *open_link = Some(crate::help::WEBSITE_URL.to_string());
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.about.source"),
            t!("settings.about.source.note"),
            |ui| {
                if link_button(ui, icons::CODE, crate::help::SOURCE_LABEL, p.accent_text)
                    .on_hover_text(crate::help::SOURCE_URL)
                    .clicked()
                {
                    *open_link = Some(crate::help::SOURCE_URL.to_string());
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.about.feedback"),
            t!("settings.about.feedback.note"),
            |ui| {
                if link_button(
                    ui,
                    icons::BUG,
                    t!("settings.about.feedback.report"),
                    p.accent_text,
                )
                .on_hover_text(crate::feedback::CHOOSE_URL)
                .clicked()
                {
                    *open_link = Some(crate::feedback::current_issue_url(
                        crate::feedback::Kind::Bug,
                    ));
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("settings.about.license"),
            t!("settings.about.license.note"),
            |ui| {
                if link_button(
                    ui,
                    icons::FILE_TEXT,
                    t!("settings.about.license.read"),
                    p.accent_text,
                )
                .on_hover_text(crate::help::LICENSE_URL)
                .clicked()
                {
                    *open_link = Some(crate::help::LICENSE_URL.to_string());
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("cmd.third_party_licenses"),
            t!("settings.about.licenses.note"),
            |ui| {
                if link_button(
                    ui,
                    icons::SCROLL,
                    t!("settings.about.licenses.show"),
                    p.accent_text,
                )
                .clicked()
                {
                    crate::help::request(ui.ctx(), crate::help::THIRD_PARTY_LICENSES);
                }
            },
        );
    });
}

/// Settings → General → Updates.
fn updates_section(
    ui: &mut egui::Ui,
    info: &crate::updates::SettingsInfo,
    reveal: bool,
) -> Option<crate::updates::SettingsAction> {
    use crate::updates::SettingsAction;
    use manta_updater::Channel;
    let p = pal(ui);
    let mut action = None;
    section_header(
        ui,
        t!("update.settings.section"),
        "16-einstellungen.md#updates",
    );
    let card = setting_card(ui, |ui| {
        if let Some(d) = &info.disabled {
            ui.horizontal_top(|ui| {
                widgets::icon(ui, icons::INFO, 16.0, p.text_secondary);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = space::XXS;
                    ui.label(RichText::new(t!("update.settings.disabled")).color(p.text));
                    ui.add(
                        egui::Label::new(
                            RichText::new(d.reason())
                                .font(theme::regular(text::SMALL))
                                .color(p.text_secondary),
                        )
                        .wrap(),
                    );
                    ui.label(
                        RichText::new(t!("update.settings.version", version = info.current))
                            .font(theme::regular(text::SMALL))
                            .color(p.text_tertiary),
                    );
                });
            });
            return;
        }
        setting_row(
            ui,
            t!("update.settings.auto"),
            t!("update.settings.auto.note"),
            |ui| {
                if widgets::toggle_switch(ui, info.auto_check, true).clicked() {
                    action = Some(SettingsAction::AutoCheck(!info.auto_check));
                }
            },
        );
        ui.add_space(space::SM);
        setting_row(
            ui,
            t!("update.settings.channel"),
            t!("update.settings.channel.note"),
            |ui| {
                let mut c = info.channel;
                let options = [
                    (Channel::Stable, t!("update.channel.stable")),
                    (Channel::Beta, t!("update.channel.beta")),
                ];
                if widgets::segmented(ui, "update-channel", &mut c, &options) {
                    action = Some(SettingsAction::Channel(c));
                }
            },
        );
        ui.add_space(space::SM);
        let last = match info.last_check {
            Some(t) => crate::util::format_time_relative(Some(t), chrono::Local::now()),
            None => t!("update.settings.never").to_string(),
        };
        setting_row(
            ui,
            &t!("update.settings.version", version = info.current),
            &t!("update.settings.last_check", time = last),
            |ui| {
                if info.checking {
                    ui.add(egui::Spinner::new().size(14.0).color(p.text_secondary));
                }
                let b = widgets::button(
                    ui,
                    t!("update.settings.check_now"),
                    ButtonKind::Secondary,
                    !info.checking,
                );
                if b.clicked() {
                    action = Some(SettingsAction::CheckNow);
                }
            },
        );
        if let Some(v) = &info.skipped {
            ui.add_space(space::SM);
            setting_row(
                ui,
                &t!("update.settings.skipped", version = v),
                t!("update.settings.skipped.note"),
                |ui| {
                    if widgets::button(ui, t!("update.settings.reset"), ButtonKind::Secondary, true)
                        .clicked()
                    {
                        action = Some(SettingsAction::ResetSkipped);
                    }
                },
            );
        }
    });
    if reveal {
        card.scroll_to_me(Some(egui::Align::Max));
    }
    action
}

/// Settings → General → Default file manager: a switch that shows the real
/// state, what it does, and where the role goes back to.
fn default_viewer_row(
    ui: &mut egui::Ui,
    v: &crate::default_viewer::ViewerStatus,
    switch: &mut Option<bool>,
) {
    let p = pal(ui);
    let note = match &v.available {
        Err(u) => u.reason(),
        Ok(()) if v.is_default => format!(
            "{}{} {}",
            if v.folders {
                t!("viewer.settings.note_on")
            } else {
                t!("viewer.settings.note_on_reveal")
            },
            if v.folders {
                ""
            } else {
                t!("viewer.settings.folders_refused")
            },
            t!("viewer.settings.back_to", name = v.restore_to.name)
        ),
        Ok(()) => format!(
            "{} {}",
            t!("viewer.settings.note_off"),
            t!("viewer.currently", name = v.holder.name)
        ),
    };
    setting_row(ui, t!("viewer.settings.title"), &note, |ui| {
        let enabled = v.available.is_ok() && !v.busy;
        let resp = widgets::toggle_switch(ui, v.is_default, enabled);
        if resp.clicked() && enabled {
            *switch = Some(!v.is_default);
        }
        if v.busy {
            ui.add(egui::Spinner::new().size(14.0).color(p.text_secondary));
        }
    });
    ui.add_space(space::SM);
    let y = ui.cursor().top();
    ui.painter()
        .hline(ui.max_rect().x_range(), y, Stroke::new(1.0, p.separator));
    ui.add_space(space::SM);
    ui.add(
        egui::Label::new(
            RichText::new(t!("viewer.settings.limits"))
                .font(theme::regular(text::SMALL))
                .color(p.text_tertiary),
        )
        .wrap(),
    );
}

/// The macOS accent colour picker: a row of round swatches, the selected one
/// with a ring and a dot.
fn accent_swatches(ui: &mut egui::Ui, accent: &mut Accent) -> bool {
    let p = pal(ui);
    let mut changed = false;
    const D: f32 = 18.0;
    ui.spacing_mut().item_spacing.x = space::SM + 1.0;
    // Right-to-left layout: add in reverse so they read Blue … Graphite.
    for &a in Accent::ALL.iter().rev() {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(D + 4.0, D + 4.0), Sense::click());
        let c = rect.center();
        let fill = a.swatch(p.dark);
        let on = *accent == a;
        let painter = ui.painter();
        painter.circle_filled(c, D / 2.0, fill);
        painter.circle_stroke(
            c,
            D / 2.0,
            Stroke::new(
                1.0,
                egui::Color32::from_black_alpha(if p.dark { 60 } else { 35 }),
            ),
        );
        if on {
            painter.circle_stroke(c, D / 2.0 + 2.0, Stroke::new(1.5, fill));
            let dot = if a == Accent::Yellow {
                egui::Color32::from_rgb(29, 29, 31)
            } else {
                egui::Color32::WHITE
            };
            painter.circle_filled(c, 3.0, dot);
        } else if resp.hovered() {
            painter.circle_stroke(c, D / 2.0 + 2.0, Stroke::new(1.0, p.border));
        }
        let name = crate::i18n::tr(a.key());
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, on, name));
        let resp = resp.on_hover_text(name);
        if resp.clicked() && !on {
            *accent = a;
            changed = true;
        }
    }
    changed
}

fn setting_card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let p = pal(ui);
    egui::Frame::new()
        .fill(p.content)
        .stroke(Stroke::new(1.0, p.separator))
        .corner_radius(CornerRadius::same(radius::PANEL + 2))
        .inner_margin(egui::Margin::symmetric(
            space::LG as i8,
            space::MD as i8 + 2,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        })
        .response
}

fn setting_row(ui: &mut egui::Ui, title: &str, note: &str, control: impl FnOnce(&mut egui::Ui)) {
    setting_row_w(ui, title, note, 200.0, control);
}

/// [`setting_row`] whose control needs `control_w` points on the right.
fn setting_row_w(
    ui: &mut egui::Ui,
    title: &str,
    note: &str,
    control_w: f32,
    control: impl FnOnce(&mut egui::Ui),
) {
    let p = pal(ui);
    ui.horizontal(|ui| {
        let title_id = ui
            .vertical(|ui| {
                ui.set_width((ui.available_width() - control_w).max(160.0));
                ui.spacing_mut().item_spacing.y = space::XXS;
                let t = ui.label(RichText::new(title).color(p.text));
                ui.add(
                    egui::Label::new(
                        RichText::new(note)
                            .font(theme::regular(text::SMALL))
                            .color(p.text_secondary),
                    )
                    .wrap(),
                );
                t.id
            })
            .inner;
        // The row's title names its control for VoiceOver ("Show hidden
        // files, switch, off").
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label_inputs_by(title_id, control)
        });
    });
}

fn status_label(ui: &mut egui::Ui, p: &Palette, status: &PluginStatus) {
    let (color, label) = match status {
        PluginStatus::Running => (p.success, t!("plugins.status.running")),
        PluginStatus::Starting => (p.accent_text, t!("plugins.status.starting")),
        PluginStatus::Stopped => (p.text_tertiary, t!("plugins.status.off")),
        PluginStatus::Error(_) => (p.danger, t!("plugins.status.error")),
    };
    let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 14.0), Sense::hover());
    ui.painter().circle_filled(r.center(), 3.5, color);
    let text_color = if matches!(status, PluginStatus::Stopped) {
        p.text_secondary
    } else {
        color
    };
    ui.label(
        RichText::new(label)
            .font(theme::medium(text::SMALL))
            .color(text_color),
    );
}

fn error_banner(ui: &mut egui::Ui, p: &Palette, msg: &str) {
    egui::Frame::new()
        .fill(p.danger_soft)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8 + 1,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                widgets::icon(ui, icons::WARNING_CIRCLE, 15.0, p.danger);
                ui.add(
                    egui::Label::new(
                        RichText::new(msg)
                            .font(theme::regular(text::SMALL + 0.5))
                            .color(p.danger),
                    )
                    .wrap()
                    .selectable(true),
                );
            });
        });
}

/// An external plugin a built-in replaced (`PluginRow::replaced_by`).
fn replaced_banner(ui: &mut egui::Ui, p: &Palette, msg: &str) {
    egui::Frame::new()
        .fill(p.accent_soft)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8 + 1,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                widgets::icon(ui, icons::INFO, 15.0, p.accent_text);
                ui.add(
                    egui::Label::new(
                        RichText::new(msg)
                            .font(theme::regular(text::SMALL + 0.5))
                            .color(p.accent_text),
                    )
                    .wrap(),
                );
            });
        });
}

fn log_view(ui: &mut egui::Ui, p: &Palette, lines: &[String]) {
    egui::Frame::new()
        .fill(if p.dark {
            egui::Color32::from_rgb(24, 24, 26)
        } else {
            p.stripe
        })
        .stroke(Stroke::new(1.0, p.separator))
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 2,
            space::SM as i8 + 1,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if lines.is_empty() {
                ui.label(
                    RichText::new(t!("plugins.log.empty"))
                        .font(theme::mono(text::CAPTION + 0.5))
                        .color(p.text_secondary),
                );
                return;
            }
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for l in lines {
                        let color = if l.starts_with("[manta]") {
                            p.text_secondary
                        } else if l.starts_with("[error]") || l.to_lowercase().contains("traceback")
                        {
                            p.danger
                        } else {
                            p.text
                        };
                        ui.add(
                            egui::Label::new(
                                RichText::new(l)
                                    .font(theme::mono(text::CAPTION + 0.5))
                                    .color(color),
                            )
                            .wrap()
                            .selectable(true),
                        );
                    }
                });
        });
}

/// "▸ Label" disclosure; returns true when clicked.
fn disclosure(ui: &mut egui::Ui, label: &str, open: bool) -> bool {
    let p = pal(ui);
    let font = theme::medium(text::SMALL);
    let g = ui
        .painter()
        .layout_no_wrap(label.to_string(), font, p.accent_text);
    let (r, resp) = ui.allocate_exact_size(egui::vec2(g.size().x + 16.0, 20.0), Sense::click());
    crate::a11y::disclosure(&resp, label, open);
    if resp.hovered() {
        ui.painter().rect_filled(
            r.expand2(egui::vec2(3.0, 0.0)),
            CornerRadius::same(radius::CONTROL),
            p.hover,
        );
    }
    let ir = egui::Rect::from_min_size(r.min, egui::vec2(14.0, r.height()));
    widgets::paint_icon(
        ui,
        ir,
        if open {
            icons::CARET_DOWN
        } else {
            icons::CARET_RIGHT
        },
        10.0,
        p.accent_text,
        false,
    );
    ui.painter().galley(
        egui::pos2(ir.right() + 2.0, r.center().y - g.size().y / 2.0),
        g,
        p.accent_text,
    );
    resp.clicked()
}

/// Small borderless "icon + label" button.
fn link_button(
    ui: &mut egui::Ui,
    glyph: &str,
    label: &str,
    color: egui::Color32,
) -> egui::Response {
    let p = pal(ui);
    let font = theme::regular(text::SMALL);
    let g = ui.painter().layout_no_wrap(label.to_string(), font, color);
    let (r, resp) = ui.allocate_exact_size(egui::vec2(g.size().x + 26.0, 22.0), Sense::click());
    if resp.hovered() {
        ui.painter()
            .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
    }
    let ir = egui::Rect::from_min_size(r.min + egui::vec2(4.0, 0.0), egui::vec2(16.0, r.height()));
    widgets::paint_icon(ui, ir, glyph, 12.0, color, false);
    ui.painter().galley(
        egui::pos2(ir.right() + 2.0, r.center().y - g.size().y / 2.0),
        g,
        color,
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Link, true, label));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Width of the key column in the plugin sheets.
const KEY_W: f32 = 92.0;

fn kv_line(ui: &mut egui::Ui, p: &Palette, key: &str, value: &str, mono: bool) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(KEY_W, 16.0), Sense::hover());
        ui.painter().text(
            r.left_center(),
            egui::Align2::LEFT_CENTER,
            key,
            theme::regular(text::SMALL),
            p.text_secondary,
        );
        let font = if mono {
            theme::mono(text::SMALL)
        } else {
            theme::regular(text::SMALL)
        };
        ui.add(
            egui::Label::new(RichText::new(value).font(font).color(p.text))
                .wrap()
                .selectable(true),
        );
    });
}
