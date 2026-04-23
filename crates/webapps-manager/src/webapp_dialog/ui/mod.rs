mod sections;
mod shell;

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::{BrowserCollection, WebApp};

pub(crate) use sections::update_browser_icon;

pub(super) const CATEGORIES: &[&str] = &[
    "Webapps",
    "Network",
    "Office",
    "Development",
    "Graphics",
    "AudioVideo",
    "Game",
    "Utility",
    "System",
    "Education",
    "Science",
];

pub(super) struct DialogWidgets {
    pub dialog: adw::Dialog,
    pub url_row: adw::EntryRow,
    pub name_row: adw::EntryRow,
    pub icon_preview: gtk::Image,
    pub icon_button: gtk::Button,
    pub favicon_flow: gtk::FlowBox,
    pub category_row: adw::ComboRow,
    pub browser_row: adw::ActionRow,
    pub browser_button: gtk::Button,
    pub browser_icon: gtk::Image,
    pub profile_row: adw::ExpanderRow,
    pub profile_entry: adw::EntryRow,
    pub detect_button: gtk::Button,
    pub spinner_box: gtk::Box,
    pub cancel_button: gtk::Button,
    pub save_button: gtk::Button,
}

pub(super) fn build_dialog(
    webapp: &WebApp,
    _is_new: bool,
    browsers: Rc<RefCell<BrowserCollection>>,
) -> DialogWidgets {
    let shell = shell::build_dialog_shell(_is_new);
    let website = sections::build_website_section(webapp);
    let appearance = sections::build_appearance_section(webapp);
    let behavior = sections::build_behavior_section(webapp, browsers);

    let group = adw::PreferencesGroup::new();
    group.add(&website.url_row);
    group.add(&website.name_row);
    group.add(&appearance.icon_row);
    group.add(&appearance.category_row);
    group.add(&behavior.browser_row);
    group.add(&behavior.profile_row);

    shell.form.append(&group);
    shell.form.append(&appearance.favicon_flow);
    shell.dialog.set_child(Some(&shell.outer));

    let sections::WebsiteSection {
        url_row,
        name_row,
        detect_button,
    } = website;
    let sections::AppearanceSection {
        icon_row: _,
        icon_preview,
        icon_button,
        favicon_flow,
        category_row,
    } = appearance;
    let sections::BehaviorSection {
        browser_row,
        browser_button,
        browser_icon,
        profile_row,
        profile_entry,
    } = behavior;

    DialogWidgets {
        dialog: shell.dialog,
        url_row,
        name_row,
        icon_preview,
        icon_button,
        favicon_flow,
        category_row,
        browser_row,
        browser_button,
        browser_icon,
        profile_row,
        profile_entry,
        detect_button,
        spinner_box: shell.spinner_box,
        cancel_button: shell.cancel_button,
        save_button: shell.save_button,
    }
}
