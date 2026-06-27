//! "Ứng dụng bỏ qua" page (Fcitx5 only): apps that default to English on focus,
//! plus a "recent apps" list the engine recorded. Rows are rebuilt on add/remove.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};
use gtk::{Align, Button};

use crate::settings::{self, ExcludedApp, Settings};

pub(super) fn page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Ứng dụng bỏ qua")
        .icon_name("application-x-executable-symbolic")
        .build();

    let excluded_group = PreferencesGroup::builder()
        .title("Ứng dụng bỏ qua")
        .description("Mặc định tiếng Anh khi vào các app này — vẫn bật lại tiếng Việt bằng phím tắt.")
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
