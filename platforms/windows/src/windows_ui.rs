//! Open the Settings / Onboarding windows on demand (tray-only app otherwise).
//! Slint windows are created lazily and kept alive (they are cheap), so reopening
//! just re-shows them. A Mica backdrop gives the native Windows 11 frosted look.
//!
//! Everything here runs on the main (Slint event-loop) thread. The tray, which
//! lives on the hook thread, reaches these via `slint::invoke_from_event_loop`.

use std::cell::RefCell;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::settings::{Hotkey, Method, ToneStyle};
use crate::{commands, shell, AppEntry, OnboardingWindow, SettingsWindow};

thread_local! {
    static SETTINGS: RefCell<Option<SettingsWindow>> = const { RefCell::new(None) };
    static ONBOARDING: RefCell<Option<OnboardingWindow>> = const { RefCell::new(None) };
}

// --- Settings --------------------------------------------------------------

pub fn open_settings() {
    if let Some(win) = SETTINGS.with(|c| c.borrow().as_ref().map(|w| w.clone_strong())) {
        populate_settings(&win);
        let _ = win.show();
        apply_backdrop(win.window());
        return;
    }
    let win = SettingsWindow::new().expect("create settings window");
    populate_settings(&win);
    wire_settings(&win);
    let _ = win.show();
    apply_backdrop(win.window());
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
    refresh_apps(win);
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

    win.on_open_link(|url| commands::open_url(url.as_str()));
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
        apply_backdrop(win.window());
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
    apply_backdrop(win.window());
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

/// Apply the Win11 Mica backdrop, matching the system light/dark setting. The
/// window must already be shown (the HWND has to exist).
fn apply_backdrop(window: &slint::Window) {
    #[cfg(windows)]
    {
        let _ = window_vibrancy::apply_mica(window.window_handle(), Some(is_dark_mode()));
    }
    #[cfg(not(windows))]
    let _ = window;
}

/// Read the system "apps" theme: `AppsUseLightTheme == 0` means dark mode.
#[cfg(windows)]
fn is_dark_mode() -> bool {
    use windows::core::w;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

    let mut data: u32 = 1;
    let mut size: u32 = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut core::ffi::c_void),
            Some(&mut size),
        )
    };
    status == ERROR_SUCCESS && data == 0
}
