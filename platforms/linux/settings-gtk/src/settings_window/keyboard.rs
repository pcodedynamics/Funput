//! "Phím chuyển" page: the VI/EN toggle hotkey and the flip hotkey.

use adw::prelude::*;
use adw::{ComboRow, PreferencesGroup, PreferencesPage};
use gtk::StringList;

use crate::settings::{FlipHotkey, Hotkey, Settings};

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

    // Flip the word being composed VN↔raw (card ⇄ cải). Presets mirror the Windows
    // build; "Tắt" disables it.
    let flip_group = PreferencesGroup::builder()
        .description("Đổi từ đang gõ giữa tiếng Việt và chữ gốc (card ⇄ cải).")
        .build();
    let flip_row = ComboRow::builder()
        .title("Phím lật từ vừa gõ")
        .model(&StringList::new(&["Tắt", "Ctrl + Shift + Z", "Ctrl + Shift + X"]))
        .build();
    flip_row.set_selected(match s.flip_hotkey {
        FlipHotkey::CtrlShiftZ => 1,
        FlipHotkey::CtrlShiftX => 2,
        FlipHotkey::Off => 0,
    });
    flip_row.connect_selected_notify(|row| {
        let h = match row.selected() {
            1 => FlipHotkey::CtrlShiftZ,
            2 => FlipHotkey::CtrlShiftX,
            _ => FlipHotkey::Off,
        };
        Settings::update(|s| s.flip_hotkey = h);
    });
    flip_group.add(&flip_row);
    page.add(&flip_group);

    page
}
