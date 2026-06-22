// No console window for the release tray app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))]
compile_error!("funput-windows builds only on Windows (global keyboard hook + SendInput).");

mod commands;
mod hook;
mod inject;
mod keymap;
mod settings;
mod shell;
mod tray;
mod windows_ui;

use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
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
            // Touch the shell to load persisted settings + apply them to the engine.
            let settings = shell::snapshot();

            // Keep the OS autostart entry in sync with the saved preference.
            let mgr = app.autolaunch();
            let _ = if settings.launch_at_login {
                mgr.enable()
            } else {
                mgr.disable()
            };

            // Background engine: install the global keyboard hook on its own thread.
            hook::spawn();
            // Tray icon + menu (the always-available UI).
            tray::setup(app.handle())?;

            // First run: walk the user through setup.
            if !settings.has_completed_onboarding {
                windows_ui::open_onboarding(app.handle());
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build Funput")
        .run(|_app, event| {
            // Tray-only app: keep running even when no window is open.
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
