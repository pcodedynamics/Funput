//! Open the Settings / Onboarding windows on demand (tray-only app otherwise).
//! Slint windows are created lazily and kept alive (they are cheap), so reopening
//! just re-shows them. A Mica backdrop gives the native Windows 11 frosted look.
//!
//! Everything here runs on the main (Slint event-loop) thread. The tray, which
//! lives on the hook thread, reaches these via `slint::invoke_from_event_loop`.

use std::cell::RefCell;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::settings::{Hotkey, Method, ToneStyle};
use crate::{commands, shell, AppEntry, OnboardingWindow, SettingsWindow, ShortcutEntry};

thread_local! {
    static SETTINGS: RefCell<Option<SettingsWindow>> = const { RefCell::new(None) };
    static ONBOARDING: RefCell<Option<OnboardingWindow>> = const { RefCell::new(None) };
}

// --- Settings --------------------------------------------------------------

pub fn open_settings() {
    if let Some(win) = SETTINGS.with(|c| c.borrow().as_ref().map(|w| w.clone_strong())) {
        populate_settings(&win);
        let _ = win.show();
        return;
    }
    let win = SettingsWindow::new().expect("create settings window");
    populate_settings(&win);
    wire_settings(&win);
    let _ = win.show();
    SETTINGS.with(|c| *c.borrow_mut() = Some(win));
}

fn populate_settings(win: &SettingsWindow) {
    let s = shell::snapshot();
    win.set_method(s.method.id().into());
    win.set_tone_style(s.tone_style.id().into());
    win.set_hotkey(s.toggle_hotkey.id().into());
    win.set_hotkey_caps(caps_model(s.toggle_hotkey));
    win.set_smart_restore(s.smart_restore);
    win.set_eager_restore(s.eager_restore);
    win.set_launch_at_login(s.launch_at_login);
    win.set_version(env!("CARGO_PKG_VERSION").into());
    // Reset the updater UI each time the window is (re)shown.
    win.set_update_state("idle".into());
    win.set_update_version("".into());
    win.set_update_message("".into());
    refresh_apps(win);
    win.set_shortcuts(shortcuts_model(&shell::shortcuts()));
}

fn wire_settings(win: &SettingsWindow) {
    let w = win.as_weak();
    win.on_pick_method(move |v| {
        if let Some(m) = Method::from_id(&v) {
            commands::set_method(m);
        }
        if let Some(win) = w.upgrade() {
            win.set_method(v);
        }
    });

    let w = win.as_weak();
    win.on_pick_tone(move |v| {
        if let Some(t) = ToneStyle::from_id(&v) {
            commands::set_tone_style(t);
        }
        if let Some(win) = w.upgrade() {
            win.set_tone_style(v);
        }
    });

    let w = win.as_weak();
    win.on_pick_hotkey(move |v| {
        if let Some(h) = Hotkey::from_id(&v) {
            commands::set_toggle_hotkey(h);
            if let Some(win) = w.upgrade() {
                win.set_hotkey(v);
                win.set_hotkey_caps(caps_model(h));
            }
        }
    });

    // The Switch values are two-way bound, so the property is already updated; we
    // only need to persist (and apply the OS side effect for launch-at-login).
    win.on_set_smart(commands::set_smart_restore);
    win.on_set_eager(commands::set_eager_restore);
    win.on_set_launch(commands::set_launch_at_login);

    let w = win.as_weak();
    win.on_add_app(move |id| {
        if let Some(app) = shell::recent_apps().into_iter().find(|a| a.id == id.as_str()) {
            commands::add_excluded_app(app);
        }
        if let Some(win) = w.upgrade() {
            refresh_apps(&win);
        }
    });

    let w = win.as_weak();
    win.on_remove_app(move |id| {
        commands::remove_excluded_app(&id);
        if let Some(win) = w.upgrade() {
            refresh_apps(&win);
        }
    });

    let w = win.as_weak();
    win.on_add_shortcut(move || {
        commands::add_shortcut();
        if let Some(win) = w.upgrade() {
            win.set_shortcuts(shortcuts_model(&shell::shortcuts()));
        }
    });

    let w = win.as_weak();
    win.on_remove_shortcut(move |index| {
        commands::remove_shortcut(index.max(0) as usize);
        if let Some(win) = w.upgrade() {
            win.set_shortcuts(shortcuts_model(&shell::shortcuts()));
        }
    });

    // Editing a field only persists (engine + settings); the model is NOT rebuilt,
    // so the LineEdit keeps its text and caret while the user types.
    win.on_edit_trigger(|index, text| {
        commands::set_shortcut_trigger(index.max(0) as usize, text.to_string());
    });
    win.on_edit_expansion(|index, text| {
        commands::set_shortcut_expansion(index.max(0) as usize, text.to_string());
    });

    win.on_open_link(|url| commands::open_url(url.as_str()));

    // Auto-update: the buttons are argument-free; state flows back through
    // `set_update_state`. `commands` runs the work off the main thread.
    win.on_check_update(commands::check_for_updates);
    win.on_install_update(commands::install_update);
    win.on_relaunch_now(commands::relaunch_after_update);
}

/// Reflect an update step on the Settings window (no-op if it is not open).
/// Called on the main thread via `slint::invoke_from_event_loop`.
pub fn set_update_state(state: &str, version: &str, message: &str) {
    SETTINGS.with(|c| {
        if let Some(win) = c.borrow().as_ref() {
            win.set_update_state(state.into());
            win.set_update_version(version.into());
            win.set_update_message(message.into());
        }
    });
}

/// Open Settings on the "Giới thiệu" tab and immediately check for updates — the
/// tray's "Kiểm tra cập nhật…" entry point.
pub fn open_settings_and_check_updates() {
    open_settings();
    SETTINGS.with(|c| {
        if let Some(win) = c.borrow().as_ref() {
            win.set_active("about".into());
        }
    });
    commands::check_for_updates();
}

/// Refresh the excluded list and the "recent" picker (recent minus already-excluded).
fn refresh_apps(win: &SettingsWindow) {
    let excluded = shell::excluded_apps();
    let addable: Vec<_> = shell::recent_apps()
        .into_iter()
        .filter(|r| !excluded.iter().any(|e| e.id == r.id))
        .collect();
    win.set_excluded_apps(apps_model(&excluded));
    win.set_recent_apps(apps_model(&addable));
}

// --- Onboarding ------------------------------------------------------------

pub fn open_onboarding() {
    if let Some(win) = ONBOARDING.with(|c| c.borrow().as_ref().map(|w| w.clone_strong())) {
        win.set_step(0);
        let _ = win.show();
        return;
    }
    let win = OnboardingWindow::new().expect("create onboarding window");
    let s = shell::snapshot();
    win.set_method(s.method.id().into());
    win.set_launch_at_login(s.launch_at_login);

    let w = win.as_weak();
    win.on_pick_method(move |v| {
        if let Some(m) = Method::from_id(&v) {
            commands::set_method(m);
        }
        if let Some(win) = w.upgrade() {
            win.set_method(v);
        }
    });
    win.on_set_launch(commands::set_launch_at_login);

    let w = win.as_weak();
    win.on_finish(move || {
        commands::complete_onboarding();
        if let Some(win) = w.upgrade() {
            let _ = win.hide();
        }
    });

    let _ = win.show();
    ONBOARDING.with(|c| *c.borrow_mut() = Some(win));
}

// --- helpers ---------------------------------------------------------------

fn caps_model(hotkey: Hotkey) -> ModelRc<SharedString> {
    let caps: Vec<SharedString> = hotkey.caps().iter().map(|c| (*c).into()).collect();
    ModelRc::new(VecModel::from(caps))
}

fn apps_model(apps: &[crate::settings::ExcludedApp]) -> ModelRc<AppEntry> {
    let rows: Vec<AppEntry> = apps
        .iter()
        .map(|a| AppEntry {
            id: a.id.clone().into(),
            name: a.name.clone().into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

fn shortcuts_model(shortcuts: &[crate::settings::Shortcut]) -> ModelRc<ShortcutEntry> {
    let rows: Vec<ShortcutEntry> = shortcuts
        .iter()
        .map(|s| ShortcutEntry {
            trigger: s.trigger.clone().into(),
            expansion: s.expansion.clone().into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}
