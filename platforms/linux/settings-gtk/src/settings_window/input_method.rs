//! "Kiểu gõ" page: input method (Telex/VNI) and tone-mark placement style.

use adw::prelude::*;
use adw::{ComboRow, PreferencesGroup, PreferencesPage};
use gtk::StringList;

use crate::settings::{Method, Settings, ToneStyle};

fn method_blurb(m: Method) -> &'static str {
    match m {
        Method::Telex => "Dấu bằng chữ cái — aa→â, ow→ơ, as→á, dd→đ",
        Method::Vni => "Dấu bằng chữ số — a6→â, o7→ơ, a1→á, d9→đ",
    }
}

fn tone_blurb(t: ToneStyle) -> &'static str {
    match t {
        ToneStyle::Traditional => "Dấu kiểu cũ — hòa, khỏe, thúy",
        ToneStyle::Modern => "Dấu kiểu mới — hoà, khoẻ, thuý",
    }
}

pub(super) fn page() -> PreferencesPage {
    let s = Settings::load();
    let page = PreferencesPage::builder()
        .title("Kiểu gõ")
        .icon_name("input-keyboard-symbolic")
        .build();
    let group = PreferencesGroup::new();

    let method_row = ComboRow::builder()
        .title("Phương thức")
        .model(&StringList::new(&["Telex", "VNI"]))
        .build();
    method_row.set_selected(match s.method {
        Method::Telex => 0,
        Method::Vni => 1,
    });
    method_row.set_subtitle(method_blurb(s.method));
    method_row.connect_selected_notify(|row| {
        let m = if row.selected() == 0 { Method::Telex } else { Method::Vni };
        row.set_subtitle(method_blurb(m));
        Settings::update(|s| s.method = m);
    });
    group.add(&method_row);

    let tone_row = ComboRow::builder()
        .title("Kiểu đặt dấu")
        .model(&StringList::new(&["Truyền thống", "Hiện đại"]))
        .build();
    tone_row.set_selected(match s.tone_style {
        ToneStyle::Traditional => 0,
        ToneStyle::Modern => 1,
    });
    tone_row.set_subtitle(tone_blurb(s.tone_style));
    tone_row.connect_selected_notify(|row| {
        let t = if row.selected() == 0 {
            ToneStyle::Traditional
        } else {
            ToneStyle::Modern
        };
        row.set_subtitle(tone_blurb(t));
        Settings::update(|s| s.tone_style = t);
    });
    group.add(&tone_row);

    page.add(&group);
    page
}
