//! The Settings window: an `AdwPreferencesWindow` with one page per settings group.
//! Every control reads the current value from `settings.json` on build and writes
//! back through `Settings::update` on change — the engine reloads on its next
//! focus-in. Mirrors the panes of the retired Svelte UI (`platforms/ui/src/lib/
//! settings/panes/*`), minus the "Gõ thử" try-typing box (dropped for a cleaner
//! native UI; users type in real apps).

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{
    AboutDialog, ActionRow, Application, ComboRow, PreferencesGroup, PreferencesPage,
    PreferencesWindow, SwitchRow,
};
use gtk::{Align, Button, Image, StringList};

use crate::settings::{self, ExcludedApp, Hotkey, Method, Settings, ToneStyle};

pub fn build(app: &Application) -> PreferencesWindow {
    let window = PreferencesWindow::builder()
        .title("Funput — Cài đặt")
        .default_width(640)
        .default_height(520)
        .build();
    window.set_application(Some(app));
    window.set_search_enabled(false);

    window.add(&input_method_page());
    window.add(&smart_page());
    window.add(&keyboard_page());
    window.add(&apps_page());
    window.add(&general_page());
    window.add(&about_page(&window));

    window
}

// --- Kiểu gõ ----------------------------------------------------------------

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

fn input_method_page() -> PreferencesPage {
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

// --- Gõ thông minh ----------------------------------------------------------

fn smart_page() -> PreferencesPage {
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
    group.add(&smart_row);
    group.add(&eager_row);
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

// --- Phím chuyển VI / EN -----------------------------------------------------

fn keyboard_page() -> PreferencesPage {
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

// --- Chung ------------------------------------------------------------------

fn general_page() -> PreferencesPage {
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

// --- Ứng dụng bỏ qua ---------------------------------------------------------

fn apps_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Ứng dụng bỏ qua")
        .icon_name("application-x-executable-symbolic")
        .build();

    let excluded_group = PreferencesGroup::builder()
        .title("Ứng dụng bỏ qua")
        .description(
            "Mặc định tiếng Anh khi vào các app này — vẫn bật lại tiếng Việt bằng phím tắt. \
             (IBus hiện chưa hỗ trợ tự chuyển theo app; cần Fcitx5.)",
        )
        .build();
    let recent_group = PreferencesGroup::builder().title("App gần đây").build();
    page.add(&excluded_group);
    page.add(&recent_group);

    // AdwPreferencesGroup (libadwaita 1.5) has no "remove all", so track the rows we
    // add and remove them by hand before each rebuild.
    let excluded_rows: Rc<RefCell<Vec<ActionRow>>> = Rc::new(RefCell::new(Vec::new()));
    let recent_rows: Rc<RefCell<Vec<ActionRow>>> = Rc::new(RefCell::new(Vec::new()));

    // Wrapped in Rc<RefCell<Option<…>>> so the per-row add/remove buttons created
    // *inside* the rebuild can call the rebuild again.
    let rebuild: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    let rebuild_impl: Rc<dyn Fn()> = {
        let excluded_group = excluded_group.clone();
        let recent_group = recent_group.clone();
        let excluded_rows = excluded_rows.clone();
        let recent_rows = recent_rows.clone();
        let rebuild = rebuild.clone();
        Rc::new(move || {
            for r in excluded_rows.borrow_mut().drain(..) {
                excluded_group.remove(&r);
            }
            for r in recent_rows.borrow_mut().drain(..) {
                recent_group.remove(&r);
            }

            let s = Settings::load();

            // Excluded list.
            if s.excluded_apps.is_empty() {
                let row = ActionRow::builder()
                    .title("Chưa có app nào")
                    .subtitle("Thêm từ danh sách bên dưới.")
                    .build();
                excluded_group.add(&row);
                excluded_rows.borrow_mut().push(row);
            } else {
                for app in &s.excluded_apps {
                    let row = ActionRow::builder()
                        .title(app.name.as_str())
                        .subtitle(app.id.as_str())
                        .build();
                    let btn = Button::builder()
                        .icon_name("user-trash-symbolic")
                        .valign(Align::Center)
                        .tooltip_text("Bỏ khỏi danh sách")
                        .build();
                    btn.add_css_class("flat");
                    let id = app.id.clone();
                    let rebuild_cb = rebuild.clone();
                    btn.connect_clicked(move |_| {
                        let id = id.clone();
                        Settings::update(move |s| s.excluded_apps.retain(|a| a.id != id));
                        if let Some(f) = rebuild_cb.borrow().as_ref() {
                            f();
                        }
                    });
                    row.add_suffix(&btn);
                    excluded_group.add(&row);
                    excluded_rows.borrow_mut().push(row);
                }
            }

            // Recent = apps the engine saw focused, minus the ones already excluded.
            let addable: Vec<ExcludedApp> = settings::recent_apps()
                .into_iter()
                .filter(|r| !s.excluded_apps.iter().any(|e| e.id == r.id))
                .collect();
            if addable.is_empty() {
                let row = ActionRow::builder()
                    .title("Chưa có app nào gần đây")
                    .subtitle("Chuyển qua lại các app một lúc để chúng hiện ở đây.")
                    .build();
                recent_group.add(&row);
                recent_rows.borrow_mut().push(row);
            } else {
                for app in addable {
                    let row = ActionRow::builder()
                        .title(app.name.as_str())
                        .subtitle(app.id.as_str())
                        .build();
                    let btn = Button::builder().label("Thêm").valign(Align::Center).build();
                    btn.add_css_class("flat");
                    let app = app.clone();
                    let rebuild_cb = rebuild.clone();
                    btn.connect_clicked(move |_| {
                        let app = app.clone();
                        Settings::update(move |s| {
                            if !s.excluded_apps.iter().any(|a| a.id == app.id) {
                                s.excluded_apps.push(app);
                            }
                        });
                        if let Some(f) = rebuild_cb.borrow().as_ref() {
                            f();
                        }
                    });
                    row.add_suffix(&btn);
                    recent_group.add(&row);
                    recent_rows.borrow_mut().push(row);
                }
            }
        })
    };

    *rebuild.borrow_mut() = Some(rebuild_impl.clone());
    rebuild_impl();

    page
}

// --- Giới thiệu --------------------------------------------------------------

fn about_page(parent: &PreferencesWindow) -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Giới thiệu")
        .icon_name("help-about-symbolic")
        .build();
    let group = PreferencesGroup::new();

    let ver_row = ActionRow::builder()
        .title("Funput")
        .subtitle(format!("Phiên bản {}", env!("CARGO_PKG_VERSION")))
        .build();
    group.add(&ver_row);

    let about_row = ActionRow::builder()
        .title("Giới thiệu Funput")
        .activatable(true)
        .build();
    about_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    let parent = parent.clone();
    about_row.connect_activated(move |_| show_about(&parent));
    group.add(&about_row);

    page.add(&group);
    page
}

fn show_about(parent: &PreferencesWindow) {
    let about = AboutDialog::builder()
        .application_name("Funput")
        .application_icon("funput")
        .version(env!("CARGO_PKG_VERSION"))
        .comments("Bộ gõ tiếng Việt — miễn phí, mã nguồn mở.")
        .developer_name("Funput")
        .website("https://funput.app/")
        .issue_url("https://github.com/Funput/Funput/issues")
        .license_type(gtk::License::MitX11)
        .build();
    about.add_link("GitHub", "https://github.com/Funput/Funput");
    // AdwDialog presents itself relative to a parent widget (no transient/modal).
    about.present(Some(parent));
}
