//! First-run welcome dialog.
//!
//! Finishes the onda6 migration: replaces the hand-built `adw::Dialog` with
//! the cataloged `big_app_kit::dialogs` content-dialog helper (an
//! `adw::AlertDialog` carrying a content box), so it matches the rest of the
//! manager's dialog surface and satisfies the component policy (no direct
//! `adw::Dialog`). The dialog is informational + a single "don't show again"
//! switch; its only effect fires on dismissal.

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;

use crate::service;

/// Show the welcome dialog on first run only. Returns immediately.
pub fn show_if_needed(parent: &adw::ApplicationWindow) {
    if !service::should_show_welcome() {
        return;
    }
    build_dialog().present(Some(parent));
}

fn build_dialog() -> adw::AlertDialog {
    let (dialog, content) = big_app_kit::dialogs::content_dialog(
        &gettext("Welcome to WebApps Manager"),
        &gettext("Let's Start"),
    );
    dialog.set_default_response(Some("close"));

    let icon = gtk::Image::from_icon_name("big-webapps");
    icon.set_pixel_size(64);
    icon.set_halign(gtk::Align::Center);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    content.append(&icon);

    let explanation = gtk::Label::builder()
        .label(format!(
            "<b>{}</b>\n\n{}\n\n<b>{}</b>\n\n\
            \u{2022} <b>{}</b>: {}\n\
            \u{2022} <b>{}</b>: {}\n\
            \u{2022} <b>{}</b>: {}",
            gettext("What are WebApps?"),
            gettext("WebApps are web applications that run in a dedicated browser window, providing a more app-like experience for your favorite websites."),
            gettext("Benefits of using WebApps:"),
            gettext("Focus"), gettext("Work without the distractions of other browser tabs"),
            gettext("Desktop Integration"), gettext("Quick access from your application menu"),
            gettext("Isolated Profiles"), gettext("Each webapp can have its own cookies and settings"),
        ))
        .use_markup(true)
        .wrap(true)
        .max_width_chars(60)
        .halign(gtk::Align::Start)
        .build();
    content.append(&explanation);

    // "Don't show again" — `AdwSwitchRow` wires its label↔switch a11y relation.
    let show_switch_row = adw::SwitchRow::builder()
        .title(gettext("Don't show this again"))
        .active(false)
        .margin_top(12)
        .build();
    let switch_list = gtk::ListBox::new();
    switch_list.set_selection_mode(gtk::SelectionMode::None);
    switch_list.add_css_class("boxed-list");
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
