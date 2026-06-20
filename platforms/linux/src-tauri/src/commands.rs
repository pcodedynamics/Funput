//! Tauri commands the web UI (`platforms/ui/src/lib/api.ts`) calls. They mirror the
//! Windows shell's command surface, but here each one just reads/writes the shared
//! settings JSON — the Fcitx5 addon picks the change up on its next focus-in. The
//! command signatures (names + arg keys) must stay identical to the Windows ones so
//! the same UI works unchanged.

use crate::settings::{Hotkey, Method, Settings};

#[tauri::command]
pub fn get_settings() -> Settings {
    Settings::load()
}

#[tauri::command]
pub fn set_method(method: Method) {
    Settings::update(|s| s.method = method);
}

#[tauri::command]
pub fn set_enabled(on: bool) {
    Settings::update(|s| s.enabled = on);
}

#[tauri::command]
pub fn set_smart_restore(on: bool) {
    Settings::update(|s| s.smart_restore = on);
}

#[tauri::command]
pub fn set_eager_restore(on: bool) {
    Settings::update(|s| s.eager_restore = on);
}

#[tauri::command]
pub fn set_toggle_hotkey(hotkey: Hotkey) {
    Settings::update(|s| s.toggle_hotkey = hotkey);
}

#[tauri::command]
pub fn set_launch_at_login(on: bool) {
    // On Linux the IME runs inside the fcitx5 daemon, whose own autostart is
    // managed by the desktop session — not by this settings GUI (autostarting a
    // window at login would be wrong). So we only persist the preference.
    Settings::update(|s| s.launch_at_login = on);
}

#[tauri::command]
pub fn complete_onboarding() {
    Settings::update(|s| s.has_completed_onboarding = true);
}
