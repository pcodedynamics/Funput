//! "Gõ thông minh" page: English auto-restore toggles + read-only examples.

use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, SwitchRow};

use crate::settings::Settings;

pub(super) fn page() -> PreferencesPage {
    let s = Settings::load();
    let page = PreferencesPage::builder()
        .title("Gõ thông minh")
        .icon_name("starred-symbolic")
        .build();

    let group = PreferencesGroup::new();
    let smart_row = SwitchRow::builder()
        .title("Tự khôi phục tiếng Anh")
        .subtitle("Từ không phải tiếng Việt giữ nguyên chữ gốc (card → card, không thành cảd).")
        .active(s.smart_restore)
        .build();
    let eager_row = SwitchRow::builder()
        .title("Khôi phục tức thì")
        .subtitle("Đổi lại ngay khi từ không thể là tiếng Việt, không chờ dấu cách.")
        .active(s.eager_restore)
        .build();
    eager_row.set_sensitive(s.smart_restore);

    let eager_for_smart = eager_row.clone();
    smart_row.connect_active_notify(move |row| {
        let on = row.is_active();
        eager_for_smart.set_sensitive(on);
        Settings::update(|s| s.smart_restore = on);
    });
    eager_row.connect_active_notify(|row| {
        let on = row.is_active();
        Settings::update(|s| s.eager_restore = on);
    });
    let spell_row = SwitchRow::builder()
        .title("Kiểm tra chính tả")
        .subtitle("Chỉ đặt dấu khi tạo thành âm tiết tiếng Việt hợp lệ.")
        .active(s.spell_check)
        .build();
    spell_row.connect_active_notify(|row| {
        let on = row.is_active();
        Settings::update(|s| s.spell_check = on);
    });
    let auto_cap_row = SwitchRow::builder()
        .title("Tự động viết hoa")
        .subtitle("Viết hoa chữ đầu câu, sau dấu chấm và đầu dòng.")
        .active(s.auto_capitalize)
        .build();
    auto_cap_row.connect_active_notify(|row| {
        let on = row.is_active();
        Settings::update(|s| s.auto_capitalize = on);
    });
    group.add(&smart_row);
    group.add(&eager_row);
    group.add(&spell_row);
    group.add(&auto_cap_row);
    page.add(&group);

    // Informational examples (read-only).
    let ex = PreferencesGroup::builder().title("Ví dụ").build();
    let r1 = ActionRow::builder()
        .title("text → text")
        .subtitle("Giữ tiếng Anh")
        .build();
    let r2 = ActionRow::builder()
        .title("tieesng → tiếng")
        .subtitle("Gõ tiếng Việt")
        .build();
    ex.add(&r1);
    ex.add(&r2);
    page.add(&ex);

    page
}
