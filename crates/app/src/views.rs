//! The pane's views besides the list (Finder's ⌘1 – ⌘4): Icons (a grid of
//! large icons and thumbnails), Columns (Miller columns with a preview) and
//! Gallery (a large preview over a filmstrip).
//!
//! Every view works on the same [`Tab`] as the list: `tab.view` is the
//! order, `tab.cursor` an index into it, `tab.selected` the selection, so
//! clicks, ⌘/⇧-clicks, ⌘A, type-to-select, Quick Look, rename, the context
//! menu, drag & drop and plugin decorations behave the same in every view.
//! What differs is the layout, the arrow keys ([`nav`]) and how the cursor
//! is scrolled into view. Thumbnails come from [`crate::thumbs`].

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use eframe::egui::{self, Color32, CornerRadius, Sense, Stroke, StrokeKind};
use manta_core::{Entry, SharedVfs, VfsPath};
use manta_plugin::host::MergedDecoration;
use serde::{Deserialize, Serialize};

use crate::app::{Action, Side};
use crate::commands::Cmd;
use crate::icons;
use crate::pane::{self, DragPayload, MenuCtx, PaneInfo, Tab};
use crate::plugin_ui;
use crate::theme::{self, pal, radius, size, space, text, Palette};
use crate::thumbs::{self, Method, Source, Thumb, ThumbKey};
use crate::util::{self, SortColumn, SortSpec};
use crate::widgets;

// ------------------------------------------------------------------ modes

/// How a pane shows its folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Icons,
    #[default]
    List,
    Columns,
    Gallery,
}

impl ViewMode {
    /// In shortcut order (⌘1 – ⌘4).
    pub const ALL: [ViewMode; 4] = [
        ViewMode::Icons,
        ViewMode::List,
        ViewMode::Columns,
        ViewMode::Gallery,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Icons => t!("view.icons"),
            ViewMode::List => t!("view.list"),
            ViewMode::Columns => t!("view.columns"),
            ViewMode::Gallery => t!("view.gallery"),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            ViewMode::Icons => icons::SQUARES_FOUR,
            ViewMode::List => icons::LIST,
            ViewMode::Columns => icons::COLUMNS,
            ViewMode::Gallery => icons::SLIDESHOW,
        }
    }

    pub fn cmd(self) -> Cmd {
        match self {
            ViewMode::Icons => Cmd::ViewIcons,
            ViewMode::List => Cmd::ViewList,
            ViewMode::Columns => Cmd::ViewColumns,
            ViewMode::Gallery => Cmd::ViewGallery,
        }
    }
}

/// Icon sizes of the Icons view (points); the slider snaps to `ICON_STEP`
/// (each glyph size is rasterised once, so free sizes would fill the font
/// atlas while dragging).
pub const ICON_MIN: f32 = 48.0;
pub const ICON_MAX: f32 = 256.0;
pub const ICON_STEP: f32 = 8.0;
pub const ICON_DEFAULT: f32 = 64.0;

pub fn clamp_icon_size(s: f32) -> f32 {
    if !s.is_finite() {
        return ICON_DEFAULT;
    }
    ((s / ICON_STEP).round() * ICON_STEP).clamp(ICON_MIN, ICON_MAX)
}

/// Per-tab state of the Icons, Columns and Gallery views.
#[derive(Default)]
pub struct ViewState {
    /// Icons: items per row in the last frame (↑ / ↓ move by this much).
    pub grid_cols: usize,
    /// Rows (Icons) or items (Gallery filmstrip) visible in the last frame.
    pub page: usize,
    /// Icons: a rubber band being dragged.
    pub band: Option<Band>,
    /// Icons: the grid and where its content started on screen in the last
    /// frame (hit testing in tests, drags).
    pub grid: Option<(Grid, egui::Pos2)>,
    /// Gallery: horizontal scroll of the filmstrip and its width.
    pub strip_offset: f32,
    pub strip_view_w: f32,
    /// Columns: the folder listings and the column root.
    pub cols: ColumnsState,
}

/// A rubber-band selection in the Icons view, in content coordinates.
pub struct Band {
    pub origin: egui::Pos2,
    pub current: egui::Pos2,
    /// The selection before the band (kept with ⌘ / ⇧).
    pub base: BTreeSet<String>,
}

// ------------------------------------------------------------------ keys

/// A navigation key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// What a navigation key asks the app to do besides moving the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavOut {
    /// Handled (or nothing to do).
    Done,
    /// Columns ←: the enclosing folder becomes the active column.
    Parent,
    /// Columns →: open the folder under the cursor as the next column.
    Enter,
    /// The key isn't used by this view (left for others).
    Unused,
}

/// Whether `mode` uses `key` (the list leaves ← → to others).
pub fn nav_uses(mode: ViewMode, key: Nav) -> bool {
    !(mode == ViewMode::List && matches!(key, Nav::Left | Nav::Right))
}

/// Move the cursor for `key` in `mode` (`extend`: ⇧ extends the selection).
pub fn nav(mode: ViewMode, tab: &mut Tab, key: Nav, extend: bool) -> NavOut {
    let n = tab.view.len() as isize;
    let cols = tab.view_state.grid_cols.max(1) as isize;
    let page = tab.view_state.page.max(1) as isize;
    let delta = match (mode, key) {
        (ViewMode::List, Nav::Left | Nav::Right) => return NavOut::Unused,
        (ViewMode::Columns, Nav::Left) => return NavOut::Parent,
        (ViewMode::Columns, Nav::Right) => {
            return if tab.cursor_entry().is_some_and(|e| e.is_dir()) {
                NavOut::Enter
            } else {
                NavOut::Done
            };
        }
        (_, Nav::Home) => -n,
        (_, Nav::End) => n,
        (ViewMode::Icons, Nav::Up) => -cols,
        (ViewMode::Icons, Nav::Down) => cols,
        (ViewMode::Icons, Nav::PageUp) => -cols * page,
        (ViewMode::Icons, Nav::PageDown) => cols * page,
        (ViewMode::Gallery, Nav::PageUp) => -page,
        (ViewMode::Gallery, Nav::PageDown) => page,
        (_, Nav::PageUp) => -20,
        (_, Nav::PageDown) => 20,
        (_, Nav::Up | Nav::Left) => -1,
        (_, Nav::Down | Nav::Right) => 1,
    };
    // ↓ on the last (partial) row of the grid stays put, like Finder.
    if mode == ViewMode::Icons && key == Nav::Down {
        if let Some(c) = tab.cursor {
            if c as isize + cols >= n && (c as isize / cols) == (n - 1) / cols {
                return NavOut::Done;
            }
        }
    }
    tab.move_cursor(delta, extend);
    NavOut::Done
}

// ------------------------------------------------------------------ grid

/// Geometry of the Icons grid for a given width and icon size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    pub cols: usize,
    /// Horizontal step between cells (cells are centred in it).
    pub step_x: f32,
    pub cell_w: f32,
    pub row_h: f32,
    pub icon: f32,
    /// Left margin.
    pub margin: f32,
}

/// Space above the icon, between icon and name, below the name.
const CELL_PAD: f32 = 6.0;
const NAME_GAP: f32 = 5.0;
const NAME_LINE: f32 = 15.0;
const GRID_MARGIN: f32 = 10.0;

impl Grid {
    pub fn new(width: f32, icon: f32) -> Grid {
        let icon = clamp_icon_size(icon);
        let cell_w = (icon + 36.0).max(96.0);
        let row_h = CELL_PAD + icon + NAME_GAP + 2.0 * NAME_LINE + CELL_PAD + 2.0;
        let avail = (width - 2.0 * GRID_MARGIN).max(cell_w);
        let cols = ((avail / cell_w).floor() as usize).max(1);
        Grid {
            cols,
            step_x: avail / cols as f32,
            cell_w,
            row_h,
            icon,
            margin: GRID_MARGIN,
        }
    }

    pub fn rows(&self, n: usize) -> usize {
        n.div_ceil(self.cols)
    }

    /// The cell of item `i` relative to the content's top-left corner.
    pub fn cell(&self, i: usize) -> egui::Rect {
        let (r, c) = (i / self.cols, i % self.cols);
        let cx = self.margin + self.step_x * (c as f32 + 0.5);
        egui::Rect::from_center_size(
            egui::pos2(cx, r as f32 * self.row_h + self.row_h / 2.0),
            egui::vec2(self.cell_w, self.row_h),
        )
    }

    /// Where the icon of item `i` is drawn (content coordinates).
    pub fn icon_rect(&self, i: usize) -> egui::Rect {
        let cell = self.cell(i);
        egui::Rect::from_min_size(
            egui::pos2(cell.center().x - self.icon / 2.0, cell.top() + CELL_PAD),
            egui::vec2(self.icon, self.icon),
        )
    }

    /// The part of the cell that belongs to the item (icon and name).
    pub fn hit_rect(&self, i: usize) -> egui::Rect {
        let ir = self.icon_rect(i).expand(4.0);
        let cell = self.cell(i);
        let name = egui::Rect::from_min_max(
            egui::pos2(cell.left() + 4.0, ir.bottom()),
            egui::pos2(cell.right() - 4.0, cell.bottom() - CELL_PAD),
        );
        ir.union(name)
    }

    /// Items whose icon or name the band `r` (content coordinates) touches.
    pub fn hits(&self, r: egui::Rect, n: usize) -> Vec<usize> {
        if n == 0 {
            return Vec::new();
        }
        let first_row = ((r.top() / self.row_h).floor().max(0.0)) as usize;
        let last_row = ((r.bottom() / self.row_h).floor().max(0.0)) as usize;
        let mut out = Vec::new();
        for row in first_row..=last_row.min(self.rows(n).saturating_sub(1)) {
            for c in 0..self.cols {
                let i = row * self.cols + c;
                if i < n && self.hit_rect(i).intersects(r) {
                    out.push(i);
                }
            }
        }
        out
    }

    /// The vertical scroll offset that shows item `i` (or `current` if it
    /// is visible already) in a viewport `view_h` high.
    pub fn reveal(&self, i: usize, current: f32, view_h: f32) -> f32 {
        reveal_span(
            (i / self.cols) as f32 * self.row_h,
            self.row_h,
            current,
            view_h,
        )
    }
}

/// The scroll offset that shows `[top, top + h]` in a viewport `view`
/// long, scrolled to `current`.
pub fn reveal_span(top: f32, h: f32, current: f32, view: f32) -> f32 {
    if top < current {
        top
    } else if top + h > current + view {
        (top + h - view).max(0.0)
    } else {
        current
    }
}

// ------------------------------------------------------------------ shared drawing

/// The view switcher in the path bar (four segments, or one button with a
/// menu when the pane is narrow) and the view options button (sort order,
/// icon size). `right` is its right edge; returns its left edge.
#[allow(clippy::too_many_arguments)]
pub fn view_switcher(
    ui: &mut egui::Ui,
    right: f32,
    cy: f32,
    compact: bool,
    side: Side,
    tab: &mut Tab,
    view: ViewMode,
    icon_size: &mut f32,
    actions: &mut Vec<Action>,
) -> f32 {
    let p = pal(ui);
    let h = size::FIELD;
    // Options (sort, icon size): every view but the list (its header sorts).
    let mut x = right;
    if view != ViewMode::List && !compact {
        let r =
            egui::Rect::from_min_max(egui::pos2(x - h, cy - h / 2.0), egui::pos2(x, cy + h / 2.0));
        let resp = ui.interact(r, egui::Id::unique(("view-options", side)), Sense::click());
        let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp));
        if resp.hovered() || open {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        widgets::paint_icon(
            ui,
            r,
            icons::SLIDERS_HORIZONTAL,
            14.0,
            if resp.hovered() {
                p.text
            } else {
                p.text_secondary
            },
            false,
        );
        let resp = widgets::tooltip(resp, t!("view.options"), "");
        egui::Popup::menu(&resp)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| options_menu(ui, tab, view, icon_size));
        x = r.left() - 2.0;
    }
    if compact {
        let r = egui::Rect::from_min_max(
            egui::pos2(x - h - 8.0, cy - h / 2.0),
            egui::pos2(x, cy + h / 2.0),
        );
        let resp = ui.interact(
            r,
            egui::Id::unique(("view-switch-compact", side)),
            Sense::click(),
        );
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, t!("view.switch_tip"))
        });
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        let ir = egui::Rect::from_min_size(r.min, egui::vec2(h, h));
        widgets::paint_icon(
            ui,
            ir,
            view.icon(),
            14.0,
            if resp.hovered() {
                p.text
            } else {
                p.text_secondary
            },
            false,
        );
        let cr = egui::Rect::from_min_max(egui::pos2(ir.right() - 4.0, r.top()), r.max);
        widgets::paint_icon(ui, cr, icons::CARET_DOWN, 8.0, p.text_tertiary, false);
        let tip = format!("{} – {}", t!("view.switch_tip"), view.label());
        let resp = widgets::tooltip(resp, &tip, view.cmd().shortcut());
        egui::Popup::menu(&resp)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_width(200.0);
                for m in ViewMode::ALL {
                    let label = format!("{}   {}", m.icon(), m.label());
                    let btn = egui::Button::selectable(m == view, label)
                        .shortcut_text(m.cmd().shortcut());
                    if ui.add(btn).clicked() {
                        actions.push(Action::SetView(side, m));
                        ui.close();
                    }
                }
                // No room for the options button: its contents are here.
                if view != ViewMode::List {
                    ui.separator();
                    options_menu(ui, tab, view, icon_size);
                }
            });
        return r.left();
    }
    const SEG: f32 = 22.0;
    let track = egui::Rect::from_min_max(
        egui::pos2(x - 4.0 * SEG - 4.0, cy - h / 2.0),
        egui::pos2(x, cy + h / 2.0),
    );
    let track_fill = if p.dark {
        Color32::from_rgb(54, 54, 58)
    } else {
        Color32::from_rgb(228, 228, 232)
    };
    ui.painter()
        .rect_filled(track, CornerRadius::same(radius::CONTROL + 1), track_fill);
    for (i, m) in ViewMode::ALL.into_iter().enumerate() {
        let r = egui::Rect::from_min_size(
            egui::pos2(track.left() + 2.0 + SEG * i as f32, track.top() + 2.0),
            egui::vec2(SEG, h - 4.0),
        );
        let resp = ui.interact(
            r,
            egui::Id::unique(("view-switch", side, i)),
            Sense::click(),
        );
        let on = m == view;
        resp.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::RadioButton, true, on, m.label())
        });
        if on {
            let thumb = if p.dark {
                Color32::from_rgb(98, 98, 104)
            } else {
                Color32::WHITE
            };
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), thumb);
            ui.painter().rect_stroke(
                r,
                CornerRadius::same(radius::CONTROL),
                Stroke::new(0.5, Color32::from_black_alpha(30)),
                StrokeKind::Outside,
            );
        } else if resp.hovered() {
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::CONTROL), p.hover);
        }
        let color = if on || resp.hovered() {
            p.text
        } else {
            p.text_secondary
        };
        widgets::paint_icon(ui, r, m.icon(), 13.0, color, false);
        if widgets::tooltip(resp, m.label(), m.cmd().shortcut()).clicked() && !on {
            actions.push(Action::SetView(side, m));
        }
    }
    track.left()
}

/// The view options popover: sort order (all views but the list) and the
/// icon size (Icons).
fn options_menu(ui: &mut egui::Ui, tab: &mut Tab, view: ViewMode, icon_size: &mut f32) {
    ui.set_min_width(210.0);
    let p = pal(ui);
    ui.label(
        egui::RichText::new(t!("view.sort_by"))
            .font(theme::medium(text::CAPTION))
            .color(p.text_secondary),
    );
    let cols = [
        (SortColumn::Name, t!("pane.col.name")),
        (SortColumn::Size, t!("pane.col.size")),
        (SortColumn::Modified, t!("pane.col.modified")),
        (SortColumn::Permissions, t!("pane.col.permissions")),
    ];
    for (c, label) in cols {
        if ui
            .add(egui::Button::selectable(tab.sort.column == c, label))
            .clicked()
        {
            tab.sort = SortSpec {
                column: c,
                ascending: if tab.sort.column == c {
                    tab.sort.ascending
                } else {
                    c == SortColumn::Name
                },
            };
            tab.rebuild_view();
        }
    }
    ui.separator();
    for (asc, label) in [(true, t!("view.ascending")), (false, t!("view.descending"))] {
        if ui
            .add(egui::Button::selectable(tab.sort.ascending == asc, label))
            .clicked()
        {
            tab.sort.ascending = asc;
            tab.rebuild_view();
        }
    }
    if view == ViewMode::Icons {
        ui.separator();
        ui.label(
            egui::RichText::new(t!("view.icon_size"))
                .font(theme::medium(text::CAPTION))
                .color(p.text_secondary),
        );
        icon_size_slider(ui, icon_size);
    }
}

/// The icon size slider: small and large icon at the ends.
pub fn icon_size_slider(ui: &mut egui::Ui, icon_size: &mut f32) -> bool {
    let p = pal(ui);
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = 130.0;
        widgets::icon(ui, icons::IMAGE, 10.0, p.text_secondary);
        let resp = ui.add(
            egui::Slider::new(icon_size, ICON_MIN..=ICON_MAX)
                .step_by(ICON_STEP as f64)
                .show_value(false),
        );
        widgets::icon(ui, icons::IMAGE, 16.0, p.text_secondary);
        changed = resp.changed();
        resp.on_hover_text(format!("{} pt", *icon_size as u32));
    });
    *icon_size = clamp_icon_size(*icon_size);
    changed
}

/// Colours of an entry's icon, name and secondary text, honouring hidden
/// files, the selection and plugin decorations (tone, dimmed).
struct Colors {
    icon: Color32,
    name: Color32,
    sub: Color32,
    dim: f32,
}

fn colors(p: &Palette, e: &Entry, deco: Option<&MergedDecoration>, on_accent: bool) -> Colors {
    let (_, tint, _) = icons::entry_icon(e, p);
    let hidden = e.hidden || e.name.starts_with('.');
    let mut icon = tint;
    let mut name = match deco.and_then(|d| d.name_tone) {
        _ if on_accent => p.text_on_accent,
        Some(tone) => theme::tone_text(p, tone),
        None => p.text,
    };
    let mut sub = if on_accent {
        p.text_on_accent.gamma_multiply(0.85)
    } else {
        p.text_secondary
    };
    if hidden {
        icon = icon.gamma_multiply(0.5);
        name = if on_accent {
            p.text_on_accent.gamma_multiply(0.75)
        } else {
            p.text_tertiary
        };
    }
    let dim = if deco.is_some_and(|d| d.dimmed) {
        if on_accent {
            0.75
        } else {
            0.5
        }
    } else {
        1.0
    };
    if dim < 1.0 {
        icon = icon.gamma_multiply(dim);
        name = name.gamma_multiply(dim);
        sub = sub.gamma_multiply(dim);
    }
    Colors {
        icon,
        name,
        sub,
        dim,
    }
}

/// The thumbnail of `e` at `pt` points (requested in the background), or
/// what to draw meanwhile.
pub(crate) fn thumb_of(ui: &egui::Ui, info: &PaneInfo<'_>, tab: &Tab, e: &Entry, pt: f32) -> Thumb {
    let Some(thumbs) = info.thumbs else {
        return Thumb::Unavailable;
    };
    if info.type_icons && tab.loc.conn.is_none() && tab.loc.archive.is_none() {
        let px = thumbs::bucket_for(pt * ui.ctx().pixels_per_point());
        if let Some(t) = icloud_app_icon(thumbs, &tab.loc.fs, e, px) {
            return t;
        }
    }
    let Some(method) = thumbs::method_for(e) else {
        return Thumb::Unavailable;
    };
    if method == Method::AppIcon && !info.type_icons {
        return Thumb::Unavailable;
    }
    if tab.loc.archive.is_some() {
        return Thumb::Unavailable;
    }
    let px = thumbs::bucket_for(pt * ui.ctx().pixels_per_point());
    let fs = &tab.loc.fs;
    if tab.loc.conn.is_none() {
        let key = ThumbKey::for_entry("", e, px);
        return thumbs.get_any(&key, || {
            fs.local_path(&e.path).map(|p| (Source::Local(p), method))
        });
    }
    if !info.remote_thumbs || method != Method::Image || e.size > thumbs::REMOTE_MAX {
        return Thumb::Unavailable;
    }
    let key = ThumbKey::for_entry(&fs.label(), e, px);
    thumbs.get_any(&key, || {
        Some((Source::Remote(fs.clone(), e.path.clone()), Method::Image))
    })
}

/// An app's folder in iCloud Drive shows the app's icon, like in Finder
/// (`None`: not such a folder, or the app isn't installed).
fn icloud_app_icon(
    thumbs: &thumbs::Thumbs,
    fs: &manta_core::SharedVfs,
    e: &Entry,
    px: u32,
) -> Option<Thumb> {
    if !e.is_dir() {
        return None;
    }
    let app = thumbs::icloud_app(&fs.local_path(&e.path)?)?;
    let key = ThumbKey {
        origin: "icloud-app".into(),
        path: app.to_string_lossy().into_owned(),
        mtime: 0,
        size: 0,
        px,
    };
    Some(thumbs.get_any(&key, || Some((Source::Local(app), Method::AppIcon))))
}

/// Real icons for small rows (List, Columns): an app's own icon, and the
/// icon macOS shows for a file's type. Built before a view borrows its tab.
pub(crate) struct RowIcons<'a> {
    thumbs: Option<&'a thumbs::Thumbs>,
    types: bool,
    /// The folder's file system when it is on this Mac (app icons).
    local: Option<manta_core::SharedVfs>,
}

impl<'a> RowIcons<'a> {
    pub fn of(info: &PaneInfo<'a>, loc: &crate::pane::Location) -> RowIcons<'a> {
        RowIcons {
            thumbs: info.thumbs,
            types: info.type_icons,
            local: (loc.conn.is_none() && loc.archive.is_none()).then(|| loc.fs.clone()),
        }
    }

    /// The icon of `e` at `pt` points, once it is made (`None`: draw the
    /// glyph; folders always do, except apps).
    pub fn get(&self, ui: &egui::Ui, e: &Entry, pt: f32) -> Option<egui::TextureHandle> {
        let thumbs = self.thumbs?;
        let px = thumbs::bucket_for(pt * ui.ctx().pixels_per_point());
        if !self.types {
            return None;
        }
        let icloud = self
            .local
            .as_ref()
            .and_then(|fs| icloud_app_icon(thumbs, fs, e, px));
        let thumb = if let Some(t) = icloud {
            t
        } else if thumbs::is_app_bundle(e) {
            let fs = self.local.as_ref()?;
            let key = ThumbKey::for_entry("", e, px);
            thumbs.get_any(&key, || {
                fs.local_path(&e.path)
                    .map(|p| (Source::Local(p), Method::AppIcon))
            })
        } else if !e.is_dir() && cfg!(target_os = "macos") {
            let ext = icons::extension(&e.name);
            thumbs.get_any(&thumbs::type_key(&ext, px), || {
                Some((Source::Type(ext.clone()), Method::TypeIcon))
            })
        } else {
            return None;
        };
        match thumb {
            Thumb::Ready(tex) => Some(tex),
            _ => None,
        }
    }
}

/// `size` scaled to fit `into` (aspect kept, never enlarged beyond 2×),
/// centred horizontally and aligned to the bottom when `bottom`.
pub fn fit(size: egui::Vec2, into: egui::Rect, bottom: bool) -> egui::Rect {
    if size.x <= 0.0 || size.y <= 0.0 {
        return into;
    }
    let k = (into.width() / size.x).min(into.height() / size.y);
    let s = size * k;
    let x = into.center().x - s.x / 2.0;
    let y = if bottom {
        into.bottom() - s.y
    } else {
        into.center().y - s.y / 2.0
    };
    egui::Rect::from_min_size(egui::pos2(x, y), s)
}

/// Draw a thumbnail texture into `r` with soft corners and a hairline.
fn paint_texture(ui: &egui::Ui, tex: &egui::TextureHandle, r: egui::Rect, rounding: u8, dim: f32) {
    let p = pal(ui);
    // A soft shadow under it, like Finder's thumbnails.
    ui.painter().rect_filled(
        r.translate(egui::vec2(0.0, 1.0)).expand(0.5),
        CornerRadius::same(rounding),
        Color32::from_black_alpha(if p.dark { 60 } else { 28 }),
    );
    egui::Image::new(egui::load::SizedTexture::new(tex.id(), r.size()))
        .corner_radius(CornerRadius::same(rounding))
        .tint(Color32::WHITE.gamma_multiply(dim))
        .paint_at(ui, r);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(rounding),
        Stroke::new(
            0.5,
            if p.dark {
                Color32::from_white_alpha(28)
            } else {
                Color32::from_black_alpha(30)
            },
        ),
        StrokeKind::Inside,
    );
}

/// Draw `e`'s thumbnail or its type icon into the square `r`.
#[allow(clippy::too_many_arguments)]
fn paint_entry_icon(
    ui: &egui::Ui,
    info: &PaneInfo<'_>,
    tab: &Tab,
    e: &Entry,
    r: egui::Rect,
    color: Color32,
    dim: f32,
    bottom: bool,
) {
    let p = pal(ui);
    match thumb_of(ui, info, tab, e, r.width()) {
        Thumb::Ready(tex) => {
            let tr = fit(tex.size_vec2(), r.shrink(r.width() * 0.04), bottom);
            paint_texture(ui, &tex, tr, if r.width() > 120.0 { 4 } else { 2 }, dim);
        }
        _ => match RowIcons::of(info, &tab.loc).get(ui, e, r.width()) {
            // No thumbnail (yet): the type's own icon, like Finder.
            Some(tex) => {
                let tr = fit(tex.size_vec2(), r, bottom);
                paint_plain(ui, &tex, tr, dim);
            }
            None => {
                let (glyph, _, fill) = icons::entry_icon(e, p);
                // Glyph sizes in whole steps (each size is rasterised once).
                let gs = ((r.width() * 0.86) / 4.0).round() * 4.0;
                widgets::paint_icon(ui, r, glyph, gs, color, fill);
            }
        },
    }
    if e.kind == manta_core::EntryKind::Symlink || e.link_target.is_some() {
        let s = (r.width() * 0.26).clamp(12.0, 28.0);
        let b = egui::Rect::from_min_size(egui::pos2(r.left(), r.bottom() - s), egui::vec2(s, s));
        ui.painter().circle_filled(b.center(), s / 2.0, p.content);
        ui.painter()
            .circle_stroke(b.center(), s / 2.0, Stroke::new(1.0, p.separator));
        widgets::paint_icon(
            ui,
            b,
            icons::ARROW_U_UP_LEFT,
            s * 0.6,
            p.text_secondary,
            false,
        );
    }
}

/// A name laid out over at most two lines, centred, cut with "…".
fn name_galley(
    ui: &egui::Ui,
    name: &str,
    width: f32,
    color: Color32,
    lines: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        name.to_string(),
        egui::TextFormat::simple(theme::regular(text::SMALL + 0.5), color),
    );
    job.wrap = egui::text::TextWrapping {
        max_width: width,
        max_rows: lines,
        break_anywhere: false,
        overflow_character: Some('…'),
    };
    job.halign = egui::Align::Center;
    ui.painter().layout_job(job)
}

fn entry_tooltip(
    e: &Entry,
    deco: Option<&MergedDecoration>,
    dir_size: Option<&manta_plugin::SizeInfo>,
) -> String {
    let mut lines = vec![util::display_name(&e.name).to_string()];
    if let Some(t) = &e.link_target {
        lines.push(t!("pane.alias_of", target = t));
    }
    if let Some(d) = dir_size {
        lines.push(pane::size_tooltip(d));
    }
    if let Some(t) = deco.and_then(|d| d.tooltip.as_ref()) {
        lines.push(t.clone());
    }
    lines.join("\n")
}

/// What happened to the items of a view this frame (applied after drawing,
/// like the list does).
#[derive(Default)]
struct Events {
    clicked: Option<(usize, egui::Modifiers)>,
    double: Option<usize>,
    drag: Option<usize>,
    context: Option<usize>,
}

impl Events {
    fn from(&mut self, vi: usize, resp: &egui::Response) {
        if resp.double_clicked() {
            self.double = Some(vi);
        } else if resp.clicked() {
            self.clicked = Some((vi, resp.ctx.input(|i| i.modifiers)));
        }
        if resp.drag_started() {
            self.drag = Some(vi);
        }
        if resp.secondary_clicked() {
            self.context = Some(vi);
        }
    }

    fn apply(self, ui: &egui::Ui, side: Side, tab: &mut Tab, actions: &mut Vec<Action>) {
        if let Some((vi, mods)) = self.clicked {
            tab.click(vi, mods);
        }
        if let Some(vi) = self.double {
            tab.select_only(vi);
            actions.push(Action::OpenSelected(side));
        }
        if let Some(vi) = self.context {
            if let Some(name) = tab.entry_at(vi).map(|e| e.name.clone()) {
                if !tab.selected.contains(&name) {
                    tab.select_only(vi);
                }
            }
        }
        if let Some(vi) = self.drag {
            let name = tab.entry_at(vi).map(|e| e.name.clone()).unwrap_or_default();
            if !tab.selected.contains(&name) {
                tab.select_only(vi);
            }
            if let Some(payload) = DragPayload::of_tab(side, tab) {
                egui::DragAndDrop::set_payload(ui.ctx(), payload);
            }
        }
    }
}

// ------------------------------------------------------------------ icons

/// The Icons view: a grid of large icons and thumbnails. Only the visible
/// rows are laid out (`show_rows`), so thousands of items stay smooth.
pub fn icons_view(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    info: &PaneInfo<'_>,
    tab: &mut Tab,
    icon_size: &mut f32,
    actions: &mut Vec<Action>,
) {
    let side = info.side;
    if pane::blocking_state(ui, rect, side, tab, actions) {
        return;
    }
    let body = rect;
    let bg = ui.interact(
        body,
        egui::Id::unique(("icons-bg", side, tab.id)),
        Sense::click_and_drag(),
    );
    if bg.clicked() {
        tab.selected.clear();
    }
    pane::list_anchor(&bg, side, tab, actions);
    let menu = MenuCtx::of(ui.ctx(), tab, info.count < 2);
    bg.context_menu(|ui| pane::background_menu(ui, side, actions, &menu));
    if pane::empty_states(ui, body, side, tab, actions) {
        return;
    }
    // ⌘ + scroll wheel / pinch zooms the icons.
    if bg.hovered() || ui.rect_contains_pointer(body) {
        let zoom = ui.input(|i| i.zoom_delta());
        if zoom != 1.0 {
            *icon_size = clamp_icon_size(*icon_size * zoom.powf(2.0));
        }
    }

    let inner = body.shrink2(egui::vec2(0.0, 4.0));
    let grid = Grid::new(inner.width() - 12.0, *icon_size);
    let n = tab.view.len();
    let rows = grid.rows(n);
    tab.view_state.grid_cols = grid.cols;
    tab.view_state.page = ((inner.height() / grid.row_h).floor() as usize).max(1);

    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(("icons", side, tab.id))
        .auto_shrink([false, false]);
    if std::mem::take(&mut tab.scroll_to_cursor) {
        if let Some(c) = tab.cursor {
            scroll =
                scroll.vertical_scroll_offset(grid.reveal(c, tab.scroll_offset, inner.height()));
        }
    }
    // Rubber band: started on empty space, auto-scrolls near the edges.
    let mut band_scroll = 0.0;
    if let (Some(b), Some(pos)) = (&tab.view_state.band, bg.interact_pointer_pos()) {
        let _ = b;
        if pos.y < inner.top() + 8.0 {
            band_scroll = -12.0;
        } else if pos.y > inner.bottom() - 8.0 {
            band_scroll = 12.0;
        }
        if band_scroll != 0.0 {
            scroll = scroll.vertical_scroll_offset((tab.scroll_offset + band_scroll).max(0.0));
            ui.ctx().request_repaint();
        }
    }

    let p = pal(ui);
    let active = info.is_active;
    let multi = tab.selected.len() > 1;
    let now = chrono::Local::now();
    let mut ev = Events::default();
    let out = ui
        .scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            scroll.show_rows(ui, grid.row_h, rows, |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let top = ui.max_rect().top() - range.start as f32 * grid.row_h;
                let origin = egui::pos2(ui.max_rect().left(), top);
                crate::a11y::list(ui, &pane::list_name(tab), n);
                // Rows are positioned from the content origin; reserve them.
                ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), grid.row_h * range.len() as f32),
                    Sense::hover(),
                );
                for row in range {
                    for c in 0..grid.cols {
                        let vi = row * grid.cols + c;
                        let Some(e) = tab.entry_at(vi) else { break };
                        let hit = grid.hit_rect(vi).translate(origin.to_vec2());
                        let resp = ui.interact(
                            hit,
                            egui::Id::unique(("icon-cell", side, tab.id, vi)),
                            Sense::CLICK | Sense::DRAG,
                        );
                        pane::a11y_entry(&resp, info, tab, vi, now);
                        if e.is_dir() {
                            pane::note_folder_target(ui, side, hit, &e.path);
                        }
                        let deco = info.decorations.get(&e.name);
                        let dir_size = e.is_dir().then(|| tab.dir_sizes.get(&e.name)).flatten();
                        paint_cell(
                            ui,
                            info,
                            tab,
                            &grid,
                            vi,
                            origin,
                            e,
                            deco,
                            CellState {
                                selected: tab.selected.contains(&e.name),
                                cursor: tab.cursor == Some(vi) && active && multi,
                                hovered: resp.hovered(),
                                active,
                            },
                        );
                        let resp = if egui::DragAndDrop::has_any_payload(ui.ctx())
                            || pane::external_drag_active(ui.ctx())
                        {
                            resp
                        } else {
                            let mut tip = entry_tooltip(e, deco, dir_size);
                            if let Some(st) = tab
                                .cloud
                                .as_ref()
                                .and_then(|c| c.get(&e.name))
                                .and_then(crate::cloud::tooltip)
                            {
                                tip.push('\n');
                                tip.push_str(&st);
                            }
                            resp.on_hover_text_at_pointer(tip)
                        };
                        ev.from(vi, &resp);
                        resp.context_menu(|ui| {
                            pane::context_menu(ui, side, actions, info.context_cmds, &menu)
                        });
                    }
                }
            })
        })
        .inner;
    tab.scroll_offset = out.state.clamped_offset().y;
    tab.viewport_h = out.inner_rect.height();
    tab.content_h = out.content_size.y;
    tab.view_state.grid = Some((
        grid,
        egui::pos2(inner.left(), inner.top() - tab.scroll_offset),
    ));

    // Rubber band (content coordinates, so it survives scrolling).
    let to_content =
        |pos: egui::Pos2, off: f32| egui::pos2(pos.x - inner.left(), pos.y - inner.top() + off);
    if bg.drag_started() {
        if let Some(pos) = bg.interact_pointer_pos() {
            let mods = ui.input(|i| i.modifiers);
            let base = if mods.command || mods.shift {
                tab.selected.names().clone()
            } else {
                BTreeSet::new()
            };
            let c = to_content(pos, tab.scroll_offset);
            tab.view_state.band = Some(Band {
                origin: c,
                current: c,
                base,
            });
        }
    }
    if let Some(pos) = bg.interact_pointer_pos().filter(|_| bg.dragged()) {
        let off = tab.scroll_offset;
        if let Some(b) = &mut tab.view_state.band {
            b.current = to_content(pos, off);
            let r = egui::Rect::from_two_pos(b.origin, b.current);
            let hits = grid.hits(r, n);
            let mut sel = b.base.clone();
            for &i in &hits {
                if let Some(e) = tab.entry_at(i) {
                    sel.insert(e.name.clone());
                }
            }
            if let Some(&last) = hits.last() {
                tab.cursor = Some(last);
                tab.anchor = Some(hits[0]);
            }
            tab.selected = sel.into();
        }
    }
    if bg.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
        tab.view_state.band = None;
    }
    if let Some(b) = &tab.view_state.band {
        let off = tab.scroll_offset;
        let r = egui::Rect::from_two_pos(b.origin, b.current)
            .translate(egui::vec2(inner.left(), inner.top() - off))
            .intersect(inner);
        let painter = ui.painter().with_clip_rect(inner);
        painter.rect_filled(r, CornerRadius::same(2), p.accent.gamma_multiply(0.12));
        painter.rect_stroke(
            r,
            CornerRadius::same(2),
            Stroke::new(1.0, p.accent.gamma_multiply(0.8)),
            StrokeKind::Inside,
        );
    }

    ev.apply(ui, side, tab, actions);
}

struct CellState {
    selected: bool,
    cursor: bool,
    hovered: bool,
    active: bool,
}

/// One item of the Icons grid: icon (or thumbnail) with a rounded
/// highlight, the name under it in a selection pill, badges at the icon.
#[allow(clippy::too_many_arguments)]
fn paint_cell(
    ui: &egui::Ui,
    info: &PaneInfo<'_>,
    tab: &Tab,
    grid: &Grid,
    vi: usize,
    origin: egui::Pos2,
    e: &Entry,
    deco: Option<&MergedDecoration>,
    s: CellState,
) {
    let p = pal(ui);
    let ir = grid.icon_rect(vi).translate(origin.to_vec2());
    let cell = grid.cell(vi).translate(origin.to_vec2());
    let icon_bg = ir.expand(4.0);
    if s.selected {
        let fill = if p.dark {
            Color32::from_white_alpha(26)
        } else {
            Color32::from_black_alpha(20)
        };
        ui.painter()
            .rect_filled(icon_bg, CornerRadius::same(radius::ROW + 2), fill);
    } else if s.hovered {
        ui.painter()
            .rect_filled(icon_bg, CornerRadius::same(radius::ROW + 2), p.hover);
    }
    let on_accent = s.selected && s.active;
    let c = colors(p, e, deco, false);
    paint_entry_icon(ui, info, tab, e, ir, c.icon, c.dim, true);

    // Cloud sync badge at the icon's upper right corner (cloud-only etc.).
    if let Some(st) = tab
        .cloud
        .as_ref()
        .and_then(|m| m.get(&e.name))
        .map(|i| i.state)
        .filter(|st| crate::cloud::badge(*st).is_some())
    {
        let d = (ir.width() * 0.3).clamp(16.0, 30.0);
        let b = egui::Rect::from_center_size(
            egui::pos2(ir.right() - d * 0.3, ir.top() + d * 0.5),
            egui::vec2(d, d),
        );
        let ring = if st == manta_vfs::cloud_local::SyncState::UploadError {
            p.danger
        } else {
            p.separator
        };
        ui.painter().circle_filled(b.center(), d / 2.0, p.content);
        ui.painter()
            .circle_stroke(b.center(), d / 2.0, Stroke::new(1.0, ring));
        crate::cloud::paint_badge(
            ui,
            b,
            st,
            (d * 0.62).round(),
            p.text_secondary.gamma_multiply(c.dim),
        );
    }

    // Plugin badges at the icon's lower right corner.
    let badges: &[manta_plugin::Badge] = deco.map_or(&[], |d| d.badges.as_slice());
    if !badges.is_empty() {
        let w = plugin_ui::badges_width(ui, badges);
        let x = (ir.right() - w + 6.0).min(cell.right() - w - 2.0);
        plugin_ui::paint_badges(ui, x, ir.bottom() - 6.0, badges, false, c.dim);
    }

    // Name: two lines, centred; selected → in a pill.
    let name_color = if on_accent {
        p.text_on_accent
    } else {
        colors(p, e, deco, false).name
    };
    // Finder tags: dots before the name's first line.
    let tags = tab.tags_of(&e.name);
    let dots_w = crate::tags::dots_width(tags, 8.0);
    let room = if dots_w > 0.0 { dots_w + 4.0 } else { 0.0 };
    let g = name_galley(
        ui,
        util::display_name(&e.name),
        cell.width() - 10.0 - room,
        name_color,
        2,
    );
    let pos = egui::pos2(cell.center().x + room / 2.0, ir.bottom() + NAME_GAP + 1.0);
    if let Some(first) = g.rows.first() {
        if dots_w > 0.0 {
            let r = first.rect().translate(pos.to_vec2());
            crate::tags::paint_dots(
                ui.painter(),
                r.left() - room - if s.selected { 5.0 } else { 0.0 },
                r.center().y,
                tags,
                8.0,
                p.content,
                p.dark,
            );
        }
    }
    if s.selected {
        let fill = if s.active {
            p.selection
        } else {
            p.selection_inactive
        };
        let mut r = egui::Rect::NOTHING;
        for row in &g.rows {
            r = r.union(row.rect().translate(pos.to_vec2()));
        }
        ui.painter().rect_filled(
            r.expand2(egui::vec2(5.0, 1.0)),
            CornerRadius::same(radius::ROW),
            fill,
        );
    }
    if s.cursor {
        ui.painter().rect_stroke(
            icon_bg,
            CornerRadius::same(radius::ROW + 2),
            Stroke::new(1.5, p.focus_ring),
            StrokeKind::Inside,
        );
    }
    ui.painter().galley(pos, g, name_color);
}

// ------------------------------------------------------------------ gallery

const STRIP_H: f32 = 96.0;
const STRIP_CELL: f32 = 68.0;
const STRIP_THUMB: f32 = 56.0;

/// The Gallery view: the selected item large on top (with its name and
/// details), a filmstrip of all items below.
pub fn gallery_view(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    info: &PaneInfo<'_>,
    tab: &mut Tab,
    actions: &mut Vec<Action>,
) {
    let side = info.side;
    if pane::blocking_state(ui, rect, side, tab, actions) {
        return;
    }
    let bg = ui.interact(
        rect,
        egui::Id::unique(("gallery-bg", side, tab.id)),
        Sense::click(),
    );
    pane::list_anchor(&bg, side, tab, actions);
    let menu = MenuCtx::of(ui.ctx(), tab, info.count < 2);
    bg.context_menu(|ui| pane::background_menu(ui, side, actions, &menu));
    if pane::empty_states(ui, rect, side, tab, actions) {
        return;
    }
    let p = pal(ui);
    // The gallery always shows something: the first item until one is chosen.
    if tab.cursor.is_none_or(|c| c >= tab.view.len()) {
        tab.select_only(0);
    }
    let strip =
        egui::Rect::from_min_max(egui::pos2(rect.left(), rect.bottom() - STRIP_H), rect.max);
    let stage = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), strip.top()));
    ui.painter()
        .rect_filled(strip, CornerRadius::ZERO, p.stripe);
    ui.painter()
        .hline(strip.x_range(), strip.top(), Stroke::new(1.0, p.separator));

    let mut ev = Events::default();
    // ---- the stage: large preview + details.
    if let Some(ci) = tab.cursor {
        if let Some(e) = tab.entry_at(ci).cloned() {
            let deco = info.decorations.get(&e.name);
            let meta_h = 58.0;
            let mut detail_top = stage.bottom() - meta_h;
            let area = egui::Rect::from_min_max(
                stage.min + egui::vec2(space::XL * 2.0, space::XL + 4.0),
                egui::pos2(
                    stage.right() - space::XL * 2.0,
                    stage.bottom() - meta_h - space::SM,
                ),
            );
            if area.height() > 40.0 && area.width() > 40.0 {
                let side_pt = area.width().min(area.height());
                let c = colors(p, &e, deco, false);
                let shown = match thumb_of(
                    ui,
                    info,
                    tab,
                    &e,
                    area.width().max(area.height()).min(1024.0),
                ) {
                    Thumb::Ready(tex) => {
                        // Never upscale a small image more than 2×.
                        let s = tex.size_vec2() / ui.ctx().pixels_per_point();
                        let max =
                            egui::Rect::from_center_size(area.center(), (s * 2.0).min(area.size()));
                        let r = fit(tex.size_vec2(), max, false);
                        paint_texture(ui, &tex, r, 6, c.dim);
                        r
                    }
                    _ => {
                        let r = egui::Rect::from_center_size(
                            area.center(),
                            egui::vec2(side_pt, side_pt).min(egui::vec2(220.0, 220.0)),
                        );
                        paint_entry_icon(ui, info, tab, &e, r, c.icon, c.dim, false);
                        r
                    }
                };
                detail_top = detail_top.min(shown.bottom() + space::LG + 2.0);
                let resp = ui.interact(
                    shown,
                    egui::Id::unique(("gallery-stage", side, tab.id)),
                    Sense::CLICK | Sense::DRAG,
                );
                crate::a11y::name(
                    &resp,
                    egui::accesskit::Role::Image,
                    &crate::a11y::entry_label(&e, chrono::Local::now(), None, None, None),
                );
                ev.from(ci, &resp);
                resp.context_menu(|ui| {
                    pane::context_menu(ui, side, actions, info.context_cmds, &menu)
                });
            }
            // Details under the preview.
            let mut y = detail_top;
            let g = name_galley(
                ui,
                util::display_name(&e.name),
                stage.width() - 4.0 * space::XL,
                p.text,
                1,
            );
            let g = {
                // Medium weight for the title.
                let mut job = (*g.job).clone();
                for s in &mut job.sections {
                    s.format.font_id = theme::medium(text::BODY);
                }
                ui.painter().layout_job(job)
            };
            // Finder tags: dots before the title.
            let tags = tab.tags_of(&e.name);
            if let Some(first) = g.rows.first() {
                let r = first.rect().translate(egui::vec2(stage.center().x, y));
                let w = crate::tags::dots_width(tags, 9.0);
                if w > 0.0 {
                    crate::tags::paint_dots(
                        ui.painter(),
                        r.left() - w - 5.0,
                        r.center().y,
                        tags,
                        9.0,
                        p.content,
                        p.dark,
                    );
                }
            }
            ui.painter()
                .galley(egui::pos2(stage.center().x, y), g, p.text);
            y += 20.0;
            let detail = details_line(&e, tab.dir_sizes.get(&e.name));
            ui.painter().text(
                egui::pos2(stage.center().x, y),
                egui::Align2::CENTER_TOP,
                detail,
                theme::regular(text::SMALL),
                p.text_secondary,
            );
            let badges: &[manta_plugin::Badge] = deco.map_or(&[], |d| d.badges.as_slice());
            if !badges.is_empty() {
                let w = plugin_ui::badges_width(ui, badges);
                plugin_ui::paint_badges(
                    ui,
                    stage.center().x - w / 2.0,
                    y + 26.0,
                    badges,
                    false,
                    1.0,
                );
            }
        }
    }

    // ---- the filmstrip (only the visible thumbnails are drawn).
    let n = tab.view.len();
    let strip_inner = strip.shrink2(egui::vec2(space::MD, 0.0));
    let mut scroll = egui::ScrollArea::horizontal()
        .id_salt(("gallery-strip", side, tab.id))
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
    if std::mem::take(&mut tab.scroll_to_cursor) {
        if let Some(c) = tab.cursor {
            let x = reveal_span(
                c as f32 * STRIP_CELL,
                STRIP_CELL,
                tab.view_state.strip_offset,
                strip_inner.width(),
            );
            scroll = scroll.horizontal_scroll_offset(x);
        }
    }
    let active = info.is_active;
    let out = ui
        .scope_builder(egui::UiBuilder::new().max_rect(strip_inner), |ui| {
            scroll.show_viewport(ui, |ui, viewport| {
                crate::a11y::list(ui, &pane::list_name(tab), n);
                let now = chrono::Local::now();
                let (full, _) = ui.allocate_exact_size(
                    egui::vec2(n as f32 * STRIP_CELL, strip_inner.height() - 4.0),
                    Sense::hover(),
                );
                let first = (viewport.left() / STRIP_CELL).floor().max(0.0) as usize;
                let last = ((viewport.right() / STRIP_CELL).ceil() as usize).min(n);
                for vi in first..last {
                    let Some(e) = tab.entry_at(vi) else { break };
                    let cell = egui::Rect::from_min_size(
                        egui::pos2(full.left() + vi as f32 * STRIP_CELL, full.top()),
                        egui::vec2(STRIP_CELL, full.height()),
                    );
                    let tr = egui::Rect::from_center_size(
                        cell.center() - egui::vec2(0.0, 2.0),
                        egui::vec2(STRIP_THUMB, STRIP_THUMB),
                    );
                    let resp = ui.interact(
                        tr.expand(4.0),
                        egui::Id::unique(("strip", side, tab.id, vi)),
                        Sense::CLICK | Sense::DRAG,
                    );
                    pane::a11y_entry(&resp, info, tab, vi, now);
                    if e.is_dir() {
                        pane::note_folder_target(ui, side, resp.rect, &e.path);
                    }
                    let selected = tab.selected.contains(&e.name);
                    let deco = info.decorations.get(&e.name);
                    let hl = tr.expand(5.0);
                    if selected {
                        ui.painter().rect_filled(
                            hl,
                            CornerRadius::same(radius::ROW + 2),
                            if p.dark {
                                Color32::from_white_alpha(26)
                            } else {
                                Color32::from_black_alpha(18)
                            },
                        );
                    } else if resp.hovered() {
                        ui.painter()
                            .rect_filled(hl, CornerRadius::same(radius::ROW + 2), p.hover);
                    }
                    let c = colors(p, e, deco, false);
                    paint_entry_icon(ui, info, tab, e, tr, c.icon, c.dim, false);
                    if selected {
                        let ring = if active { p.accent } else { p.border };
                        ui.painter().rect_stroke(
                            hl,
                            CornerRadius::same(radius::ROW + 2),
                            Stroke::new(2.0, ring),
                            StrokeKind::Inside,
                        );
                    }
                    let badges: &[manta_plugin::Badge] = deco.map_or(&[], |d| d.badges.as_slice());
                    if let Some(b) = badges.first() {
                        plugin_ui::paint_badges(
                            ui,
                            tr.right() - plugin_ui::badges_width(ui, std::slice::from_ref(b)) + 4.0,
                            tr.bottom() - 4.0,
                            std::slice::from_ref(b),
                            false,
                            c.dim,
                        );
                    }
                    let resp = if egui::DragAndDrop::has_any_payload(ui.ctx())
                        || pane::external_drag_active(ui.ctx())
                    {
                        resp
                    } else {
                        resp.on_hover_text_at_pointer(entry_tooltip(
                            e,
                            deco,
                            tab.dir_sizes.get(&e.name),
                        ))
                    };
                    ev.from(vi, &resp);
                    resp.context_menu(|ui| {
                        pane::context_menu(ui, side, actions, info.context_cmds, &menu)
                    });
                }
            })
        })
        .inner;
    tab.view_state.strip_offset = out.state.clamped_offset().x;
    tab.view_state.strip_view_w = out.inner_rect.width();
    tab.view_state.page = ((out.inner_rect.width() / STRIP_CELL).floor() as usize).max(1);
    ev.apply(ui, side, tab, actions);
}

/// "Kind · size · modified" under a preview.
fn details_line(e: &Entry, dir_size: Option<&manta_plugin::SizeInfo>) -> String {
    let kind = icons::file_kind(&e.name, e.is_dir()).label();
    let mut parts = vec![kind.to_string()];
    if e.is_dir() {
        if let Some(s) = dir_size {
            parts.push(pane::size_label(s));
        }
    } else {
        parts.push(util::format_size(e.size));
    }
    if let Some(t) = e.modified {
        parts.push(format_date(t));
    }
    parts.join(" · ")
}

/// A full local date, e.g. "21.10.2026, 14:05" / "Oct 21, 2026 at 2:05 PM" /
/// "21 oct. 2026 à 14:05".
pub fn format_date(t: chrono::DateTime<chrono::Utc>) -> String {
    use crate::i18n::Locale;
    use chrono::Datelike;
    let l = t.with_timezone(&chrono::Local);
    let time = l.format("%H:%M").to_string();
    match crate::i18n::locale() {
        Locale::En => l.format("%b %-d, %Y at %-I:%M %p").to_string(),
        Locale::De => l.format("%d.%m.%Y, %H:%M").to_string(),
        Locale::Zh => crate::i18n::format_day(l.day(), l.month0(), Some(l.year()), Some(&time)),
        Locale::Fr => format!(
            "{} à {time}",
            crate::i18n::format_day(l.day(), l.month0(), Some(l.year()), None)
        ),
        _ => format!(
            "{}, {time}",
            crate::i18n::format_day(l.day(), l.month0(), Some(l.year()), None)
        ),
    }
}

// ------------------------------------------------------------------ columns

/// Default and limits of a column's width.
pub const COLUMN_W: f32 = 220.0;
pub const COLUMN_MIN: f32 = 140.0;
pub const COLUMN_MAX: f32 = 600.0;
const PREVIEW_W: f32 = 280.0;
const COL_ROW: f32 = 22.0;

/// A folder listed for a column other than the active one.
pub struct ColListing {
    pub entries: Vec<Entry>,
    /// Indices into `entries` in display order, for `order_key`.
    order: Vec<usize>,
    order_key: Option<(SortSpec, bool)>,
    pub loading: bool,
    pub error: Option<String>,
    /// The tab's `listing_gen` when it was requested (refreshed after a
    /// reload of the tab).
    gen: u64,
}

impl ColListing {
    fn ordered(&mut self, sort: SortSpec, hidden: bool) -> &[usize] {
        if self.order_key != Some((sort, hidden)) {
            self.order = util::visible_indices(&self.entries, sort, hidden, "", &HashMap::new());
            self.order_key = Some((sort, hidden));
        }
        &self.order
    }
}

struct ColReq {
    path: VfsPath,
    /// The column right of the active one: only the newest is listed.
    child: bool,
}

struct ColDone {
    path: VfsPath,
    result: Result<Vec<Entry>, String>,
}

/// The Columns view's folders: the leftmost column (`root`), and the
/// listings of the columns that aren't the active one (loaded on a
/// background thread; the active column is the tab itself).
#[derive(Default)]
pub struct ColumnsState {
    pub root: Option<VfsPath>,
    fs_id: usize,
    pub listings: HashMap<VfsPath, ColListing>,
    tx: Option<mpsc::Sender<ColReq>>,
    rx: Option<mpsc::Receiver<ColDone>>,
    /// The active folder in the last frame (reveal it when it changes).
    last_active: Option<VfsPath>,
    /// Frames left to keep revealing the active column (egui clamps a
    /// scroll offset to the previous frame's content width).
    reveal_frames: u8,
    /// The folder shown right of the active column in the last frame.
    last_next: Option<VfsPath>,
    /// Columns whose highlighted row was scrolled into view once.
    revealed: HashSet<VfsPath>,
}

fn fs_id(fs: &SharedVfs) -> usize {
    std::sync::Arc::as_ptr(fs) as *const () as usize
}

/// `path` is `root` or inside it.
pub fn is_within(path: &VfsPath, root: &VfsPath) -> bool {
    let (p, r) = (
        path.as_str().trim_end_matches('/'),
        root.as_str().trim_end_matches('/'),
    );
    r.is_empty() || p == r || p.starts_with(&format!("{r}/"))
}

/// The folders shown as columns, root first, ending with `active`.
pub fn column_chain(root: &VfsPath, active: &VfsPath) -> Vec<VfsPath> {
    let mut chain = vec![active.clone()];
    let mut cur = active.clone();
    while &cur != root {
        match cur.parent() {
            Some(p) if is_within(&p, root) || &p == root => {
                chain.push(p.clone());
                cur = p;
            }
            _ => break,
        }
    }
    chain.reverse();
    chain
}

impl ColumnsState {
    /// Keep the root valid for `tab`: a new backend, or an active folder
    /// outside the root (↑ past it, the sidebar, a breadcrumb) restarts the
    /// columns at the active folder.
    pub fn sync_root(&mut self, tab: &Tab) {
        let id = fs_id(&tab.loc.fs);
        if self.fs_id != id {
            self.fs_id = id;
            self.listings.clear();
            self.revealed.clear();
            self.root = None;
        }
        let active = &tab.loc.path;
        if self.root.as_ref().is_none_or(|r| !is_within(active, r)) {
            self.root = Some(active.clone());
        }
    }

    /// A cached listing of `path` (to show a folder at once while the tab
    /// loads it again).
    pub fn cached(&self, fs: &SharedVfs, path: &VfsPath) -> Option<Vec<Entry>> {
        if self.fs_id != fs_id(fs) {
            return None;
        }
        self.listings
            .get(path)
            .filter(|l| !l.loading && l.error.is_none())
            .map(|l| l.entries.clone())
    }

    /// Remember the active folder's listing (the column left of the next
    /// active folder shows it without loading it again).
    pub fn remember(&mut self, tab: &Tab) {
        if self.fs_id != fs_id(&tab.loc.fs) || tab.loading {
            return;
        }
        let gen = tab.listing_gen;
        let e = self
            .listings
            .entry(tab.loc.path.clone())
            .or_insert_with(|| ColListing {
                entries: Vec::new(),
                order: Vec::new(),
                order_key: None,
                loading: false,
                error: None,
                gen: 0,
            });
        if e.gen != gen || e.loading {
            e.entries = tab.entries.clone();
            e.order_key = None;
            e.loading = false;
            e.error = None;
            e.gen = gen;
        }
    }

    /// Ask for `path`'s listing (no-op if it is there and fresh).
    pub fn request(
        &mut self,
        ctx: &egui::Context,
        fs: &SharedVfs,
        path: &VfsPath,
        gen: u64,
        child: bool,
    ) {
        if let Some(l) = self.listings.get(path) {
            if l.loading || l.gen == gen {
                return;
            }
        }
        if self.tx.is_none() {
            let (tx, rx) = mpsc::channel::<ColReq>();
            let (dtx, drx) = mpsc::channel::<ColDone>();
            let fs = fs.clone();
            let ctx = ctx.clone();
            let _ = std::thread::Builder::new()
                .name("manta-columns".into())
                .spawn(move || columns_loader(fs, rx, dtx, ctx));
            self.tx = Some(tx);
            self.rx = Some(drx);
        }
        let old = self.listings.remove(path);
        self.listings.insert(
            path.clone(),
            ColListing {
                entries: old.map(|o| o.entries).unwrap_or_default(),
                order: Vec::new(),
                order_key: None,
                loading: true,
                error: None,
                gen,
            },
        );
        if let Some(tx) = &self.tx {
            let _ = tx.send(ColReq {
                path: path.clone(),
                child,
            });
        }
    }

    /// Take finished listings.
    pub fn poll(&mut self) {
        let Some(rx) = &self.rx else { return };
        while let Ok(d) = rx.try_recv() {
            if let Some(l) = self.listings.get_mut(&d.path) {
                l.loading = false;
                l.order_key = None;
                match d.result {
                    Ok(entries) => {
                        l.entries = entries;
                        l.error = None;
                    }
                    Err(e) => l.error = Some(e),
                }
            }
        }
    }

    /// Wait until no listing is loading (tests).
    #[cfg(test)]
    pub fn wait_loaded(&mut self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        loop {
            self.poll();
            if !self.listings.values().any(|l| l.loading) {
                return true;
            }
            if start.elapsed() > timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

/// The background thread that lists folders for the columns. Ancestor
/// folders are all listed; of the "next column" requests only the newest
/// (holding ↓ through a folder of folders asks for many).
fn columns_loader(
    fs: SharedVfs,
    rx: mpsc::Receiver<ColReq>,
    tx: mpsc::Sender<ColDone>,
    ctx: egui::Context,
) {
    while let Ok(first) = rx.recv() {
        let mut batch = vec![first];
        while let Ok(r) = rx.try_recv() {
            batch.push(r);
        }
        let newest_child = batch.iter().rposition(|r| r.child);
        let mut skipped = Vec::new();
        for (i, r) in batch.into_iter().enumerate() {
            if r.child && Some(i) != newest_child {
                skipped.push(r.path);
                continue;
            }
            let result =
                crate::worker::with_reconnect(&fs, || fs.list(&r.path)).map_err(|e| e.to_string());
            if tx
                .send(ColDone {
                    path: r.path,
                    result,
                })
                .is_err()
            {
                return;
            }
            ctx.request_repaint();
        }
        // Skipped folders aren't loading any more (asked again if shown).
        for path in skipped {
            if tx
                .send(ColDone {
                    path,
                    result: Err(String::new()),
                })
                .is_err()
            {
                return;
            }
        }
    }
}

/// The Columns view: one column per folder from the root to the active
/// folder (the tab), then the selected folder's contents or a preview of
/// the selected file. Columns scroll sideways and can be resized.
pub fn columns_view(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    info: &PaneInfo<'_>,
    tab: &mut Tab,
    widths: &mut Vec<f32>,
    actions: &mut Vec<Action>,
) {
    let row_icons = &RowIcons::of(info, &tab.loc);
    let side = info.side;
    if pane::blocking_state(ui, rect, side, tab, actions) {
        return;
    }
    let p = pal(ui);
    let ctx = ui.ctx().clone();
    {
        let mut cols = std::mem::take(&mut tab.view_state.cols);
        cols.sync_root(tab);
        cols.poll();
        cols.remember(tab);
        tab.view_state.cols = cols;
    }
    let root = tab
        .view_state
        .cols
        .root
        .clone()
        .unwrap_or_else(|| tab.loc.path.clone());
    let chain = column_chain(&root, &tab.loc.path);
    let gen = tab.listing_gen;
    let fs = tab.loc.fs.clone();
    for path in &chain[..chain.len() - 1] {
        tab.view_state.cols.request(&ctx, &fs, path, gen, false);
    }
    // What follows the active column: a folder's contents or a preview.
    let single = tab.selected.len() <= 1;
    let next_dir = tab
        .cursor_entry()
        .filter(|e| e.is_dir() && single)
        .map(|e| e.path.clone());
    if let Some(d) = &next_dir {
        tab.view_state.cols.request(&ctx, &fs, d, gen, true);
    }
    let preview = !tab.view.is_empty()
        && next_dir.is_none()
        && (tab.cursor_entry().is_some() || !tab.selected.is_empty());

    let width_of = |i: usize, widths: &Vec<f32>| {
        widths
            .get(i)
            .copied()
            .unwrap_or(COLUMN_W)
            .clamp(COLUMN_MIN, COLUMN_MAX)
    };
    let n_cols = chain.len() + usize::from(next_dir.is_some());
    let mut xs = Vec::with_capacity(n_cols + 1);
    let mut x = 0.0;
    for i in 0..n_cols {
        xs.push(x);
        x += width_of(i, widths);
    }
    let preview_w = if preview {
        PREVIEW_W.max(rect.width() - x)
    } else {
        (rect.width() - x).max(0.0)
    };
    let total_w = x + preview_w;

    // Reveal the active column and the one after it when the folder changes.
    let active_i = chain.len() - 1;
    let changed = tab.view_state.cols.last_active.as_ref() != Some(&tab.loc.path)
        || tab.view_state.cols.last_next != next_dir;
    tab.view_state.cols.last_active = Some(tab.loc.path.clone());
    tab.view_state.cols.last_next = next_dir.clone();
    let scroll = egui::ScrollArea::horizontal()
        .id_salt(("columns", side, tab.id))
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
    if changed || tab.scroll_to_cursor {
        tab.view_state.cols.reveal_frames = 3;
    }
    // The span to keep in view: the active column and what follows it.
    let mut reveal: Option<(f32, f32)> = None;
    if tab.view_state.cols.reveal_frames > 0 {
        tab.view_state.cols.reveal_frames -= 1;
        ctx.request_repaint();
        let right_edge = match (&next_dir, preview) {
            (Some(_), _) => {
                xs[active_i] + width_of(active_i, widths) + width_of(active_i + 1, widths)
            }
            (None, true) => x + preview_w.min(PREVIEW_W),
            _ => x,
        };
        // Wider than the pane: the active column's left edge wins.
        let left = xs[active_i];
        reveal = Some((left.max(right_edge - rect.width()), right_edge));
    }

    let menu = MenuCtx::of(ui.ctx(), tab, info.count < 2);
    let mut ev = Events::default();
    let mut out_actions: Vec<Action> = Vec::new();
    let mut resize: Option<(usize, f32)> = None;
    let mut reset: Option<usize> = None;
    let out = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            scroll.show(ui, |ui| {
                let (full, _) = ui
                    .allocate_exact_size(egui::vec2(total_w, rect.height() - 1.0), Sense::hover());
                if let Some((a, b)) = reveal {
                    let r = egui::Rect::from_x_y_ranges(
                        egui::Rangef::new(full.left() + a.min(b), full.left() + b),
                        full.y_range(),
                    );
                    ui.scroll_to_rect(r, None);
                }
                for (i, path) in chain.iter().enumerate() {
                    let col = egui::Rect::from_min_size(
                        egui::pos2(full.left() + xs[i], full.top()),
                        egui::vec2(width_of(i, widths), full.height()),
                    );
                    if i == active_i {
                        active_column(ui, col, info, tab, &mut ev, &menu, actions);
                    } else {
                        let highlight = chain.get(i + 1).map(|c| c.file_name().to_string());
                        let tags = folder_tags(info, tab, path);
                        other_column(
                            ui,
                            col,
                            side,
                            tab,
                            path,
                            highlight.as_deref(),
                            false,
                            tags.as_deref(),
                            row_icons,
                            &mut out_actions,
                        );
                    }
                    column_edge(ui, col, side, tab.id, i, &mut resize, &mut reset);
                }
                if let Some(d) = &next_dir {
                    let i = chain.len();
                    let col = egui::Rect::from_min_size(
                        egui::pos2(full.left() + xs[i], full.top()),
                        egui::vec2(width_of(i, widths), full.height()),
                    );
                    let tags = folder_tags(info, tab, d);
                    other_column(
                        ui,
                        col,
                        side,
                        tab,
                        d,
                        None,
                        true,
                        tags.as_deref(),
                        row_icons,
                        &mut out_actions,
                    );
                    column_edge(ui, col, side, tab.id, i, &mut resize, &mut reset);
                }
                if preview {
                    let col =
                        egui::Rect::from_min_max(egui::pos2(full.left() + x, full.top()), full.max);
                    preview_column(ui, col, info, tab);
                }
            })
        })
        .inner;
    tab.view_state.strip_offset = out.state.clamped_offset().x;
    tab.view_state.strip_view_w = out.inner_rect.width();
    let _ = p;
    if let Some((i, dx)) = resize {
        if widths.len() <= i {
            widths.resize(i + 1, COLUMN_W);
        }
        widths[i] = (widths[i] + dx).clamp(COLUMN_MIN, COLUMN_MAX);
    }
    if let Some(i) = reset {
        if let Some(w) = widths.get_mut(i) {
            *w = COLUMN_W;
        }
    }
    ev.apply(ui, side, tab, actions);
    actions.extend(out_actions);
}

/// The drag handle on a column's right edge (and its hairline).
fn column_edge(
    ui: &mut egui::Ui,
    col: egui::Rect,
    side: Side,
    tab_id: u64,
    i: usize,
    resize: &mut Option<(usize, f32)>,
    reset: &mut Option<usize>,
) {
    let p = pal(ui);
    let x = (col.right() - 0.5).round() + 0.5;
    ui.painter()
        .vline(x - 0.5, col.y_range(), Stroke::new(1.0, p.separator));
    let h = egui::Rect::from_x_y_ranges(
        egui::Rangef::new(col.right() - 4.0, col.right() + 3.0),
        col.y_range(),
    );
    let r = ui.interact(
        h,
        egui::Id::unique(("col-resize", side, tab_id, i)),
        Sense::CLICK | Sense::DRAG,
    );
    if r.hovered() || r.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
    }
    if r.dragged() {
        *resize = Some((i, r.drag_delta().x));
    }
    if r.double_clicked() {
        *reset = Some(i);
    }
    // Finder's grip at the bottom of the edge.
    let grip = egui::Rect::from_center_size(
        egui::pos2(col.right() - 6.0, col.bottom() - 12.0),
        egui::vec2(8.0, 12.0),
    );
    let c = if r.hovered() || r.dragged() {
        p.text_secondary
    } else {
        p.text_tertiary.gamma_multiply(0.7)
    };
    for dx in [-1.5, 1.5] {
        ui.painter().vline(
            grip.center().x + dx,
            grip.y_range().shrink(2.0),
            Stroke::new(1.0, c),
        );
    }
}

/// An icon texture as is (no shadow, unlike thumbnails), faded by `dim`.
pub(crate) fn paint_plain(ui: &egui::Ui, tex: &egui::TextureHandle, r: egui::Rect, dim: f32) {
    ui.painter().image(
        tex.id(),
        r,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE.gamma_multiply(dim),
    );
}

/// One row of a column: icon, name (cut), and at the right a folder chevron
/// (plus a size or badges where known).
#[allow(clippy::too_many_arguments)]
fn column_row(
    ui: &egui::Ui,
    r: egui::Rect,
    e: &Entry,
    icons_: &RowIcons<'_>,
    fill: Option<Color32>,
    on_accent: bool,
    deco: Option<&MergedDecoration>,
    right_text: Option<String>,
    tags: &[crate::tags::Tag],
) {
    let p = pal(ui);
    let inset = r.shrink2(egui::vec2(space::XS, 0.0));
    if let Some(f) = fill {
        ui.painter()
            .rect_filled(inset, CornerRadius::same(radius::ROW), f);
    }
    let c = colors(p, e, deco, on_accent);
    let (glyph, _, filled) = icons::entry_icon(e, p);
    let ir = egui::Rect::from_center_size(
        egui::pos2(inset.left() + 13.0, r.center().y),
        egui::vec2(16.0, 16.0),
    );
    match icons_.get(ui, e, 16.0) {
        Some(tex) => paint_plain(ui, &tex, ir, c.dim),
        None => widgets::paint_icon(
            ui,
            ir,
            glyph,
            15.0,
            if on_accent { p.text_on_accent } else { c.icon },
            filled,
        ),
    }
    let mut right = inset.right() - space::SM;
    if e.is_dir() {
        let cr = egui::Rect::from_center_size(
            egui::pos2(right - 4.0, r.center().y),
            egui::vec2(10.0, 12.0),
        );
        widgets::paint_icon(
            ui,
            cr,
            icons::CARET_RIGHT,
            10.0,
            if on_accent {
                p.text_on_accent
            } else {
                p.text_tertiary
            },
            false,
        );
        right = cr.left() - space::XS;
    }
    if let Some(t) = right_text {
        let g = ui.painter().layout_no_wrap(
            t,
            theme::regular(text::CAPTION),
            if on_accent { c.sub } else { p.text_tertiary },
        );
        let w = g.size().x;
        ui.painter().galley(
            egui::pos2(right - w, r.center().y - g.size().y / 2.0),
            g,
            p.text_tertiary,
        );
        right -= w + space::SM;
    }
    let badges: &[manta_plugin::Badge] = deco.map_or(&[], |d| d.badges.as_slice());
    if !badges.is_empty() {
        let w = plugin_ui::badges_width(ui, badges);
        if right - w > ir.right() + 60.0 {
            plugin_ui::paint_badges(ui, right - w, r.center().y, badges, on_accent, c.dim);
            right -= w + space::SM;
        }
    }
    let mut job = egui::text::LayoutJob::simple_singleline(
        e.name.clone(),
        theme::regular(text::BODY),
        c.name,
    );
    // Finder tags: dots after the name.
    let dots_w = crate::tags::dots_width(tags, 8.0);
    let reserve = if dots_w > 0.0 {
        dots_w + space::SM
    } else {
        0.0
    };
    job.wrap =
        egui::text::TextWrapping::truncate_at_width((right - ir.right() - 7.0 - reserve).max(10.0));
    let g = ui.painter().layout_job(job);
    let name_w = g.size().x;
    ui.painter().galley(
        egui::pos2(ir.right() + 6.0, r.center().y - g.size().y / 2.0),
        g,
        c.name,
    );
    if dots_w > 0.0 {
        let ring = fill.unwrap_or(p.content);
        crate::tags::paint_dots(
            ui.painter(),
            ir.right() + 6.0 + name_w + space::SM,
            r.center().y,
            tags,
            8.0,
            ring,
            p.dark,
        );
    }
}

/// The tags of a column's folder (local folders, once read).
fn folder_tags(
    info: &PaneInfo<'_>,
    tab: &Tab,
    path: &VfsPath,
) -> Option<std::sync::Arc<crate::tags::FolderTags>> {
    let store = info.tags?;
    if !crate::tags::taggable(&tab.loc) {
        return None;
    }
    store.get(&tab.loc.fs.local_path(path)?)
}

/// The active column: the tab's own entries, selection and cursor.
fn active_column(
    ui: &mut egui::Ui,
    col: egui::Rect,
    info: &PaneInfo<'_>,
    tab: &mut Tab,
    ev: &mut Events,
    menu: &MenuCtx,
    actions: &mut Vec<Action>,
) {
    let row_icons = &RowIcons::of(info, &tab.loc);
    let side = info.side;
    let p = pal(ui);
    let bg = ui.interact(
        col,
        egui::Id::unique(("col-active-bg", side, tab.id)),
        Sense::click(),
    );
    if bg.clicked() {
        tab.selected.clear();
    }
    pane::list_anchor(&bg, side, tab, actions);
    bg.context_menu(|ui| pane::background_menu(ui, side, actions, menu));
    if tab.view.is_empty() {
        column_note(ui, col, tab.loading, &tab.filter, tab.loading_since);
        return;
    }
    let inner = col.shrink2(egui::vec2(0.0, space::XS));
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(("col-active", side, tab.id, tab.loc.path.as_str()))
        .auto_shrink([false, false]);
    if std::mem::take(&mut tab.scroll_to_cursor) {
        if let Some(c) = tab.cursor {
            scroll = scroll.vertical_scroll_offset(reveal_span(
                c as f32 * COL_ROW,
                COL_ROW,
                tab.scroll_offset,
                inner.height(),
            ));
        }
    }
    let active = info.is_active;
    let out = ui
        .scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            scroll.show_rows(ui, COL_ROW, tab.view.len(), |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;
                crate::a11y::list(ui, &pane::list_name(tab), tab.view.len());
                let now = chrono::Local::now();
                for vi in range {
                    let (r, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), COL_ROW),
                        Sense::CLICK | Sense::DRAG,
                    );
                    pane::a11y_entry(&resp, info, tab, vi, now);
                    let Some(e) = tab.entry_at(vi) else { continue };
                    if e.is_dir() {
                        pane::note_folder_target(ui, side, r, &e.path);
                    }
                    let selected = tab.selected.contains(&e.name);
                    let fill = if selected {
                        Some(if active {
                            p.selection
                        } else {
                            p.selection_inactive
                        })
                    } else if resp.hovered() {
                        Some(p.hover)
                    } else {
                        None
                    };
                    let deco = info.decorations.get(&e.name);
                    let size = e.is_dir().then(|| tab.dir_sizes.get(&e.name)).flatten();
                    column_row(
                        ui,
                        r,
                        e,
                        row_icons,
                        fill,
                        selected && active,
                        deco,
                        size.map(pane::size_label),
                        tab.tags_of(&e.name),
                    );
                    if tab.cursor == Some(vi) && active && tab.selected.len() > 1 {
                        ui.painter().rect_stroke(
                            r.shrink2(egui::vec2(space::XS, 0.0)).shrink(1.0),
                            CornerRadius::same(radius::ROW),
                            Stroke::new(
                                1.5,
                                if selected {
                                    p.text_on_accent.gamma_multiply(0.7)
                                } else {
                                    p.focus_ring
                                },
                            ),
                            StrokeKind::Inside,
                        );
                    }
                    let resp = if egui::DragAndDrop::has_any_payload(ui.ctx())
                        || pane::external_drag_active(ui.ctx())
                    {
                        resp
                    } else {
                        match (deco.and_then(|d| d.tooltip.as_ref()), size) {
                            (_, Some(s)) => resp.on_hover_text(pane::size_tooltip(s)),
                            (Some(t), None) => resp.on_hover_text(t),
                            _ => resp,
                        }
                    };
                    ev.from(vi, &resp);
                    resp.context_menu(|ui| {
                        pane::context_menu(ui, side, actions, info.context_cmds, menu)
                    });
                }
            })
        })
        .inner;
    tab.scroll_offset = out.state.clamped_offset().y;
    tab.viewport_h = out.inner_rect.height();
    tab.content_h = out.content_size.y;
}

/// "Loading…" / "Empty" inside a column.
fn column_note(
    ui: &mut egui::Ui,
    col: egui::Rect,
    loading: bool,
    filter: &str,
    since: Option<std::time::Instant>,
) {
    let p = pal(ui);
    let c = egui::pos2(col.center().x, col.top() + 40.0);
    if loading {
        if since.is_some_and(|t| t.elapsed() > Duration::from_millis(200)) {
            ui.put(
                egui::Rect::from_center_size(c, egui::vec2(16.0, 16.0)),
                egui::Spinner::new().size(14.0).color(p.text_tertiary),
            );
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(220));
        }
        return;
    }
    let text = if filter.is_empty() {
        t!("view.column_empty").to_string()
    } else {
        t!("pane.no_match", filter = filter.trim())
    };
    ui.painter().text(
        c,
        egui::Align2::CENTER_CENTER,
        text,
        theme::regular(text::SMALL),
        p.text_tertiary,
    );
}

/// A column that isn't the active one: an enclosing folder (`highlight` =
/// the folder that is open to its right) or the selected folder's contents
/// (`next`). A click makes that folder the active column.
#[allow(clippy::too_many_arguments)]
fn other_column(
    ui: &mut egui::Ui,
    col: egui::Rect,
    side: Side,
    tab: &mut Tab,
    path: &VfsPath,
    highlight: Option<&str>,
    next: bool,
    tags: Option<&crate::tags::FolderTags>,
    row_icons: &RowIcons<'_>,
    actions: &mut Vec<Action>,
) {
    let p = pal(ui);
    let sort = tab.sort;
    let hidden = tab.show_hidden;
    let tab_id = tab.id;
    let dir_size = next
        .then(|| {
            tab.cursor_entry()
                .and_then(|e| tab.dir_sizes.get(&e.name))
                .cloned()
        })
        .flatten();
    let bg = ui.interact(
        col,
        egui::Id::unique(("col-bg", side, tab_id, path.as_str())),
        Sense::CLICK,
    );
    // Files from another app dropped onto this column go into its folder.
    pane::note_folder_target(ui, side, col, path);
    if bg.clicked() {
        actions.push(Action::ColumnsSelect(side, path.clone(), None));
    }
    let reveal = highlight.is_some() && !tab.view_state.cols.revealed.contains(path);
    let Some(listing) = tab.view_state.cols.listings.get_mut(path) else {
        column_note(
            ui,
            col,
            true,
            "",
            Some(std::time::Instant::now() - Duration::from_secs(1)),
        );
        return;
    };
    if listing.entries.is_empty() {
        if let Some(err) = listing.error.as_ref().filter(|e| !e.is_empty()) {
            let (title, _) = pane::friendly_listing_error(err);
            ui.painter().text(
                egui::pos2(col.center().x, col.top() + 40.0),
                egui::Align2::CENTER_CENTER,
                title,
                theme::regular(text::SMALL),
                p.text_tertiary,
            );
        } else {
            column_note(
                ui,
                col,
                listing.loading,
                "",
                Some(std::time::Instant::now() - Duration::from_secs(1)),
            );
        }
        return;
    }
    let order: Vec<usize> = listing.ordered(sort, hidden).to_vec();
    let entries = &listing.entries;
    let footer_h = if next { 24.0 } else { 0.0 };
    let inner = egui::Rect::from_min_max(
        col.min + egui::vec2(0.0, space::XS),
        egui::pos2(col.right(), col.bottom() - footer_h),
    );
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(("col", side, tab_id, path.as_str()))
        .auto_shrink([false, false]);
    let hl_index = highlight.and_then(|h| order.iter().position(|&i| entries[i].name == h));
    if reveal {
        if let Some(i) = hl_index {
            scroll = scroll.vertical_scroll_offset(reveal_span(
                i as f32 * COL_ROW,
                COL_ROW,
                0.0,
                inner.height(),
            ));
        }
    }
    let mut clicked: Option<String> = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        scroll.show_rows(ui, COL_ROW, order.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            crate::a11y::list(ui, path.file_name(), order.len());
            for k in range {
                let (r, resp) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), COL_ROW), Sense::CLICK);
                let Some(e) = order.get(k).and_then(|&i| entries.get(i)) else {
                    continue;
                };
                crate::a11y::file_item(
                    &resp,
                    &crate::a11y::entry_label(e, chrono::Local::now(), None, None, None),
                    hl_index == Some(k),
                    k,
                    order.len(),
                );
                if e.is_dir() {
                    pane::note_folder_target(ui, side, r, &e.path);
                }
                let on = hl_index == Some(k);
                let fill = if on {
                    Some(p.selection_inactive)
                } else if resp.hovered() {
                    Some(p.hover)
                } else {
                    None
                };
                let t = tags
                    .and_then(|t| t.get(&e.name))
                    .map_or(&[][..], |v| v.as_slice());
                column_row(ui, r, e, row_icons, fill, false, None, None, t);
                if resp.clicked() {
                    clicked = Some(e.name.clone());
                }
            }
        });
    });
    if next {
        let f = egui::Rect::from_min_max(egui::pos2(col.left(), col.bottom() - footer_h), col.max);
        ui.painter().hline(
            f.x_range().shrink(space::SM),
            f.top(),
            Stroke::new(1.0, p.separator),
        );
        let count = order.len();
        let mut s = t!("view.items", count = count);
        if let Some(d) = &dir_size {
            s = format!("{s} · {}", pane::size_label(d));
        }
        ui.painter().text(
            f.center(),
            egui::Align2::CENTER_CENTER,
            s,
            theme::regular(text::CAPTION),
            p.text_secondary,
        );
    }
    if reveal {
        tab.view_state.cols.revealed.insert(path.clone());
    }
    if let Some(name) = clicked {
        actions.push(Action::ColumnsSelect(side, path.clone(), Some(name)));
    }
}

/// The last column: a preview of the selected file (thumbnail or icon,
/// name, kind, size, dates) or a summary of several selected items.
fn preview_column(ui: &mut egui::Ui, col: egui::Rect, info: &PaneInfo<'_>, tab: &Tab) {
    let p = pal(ui);
    let sum = tab.selection_summary();
    let w = (col.width() - 2.0 * space::XL).clamp(80.0, PREVIEW_W - 2.0 * space::XL);
    let x = col.left() + space::XL;
    let mut y = col.top() + space::XL + 4.0;
    let cx = x + w / 2.0;
    if sum.view_idx.len() > 1 {
        let r = egui::Rect::from_center_size(egui::pos2(cx, y + 50.0), egui::vec2(96.0, 96.0));
        widgets::paint_icon(ui, r, icons::COPY, 64.0, p.text_tertiary, false);
        y = r.bottom() + space::LG;
        let bytes = sum.size;
        ui.painter().text(
            egui::pos2(cx, y),
            egui::Align2::CENTER_TOP,
            t!("view.selected_n", count = sum.view_idx.len()),
            theme::medium(text::BODY),
            p.text,
        );
        ui.painter().text(
            egui::pos2(cx, y + 20.0),
            egui::Align2::CENTER_TOP,
            util::format_size(bytes),
            theme::regular(text::SMALL),
            p.text_secondary,
        );
        return;
    }
    let first = sum.view_idx.first().and_then(|&vi| tab.entry_at(vi));
    let Some(e) = first.or_else(|| tab.cursor_entry()) else {
        return;
    };
    let deco = info.decorations.get(&e.name);
    let c = colors(p, e, deco, false);
    let pr = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, w.min(col.height() * 0.45)));
    // Square for icons, the full width for wide pictures.
    let side_len = pr.width().min(pr.height());
    let sq = egui::Rect::from_center_size(pr.center(), egui::vec2(side_len, side_len));
    let pr = match thumb_of(ui, info, tab, e, w) {
        Thumb::Ready(tex) => {
            let r = fit(tex.size_vec2(), pr, false);
            let r = egui::Rect::from_min_size(egui::pos2(r.left(), pr.top()), r.size());
            paint_texture(ui, &tex, r, 4, c.dim);
            r
        }
        _ => {
            paint_entry_icon(ui, info, tab, e, sq, c.icon, c.dim, false);
            sq
        }
    };
    y = pr.bottom() + space::LG;
    let g = name_galley(ui, &e.name, w, p.text, 3);
    let g = {
        let mut job = (*g.job).clone();
        for s in &mut job.sections {
            s.format.font_id = theme::medium(text::BODY);
        }
        ui.painter().layout_job(job)
    };
    let gh = g.size().y;
    ui.painter().galley(egui::pos2(cx, y), g, p.text);
    y += gh + 2.0;
    let kind = icons::file_kind(&e.name, e.is_dir()).label();
    let size = if e.is_dir() {
        tab.dir_sizes
            .get(&e.name)
            .map(pane::size_label)
            .unwrap_or_default()
    } else {
        util::format_size(e.size)
    };
    let sub = if size.is_empty() {
        kind.to_string()
    } else {
        format!("{kind} – {size}")
    };
    ui.painter().text(
        egui::pos2(cx, y),
        egui::Align2::CENTER_TOP,
        sub,
        theme::regular(text::SMALL),
        p.text_secondary,
    );
    y += 26.0;
    let badges: &[manta_plugin::Badge] = deco.map_or(&[], |d| d.badges.as_slice());
    if !badges.is_empty() {
        let bw = plugin_ui::badges_width(ui, badges);
        plugin_ui::paint_badges(ui, cx - bw / 2.0, y + 6.0, badges, false, 1.0);
        y += 22.0;
    }
    ui.painter().hline(
        egui::Rangef::new(x, x + w),
        y,
        Stroke::new(1.0, p.separator),
    );
    y += space::MD;
    ui.painter().text(
        egui::pos2(x, y),
        egui::Align2::LEFT_TOP,
        t!("view.information"),
        theme::semibold(text::SMALL),
        p.text,
    );
    y += 20.0;
    let mut rows: Vec<(&str, String)> = Vec::new();
    rows.push((t!("view.info.kind"), kind.to_string()));
    if !size.is_empty() {
        let bytes = if e.is_dir() {
            tab.dir_sizes.get(&e.name).map(|s| s.bytes)
        } else {
            Some(e.size)
        };
        let full = match bytes {
            Some(b) if b >= 1000 => format!("{size} ({})", manta_plugin::i18n::format_bytes(b)),
            _ => size.clone(),
        };
        rows.push((t!("view.info.size"), full));
    }
    if let Some(t) = e.modified {
        rows.push((t!("view.info.modified"), format_date(t)));
    }
    if e.permissions.is_some() {
        rows.push((
            t!("view.info.permissions"),
            util::format_permissions(e.permissions, e.kind),
        ));
    }
    if let Some(t) = &e.link_target {
        rows.push((t!("view.info.original"), t.clone()));
    }
    for (k, v) in rows {
        ui.painter().text(
            egui::pos2(x, y),
            egui::Align2::LEFT_TOP,
            k,
            theme::regular(text::SMALL),
            p.text_secondary,
        );
        let mut job =
            egui::text::LayoutJob::simple_singleline(v, theme::regular(text::SMALL), p.text);
        job.wrap = egui::text::TextWrapping::truncate_at_width((w * 0.62).max(40.0));
        job.halign = egui::Align::RIGHT;
        let g = ui.painter().layout_job(job);
        ui.painter().galley(egui::pos2(x + w, y), g, p.text);
        y += 19.0;
    }
}

// ------------------------------------------------------------------ per-folder views

/// "Remember view per folder": the last view of each folder, saved in
/// `config_dir/Manta/folder-views.json` (the most recent [`Self::MAX`]).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderViews {
    /// (folder key, view), oldest first.
    #[serde(default)]
    pub items: VecDeque<(String, ViewMode)>,
}

impl FolderViews {
    pub const MAX: usize = 2000;

    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("Manta").join("folder-views.json"))
    }

    pub fn load(path: &Path) -> FolderViews {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        std::fs::write(
            path,
            serde_json::to_string(self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    }

    /// The key of a folder: its backend (empty for this Mac) and path.
    pub fn key(origin: &str, path: &VfsPath) -> String {
        format!("{origin}:{}", path.as_str().trim_end_matches('/'))
    }

    pub fn get(&self, key: &str) -> Option<ViewMode> {
        self.items
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| *v)
    }

    pub fn set(&mut self, key: String, view: ViewMode) {
        self.items.retain(|(k, _)| *k != key);
        self.items.push_back((key, view));
        while self.items.len() > Self::MAX {
            self.items.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use manta_core::EntryKind;

    fn entry(name: &str, dir: bool) -> Entry {
        Entry {
            name: name.into(),
            path: VfsPath::new(&format!("/d/{name}")),
            kind: if dir {
                EntryKind::Directory
            } else {
                EntryKind::File
            },
            size: 1,
            modified: None,
            permissions: None,
            link_target: None,
            hidden: false,
        }
    }

    fn tab(n: usize) -> Tab {
        let fs: SharedVfs = std::sync::Arc::new(manta_vfs::LocalFs::new());
        let mut t = Tab::new(1, pane::Location::new(fs, None, VfsPath::new("/d")));
        let names: Vec<Entry> = (0..n).map(|i| entry(&format!("f{i:02}"), i == 0)).collect();
        t.apply_listing(VfsPath::new("/d"), names);
        t
    }

    fn cursor(t: &Tab) -> Option<usize> {
        t.cursor
    }

    #[test]
    fn grid_geometry() {
        let g = Grid::new(500.0, 64.0);
        assert_eq!(g.cols, 4, "{g:?}");
        assert!(g.step_x >= g.cell_w);
        assert_eq!(g.rows(9), 3);
        let a = g.cell(0);
        let b = g.cell(5);
        assert!(
            (b.top() - a.top() - g.row_h).abs() < 0.01,
            "item 5 is on the second row"
        );
        assert!(g.icon_rect(1).left() > g.icon_rect(0).right());
        // Bigger icons: fewer columns; never zero.
        assert!(Grid::new(500.0, 256.0).cols < g.cols);
        assert_eq!(Grid::new(50.0, 256.0).cols, 1);
        // Snapped sizes.
        assert_eq!(clamp_icon_size(10.0), ICON_MIN);
        assert_eq!(clamp_icon_size(1000.0), ICON_MAX);
        assert_eq!(clamp_icon_size(67.0), 64.0);
        assert_eq!(clamp_icon_size(f32::NAN), ICON_DEFAULT);
    }

    #[test]
    fn rubber_band_hits() {
        let g = Grid::new(500.0, 64.0);
        let a = g.icon_rect(1).center();
        let b = g.icon_rect(6).center();
        let mut hits = g.hits(egui::Rect::from_two_pos(a, b), 20);
        hits.sort();
        assert_eq!(hits, vec![1, 2, 5, 6]);
        // Empty space between rows / outside the items: nothing.
        let gap = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(3.0, 3.0));
        assert!(g.hits(gap, 20).is_empty());
        assert!(g.hits(egui::Rect::from_two_pos(a, b), 0).is_empty());
    }

    #[test]
    fn reveal_keeps_or_scrolls() {
        assert_eq!(
            reveal_span(100.0, 20.0, 50.0, 200.0),
            50.0,
            "visible: stays"
        );
        assert_eq!(
            reveal_span(10.0, 20.0, 50.0, 200.0),
            10.0,
            "above: scroll up"
        );
        assert_eq!(
            reveal_span(300.0, 20.0, 50.0, 200.0),
            120.0,
            "below: scroll down"
        );
        let g = Grid::new(500.0, 64.0);
        let off = g.reveal(39, 0.0, 300.0);
        let top = (39 / g.cols) as f32 * g.row_h;
        assert!(top >= off && top + g.row_h <= off + 300.0 + 0.01);
    }

    #[test]
    fn keys_in_the_grid() {
        let mut t = tab(10);
        t.view_state.grid_cols = 4;
        t.view_state.page = 2;
        assert_eq!(
            nav(ViewMode::Icons, &mut t, Nav::Right, false),
            NavOut::Done
        );
        assert_eq!(cursor(&t), Some(0), "first key selects the first item");
        nav(ViewMode::Icons, &mut t, Nav::Down, false);
        assert_eq!(cursor(&t), Some(4));
        nav(ViewMode::Icons, &mut t, Nav::Right, false);
        assert_eq!(cursor(&t), Some(5));
        nav(ViewMode::Icons, &mut t, Nav::Down, true);
        assert_eq!(cursor(&t), Some(9));
        assert_eq!(t.selected.len(), 5, "⇧↓ selects the range 5…9");
        // Last row: ↓ stays.
        nav(ViewMode::Icons, &mut t, Nav::Down, false);
        assert_eq!(cursor(&t), Some(9));
        nav(ViewMode::Icons, &mut t, Nav::Up, false);
        assert_eq!(cursor(&t), Some(5));
        nav(ViewMode::Icons, &mut t, Nav::Left, false);
        assert_eq!(cursor(&t), Some(4));
        nav(ViewMode::Icons, &mut t, Nav::Home, false);
        assert_eq!(cursor(&t), Some(0));
        nav(ViewMode::Icons, &mut t, Nav::PageDown, false);
        assert_eq!(cursor(&t), Some(8), "a page = rows × columns");
        nav(ViewMode::Icons, &mut t, Nav::End, false);
        assert_eq!(cursor(&t), Some(9));
    }

    #[test]
    fn keys_in_gallery_columns_and_list() {
        let mut t = tab(5);
        nav(ViewMode::Gallery, &mut t, Nav::Right, false);
        nav(ViewMode::Gallery, &mut t, Nav::Right, false);
        assert_eq!(cursor(&t), Some(1));
        nav(ViewMode::Gallery, &mut t, Nav::Left, false);
        assert_eq!(cursor(&t), Some(0));
        nav(ViewMode::Gallery, &mut t, Nav::Down, false);
        assert_eq!(cursor(&t), Some(1), "↓ also steps in the filmstrip");

        // Columns: ↑ ↓ in the column, ← parent, → into a folder only.
        let mut t = tab(5);
        nav(ViewMode::Columns, &mut t, Nav::Down, false);
        assert_eq!(cursor(&t), Some(0));
        assert_eq!(
            nav(ViewMode::Columns, &mut t, Nav::Right, false),
            NavOut::Enter,
            "f00 is a folder"
        );
        nav(ViewMode::Columns, &mut t, Nav::Down, false);
        assert_eq!(
            nav(ViewMode::Columns, &mut t, Nav::Right, false),
            NavOut::Done,
            "a file: nothing to open"
        );
        assert_eq!(
            nav(ViewMode::Columns, &mut t, Nav::Left, false),
            NavOut::Parent
        );

        // The list leaves ← → alone.
        assert_eq!(
            nav(ViewMode::List, &mut t, Nav::Left, false),
            NavOut::Unused
        );
        nav(ViewMode::List, &mut t, Nav::Down, false);
        assert_eq!(cursor(&t), Some(2));
    }

    #[test]
    fn column_chain_from_root() {
        let root = VfsPath::new("/a");
        let chain = column_chain(&root, &VfsPath::new("/a/b/c"));
        assert_eq!(
            chain.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            vec!["/a", "/a/b", "/a/b/c"]
        );
        assert_eq!(column_chain(&root, &root).len(), 1);
        assert!(is_within(&VfsPath::new("/a/b"), &root));
        assert!(!is_within(&VfsPath::new("/ab"), &root));
        assert!(is_within(&VfsPath::new("/x"), &VfsPath::root()));
        let all = column_chain(&VfsPath::root(), &VfsPath::new("/a/b"));
        assert_eq!(all.len(), 3, "{all:?}");
    }

    #[test]
    fn root_follows_the_active_folder() {
        let mut t = tab(3);
        let mut s = ColumnsState::default();
        s.sync_root(&t);
        assert_eq!(s.root, Some(VfsPath::new("/d")));
        t.loc.path = VfsPath::new("/d/x/y");
        s.sync_root(&t);
        assert_eq!(s.root, Some(VfsPath::new("/d")), "deeper: the root stays");
        t.loc.path = VfsPath::new("/");
        s.sync_root(&t);
        assert_eq!(s.root, Some(VfsPath::root()), "above the root: it moves up");
    }

    #[test]
    fn folder_views_are_remembered() {
        let mut v = FolderViews::default();
        let k = FolderViews::key("", &VfsPath::new("/Users/me/Pictures/"));
        assert_eq!(k, ":/Users/me/Pictures");
        assert_eq!(v.get(&k), None);
        v.set(k.clone(), ViewMode::Icons);
        v.set(k.clone(), ViewMode::Gallery);
        assert_eq!(v.get(&k), Some(ViewMode::Gallery));
        assert_eq!(v.items.len(), 1);
        for i in 0..FolderViews::MAX + 5 {
            v.set(format!(":/f{i}"), ViewMode::Columns);
        }
        assert_eq!(v.items.len(), FolderViews::MAX);
        assert_eq!(v.get(&k), None, "the oldest are forgotten");
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("fv.json");
        v.save(&p).unwrap();
        assert_eq!(FolderViews::load(&p), v);
    }

    #[test]
    fn view_mode_serializes_as_words() {
        assert_eq!(
            serde_json::to_string(&ViewMode::Gallery).unwrap(),
            "\"gallery\""
        );
        assert_eq!(
            serde_json::from_str::<ViewMode>("\"icons\"").unwrap(),
            ViewMode::Icons
        );
        assert_eq!(ViewMode::default(), ViewMode::List);
    }
}
