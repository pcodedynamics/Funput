// Funput Linux — Settings & Onboarding window. Typing is handled by the Fcitx5
// addon (platforms/linux/fcitx5); this binary only edits the shared settings file.
// No tray: VI/EN toggling and the status icon are provided by Fcitx5 itself.

#[cfg(not(target_os = "linux"))]
compile_error!("funput-linux builds only on Linux (the typing engine ships as a Fcitx5 addon).");

mod commands;
mod settings;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::settings::Settings;

/// Open (or focus) the Settings or Onboarding window. Mirrors the Windows shell's
/// `windows_ui::open`, minus the Acrylic vibrancy (Linux has no equivalent).
fn open(app: &AppHandle, label: &str, title: &str, w: f64, h: f64) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.set_focus();
        return;
    }
    // Opaque window: Linux has no Acrylic/Mica material, so the UI paints its own
    // solid background (see tokens.css `[data-platform="linux"]`). `platform=linux`
    // tells the shared UI which styling + copy to use.
    let url = WebviewUrl::App(format!("index.html?view={label}&platform=linux").into());
    let _ = WebviewWindowBuilder::new(app, label, url)
        .title(title)
        .inner_size(w, h)
        .resizable(false)
        .transparent(false)
        .center()
        .build();
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::set_method,
            commands::set_tone_style,
            commands::set_enabled,
            commands::set_smart_restore,
            commands::set_eager_restore,
            commands::set_toggle_hotkey,
            commands::set_launch_at_login,
            commands::complete_onboarding,
            commands::open_url,
            commands::get_excluded_apps,
            commands::add_excluded_app,
            commands::remove_excluded_app,
            commands::list_recent_apps,
        ])
        .setup(|app| {
            let settings = Settings::load();
            // First run walks the user through setup; afterwards it opens Settings.
            if settings.has_completed_onboarding {
                open(app.handle(), "settings", "Funput — Cài đặt", 720.0, 480.0);
            } else {
                open(app.handle(), "onboarding", "Chào mừng đến Funput", 460.0, 540.0);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Funput");
}
