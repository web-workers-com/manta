//! The macOS menu bar: adds "Check for Updates…" (after About, as usual on
//! macOS) and "Settings… ⌘," to the Manta menu (winit's default app menu has
//! neither), a Help menu with "Manta Manual ⌘?", "Report an Issue…",
//! "Suggest an Idea…", "Show Log File in Finder", "Manta Website & Support…",
//! "Source Code on GitHub…" and "Third-Party Licences", and shows the
//! standard items (About, Hide, Quit, Services) in the app's language.
//!
//! The menu items only set flags; the app checks them once per frame with
//! [`take_settings_request`] / [`take_update_request`] and acts like the
//! toolbar gear / the palette command.
//!
//! File, Edit, View, Go and Window are generated from the command registry
//! ([`command_menus`]): every command with a shortcut is listed with its
//! key equivalent, so the menu bar (and cheat-sheet apps reading it through
//! Accessibility) shows Manta's shortcuts. Those menus never *perform* a key
//! equivalent: a delegate answers `menuHasKeyEquivalent:…` with NO, so every
//! key still reaches egui exactly as before (text fields, type-to-select,
//! ⌘C / ⌘V). Clicking an item queues the command ([`take_commands`]); the
//! app runs it like the command palette.

use std::sync::atomic::{AtomicBool, Ordering};

static SETTINGS_REQUESTED: AtomicBool = AtomicBool::new(false);
static UPDATE_REQUESTED: AtomicBool = AtomicBool::new(false);
static HELP_REQUESTED: AtomicBool = AtomicBool::new(false);
static SUPPORT_REQUESTED: AtomicBool = AtomicBool::new(false);
static SOURCE_REQUESTED: AtomicBool = AtomicBool::new(false);
static LICENSES_REQUESTED: AtomicBool = AtomicBool::new(false);
static LOG_REQUESTED: AtomicBool = AtomicBool::new(false);
static VIEWER_TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);
/// "Manta as Default File Manager": 0 = not offered (not macOS), 1 = off,
/// 2 = on; +4 = can't switch right now (not in Applications, busy).
static VIEWER_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
/// Updates are set up in this build ("Check for Updates…" vs. "Updates Not Set Up").
static UPDATES_ENABLED: AtomicBool = AtomicBool::new(true);

/// True once after the user picked Manta → Settings… in the menu bar.
pub fn take_settings_request() -> bool {
    SETTINGS_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Manta → Check for Updates….
pub fn take_update_request() -> bool {
    UPDATE_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Help → Manta Manual (or pressed ⌘?
/// while AppKit handled the key equivalent).
pub fn take_help_request() -> bool {
    HELP_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Help → Manta Website & Support….
pub fn take_support_request() -> bool {
    SUPPORT_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Help → Source Code on GitHub….
pub fn take_source_request() -> bool {
    SOURCE_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Help → Third-Party Licences.
pub fn take_licenses_request() -> bool {
    LICENSES_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Help → Show Log File in Finder.
pub fn take_log_request() -> bool {
    LOG_REQUESTED.swap(false, Ordering::Relaxed)
}

/// True once after the user picked Manta → Manta as Default File Manager.
pub fn take_viewer_toggle_request() -> bool {
    VIEWER_TOGGLE_REQUESTED.swap(false, Ordering::Relaxed)
}

/// The default-file-manager item's state (call every frame; cheap):
/// `on` = `None` hides it (not supported here), `can` = it can switch now.
pub fn set_viewer_state(on: Option<bool>, can: bool) {
    let v = match on {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    } + if can { 0 } else { 4 };
    VIEWER_STATE.store(v, Ordering::Relaxed);
}

/// Title the update item for this build (call every frame; cheap).
pub fn set_updates_enabled(on: bool) {
    UPDATES_ENABLED.store(on, Ordering::Relaxed);
}

/// The menu titles in the active language.
struct Titles {
    about: String,
    check_updates: &'static str,
    settings: &'static str,
    services: &'static str,
    hide: String,
    hide_others: &'static str,
    show_all: &'static str,
    quit: String,
    quit_keep: &'static str,
    help: &'static str,
    manual: &'static str,
    support: &'static str,
    source: &'static str,
    report: &'static str,
    idea: &'static str,
    log: &'static str,
    licenses: &'static str,
    viewer: &'static str,
}

fn titles() -> Titles {
    Titles {
        about: t!("menu.about", app = "Manta"),
        check_updates: if UPDATES_ENABLED.load(Ordering::Relaxed) {
            t!("menu.check_updates")
        } else {
            t!("menu.updates_off")
        },
        settings: t!("cmd.settings"),
        services: t!("menu.services"),
        hide: t!("menu.hide", app = "Manta"),
        hide_others: t!("menu.hide_others"),
        show_all: t!("menu.show_all"),
        quit: t!("menu.quit", app = "Manta"),
        quit_keep: t!("menu.quit_keep"),
        help: t!("menu.help"),
        manual: t!("menu.manual"),
        support: t!("cmd.support"),
        source: t!("cmd.source_code"),
        report: t!("cmd.report_issue"),
        idea: t!("cmd.suggest_idea"),
        log: t!("palette.show_log"),
        licenses: t!("cmd.third_party_licenses"),
        viewer: t!("viewer.menu"),
    }
}

/// `MANTA_MENU_SELFTEST`: check the generated menus in the real menu bar
/// (macOS; elsewhere nothing to check).
#[cfg_attr(test, allow(dead_code))]
pub fn selftest() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        native::selftest()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

// ------------------------------------------------------------------ command menus

use crate::commands::Cmd;

/// One entry of a generated menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Cmd(Cmd),
    Separator,
}

/// A menu of the menu bar built from the command registry: its title and
/// entries. The items show the commands' shortcuts but never perform them
/// (egui handles the keys, see the module docs); a click runs the command.
pub struct MenuDef {
    pub title: &'static str,
    pub entries: Vec<Entry>,
}

/// File, Edit, View, Go and Window, like Finder's menu bar. Settings… (⌘,)
/// is in the Manta menu, the manual (⌘?), website, source code and licences in Help.
pub fn command_menus() -> Vec<MenuDef> {
    use Cmd::*;
    use Entry::Separator as S;
    let m = |title, cmds: &[Option<Cmd>]| MenuDef {
        title,
        entries: cmds.iter().map(|c| c.map_or(S, Entry::Cmd)).collect(),
    };
    let c = Some;
    vec![
        m(
            t!("menu.file"),
            &[
                c(NewTab),
                c(NewFolder),
                c(NewFile),
                c(Open),
                c(OpenWithApp),
                c(QuickLook),
                c(GetInfo),
                c(CloseTab),
                None,
                c(Rename),
                c(Compress),
                c(ExtractHere),
                None,
                c(CopyToOther),
                c(MoveToOther),
                None,
                c(AddToFavorites),
                None,
                c(MoveToTrash),
                c(DeletePermanently),
                c(EmptyTrash),
            ],
        ),
        m(
            t!("menu.edit"),
            &[
                c(Undo),
                c(Redo),
                None,
                c(SelectAll),
                None,
                c(Filter),
                c(EditPath),
            ],
        ),
        m(
            t!("menu.view"),
            &[
                c(ViewIcons),
                c(ViewList),
                c(ViewColumns),
                c(ViewGallery),
                None,
                c(ToggleHidden),
                c(Refresh),
                None,
                c(SplitPane),
                c(ClosePane),
                c(SwitchPane),
                c(FocusPaneLeft),
                c(FocusPaneRight),
                None,
                c(AppearanceLight),
                c(AppearanceDark),
                c(AppearanceSystem),
            ],
        ),
        m(
            t!("menu.go"),
            &[
                c(Back),
                c(Forward),
                c(EnclosingFolder),
                None,
                c(GoToPath),
                None,
                c(Connect),
                c(OpenShellInContainer),
            ],
        ),
        m(
            t!("menu.window"),
            &[
                c(Transfers),
                c(CompareSync),
                c(CommandPalette),
                None,
                c(ReloadPlugins),
                c(BrowsePlugins),
            ],
        ),
    ]
}

/// Commands that live in the Manta / Help menus instead (with real key
/// equivalents handled there).
#[cfg(test)]
pub const ELSEWHERE: &[Cmd] = &[
    Cmd::Settings,
    Cmd::Help,
    Cmd::Support,
    Cmd::SourceCode,
    Cmd::ReportIssue,
    Cmd::SuggestIdea,
    Cmd::ThirdPartyLicenses,
];

/// AppKit modifier flags (`NSEventModifierFlags`).
pub mod mods {
    pub const SHIFT: u64 = 1 << 17;
    pub const CONTROL: u64 = 1 << 18;
    pub const OPTION: u64 = 1 << 19;
    pub const COMMAND: u64 = 1 << 20;
}

/// The key equivalent of `cmd` for the menu item: (key string, modifier
/// mask), from its shortcut as shown in the app (`⇧⌘N`, `F2`, `⌥⌘←` …).
/// `None` for commands without a shortcut.
pub fn key_equivalent(cmd: Cmd) -> Option<(String, u64)> {
    let sc = cmd.shortcut();
    if sc.is_empty() {
        return None;
    }
    if cmd == Cmd::QuickLook {
        // "Space" / "Leertaste" (localised).
        return Some((" ".into(), 0));
    }
    let mut mask = 0;
    let mut rest = sc;
    loop {
        let mut chars = rest.chars();
        let m = match chars.next() {
            Some('⇧') => mods::SHIFT,
            Some('⌃') => mods::CONTROL,
            Some('⌥') => mods::OPTION,
            Some('⌘') => mods::COMMAND,
            _ => break,
        };
        mask |= m;
        rest = chars.as_str();
        if rest.is_empty() {
            return None;
        }
    }
    let key = match rest {
        "⌫" => "\u{8}".to_string(),
        "⌦" => "\u{7f}".to_string(),
        "↩" => "\r".to_string(),
        r if r == crate::icons::RETURN => "\r".to_string(),
        "⇥" => "\t".to_string(),
        "↑" => "\u{F700}".to_string(),
        "↓" => "\u{F701}".to_string(),
        "←" => "\u{F702}".to_string(),
        "→" => "\u{F703}".to_string(),
        f if f.len() >= 2
            && f.starts_with('F')
            && f[1..].parse::<u32>().is_ok_and(|n| (1..=35).contains(&n)) =>
        {
            let n: u32 = f[1..].parse().ok()?;
            char::from_u32(0xF704 + n - 1)?.to_string()
        }
        k if k.chars().count() == 1 => k.to_lowercase(),
        _ => return None,
    };
    Some((key, mask))
}

/// Menu item tag of a command (its place in [`crate::commands::ALL`];
/// the palette, which isn't listed there, comes after).
pub fn cmd_tag(cmd: Cmd) -> usize {
    crate::commands::ALL
        .iter()
        .position(|c| *c == cmd)
        .unwrap_or(crate::commands::ALL.len())
}

fn cmd_of_tag(tag: usize) -> Option<Cmd> {
    match crate::commands::ALL.get(tag) {
        Some(c) => Some(*c),
        None if tag == crate::commands::ALL.len() => Some(Cmd::CommandPalette),
        None => None,
    }
}

/// Commands picked in the menu bar, run by the app on its next frame.
static PICKED: std::sync::Mutex<Vec<Cmd>> = std::sync::Mutex::new(Vec::new());
/// Which commands can run now (bit = [`cmd_tag`]); all on until the app
/// says otherwise.
static ENABLED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);

/// Commands picked in the menu bar since the last call.
pub fn take_commands() -> Vec<Cmd> {
    PICKED
        .lock()
        .map(|mut v| std::mem::take(&mut *v))
        .unwrap_or_default()
}

fn pick(tag: usize) {
    if let (Some(cmd), Ok(mut v)) = (cmd_of_tag(tag), PICKED.lock()) {
        v.push(cmd);
    }
}

/// The app's view of which commands can run (greys out menu items).
pub fn set_enabled(f: impl Fn(Cmd) -> bool) {
    let mut mask = 0u64;
    for (i, c) in crate::commands::ALL
        .iter()
        .chain(std::iter::once(&Cmd::CommandPalette))
        .enumerate()
    {
        if f(*c) {
            mask |= 1 << i;
        }
    }
    ENABLED.store(mask, Ordering::Relaxed);
}

fn tag_enabled(tag: usize) -> bool {
    tag >= 64 || ENABLED.load(Ordering::Relaxed) & (1 << tag) != 0
}

/// Add the Settings item (first call) and (re)title the app menu when the
/// language changed since the last call. Call every frame on the main
/// thread; cheap when nothing changed. Does nothing off macOS or in tests.
pub fn sync(ctx: &eframe::egui::Context) {
    use std::sync::atomic::AtomicU32;
    static SYNCED: AtomicU32 = AtomicU32::new(u32::MAX);
    if cfg!(test) {
        return;
    }
    // Locale in the low byte (7 languages), then the other states.
    let l = crate::i18n::locale() as u32
        | (UPDATES_ENABLED.load(Ordering::Relaxed) as u32) << 8
        | (VIEWER_STATE.load(Ordering::Relaxed) as u32) << 9;
    if SYNCED.swap(l, Ordering::Relaxed) == l {
        return;
    }
    #[cfg(target_os = "macos")]
    native::sync(ctx, &titles());
    #[cfg(not(target_os = "macos"))]
    let _ = (ctx, titles);
}

#[cfg(target_os = "macos")]
mod native {
    use std::cell::RefCell;
    use std::sync::atomic::Ordering;
    use std::sync::OnceLock;

    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool, Imp, NSObject, NSObjectProtocol, ProtocolObject, Sel};
    use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSEvent, NSEventModifierFlags, NSMenu, NSMenuDelegate, NSMenuItem,
    };
    use objc2_foundation::{NSInteger, NSString};

    use super::{
        Titles, HELP_REQUESTED, LICENSES_REQUESTED, LOG_REQUESTED, SETTINGS_REQUESTED,
        SOURCE_REQUESTED, SUPPORT_REQUESTED, UPDATE_REQUESTED, VIEWER_STATE,
        VIEWER_TOGGLE_REQUESTED,
    };

    static CTX: OnceLock<eframe::egui::Context> = OnceLock::new();
    /// The self-test is sending key equivalents: clicks are ignored.
    static SELFTEST_RUNNING: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "MantaMenuTarget"]
        pub struct MenuTarget;

        impl MenuTarget {
            #[unsafe(method(mantaShowSettings:))]
            fn show_settings(&self, _sender: Option<&AnyObject>) {
                SETTINGS_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaShowHelp:))]
            fn show_help(&self, _sender: Option<&AnyObject>) {
                HELP_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaSupport:))]
            fn support(&self, _sender: Option<&AnyObject>) {
                SUPPORT_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaSourceCode:))]
            fn source_code(&self, _sender: Option<&AnyObject>) {
                SOURCE_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaShowLog:))]
            fn show_log(&self, _sender: Option<&AnyObject>) {
                LOG_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaShowLicenses:))]
            fn show_licenses(&self, _sender: Option<&AnyObject>) {
                LICENSES_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            /// Greys out "Manta as Default File Manager" while it can't
            /// switch (not in Applications, a switch running), and commands
            /// that can't run now.
            #[unsafe(method(validateMenuItem:))]
            fn validate_menu_item(&self, item: &NSMenuItem) -> Bool {
                if item.action() == Some(sel!(mantaToggleDefaultViewer:)) {
                    return Bool::new(VIEWER_STATE.load(Ordering::Relaxed) & 4 == 0);
                }
                if item.action() == Some(sel!(mantaRunCommand:))
                    || item.action() == Some(sel!(mantaDockCommand:))
                {
                    return Bool::new(super::tag_enabled(item.tag().max(0) as usize));
                }
                Bool::YES
            }

            /// A generated command item was clicked.
            #[unsafe(method(mantaRunCommand:))]
            fn run_command(&self, sender: Option<&NSMenuItem>) {
                if SELFTEST_RUNNING.load(Ordering::Relaxed) {
                    return;
                }
                if let Some(item) = sender {
                    super::pick(item.tag().max(0) as usize);
                }
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            /// An item of the Dock menu: like a menu bar command, and
            /// Manta comes to the front (the Dock doesn't do that itself).
            #[unsafe(method(mantaDockCommand:))]
            fn dock_command(&self, sender: Option<&NSMenuItem>) {
                if let Some(item) = sender {
                    super::pick(item.tag().max(0) as usize);
                }
                if let Some(mtm) = MainThreadMarker::new() {
                    let app = NSApplication::sharedApplication(mtm);
                    let _: () = unsafe { msg_send![&*app, activateIgnoringOtherApps: true] };
                }
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaToggleDefaultViewer:))]
            fn toggle_default_viewer(&self, _sender: Option<&AnyObject>) {
                VIEWER_TOGGLE_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }

            #[unsafe(method(mantaCheckForUpdates:))]
            fn check_for_updates(&self, _sender: Option<&AnyObject>) {
                UPDATE_REQUESTED.store(true, Ordering::Relaxed);
                if let Some(ctx) = CTX.get() {
                    ctx.request_repaint();
                }
            }
        }
    );

    define_class!(
        /// Delegate of the generated menus: they never perform a key
        /// equivalent, so every key goes to egui as before; the items only
        /// show the shortcuts.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "MantaCommandMenuDelegate"]
        pub struct CommandMenuDelegate;

        impl CommandMenuDelegate {
            #[unsafe(method(menuHasKeyEquivalent:forEvent:target:action:))]
            fn has_key_equivalent(
                &self,
                _menu: &NSMenu,
                _event: &NSEvent,
                _target: *mut *mut AnyObject,
                _action: *mut *const std::ffi::c_void,
            ) -> Bool {
                Bool::NO
            }
        }

        unsafe impl NSObjectProtocol for CommandMenuDelegate {}
        unsafe impl NSMenuDelegate for CommandMenuDelegate {}
    );

    define_class!(
        /// The generated menus: `performKeyEquivalent:` always says NO, so
        /// AppKit never fires their items from the keyboard (egui gets
        /// every key); the items still show their shortcuts.
        #[unsafe(super(NSMenu))]
        #[thread_kind = MainThreadOnly]
        #[name = "MantaCommandMenu"]
        pub struct CommandMenu;

        impl CommandMenu {
            #[unsafe(method(performKeyEquivalent:))]
            fn perform_key_equivalent(&self, _event: &NSEvent) -> Bool {
                Bool::NO
            }
        }
    );

    impl CommandMenu {
        fn new(mtm: MainThreadMarker, title: &str) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), initWithTitle: &*NSString::from_str(title)] }
        }
    }

    impl CommandMenuDelegate {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    impl MenuTarget {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    /// The Help menu's items (kept to retitle them when the language changes).
    struct HelpItems {
        top: Retained<NSMenuItem>,
        menu: Retained<NSMenu>,
        manual: Retained<NSMenuItem>,
        report: Retained<NSMenuItem>,
        idea: Retained<NSMenuItem>,
        log: Retained<NSMenuItem>,
        support: Retained<NSMenuItem>,
        source: Retained<NSMenuItem>,
        licenses: Retained<NSMenuItem>,
    }

    thread_local! {
        /// The menu item's target (menu items don't retain their target).
        static TARGET: RefCell<Option<Retained<MenuTarget>>> = const { RefCell::new(None) };
        static SETTINGS_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
        static UPDATE_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
        /// "Manta as Default File Manager" (checkmark when on).
        static VIEWER_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
        /// File, Edit, View, Go, Window (title items in the menu bar).
        static COMMAND_MENUS: RefCell<Vec<Retained<NSMenuItem>>> = const { RefCell::new(Vec::new()) };
        static DELEGATE: RefCell<Option<Retained<CommandMenuDelegate>>> = const { RefCell::new(None) };
        /// Language the command menus were built in.
        static COMMAND_MENUS_LOCALE: std::cell::Cell<u8> = const { std::cell::Cell::new(u8::MAX) };
        /// The Help menu's title item, the menu and its "Manta Manual" item.
        static HELP_ITEMS: RefCell<Option<HelpItems>> = const { RefCell::new(None) };
        /// The Dock menu and its "New Tab" item (retitled with the language).
        static DOCK_MENU: RefCell<Option<(Retained<NSMenu>, Retained<NSMenuItem>)>> =
            const { RefCell::new(None) };
    }

    fn app_menu(mtm: MainThreadMarker) -> Option<Retained<NSMenu>> {
        let app = NSApplication::sharedApplication(mtm);
        app.mainMenu()?.itemAtIndex(0)?.submenu()
    }

    /// The Dock icon's menu: "New Tab" (Finder has "New Finder Window";
    /// Manta has one window with tabs). AppKit asks the app delegate's
    /// `applicationDockMenu:`, which winit's delegate doesn't have: the
    /// method is added to its class.
    fn dock_menu(mtm: MainThreadMarker, target: &MenuTarget) {
        let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str(""));
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(crate::commands::Cmd::NewTab.label()),
                Some(sel!(mantaDockCommand:)),
                &NSString::from_str(""),
            )
        };
        unsafe { item.setTarget(Some(target)) };
        item.setTag(super::cmd_tag(super::Cmd::NewTab) as NSInteger);
        menu.addItem(&item);
        DOCK_MENU.with(|d| *d.borrow_mut() = Some((menu, item)));

        unsafe extern "C-unwind" fn application_dock_menu(
            _this: *mut AnyObject,
            _cmd: Sel,
            _app: *mut AnyObject,
        ) -> *mut NSMenu {
            DOCK_MENU.with(|d| {
                d.borrow().as_ref().map_or(std::ptr::null_mut(), |(m, _)| {
                    Retained::as_ptr(m).cast_mut()
                })
            })
        }
        let app = NSApplication::sharedApplication(mtm);
        let delegate: *mut AnyObject = unsafe { msg_send![&*app, delegate] };
        if delegate.is_null() {
            return;
        }
        // SAFETY: the signature matches "@@:@" (returns the menu, takes the
        // NSApplication); the menu lives in DOCK_MENU for the app's life.
        unsafe {
            let class = objc2::ffi::object_getClass(delegate).cast_mut();
            let imp: Imp = std::mem::transmute(
                application_dock_menu
                    as unsafe extern "C-unwind" fn(
                        *mut AnyObject,
                        Sel,
                        *mut AnyObject,
                    ) -> *mut NSMenu,
            );
            objc2::ffi::class_addMethod(class, sel!(applicationDockMenu:), imp, c"@@:@".as_ptr());
            // Set again so AppKit looks at what the delegate answers anew
            // (setting the same delegate does nothing).
            let none: *mut AnyObject = std::ptr::null_mut();
            let _: () = msg_send![&*app, setDelegate: none];
            let _: () = msg_send![&*app, setDelegate: delegate];
        }
    }

    /// Add the Help menu (last in the menu bar, as on macOS) with
    /// "Manta Manual ⌘?", ─, "Report an Issue…", "Suggest an Idea…",
    /// "Show Log File in Finder", ─, "Manta Website & Support…",
    /// "Source Code on GitHub…", ─, "Third-Party Licences".
    /// Registered as the app's help menu, so macOS adds
    /// its search field for menu items.
    fn help_menu(mtm: MainThreadMarker, target: &MenuTarget, titles: &Titles) {
        let app = NSApplication::sharedApplication(mtm);
        let Some(main) = app.mainMenu() else { return };
        let sub = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str(titles.help));
        // "?" as key equivalent: AppKit matches the typed character, so
        // this is ⇧⌘/ on US and ⇧⌘ß on German keyboards.
        let manual = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(titles.manual),
                Some(sel!(mantaShowHelp:)),
                &NSString::from_str("?"),
            )
        };
        unsafe { manual.setTarget(Some(target)) };
        sub.addItem(&manual);
        let item = |title: &str, action| {
            let it = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    &NSString::from_str(title),
                    Some(action),
                    &NSString::from_str(""),
                )
            };
            unsafe { it.setTarget(Some(target)) };
            it
        };
        // Report / idea run as commands (the palette has them too).
        let command = |title: &str, cmd: super::Cmd| {
            let it = item(title, sel!(mantaRunCommand:));
            it.setTag(super::cmd_tag(cmd) as NSInteger);
            it
        };
        sub.addItem(&NSMenuItem::separatorItem(mtm));
        let report = command(titles.report, super::Cmd::ReportIssue);
        sub.addItem(&report);
        let idea = command(titles.idea, super::Cmd::SuggestIdea);
        sub.addItem(&idea);
        let log = item(titles.log, sel!(mantaShowLog:));
        sub.addItem(&log);
        sub.addItem(&NSMenuItem::separatorItem(mtm));
        let support = item(titles.support, sel!(mantaSupport:));
        sub.addItem(&support);
        let source = item(titles.source, sel!(mantaSourceCode:));
        sub.addItem(&source);
        sub.addItem(&NSMenuItem::separatorItem(mtm));
        let licenses = item(titles.licenses, sel!(mantaShowLicenses:));
        sub.addItem(&licenses);
        let top = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(titles.help),
                None,
                &NSString::from_str(""),
            )
        };
        top.setSubmenu(Some(&sub));
        main.addItem(&top);
        app.setHelpMenu(Some(&sub));
        HELP_ITEMS.with(|h| {
            *h.borrow_mut() = Some(HelpItems {
                top,
                menu: sub,
                manual,
                report,
                idea,
                log,
                support,
                source,
                licenses,
            })
        });
    }

    /// (Re)build File, Edit, View, Go and Window from the command
    /// registry, after the Manta menu.
    fn command_menus(mtm: MainThreadMarker, target: &MenuTarget) {
        let app = NSApplication::sharedApplication(mtm);
        let Some(main) = app.mainMenu() else { return };
        COMMAND_MENUS.with(|m| {
            for it in m.borrow_mut().drain(..) {
                main.removeItem(&it);
            }
        });
        let delegate = DELEGATE.with(|d| {
            d.borrow_mut()
                .get_or_insert_with(|| CommandMenuDelegate::new(mtm))
                .clone()
        });
        let mut tops = Vec::new();
        for (i, def) in super::command_menus().iter().enumerate() {
            let sub = CommandMenu::new(mtm, def.title);
            sub.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            for e in &def.entries {
                let super::Entry::Cmd(c) = e else {
                    sub.addItem(&NSMenuItem::separatorItem(mtm));
                    continue;
                };
                let (key, mask) = super::key_equivalent(*c).unwrap_or_default();
                let it = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &NSString::from_str(c.label()),
                        Some(sel!(mantaRunCommand:)),
                        &NSString::from_str(&key),
                    )
                };
                it.setKeyEquivalentModifierMask(NSEventModifierFlags(mask as usize));
                it.setTag(super::cmd_tag(*c) as NSInteger);
                // Show the shortcut the app uses (⌘[), not AppKit's layout
                // translation (⌘ö on German keyboards); macOS 12+.
                if it.respondsToSelector(sel!(setAllowsAutomaticKeyEquivalentLocalization:)) {
                    it.setAllowsAutomaticKeyEquivalentLocalization(false);
                }
                unsafe { it.setTarget(Some(target)) };
                sub.addItem(&it);
            }
            let top = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    &NSString::from_str(def.title),
                    None,
                    &NSString::from_str(""),
                )
            };
            top.setSubmenu(Some(&sub));
            main.insertItem_atIndex(&top, (1 + i) as NSInteger);
            tops.push(top);
        }
        COMMAND_MENUS.with(|m| *m.borrow_mut() = tops);
    }

    /// `MANTA_MENU_SELFTEST`: the generated items are in the menu bar with
    /// their key equivalents, and the menu bar performs none of them (they
    /// go to egui). Returns the problems found (empty = fine).
    #[cfg_attr(test, allow(dead_code))]
    pub fn selftest() -> Vec<String> {
        let Some(mtm) = MainThreadMarker::new() else {
            return vec!["not on the main thread".into()];
        };
        let app = NSApplication::sharedApplication(mtm);
        let Some(main) = app.mainMenu() else {
            return vec!["no main menu".into()];
        };
        // Nothing a stray key equivalent fires may run while checking.
        SELFTEST_RUNNING.store(true, Ordering::Relaxed);
        let mut problems = Vec::new();
        let mut items: Vec<Retained<NSMenuItem>> = Vec::new();
        for i in 0..main.numberOfItems() {
            if let Some(sub) = main.itemAtIndex(i).and_then(|t| t.submenu()) {
                for j in 0..sub.numberOfItems() {
                    items.extend(
                        sub.itemAtIndex(j)
                            .filter(|it| it.action() == Some(sel!(mantaRunCommand:))),
                    );
                }
            }
        }
        for def in super::command_menus() {
            for e in def.entries {
                let super::Entry::Cmd(c) = e else { continue };
                let Some(it) = items
                    .iter()
                    .find(|it| it.tag() == super::cmd_tag(c) as NSInteger)
                else {
                    problems.push(format!("{c:?} missing"));
                    continue;
                };
                let Some((key, mask)) = super::key_equivalent(c) else {
                    continue;
                };
                let shown = (
                    it.keyEquivalent().to_string(),
                    it.keyEquivalentModifierMask().0 as u64,
                );
                if shown != (key.clone(), mask) {
                    problems.push(format!(
                        "{c:?}: key equivalent {shown:?}, expected {:?}",
                        (key.clone(), mask)
                    ));
                }
                // The menu bar must not take the key from egui.
                let ev = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
                    objc2_app_kit::NSEventType::KeyDown,
                    objc2_foundation::NSPoint::new(0.0, 0.0),
                    NSEventModifierFlags(mask as usize),
                    0.0,
                    0,
                    None,
                    &NSString::from_str(&key),
                    &NSString::from_str(&key),
                    false,
                    0,
                );
                if let Some(ev) = ev {
                    if main.performKeyEquivalent(&ev) {
                        problems.push(format!("{c:?}: the menu bar performed {}", c.shortcut()));
                    }
                }
            }
        }
        if items.is_empty() {
            problems.push("no generated items".into());
        }
        SELFTEST_RUNNING.store(false, Ordering::Relaxed);
        problems
    }

    pub fn sync(ctx: &eframe::egui::Context, titles: &Titles) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let _ = CTX.set(ctx.clone());
        let Some(menu) = app_menu(mtm) else { return };

        let existing = SETTINGS_ITEM.with(|s| s.borrow().clone());
        let item = match existing {
            Some(item) => item,
            None => {
                let target = MenuTarget::new(mtm);
                let item = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &NSString::from_str(titles.settings),
                        Some(sel!(mantaShowSettings:)),
                        &NSString::from_str(","),
                    )
                };
                unsafe { item.setTarget(Some(&target)) };
                // Standard place: About, ─, Settings…, ─, Services …
                let n = menu.numberOfItems();
                let about_at = (0..n).find(|&i| {
                    menu.itemAtIndex(i)
                        .and_then(|it| it.action())
                        .is_some_and(|a| a == sel!(orderFrontStandardAboutPanel:))
                });
                let at: NSInteger = match about_at {
                    Some(i) => i + 1,
                    None => 0,
                };
                // About, Check for Updates…, ─, Settings…, ─, Services …
                let update = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &NSString::from_str(titles.check_updates),
                        Some(sel!(mantaCheckForUpdates:)),
                        &NSString::from_str(""),
                    )
                };
                unsafe { update.setTarget(Some(&target)) };
                // …, Settings…, Manta as Default File Manager, ─, Services …
                let viewer = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &NSString::from_str(titles.viewer),
                        Some(sel!(mantaToggleDefaultViewer:)),
                        &NSString::from_str(""),
                    )
                };
                unsafe { viewer.setTarget(Some(&target)) };
                menu.insertItem_atIndex(&update, at);
                menu.insertItem_atIndex(&NSMenuItem::separatorItem(mtm), at + 1);
                menu.insertItem_atIndex(&item, at + 2);
                menu.insertItem_atIndex(&viewer, at + 3);
                VIEWER_ITEM.with(|s| *s.borrow_mut() = Some(viewer));
                help_menu(mtm, &target, titles);
                dock_menu(mtm, &target);
                TARGET.with(|t| *t.borrow_mut() = Some(target));
                SETTINGS_ITEM.with(|s| *s.borrow_mut() = Some(item.clone()));
                UPDATE_ITEM.with(|s| *s.borrow_mut() = Some(update));
                item
            }
        };
        item.setTitle(&NSString::from_str(titles.settings));
        let locale = crate::i18n::locale() as u8;
        if COMMAND_MENUS_LOCALE.with(|l| l.replace(locale)) != locale {
            if let Some(target) = TARGET.with(|t| t.borrow().clone()) {
                command_menus(mtm, &target);
            }
        }
        HELP_ITEMS.with(|h| {
            if let Some(h) = &*h.borrow() {
                h.top.setTitle(&NSString::from_str(titles.help));
                h.menu.setTitle(&NSString::from_str(titles.help));
                h.manual.setTitle(&NSString::from_str(titles.manual));
                h.report.setTitle(&NSString::from_str(titles.report));
                h.idea.setTitle(&NSString::from_str(titles.idea));
                h.log.setTitle(&NSString::from_str(titles.log));
                h.support.setTitle(&NSString::from_str(titles.support));
                h.source.setTitle(&NSString::from_str(titles.source));
                h.licenses.setTitle(&NSString::from_str(titles.licenses));
            }
        });
        DOCK_MENU.with(|d| {
            if let Some((_, item)) = &*d.borrow() {
                item.setTitle(&NSString::from_str(crate::commands::Cmd::NewTab.label()));
            }
        });
        if let Some(update) = UPDATE_ITEM.with(|s| s.borrow().clone()) {
            update.setTitle(&NSString::from_str(titles.check_updates));
        }
        if let Some(viewer) = VIEWER_ITEM.with(|s| s.borrow().clone()) {
            let st = VIEWER_STATE.load(Ordering::Relaxed);
            viewer.setTitle(&NSString::from_str(titles.viewer));
            viewer.setHidden(st & 3 == 0);
            // A checkmark when Manta has the role (NSControlStateValueOn).
            let state: NSInteger = if st & 3 == 2 { 1 } else { 0 };
            let _: () = unsafe { msg_send![&*viewer, setState: state] };
        }

        // The standard items winit created, found by their action.
        for i in 0..menu.numberOfItems() {
            let Some(it) = menu.itemAtIndex(i) else {
                continue;
            };
            let title: Option<&str> = match it.action() {
                Some(a) if a == sel!(orderFrontStandardAboutPanel:) => Some(&titles.about),
                Some(a) if a == sel!(hide:) => Some(&titles.hide),
                Some(a) if a == sel!(hideOtherApplications:) => Some(titles.hide_others),
                Some(a) if a == sel!(unhideAllApplications:) => Some(titles.show_all),
                Some(a) if a == sel!(terminate:) => Some(&titles.quit),
                _ if it.submenu().is_some() => Some(titles.services),
                // AppKit's hidden ⌥ alternate of Quit.
                _ if it.title().to_string() == "Quit and Keep Windows" => Some(titles.quit_keep),
                _ => None,
            };
            if let Some(t) = title {
                it.setTitle(&NSString::from_str(t));
                if let Some(sub) = it.submenu() {
                    sub.setTitle(&NSString::from_str(t));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ALL;

    /// Every command with a shortcut is in the generated menus (once),
    /// with a key equivalent that matches the shortcut shown in the app.
    #[test]
    fn menus_list_every_command_with_its_shortcut() {
        let menus = command_menus();
        let listed: Vec<Cmd> = menus
            .iter()
            .flat_map(|m| m.entries.iter())
            .filter_map(|e| match e {
                Entry::Cmd(c) => Some(*c),
                Entry::Separator => None,
            })
            .collect();
        for c in ALL.iter().chain(std::iter::once(&Cmd::CommandPalette)) {
            let n = listed.iter().filter(|l| *l == c).count();
            if ELSEWHERE.contains(c) {
                assert_eq!(n, 0, "{c:?} is in the Manta / Help menu");
                continue;
            }
            assert_eq!(n, 1, "{c:?} listed {n} times");
            assert_eq!(
                key_equivalent(*c).is_some(),
                !c.shortcut().is_empty(),
                "{c:?}: {}",
                c.shortcut()
            );
        }
        assert!(ALL.len() < 64, "the enabled mask has room");
        for m in &menus {
            assert!(!m.title.is_empty());
            assert!(
                m.entries.first() != Some(&Entry::Separator)
                    && m.entries.last() != Some(&Entry::Separator)
            );
        }
    }

    #[test]
    fn key_equivalents_from_shortcuts() {
        use mods::*;
        let k = |c| key_equivalent(c).unwrap();
        assert_eq!(k(Cmd::NewFolder), ("n".into(), SHIFT | COMMAND));
        assert_eq!(k(Cmd::NewFile), ("n".into(), OPTION | COMMAND));
        assert_eq!(k(Cmd::MoveToTrash), ("\u{8}".into(), COMMAND));
        assert_eq!(
            k(Cmd::DeletePermanently),
            ("\u{8}".into(), OPTION | COMMAND)
        );
        assert_eq!(k(Cmd::EmptyTrash), ("\u{8}".into(), SHIFT | COMMAND));
        assert_eq!(k(Cmd::Undo), ("z".into(), COMMAND));
        assert_eq!(k(Cmd::Redo), ("z".into(), SHIFT | COMMAND));
        assert_eq!(k(Cmd::AddToFavorites), ("t".into(), CONTROL | COMMAND));
        assert_eq!(k(Cmd::ToggleHidden), (".".into(), SHIFT | COMMAND));
        assert_eq!(k(Cmd::Rename), ("\u{F705}".into(), 0), "F2");
        assert_eq!(k(Cmd::CopyToOther), ("\u{F708}".into(), 0), "F5");
        assert_eq!(k(Cmd::FocusPaneLeft), ("\u{F702}".into(), OPTION | COMMAND));
        assert_eq!(k(Cmd::EnclosingFolder), ("\u{F700}".into(), COMMAND));
        assert_eq!(k(Cmd::Open), ("\r".into(), 0));
        assert_eq!(k(Cmd::OpenWithApp), ("\r".into(), OPTION));
        assert_eq!(k(Cmd::SwitchPane), ("\t".into(), 0));
        assert_eq!(k(Cmd::QuickLook), (" ".into(), 0));
        assert_eq!(k(Cmd::ViewColumns), ("3".into(), COMMAND));
        assert_eq!(k(Cmd::Back), ("[".into(), COMMAND));
        assert_eq!(key_equivalent(Cmd::Support), None);
        assert_eq!(key_equivalent(Cmd::SourceCode), None);
    }

    /// Menu item tags map back to their commands (the palette included).
    #[test]
    fn tags_round_trip() {
        for c in ALL.iter().chain(std::iter::once(&Cmd::CommandPalette)) {
            assert_eq!(cmd_of_tag(cmd_tag(*c)), Some(*c));
        }
        assert_eq!(cmd_of_tag(999), None);
    }
}
