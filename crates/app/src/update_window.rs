//! The "Software Update" window: a native window (like macOS's own update
//! windows) that shows the update on offer with its release notes, then the
//! download, the install and the relaunch, all in place. Results of a
//! manual check ("up to date", errors) show here too.

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke};
use manta_plugin_markdown::doc::Doc;
use manta_plugin_markdown::render::{Action as MdAction, Env};
use manta_updater::{LocationProblem, Release};

use crate::aux_window::{AuxKind, Spec};
use crate::icons;
use crate::theme::{self, pal, radius, space, text, Palette};
use crate::updates::{FailKind, Failure, Op, Updates, View};
use crate::widgets::{self, ButtonKind};

/// Inner size when first opened.
pub const SIZE: [f32; 2] = [580.0, 470.0];

const ICON_URI: &str = "bytes://manta-app-icon.png";
const ICON_PNG: &[u8] = include_bytes!("../assets/app-icon.png");

/// Draw the window while it is open (call every frame).
pub fn window(ctx: &egui::Context, u: &mut Updates) {
    if !u.open {
        u.window.hidden(ctx, AuxKind::Update);
        return;
    }
    let spec = Spec {
        kind: AuxKind::Update,
        title: t!("update.window_title"),
        size: egui::vec2(SIZE[0], SIZE[1]),
        min_size: egui::vec2(460.0, 340.0),
        margin: egui::Margin::ZERO,
        activate: true,
        zoomable: false,
    };
    let mut win = std::mem::take(&mut u.window);
    let (_, closed) = win.show(ctx, &spec, |ui| contents(ui, u));
    u.window = win;
    if closed.0 {
        u.close();
        if !u.open {
            u.window.hidden(ctx, AuxKind::Update);
        }
    }
}

/// The window content (also rendered on its own by the tests).
pub fn contents(ui: &mut egui::Ui, u: &mut Updates) {
    let full = matches!(
        u.view,
        View::Available
            | View::Downloading { .. }
            | View::Ready
            | View::Installing
            | View::Installed
    ) || matches!(&u.view, View::Failed(f) if f.during != Op::Check && u.release.is_some());
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(22, 20))
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            if full && u.release.is_some() {
                release_page(ui, u);
            } else {
                compact_page(ui, u);
            }
        });
}

fn app_icon(ui: &mut egui::Ui, size: f32) {
    ui.add(
        egui::Image::from_bytes(ICON_URI, ICON_PNG)
            .fit_to_exact_size(egui::vec2(size, size))
            .sense(Sense::hover()),
    );
}

/// "21 Oct 2026" / "21. Okt. 2026".
fn format_date(d: &chrono::DateTime<chrono::FixedOffset>) -> String {
    use chrono::Datelike;
    let local = d.with_timezone(&chrono::Local);
    crate::i18n::format_day(local.day(), local.month0(), Some(local.year()), None)
}

/// Download sizes with one decimal from 1 MB on ("21.7 MB"), so the
/// progress visibly moves.
fn mb(bytes: u64) -> String {
    if bytes < 1_000_000 {
        return crate::util::format_size(bytes);
    }
    format!(
        "{}{}{}",
        crate::i18n::format_decimal(bytes as f64 / 1e6, 1),
        manta_plugin::i18n::unit_space(crate::i18n::locale()),
        manta_plugin::i18n::size_unit(1)
    )
}

/// "12.3 MB of 21.7 MB".
fn progress_text(done: u64, total: u64) -> String {
    t!("update.progress", done = mb(done), total = mb(total))
}

/// Header, release notes and the footer for the current step.
fn release_page(ui: &mut egui::Ui, u: &mut Updates) {
    let p = pal(ui);
    let r = u.release.clone().expect("release_page needs a release");
    // Header: icon, title, size and date.
    ui.horizontal_top(|ui| {
        app_icon(ui, 64.0);
        ui.add_space(space::LG);
        ui.vertical(|ui| {
            ui.add_space(space::SM);
            ui.spacing_mut().item_spacing.y = space::XS;
            let title = match u.view {
                View::Downloading { .. } => t!("update.title.downloading", version = r.version),
                View::Ready => t!("update.title.ready", version = r.version),
                View::Installing | View::Installed => {
                    t!("update.title.installing", version = r.version)
                }
                _ => t!(
                    "update.title.available",
                    version = r.version,
                    current = u.current
                ),
            };
            ui.add(
                egui::Label::new(
                    RichText::new(title)
                        .font(theme::semibold(text::HEADLINE))
                        .color(p.text),
                )
                .wrap(),
            );
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::SM;
                let meta = t!(
                    "update.meta",
                    size = mb(r.size),
                    date = format_date(&r.published)
                );
                ui.label(
                    RichText::new(meta)
                        .font(theme::regular(text::SMALL + 0.5))
                        .color(p.text_secondary),
                );
                if r.version.pre.is_empty() {
                    return;
                }
                let (fg, bg) = (p.purple, p.purple_soft);
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(CornerRadius::same(radius::PILL))
                    .inner_margin(egui::Margin::symmetric(7, 1))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(t!("update.beta"))
                                .font(theme::medium(text::CAPTION))
                                .color(fg),
                        );
                    });
            });
        });
    });
    ui.add_space(space::LG + 2.0);
    ui.label(
        RichText::new(t!("update.whats_new"))
            .font(theme::semibold(text::SMALL))
            .color(p.text_secondary),
    );
    ui.add_space(space::XS);

    // Footer first (bottom up), so the notes take the rest.
    let footer_h = footer_height(u);
    let notes_h = (ui.available_height() - footer_h - space::LG).max(80.0);
    notes_box(ui, u, &r, notes_h);
    ui.add_space(space::LG);
    footer(ui, u, &r);
}

fn footer_height(u: &Updates) -> f32 {
    let banner = if u.blocker.is_some() && matches!(u.view, View::Available | View::Ready) {
        78.0
    } else {
        0.0
    };
    match &u.view {
        View::Failed(_) => 112.0,
        View::Downloading { .. } => 52.0,
        View::Ready if u.waiting_for_transfers => 60.0 + banner,
        _ => 30.0 + banner,
    }
}

/// The release notes (Markdown, in the UI language) in a scrolling box.
fn notes_box(ui: &mut egui::Ui, u: &mut Updates, r: &Release, height: f32) {
    let p = pal(ui);
    let lang = crate::i18n::locale().code();
    let stale = !matches!(&u.notes, Some((v, l, _)) if *v == r.version && *l == lang);
    if stale {
        u.notes = Some((
            r.version.clone(),
            lang,
            Doc::parse(r.notes.for_language(lang), false),
        ));
        u.heights = Default::default();
    }
    let Some((_, _, doc)) = &u.notes else { return };
    let images = Default::default();
    let mut opened = Vec::new();
    egui::Frame::new()
        .fill(p.content)
        .stroke(Stroke::new(1.0, p.separator))
        .corner_radius(CornerRadius::same(radius::PANEL + 2))
        .inner_margin(egui::Margin::ZERO)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(height);
            egui::ScrollArea::vertical()
                .id_salt("update-notes")
                .auto_shrink([false, false])
                .show_viewport(ui, |ui, vp| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(
                            space::XL as i8 + 2,
                            space::LG as i8 + 2,
                        ))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let mut env =
                                Env::new(ui, doc, manta_core::VfsPath::root(), &images, false);
                            env.show(ui, vp, &mut u.heights);
                            opened = env.actions;
                        });
                });
        });
    for a in opened {
        // Only web links; relative links and images have nothing to point at.
        if let MdAction::OpenUrl(url) = a {
            if url.starts_with("https://") || url.starts_with("http://") {
                if let Err(e) = open::that_detached(&url) {
                    log::warn!("updates: can't open {url}: {e}");
                }
            }
        }
    }
}

/// Buttons right-aligned, the first one rightmost (the default, macOS order).
fn button_row(
    ui: &mut egui::Ui,
    left: Option<(&str, &mut bool)>,
    right: &mut [(&str, ButtonKind, bool, &mut bool)],
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::MD;
        if let Some((label, clicked)) = left {
            *clicked = widgets::button(ui, label, ButtonKind::Secondary, true).clicked();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (label, kind, enabled, clicked) in right.iter_mut() {
                **clicked = widgets::button(ui, label, *kind, *enabled).clicked();
            }
        });
    });
}

fn footer(ui: &mut egui::Ui, u: &mut Updates, r: &Release) {
    let p = pal(ui);
    let blocked = u.blocker.clone();
    match u.view.clone() {
        View::Available | View::Ready => {
            if let Some(b) = &blocked {
                blocker_banner(ui, p, b);
                ui.add_space(space::MD);
            }
            if u.view == View::Ready && u.waiting_for_transfers {
                note_line(
                    ui,
                    p,
                    icons::CLOCK,
                    p.text_secondary,
                    t!("update.waiting_for_transfers"),
                );
                ui.add_space(space::SM);
            }
            let (mut skip, mut later, mut install) = (false, false, false);
            let skip_label = t!("update.skip");
            let mut right = [
                (
                    t!("update.install"),
                    ButtonKind::Primary,
                    blocked.is_none() && !u.waiting_for_transfers,
                    &mut install,
                ),
                (t!("update.later"), ButtonKind::Secondary, true, &mut later),
            ];
            button_row(ui, Some((skip_label, &mut skip)), &mut right);
            if install {
                u.install();
            } else if later {
                u.later();
            } else if skip {
                u.skip();
            }
        }
        View::Downloading { done, total } => {
            let frac = if total > 0 {
                done as f32 / total as f32
            } else {
                0.0
            };
            progress_bar(ui, p, frac.clamp(0.0, 1.0));
            ui.add_space(space::SM);
            let mut cancel = false;
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(progress_text(done, total.max(r.size)))
                        .font(theme::regular(text::SMALL + 0.5))
                        .color(p.text_secondary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    cancel = widgets::button(ui, t!("common.cancel"), ButtonKind::Secondary, true)
                        .clicked();
                });
            });
            if cancel {
                u.cancel_download();
            }
        }
        View::Installing | View::Installed => {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(14.0).color(p.text_secondary));
                ui.add_space(space::XS);
                let msg = if u.view == View::Installed {
                    t!("update.relaunching")
                } else {
                    t!("update.installing")
                };
                ui.label(RichText::new(msg).color(p.text_secondary));
            });
        }
        View::Failed(f) => {
            failure_banner(ui, p, &f);
            ui.add_space(space::MD);
            let (mut close, mut retry) = (false, false);
            let mut right = [
                (t!("update.retry"), ButtonKind::Primary, true, &mut retry),
                (t!("update.close"), ButtonKind::Secondary, true, &mut close),
            ];
            button_row(ui, None, &mut right);
            if retry {
                u.retry();
            } else if close {
                u.later();
            }
        }
        View::Checking | View::UpToDate => {}
    }
}

fn progress_bar(ui: &mut egui::Ui, p: &Palette, frac: f32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), Sense::hover());
    ui.painter()
        .rect_filled(r, CornerRadius::same(3), p.separator);
    let mut fr = r;
    fr.set_width((r.width() * frac).max(6.0));
    ui.painter()
        .rect_filled(fr, CornerRadius::same(3), p.accent);
}

fn note_line(ui: &mut egui::Ui, _p: &Palette, glyph: &str, color: Color32, msg: &str) {
    ui.horizontal(|ui| {
        widgets::icon(ui, glyph, 14.0, color);
        ui.add(
            egui::Label::new(
                RichText::new(msg)
                    .font(theme::regular(text::SMALL + 0.5))
                    .color(color),
            )
            .wrap(),
        );
    });
}

/// What to do about a location that keeps Manta from replacing itself.
pub fn blocker_text(b: &LocationProblem) -> (&'static str, &'static str) {
    match b {
        LocationProblem::ReadOnlyVolume => (t!("update.blocker.dmg"), t!("update.blocker.dmg.fix")),
        LocationProblem::Translocated => (
            t!("update.blocker.translocated"),
            t!("update.blocker.translocated.fix"),
        ),
        LocationProblem::NotBundled => (t!("update.blocker.dev"), t!("update.blocker.dev.fix")),
    }
}

fn banner(
    ui: &mut egui::Ui,
    fill: Color32,
    fg: Color32,
    glyph: &str,
    title: &str,
    body: &str,
    detail: Option<&str>,
) {
    let p = pal(ui);
    egui::Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(radius::PANEL))
        .inner_margin(egui::Margin::symmetric(
            space::MD as i8 + 4,
            space::MD as i8 + 1,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                widgets::icon(ui, glyph, 16.0, fg);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = space::XXS;
                    ui.add(
                        egui::Label::new(
                            RichText::new(title)
                                .font(theme::medium(text::BODY))
                                .color(p.text),
                        )
                        .wrap(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(body)
                                .font(theme::regular(text::SMALL + 0.5))
                                .color(p.text_secondary),
                        )
                        .wrap(),
                    );
                    if let Some(d) = detail {
                        ui.add(
                            egui::Label::new(
                                RichText::new(d)
                                    .font(theme::regular(text::CAPTION))
                                    .color(p.text_tertiary),
                            )
                            .wrap()
                            .truncate(),
                        );
                    }
                });
            });
        });
}

fn blocker_banner(ui: &mut egui::Ui, p: &Palette, b: &LocationProblem) {
    let (title, fix) = blocker_text(b);
    banner(
        ui,
        p.warning_soft,
        p.warning,
        icons::WARNING,
        title,
        fix,
        None,
    );
}

/// Friendly text for a failure: (title, what to do).
pub fn failure_text(f: &Failure) -> (String, String) {
    let title = match f.during {
        Op::Check | Op::Restore => t!("update.failed.check"),
        Op::Download => t!("update.failed.download"),
        Op::Install => t!("update.failed.install"),
    };
    let body = match &f.kind {
        FailKind::Offline => t!("update.error.offline").to_string(),
        FailKind::Server(code) => t!("update.error.server", code = code),
        FailKind::Feed => t!("update.error.feed").to_string(),
        FailKind::Signature => t!("update.error.signature").to_string(),
        FailKind::Damaged => t!("update.error.damaged").to_string(),
        FailKind::Rejected => t!("update.error.rejected").to_string(),
        FailKind::Location(Some(b)) => blocker_text(b).1.to_string(),
        FailKind::Location(None) => t!("update.error.location").to_string(),
        FailKind::Install => t!("update.error.install").to_string(),
        FailKind::Other => t!("update.error.unknown").to_string(),
    };
    (title.to_string(), body)
}

fn failure_banner(ui: &mut egui::Ui, p: &Palette, f: &Failure) {
    let (title, body) = failure_text(f);
    banner(
        ui,
        p.danger_soft,
        p.danger,
        icons::WARNING_CIRCLE,
        &title,
        &body,
        Some(&f.detail),
    );
}

/// Checking, up to date, or a failed check: centred, like a macOS alert.
fn compact_page(ui: &mut egui::Ui, u: &mut Updates) {
    let p = pal(ui);
    let avail = ui.available_height();
    let block_h = 250.0;
    ui.add_space(((avail - block_h) / 2.0).max(0.0));
    ui.vertical_centered(|ui| {
        app_icon(ui, 80.0);
        ui.add_space(space::LG);
        let (title, body): (String, String) = match &u.view {
            View::UpToDate => (
                t!("update.up_to_date").into(),
                t!("update.up_to_date.body", version = u.current),
            ),
            View::Failed(f) => failure_text(f),
            _ => (
                t!("update.checking").into(),
                t!("update.current", version = u.current),
            ),
        };
        ui.label(
            RichText::new(title)
                .font(theme::semibold(text::HEADLINE))
                .color(p.text),
        );
        ui.add_space(space::XS);
        ui.set_max_width(400.0);
        ui.add(egui::Label::new(RichText::new(body).color(p.text_secondary)).wrap());
        if let View::Failed(f) = &u.view {
            ui.add_space(space::XS);
            ui.add(
                egui::Label::new(
                    RichText::new(&f.detail)
                        .font(theme::regular(text::CAPTION))
                        .color(p.text_tertiary),
                )
                .wrap(),
            );
        }
        ui.add_space(space::XL);
        match u.view.clone() {
            View::Checking => {
                ui.add(egui::Spinner::new().size(18.0).color(p.text_secondary));
            }
            View::Failed(_) => {
                ui.horizontal(|ui| {
                    // Centre the two buttons, measured as they are drawn.
                    let w = widgets::button_width(ui, t!("update.close"), ButtonKind::Secondary)
                        + widgets::button_width(ui, t!("update.retry"), ButtonKind::Primary)
                        + space::MD;
                    ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                    ui.spacing_mut().item_spacing.x = space::MD;
                    if widgets::button(ui, t!("update.close"), ButtonKind::Secondary, true)
                        .clicked()
                    {
                        u.later();
                    }
                    if widgets::button(ui, t!("update.retry"), ButtonKind::Primary, true).clicked()
                    {
                        u.retry();
                    }
                });
            }
            _ => {
                if widgets::button(ui, t!("common.ok"), ButtonKind::Primary, true).clicked() {
                    u.later();
                }
            }
        }
    });
}

fn status_pill_label(n: &crate::updates::Notice) -> String {
    if n.ready {
        t!("update.pill.ready", version = n.version)
    } else {
        t!("update.pill", version = n.version)
    }
}

/// The width [`status_pill`] takes (0 without an update on offer).
pub fn status_pill_width(ui: &egui::Ui, u: &Updates) -> f32 {
    let Some(n) = &u.notice else { return 0.0 };
    let font = theme::medium(text::CAPTION + 0.5);
    ui.painter()
        .layout_no_wrap(status_pill_label(n), font, egui::Color32::WHITE)
        .size()
        .x
        + 30.0
}

/// The status bar hint after the launch check found an update. True when clicked.
pub fn status_pill(ui: &mut egui::Ui, u: &Updates) -> bool {
    let Some(n) = &u.notice else { return false };
    let p = pal(ui);
    let label = status_pill_label(n);
    let font = theme::medium(text::CAPTION + 0.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.clone(), font, p.accent_text);
    let size = egui::vec2(galley.size().x + 30.0, 18.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() {
        p.accent_soft.gamma_multiply(1.4)
    } else {
        p.accent_soft
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius::PILL), fill);
    let ir = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 11.0, rect.center().y),
        egui::vec2(12.0, 12.0),
    );
    widgets::paint_icon(ui, ir, icons::ARROW_UP, 11.0, p.accent_text, false);
    ui.painter().galley(
        egui::pos2(rect.left() + 20.0, rect.center().y - galley.size().y / 2.0),
        galley,
        p.accent_text,
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
    widgets::tooltip(
        resp.on_hover_cursor(egui::CursorIcon::PointingHand),
        t!("update.pill.tip"),
        "",
    )
    .clicked()
}
