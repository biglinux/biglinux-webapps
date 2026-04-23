//! In-app cheat-sheet for keyboard shortcuts (Ctrl+? / F1).
//!
//! Uses `AdwPreferencesDialog` + `AdwActionRow` instead of `GtkShortcutsWindow`.
//! The gtk-rs bindings for `ShortcutsWindow` don't expose programmatic section
//! insertion, and `AdwPreferencesDialog` covers the same use case with better
//! AT-SPI semantics (each row is announced individually).

#[allow(unused_imports)]
use adw::prelude::*;
use gettextrs::gettext;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

pub(super) fn present(parent: &adw::ApplicationWindow) {
    let group = adw::PreferencesGroup::builder()
        .title(gettext("General"))
        .build();
    add_shortcut(&group, &gettext("New WebApp"), "Ctrl+N");
    add_shortcut(&group, &gettext("Search"), "Ctrl+F");
    add_shortcut(&group, &gettext("Quit"), "Ctrl+Q");
    add_shortcut(&group, &gettext("Show this dialog"), "F1");

    let page = adw::PreferencesPage::builder()
        .title(gettext("Keyboard Shortcuts"))
        .icon_name("input-keyboard-symbolic")
        .build();
    page.add(&group);

    let dialog = adw::PreferencesDialog::builder()
        .title(gettext("Keyboard Shortcuts"))
        .build();
    dialog.add(&page);
    dialog.present(Some(parent));
}

fn add_shortcut(group: &adw::PreferencesGroup, label: &str, accel: &str) {
    let row = adw::ActionRow::builder().title(label).build();
    let accel_label = gtk::Label::builder()
        .label(format!("<tt>{}</tt>", glib_escape(accel)))
        .use_markup(true)
        .build();
    accel_label.add_css_class("dim-label");
    // Announce the accelerator alongside the row title for screen readers.
    row.update_property(&[gtk::accessible::Property::Label(&format!(
        "{label}: {accel}"
    ))]);
    row.add_suffix(&accel_label);
    group.add(&row);
}

fn glib_escape(input: &str) -> String {
    input.replace('<', "&lt;").replace('>', "&gt;")
}
