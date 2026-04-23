#[allow(unused_imports)]
use adw::prelude::*;
use glib::clone;
use gtk::gio;
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
    let add_action = gio::SimpleAction::new("add-webapp", None);
    add_action.connect_activate(clone!(
        #[weak]
        add_btn,
        move |_, _| add_btn.emit_clicked()
    ));
    window.add_action(&add_action);
    app.set_accels_for_action("win.add-webapp", &["<Ctrl>n"]);

    let search_action = gio::SimpleAction::new("toggle-search", None);
    search_action.connect_activate(clone!(
        #[weak]
        search_btn,
        move |_, _| search_btn.set_active(!search_btn.is_active())
    ));
    window.add_action(&search_action);
    app.set_accels_for_action("win.toggle-search", &["<Ctrl>f"]);

    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate(clone!(
        #[weak]
        app,
        move |_, _| app.quit()
    ));
    window.add_action(&quit_action);
    app.set_accels_for_action("win.quit", &["<Ctrl>q"]);

    let shortcuts_action = gio::SimpleAction::new("shortcuts", None);
    shortcuts_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| super::shortcuts_window::present(&window)
    ));
    window.add_action(&shortcuts_action);
    app.set_accels_for_action("win.shortcuts", &["<Ctrl>question", "F1"]);
}
