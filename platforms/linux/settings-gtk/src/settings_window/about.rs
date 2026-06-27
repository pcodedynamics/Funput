//! "Giới thiệu" page: version info and the About dialog.

use adw::prelude::*;
use adw::{AboutDialog, ActionRow, PreferencesGroup, PreferencesPage, PreferencesWindow};
use gtk::Image;

pub(super) fn page(parent: &PreferencesWindow) -> PreferencesPage {
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
