//! "Phím chuyển" page: the VI/EN toggle hotkey.

use adw::prelude::*;
use adw::{ComboRow, PreferencesGroup, PreferencesPage};
use gtk::StringList;

use crate::settings::{Hotkey, Settings};

pub(super) fn page() -> PreferencesPage {
    let s = Settings::load();
    let page = PreferencesPage::builder()
        .title("Phím chuyển")
        .icon_name("preferences-desktop-keyboard-shortcuts-symbolic")
        .build();
    let group = PreferencesGroup::builder()
        .description("Bật/tắt gõ tiếng Việt nhanh.")
        .build();

    // Only the combos the Linux engines actually handle. Alt+Shift (a modifier-only
    // combo) is unsupported by both shells (funput_engine.cpp / engine.cpp return
    // false), so it is not offered here.
    let row = ComboRow::builder()
        .title("Phím tắt")
        .model(&StringList::new(&["Ctrl + `", "Ctrl + Space"]))
        .build();
    row.set_selected(match s.toggle_hotkey {
        Hotkey::CtrlSpace => 1,
        // CtrlBacktick — and any legacy AltShift value — show as Ctrl+` (the default).
        _ => 0,
    });
    row.connect_selected_notify(|row| {
        let h = if row.selected() == 1 {
            Hotkey::CtrlSpace
        } else {
            Hotkey::CtrlBacktick
        };
        Settings::update(|s| s.toggle_hotkey = h);
    });
    group.add(&row);
    page.add(&group);
    page
}
