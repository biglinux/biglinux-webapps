use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::{BrowserCollection, WebApp};

pub(crate) struct BehaviorSection {
    pub browser_row: adw::ActionRow,
    pub browser_button: gtk::Button,
    pub browser_icon: gtk::Image,
    pub profile_row: adw::ExpanderRow,
    pub profile_entry: adw::EntryRow,
}

pub(crate) fn build_behavior_section(
    webapp: &WebApp,
    browsers: Rc<RefCell<BrowserCollection>>,
) -> BehaviorSection {
    let browser_row = adw::ActionRow::builder().title(gettext("Browser")).build();
    let browser_icon = gtk::Image::new();
    browser_icon.set_pixel_size(20);
    browser_icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    update_browser_icon(&browser_icon, webapp);
    browser_row.add_prefix(&browser_icon);
    browser_row.set_subtitle(&browser_display_name(webapp, &browsers.borrow()));

    let browser_button = gtk::Button::with_label(&gettext("Select"));
    browser_button.set_valign(gtk::Align::Center);
    browser_row.add_suffix(&browser_button);
    browser_row.set_activatable_widget(Some(&browser_button));

    let profile_row = build_profile_row(webapp);
    let profile_entry = build_profile_entry(webapp);
    profile_row.add_row(&profile_entry);
    profile_row.set_visible(!webapp.browser_id().is_viewer());

    BehaviorSection {
        browser_row,
        browser_button,
        browser_icon,
        profile_row,
        profile_entry,
    }
}

pub(crate) fn update_browser_icon(image: &gtk::Image, webapp: &WebApp) {
    let icon_name = if webapp.browser_id().is_viewer() {
        "big-webapps".to_string()
    } else {
        webapps_core::models::Browser {
            browser_id: webapp.browser.clone(),
            is_default: false,
        }
        .icon_name()
    };
    crate::webapp_row::load_icon(image, &icon_name);
}

fn browser_display_name(webapp: &WebApp, browsers: &BrowserCollection) -> String {
    if webapp.browser_id().is_viewer() {
        return gettext("Internal Browser");
    }
    browsers
        .get_by_id(&webapp.browser)
        .map(|browser| browser.display_name().to_string())
        .unwrap_or_else(|| webapp.browser.clone())
}

fn build_profile_row(webapp: &WebApp) -> adw::ExpanderRow {
    adw::ExpanderRow::builder()
        .title(gettext("Use separate profile"))
        .subtitle(gettext(
            "Allows independent cookies and sessions for this webapp",
        ))
        .show_enable_switch(true)
        .enable_expansion(webapp.has_custom_profile())
        .build()
}

fn build_profile_entry(webapp: &WebApp) -> adw::EntryRow {
    adw::EntryRow::builder()
        .title(gettext("Profile Name"))
        .text(&webapp.app_profile)
        .build()
}
