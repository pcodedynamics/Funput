//! "Chung" page: general preferences (launch at login).

use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, SwitchRow};

use crate::settings::Settings;

pub(super) fn page() -> PreferencesPage {
    let s = Settings::load();
    let page = PreferencesPage::builder()
        .title("Chung")
        .icon_name("preferences-system-symbolic")
        .build();
    let group = PreferencesGroup::new();

    // On Linux the engine runs inside the fcitx5/ibus daemon, whose autostart is
    // managed by the desktop session — this toggle only persists the preference.
    let row = SwitchRow::builder()
        .title("Khởi động cùng phiên đăng nhập")
        .subtitle("Bộ gõ do desktop quản lý tự khởi động; tuỳ chọn này chỉ được lưu lại.")
        .active(s.launch_at_login)
        .build();
    row.connect_active_notify(|row| {
        let on = row.is_active();
        Settings::update(|s| s.launch_at_login = on);
    });
    group.add(&row);
    page.add(&group);
    page
}
