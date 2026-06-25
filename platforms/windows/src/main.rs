// No console window for the release tray app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// The shell exposes a broad state API; not every accessor is wired to UI yet.
#![allow(dead_code)]

#[cfg(not(windows))]
compile_error!("funput-windows builds only on Windows (global keyboard hook + SendInput).");

// Slint-generated components (SettingsWindow, OnboardingWindow, AppEntry).
slint::include_modules!();

mod commands;
mod hook;
mod inject;
mod keymap;
mod settings;
mod shell;
mod tray;
mod windows_ui;

fn main() {
    // Touch the shell to load persisted settings + apply them to the engine.
    let settings = shell::snapshot();

    // Keep the OS autostart entry in sync with the saved preference.
    commands::sync_autostart(settings.launch_at_login);

    // Background engine + tray: install the global keyboard hook (and the tray) on
    // their own thread with a Win32 message loop.
    hook::spawn();

    // First run: walk the user through setup. Created before the loop starts; it
    // shows once `run_event_loop_until_quit` begins pumping.
    if !settings.has_completed_onboarding {
        windows_ui::open_onboarding();
    }

    // Tray-only app: the loop keeps running with no window open, until the tray's
    // "Thoát" calls `quit_event_loop`.
    slint::run_event_loop_until_quit().expect("Slint event loop failed");
    std::process::exit(0);
}
