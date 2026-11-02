//! Software updates in the real app (fake engine): the toast after the
//! launch check, the status bar hint, the menu / palette command, the
//! update window's buttons, Settings → General → Updates. Plus renders of
//! every window state in German and English, light and dark. Child of
//! `ux_tests`.

use eframe::egui::{self, Theme};
use egui_kittest::kittest::Queryable;
use manta_updater::{LocationProblem, UpdateError, UpdateState, UpdateStatus};

use super::*;
use crate::i18n::{self, Locale};
use crate::settings::{SettingsOutcome, SettingsTab, SettingsView};
use crate::updates::tests::{release, v, Fake};
use crate::updates::{Disabled, FailKind, Failure, Op, Updates, View};

const LOCALES: [(Locale, &str); 2] = [(Locale::De, "de"), (Locale::En, "en")];
const THEMES: [(Theme, &str); 2] = [(Theme::Light, "light"), (Theme::Dark, "dark")];

pub(super) fn fake_updates(state: UpdateState) -> (Updates, Fake) {
    let fake = Fake::default();
    (
        Updates::new(Some(Box::new(fake.clone())), None, state, v("0.1.0")),
        fake,
    )
}

/// The app with a fake engine whose launch check finds 0.2.0.
fn app_with_update(fx: &Fixture, theme: Theme, gpu: bool) -> (H, Fake) {
    let mut h = harness_with(fx, WIDE, theme, gpu);
    let (u, fake) = fake_updates(UpdateState::default());
    app(&mut h).updates = u;
    h.run_steps(2);
    assert_eq!(fake.calls(), ["check()"], "checked at launch");
    fake.push(UpdateStatus::Available(release("0.2.0")));
    h.run_steps(3);
    (h, fake)
}

#[test]
fn launch_check_shows_a_toast_and_a_status_hint_that_open_the_window() {
    let fx = fixture();
    let (mut h, _fake) = app_with_update(&fx, Theme::Light, false);
    i18n::set_locale(Locale::En);
    h.run_steps(2);
    assert!(app(&mut h)
        .toasts
        .titles()
        .contains(&"Update available: 0.2.0".to_string()));
    assert!(!app(&mut h).updates.open, "no window over the user's work");
    h.get_by_label("Show").click();
    h.run_steps(3);
    assert!(
        app(&mut h).updates.open,
        "the toast's Show opens the window"
    );
    assert_eq!(app(&mut h).updates.view, View::Available);
    h.get_by_label("Later").click();
    h.run_steps(3);
    assert!(!app(&mut h).updates.open);
    assert!(
        app(&mut h).updates.notice.is_none(),
        "Later also hides the hint"
    );

    // Status bar hint (as long as the toast is gone).
    let (mut h, _fake) = app_with_update(&fx, Theme::Light, false);
    i18n::set_locale(Locale::En);
    app(&mut h).toasts.clear();
    h.run_steps(2);
    h.get_by_label("Update 0.2.0").click();
    h.run_steps(3);
    assert!(app(&mut h).updates.open);
    h.get_by_label("Skip This Version").click();
    h.run_steps(3);
    assert!(!app(&mut h).updates.open);
    assert_eq!(app(&mut h).updates.state.skipped_version, Some(v("0.2.0")));
}

#[test]
fn install_button_downloads_and_cancel_stops() {
    let fx = fixture();
    let (mut h, fake) = app_with_update(&fx, Theme::Light, false);
    i18n::set_locale(Locale::En);
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, super::super::Action::ShowUpdate);
    h.run_steps(3);
    h.get_by_label("Install and Relaunch").click();
    h.run_steps(3);
    assert_eq!(fake.calls().last().unwrap(), "download(0.2.0)");
    fake.push(UpdateStatus::Downloading {
        done: 5_000_000,
        total: 21_734_521,
    });
    h.run_steps(3);
    assert!(
        h.query_by_label("5.0 MB of 21.7 MB").is_some(),
        "MB done / total"
    );
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert_eq!(fake.calls().last().unwrap(), "cancel");
    fake.push(UpdateStatus::Idle);
    h.run_steps(3);
    assert_eq!(app(&mut h).updates.view, View::Available);
}

#[test]
fn check_for_updates_when_disabled_opens_settings() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    i18n::set_locale(Locale::En);
    h.run_steps(2);
    assert!(!app(&mut h).updates.enabled(), "tests build without a key");
    // The palette lists the command under its disabled title.
    let entries = app(&mut h).palette_entries();
    let e = entries
        .iter()
        .find(|e| e.id == crate::palette::PaletteId::CheckForUpdates)
        .expect("palette entry");
    assert_eq!(e.label, "Updates Not Set Up");
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, super::super::Action::CheckForUpdates);
    h.run_steps(3);
    assert!(!app(&mut h).updates.open, "nothing to check");
    assert!(app(&mut h).settings.open, "Settings explains why");
    assert_eq!(app(&mut h).settings.tab, SettingsTab::General);
    assert!(h
        .query_by_label("Updates aren’t set up in this build")
        .is_some());
}

#[test]
fn manual_check_shows_up_to_date_and_settings_controls_work() {
    let fx = fixture();
    let mut h = harness_with(&fx, WIDE, Theme::Light, false);
    let (u, fake) = fake_updates(UpdateState {
        auto_check: false,
        ..Default::default()
    });
    app(&mut h).updates = u;
    i18n::set_locale(Locale::En);
    h.run_steps(2);
    assert!(fake.calls().is_empty(), "launch check switched off");
    let ctx = h.ctx.clone();
    app(&mut h).handle_action(&ctx, super::super::Action::CheckForUpdates);
    h.run_steps(2);
    fake.push(UpdateStatus::UpToDate);
    h.run_steps(3);
    assert!(h.query_by_label("Manta is up to date").is_some());
    h.get_by_label("OK").click();
    h.run_steps(2);
    assert!(!app(&mut h).updates.open);

    app(&mut h).settings.show(Some(SettingsTab::General));
    app(&mut h).settings.reveal_updates = true;
    h.run_steps(4);
    h.get_by_label("Beta").click();
    h.run_steps(2);
    assert_eq!(
        app(&mut h).updates.state.channel,
        manta_updater::Channel::Beta
    );
    assert!(fake.calls().contains(&"channel(beta)".to_string()));
    h.get_by_label("Check Now").click();
    h.run_steps(2);
    assert!(
        app(&mut h).updates.open,
        "Check Now shows the result in the window"
    );
}

// ------------------------------------------------------------------ renders

fn out(name: &str) -> std::path::PathBuf {
    out_dir().join(format!("{name}.png"))
}

pub(super) fn content_harness(
    theme: Theme,
    size: [f32; 2],
    u: Updates,
) -> Harness<'static, Updates> {
    Harness::builder()
        .with_size(egui::vec2(size[0], size[1]))
        .with_pixels_per_point(2.0)
        .with_theme(theme)
        .allow_missing_glyphs()
        .with_accessibility_check(false)
        .with_context_setup(|ctx| {
            ctx.add_font_provider(std::sync::Arc::new(
                egui_system_font_provider::SystemFontProvider::new(),
            ));
            crate::theme::install(ctx);
            egui_extras::install_image_loaders(ctx);
        })
        .wgpu()
        .build_ui_state(
            |ui, u: &mut Updates| {
                let p = crate::theme::pal(ui);
                ui.painter()
                    .rect_filled(ui.ctx().content_rect(), 0.0, p.elevated);
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(p.elevated))
                    .show(ui, |ui| crate::update_window::contents(ui, u));
            },
            u,
        )
}

fn render_state(name: &str, view: View, rel: bool, blocker: Option<LocationProblem>) {
    for (locale, lname) in LOCALES {
        for (theme, tname) in THEMES {
            let (mut u, _) = fake_updates(UpdateState::default());
            u.show_state(view.clone(), rel.then(|| release("0.2.0")));
            u.set_blocker(blocker.clone());
            let mut h = content_harness(theme, crate::update_window::SIZE, u);
            i18n::set_locale(locale);
            // Image loading (the app icon) needs a few frames.
            for _ in 0..6 {
                h.run_steps(2);
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
            let img = h.render().expect("render");
            let path = out(&format!("update_{name}_{lname}_{tname}"));
            img.save(&path).expect("save png");
            eprintln!("ux_tests: wrote {}", path.display());
            i18n::set_locale(Locale::En);
        }
    }
}

#[test]
fn render_update_window_states() {
    require_gpu!();
    render_state("available", View::Available, true, None);
    render_state(
        "downloading",
        View::Downloading {
            done: 12_300_000,
            total: 21_734_521,
        },
        true,
        None,
    );
    render_state("ready", View::Ready, true, None);
    render_state("installing", View::Installing, true, None);
    render_state(
        "error_download",
        View::Failed(Failure {
            during: Op::Download,
            kind: FailKind::Offline,
            detail: UpdateError::Network("timed out after 30 s".into()).to_string(),
        }),
        true,
        None,
    );
    render_state(
        "error_check",
        View::Failed(Failure {
            during: Op::Check,
            kind: FailKind::Server(503),
            detail: UpdateError::Http {
                status: 503,
                url: "https://manta-filemanager.com/updates/update.json".into(),
            }
            .to_string(),
        }),
        false,
        None,
    );
    render_state(
        "blocker_dmg",
        View::Available,
        true,
        Some(LocationProblem::ReadOnlyVolume),
    );
    render_state("up_to_date", View::UpToDate, false, None);
    render_state("checking", View::Checking, false, None);
}

/// Issue #10: Close and Try Again sit centred under the message.
#[test]
fn failed_check_buttons_are_centred() {
    for (locale, _) in LOCALES {
        let (mut u, _) = fake_updates(UpdateState::default());
        u.show_state(
            View::Failed(Failure {
                during: Op::Check,
                kind: FailKind::Offline,
                detail: UpdateError::Network("offline".into()).to_string(),
            }),
            None,
        );
        i18n::set_locale(locale);
        let size = crate::update_window::SIZE;
        let mut h = Harness::builder()
            .with_size(egui::vec2(size[0], size[1]))
            .with_accessibility_check(false)
            .with_context_setup(crate::theme::install)
            .build_ui_state(
                |ui, u: &mut Updates| {
                    egui::CentralPanel::default()
                        .show(ui, |ui| crate::update_window::contents(ui, u));
                },
                u,
            );
        h.run_steps(2);
        let left = h.get_by_label(t!("update.close")).rect();
        let right = h.get_by_label(t!("update.retry")).rect();
        let centre = (left.min.x + right.max.x) / 2.0;
        assert!(
            (centre - size[0] / 2.0).abs() < 2.0,
            "{locale:?}: buttons span {}..{} in a {} wide window",
            left.min.x,
            right.max.x,
            size[0]
        );
        i18n::set_locale(Locale::En);
    }
}

#[test]
fn render_update_settings() {
    require_gpu!();
    for (locale, lname) in LOCALES {
        for (theme, tname) in THEMES {
            for (variant, disabled) in [("enabled", false), ("disabled", true)] {
                struct State {
                    view: SettingsView,
                    host: Option<PluginHost>,
                    prefs: crate::prefs::Prefs,
                }
                let mut view = SettingsView::default();
                view.show(Some(SettingsTab::General));
                view.reveal_updates = true;
                let (mut u, _) = fake_updates(UpdateState {
                    last_check: Some(chrono::Utc::now() - chrono::Duration::minutes(20)),
                    skipped_version: Some(v("0.1.5")),
                    ..Default::default()
                });
                if disabled {
                    u = Updates::new(
                        None,
                        Some(Disabled::NoKey),
                        UpdateState::default(),
                        v("0.1.0"),
                    );
                }
                view.updates = Some(u.settings_info());
                let mut h = Harness::builder()
                    .with_size(egui::vec2(
                        crate::settings::SIZE[0],
                        crate::settings::SIZE[1],
                    ))
                    .with_pixels_per_point(2.0)
                    .with_theme(theme)
                    .allow_missing_glyphs()
                    .with_accessibility_check(false)
                    .with_context_setup(|ctx| {
                        ctx.add_font_provider(std::sync::Arc::new(
                            egui_system_font_provider::SystemFontProvider::new(),
                        ));
                        crate::theme::install(ctx);
                    })
                    .wgpu()
                    .build_ui_state(
                        |ui, s: &mut State| {
                            let host = s
                                .host
                                .get_or_insert_with(|| plugin_host(ui.ctx(), Vec::new()));
                            let p = crate::theme::pal(ui);
                            ui.painter()
                                .rect_filled(ui.ctx().content_rect(), 0.0, p.elevated);
                            egui::CentralPanel::default()
                                .frame(egui::Frame::new().fill(p.elevated))
                                .show(ui, |ui| {
                                    let mut out = SettingsOutcome::default();
                                    s.view.contents(ui, host, &mut s.prefs, &mut out);
                                });
                        },
                        State {
                            view,
                            host: None,
                            prefs: Default::default(),
                        },
                    );
                i18n::set_locale(locale);
                h.run_steps(4);
                h.state_mut().view.reveal_updates = true;
                h.run_steps(4);
                let img = h.render().expect("render");
                let path = out(&format!("update_settings_{variant}_{lname}_{tname}"));
                img.save(&path).expect("save png");
                eprintln!("ux_tests: wrote {}", path.display());
                i18n::set_locale(Locale::En);
            }
        }
    }
}

#[test]
fn render_update_toast() {
    require_gpu!();
    let fx = fixture();
    for (locale, lname) in LOCALES {
        for (theme, tname) in THEMES {
            let mut h = harness(&fx, WIDE, theme);
            i18n::set_locale(locale);
            let (u, fake) = fake_updates(UpdateState::default());
            app(&mut h).updates = u;
            h.run_steps(2);
            fake.push(UpdateStatus::Available(release("0.2.0")));
            h.run_steps(4);
            save(&mut h, &format!("update_toast_{lname}_{tname}"));
            i18n::set_locale(Locale::En);
        }
    }
}
