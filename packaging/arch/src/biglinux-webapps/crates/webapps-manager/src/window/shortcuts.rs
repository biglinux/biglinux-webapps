#[allow(unused_imports)]
use adw::prelude::*;
use big_app_kit::desktop;
use glib::clone;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;

/// Register window-scoped GActions backed by [`Gtk::Application::set_accels_for_action`].
///
/// Going through GActions (instead of a raw `ShortcutController`) lets the
/// accelerators surface in `AdwAboutDialog`, `GtkShortcutsWindow`, and the
/// keyboard-shortcut cheat-sheet, and keeps them discoverable via Orca / other
/// AT-SPI consumers.
pub(super) fn install_shortcuts(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    add_btn: &gtk::Button,
    search_btn: &gtk::ToggleButton,
) {
    desktop::install_action(
        window,
        "add-webapp",
        clone!(
            #[weak]
            add_btn,
            move || add_btn.emit_clicked()
        ),
    );
    app.set_accels_for_action("win.add-webapp", &["<Ctrl>n"]);

    desktop::install_action(
        window,
        "toggle-search",
        clone!(
            #[weak]
            search_btn,
            move || search_btn.set_active(!search_btn.is_active())
        ),
    );
    app.set_accels_for_action("win.toggle-search", &["<Ctrl>f"]);

    desktop::install_action(
        window,
        "quit",
        clone!(
            #[weak]
            app,
            move || app.quit()
        ),
    );
    app.set_accels_for_action("win.quit", &["<Ctrl>q"]);

    desktop::install_action(
        window,
        "shortcuts",
        clone!(
            #[weak]
            window,
            move || super::shortcuts_window::present(&window)
        ),
    );
    app.set_accels_for_action("win.shortcuts", &["<Ctrl>question", "F1"]);
}
