//! Plain functions the UI callbacks (`windows_ui`) and the tray invoke. Each
//! mutates the shared `shell` state (which applies to the engine + persists); a
//! couple also apply an OS side effect (autostart registry, opening a link).

use crate::settings::{ExcludedApp, Hotkey, Method, ToneStyle};
use crate::shell;

pub fn set_method(method: Method) {
    shell::set_method(method.core());
}

pub fn set_tone_style(tone_style: ToneStyle) {
    shell::set_tone_style(tone_style.core());
}

pub fn set_smart_restore(on: bool) {
    shell::set_smart_restore(on);
}

pub fn set_eager_restore(on: bool) {
    shell::set_eager_restore(on);
}

pub fn set_toggle_hotkey(hotkey: Hotkey) {
    shell::set_toggle_hotkey(hotkey);
}

pub fn complete_onboarding() {
    shell::complete_onboarding();
}

pub fn add_excluded_app(app: ExcludedApp) {
    shell::add_excluded_app(app);
}

pub fn remove_excluded_app(id: &str) {
    shell::remove_excluded_app(id);
}

/// Persist the launch-at-login preference and mirror it into the OS autostart
/// (HKCU `…\Run`) via `auto-launch`.
pub fn set_launch_at_login(on: bool) {
    shell::set_launch_at_login(on);
    sync_autostart(on);
}

/// Bring the OS autostart entry in line with `on`. Called on startup (from the
/// persisted preference) and whenever the toggle changes.
pub fn sync_autostart(on: bool) {
    let Some(auto) = autolaunch() else { return };
    let _ = if on { auto.enable() } else { auto.disable() };
}

fn autolaunch() -> Option<auto_launch::AutoLaunch> {
    let exe = std::env::current_exe().ok()?;
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Funput")
        .set_app_path(&exe.to_string_lossy())
        .build()
        .ok()
}

/// Open an external link (GitHub / Website) in the system browser.
pub fn open_url(url: &str) {
    let _ = open::that(url);
}
