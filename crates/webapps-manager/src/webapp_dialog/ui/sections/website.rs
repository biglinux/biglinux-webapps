use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::WebApp;

pub(crate) struct WebsiteSection {
    pub url_row: adw::EntryRow,
    pub name_row: adw::EntryRow,
    pub detect_button: gtk::Button,
}

pub(crate) fn build_website_section(webapp: &WebApp) -> WebsiteSection {
    let url_row = adw::EntryRow::builder()
        .title(gettext("URL"))
        .text(&webapp.app_url)
        .build();
    let detect_button = gtk::Button::with_label(&gettext("Detect"));
    detect_button.set_tooltip_text(Some(&gettext("Detect name and icon from website")));
    detect_button.set_valign(gtk::Align::Center);
    url_row.add_suffix(&detect_button);

    let name_row = adw::EntryRow::builder()
        .title(gettext("Name"))
        .text(&webapp.app_name)
        .build();

    WebsiteSection {
        url_row,
        name_row,
        detect_button,
    }
}
