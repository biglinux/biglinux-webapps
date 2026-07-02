//! Keyboard shortcut actions for the viewer window: webview navigation,
//! window-level commands, and zoom controls.

mod webview_actions;
mod window_actions;
mod zoom_actions;

use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use glib::clone;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;

use crate::platform::desktop;

pub(super) fn setup_shortcuts(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    toolbar: &adw::ToolbarView,
    is_fullscreen: &Rc<Cell<bool>>,
    url_bar: &gtk::Revealer,
    url_entry: &gtk::Entry,
) {
    let app = window
        .application()
        .expect("Window must belong to an Application");

    window_actions::register_fullscreen_actions(window, &app, toolbar, is_fullscreen);
    webview_actions::register_navigation_actions(window, &app, webview);
    window_actions::register_close_action(window, &app);
    zoom_actions::register_zoom_actions(window, &app, webview);
    webview_actions::register_devtools_action(window, &app, webview);
    window_actions::register_url_focus_action(window, &app, url_bar, url_entry, webview);
    register_shortcuts_window(window, &app);
}

fn register_shortcuts_window(window: &adw::ApplicationWindow, app: &gtk::Application) {
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
