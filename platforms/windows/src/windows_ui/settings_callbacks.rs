//! Settings callbacks and live model synchronization.

use std::cell::RefCell;

use slint::{ComponentHandle, Model};

use super::{models, settings_window};
use crate::compose::FieldComposer;
use crate::settings::{Hotkey, Method, ToneStyle};
use crate::{commands, shell, Compose, SettingsWindow};

thread_local! {
    /// Vietnamese composer for the gõ tắt expansion field (UI thread only).
    static COMPOSER: RefCell<FieldComposer> = RefCell::new(FieldComposer::new());
}

pub(super) fn wire(window: &SettingsWindow) {
    let weak = window.as_weak();
    window.on_pick_method(move |value| {
        if let Some(method) = Method::from_id(&value) {
            commands::set_method(method);
        }
        if let Some(window) = weak.upgrade() {
            window.set_method(value);
        }
    });

    let weak = window.as_weak();
    window.on_pick_tone(move |value| {
        if let Some(tone) = ToneStyle::from_id(&value) {
            commands::set_tone_style(tone);
        }
        if let Some(window) = weak.upgrade() {
            window.set_tone_style(value);
        }
    });

    let weak = window.as_weak();
    window.on_pick_hotkey(move |value| {
        if let Some(hotkey) = Hotkey::from_id(&value) {
            commands::set_toggle_hotkey(hotkey);
            if let Some(window) = weak.upgrade() {
                window.set_hotkey(value);
                window.set_hotkey_caps(models::caps(hotkey));
            }
        }
    });

    window.on_set_smart(commands::set_smart_restore);
    window.on_set_eager(commands::set_eager_restore);
    window.on_set_spell(commands::set_spell_check);
    window.on_set_auto_cap(commands::set_auto_capitalize);
    window.on_set_launch(commands::set_launch_at_login);

    wire_apps(window);
    wire_shortcuts(window);
    wire_composer(window);

    window.on_open_link(|url| commands::open_url(url.as_str()));
    window.on_check_update(commands::check_for_updates);
    window.on_install_update(commands::install_update);
    window.on_relaunch_now(commands::relaunch_after_update);
}

fn wire_apps(window: &SettingsWindow) {
    let weak = window.as_weak();
    window.on_add_app(move |id| {
        if let Some(app) = shell::recent_apps()
            .into_iter()
            .find(|app| app.id == id.as_str())
        {
            commands::add_excluded_app(app);
        }
        if let Some(window) = weak.upgrade() {
            settings_window::refresh_apps(&window);
        }
    });

    let weak = window.as_weak();
    window.on_remove_app(move |id| {
        commands::remove_excluded_app(&id);
        if let Some(window) = weak.upgrade() {
            settings_window::refresh_apps(&window);
        }
    });
}

fn wire_shortcuts(window: &SettingsWindow) {
    let weak = window.as_weak();
    window.on_add_shortcut(move || {
        commands::add_shortcut();
        if let Some(window) = weak.upgrade() {
            window.set_shortcuts(models::shortcuts(&shell::shortcuts()));
        }
    });

    let weak = window.as_weak();
    window.on_remove_shortcut(move |index| {
        commands::remove_shortcut(index.max(0) as usize);
        if let Some(window) = weak.upgrade() {
            window.set_shortcuts(models::shortcuts(&shell::shortcuts()));
        }
    });

    let weak = window.as_weak();
    window.on_edit_trigger(move |index, text| {
        let index = index.max(0) as usize;
        commands::set_shortcut_trigger(index, text.to_string());
        if let Some(window) = weak.upgrade() {
            let model = window.get_shortcuts();
            if let Some(mut entry) = model.row_data(index) {
                entry.trigger = text;
                model.set_row_data(index, entry);
            }
        }
    });

    let weak = window.as_weak();
    window.on_edit_expansion(move |index, text| {
        let index = index.max(0) as usize;
        commands::set_shortcut_expansion(index, text.to_string());
        if let Some(window) = weak.upgrade() {
            let model = window.get_shortcuts();
            if let Some(mut entry) = model.row_data(index) {
                entry.expansion = text;
                model.set_row_data(index, entry);
            }
        }
    });
}

fn wire_composer(window: &SettingsWindow) {
    let compose = window.global::<Compose>();
    compose.on_reset(|text| {
        let (method, tone) = shell::method_and_tone();
        COMPOSER.with(|composer| composer.borrow_mut().reset(text.as_str(), method, tone));
    });
    compose.on_key(|text| {
        let character = text.chars().next().unwrap_or('\0');
        COMPOSER
            .with(|composer| composer.borrow_mut().key(character))
            .into()
    });
    compose.on_backspace(|| {
        COMPOSER
            .with(|composer| composer.borrow_mut().backspace())
            .into()
    });
}
