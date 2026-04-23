//! Cheat-sheet dialog (Ctrl+? / F1) listing viewer shortcuts.
//!
//! Uses `AdwPreferencesDialog` instead of `GtkShortcutsWindow` because the
//! gtk-rs bindings don't expose programmatic section insertion; the preferences
//! dialog gives the same result with better AT-SPI semantics.

#[allow(unused_imports)]
use adw::prelude::*;
use gettextrs::gettext;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

pub(super) fn present(parent: &adw::ApplicationWindow) {
    let navigation = build_group(
        &gettext("Navigation"),
        &[
            (gettext("Focus address bar"), "Ctrl+L"),
            (gettext("Reload page"), "Ctrl+R"),
            (gettext("Go back"), "Alt+←"),
            (gettext("Go forward"), "Alt+→"),
        ],
    );

    let view = build_group(
        &gettext("View"),
        &[
            (gettext("Toggle fullscreen"), "F11"),
            (gettext("Zoom in"), "Ctrl++"),
            (gettext("Zoom out"), "Ctrl+-"),
            (gettext("Reset zoom"), "Ctrl+0"),
        ],
    );

    let window_group = build_group(
        &gettext("Window"),
        &[
            (gettext("Close window"), "Ctrl+W"),
            (gettext("Show this dialog"), "F1"),
        ],
    );

    let page = adw::PreferencesPage::builder()
        .title(gettext("Keyboard Shortcuts"))
        .icon_name("input-keyboard-symbolic")
        .build();
    page.add(&navigation);
    page.add(&view);
    page.add(&window_group);

    let dialog = adw::PreferencesDialog::builder()
        .title(gettext("Keyboard Shortcuts"))
        .build();
    dialog.add(&page);
    dialog.present(Some(parent));
}

fn build_group(title: &str, rows: &[(String, &str)]) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder().title(title).build();
    for (label, accel) in rows {
        let row = adw::ActionRow::builder().title(label.as_str()).build();
        let accel_label = gtk::Label::builder()
            .label(format!("<tt>{}</tt>", escape_markup(accel)))
            .use_markup(true)
            .build();
        accel_label.add_css_class("dim-label");
        row.update_property(&[gtk::accessible::Property::Label(&format!(
            "{label}: {accel}"
        ))]);
        row.add_suffix(&accel_label);
        group.add(&row);
    }
    group
}

fn escape_markup(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
