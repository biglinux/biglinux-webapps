//! First-run welcome dialog.

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;

use crate::platform::dialogs;
use crate::service;

/// Show the welcome dialog on first run only. Returns immediately.
pub fn show_if_needed(parent: &adw::ApplicationWindow) {
    if !service::should_show_welcome() {
        return;
    }
    build_dialog().present(Some(parent));
}

fn build_dialog() -> adw::AlertDialog {
    let (dialog, content) = dialogs::content_dialog(
        &gettext("Welcome to WebApps Manager"),
        &gettext("Let's Start"),
    );
    dialog.set_default_response(Some("close"));

    let icon = gtk::Image::from_icon_name("big-webapps");
    icon.set_pixel_size(64);
    icon.set_halign(gtk::Align::Center);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    content.append(&icon);

    let intro = gtk::Label::builder()
        .label(gettext(
            "WebApps run your favorite websites in their own dedicated window, \
             for a focused, app-like experience.",
        ))
        .wrap(true)
        .justify(gtk::Justification::Center)
        .halign(gtk::Align::Center)
        .max_width_chars(46)
        .build();
    content.append(&intro);

    // Benefits as a native boxed list — scannable and individually labelled for
    // AT-SPI, replacing the former single markup blob.
    let benefits = adw::PreferencesGroup::builder()
        .title(gettext("Benefits"))
        .margin_top(6)
        .build();
    benefits.add(&benefit_row(
        "view-fullscreen-symbolic",
        &gettext("Focus"),
        &gettext("Work without the distractions of other browser tabs"),
    ));
    benefits.add(&benefit_row(
        "view-grid-symbolic",
        &gettext("Desktop Integration"),
        &gettext("Quick access from your application menu"),
    ));
    benefits.add(&benefit_row(
        "channel-secure-symbolic",
        &gettext("Isolated Profiles"),
        &gettext("Each webapp can have its own cookies and settings"),
    ));
    content.append(&benefits);

    // "Don't show again" — `AdwSwitchRow` wires its label↔switch a11y relation.
    let show_switch_row = adw::SwitchRow::builder()
        .title(gettext("Don't show this again"))
        .active(false)
        .build();
    let switch_list = gtk::ListBox::new();
    switch_list.set_selection_mode(gtk::SelectionMode::None);
    switch_list.add_css_class("boxed-list");
    switch_list.set_margin_top(6);
    switch_list.append(&show_switch_row);
    content.append(&switch_list);

    dialog.connect_response(None, move |_, response| {
        // Switch ON = "don't show" → persist the shown flag. The dialog closes
        // itself on any response (close_response = "close").
        if response == "close" && show_switch_row.is_active() {
            service::mark_welcome_shown();
        }
    });

    dialog
}

/// A single benefit line: symbolic icon + title + explanation.
fn benefit_row(icon_name: &str, title: &str, subtitle: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    row.add_prefix(&icon);
    row
}
