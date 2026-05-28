use std::cell::Cell;
use std::rc::Rc;

#[allow(unused_imports)]
use adw::prelude::*;
use big_app_kit::desktop;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) fn register_fullscreen_actions(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    toolbar: &adw::ToolbarView,
    is_fullscreen: &Rc<Cell<bool>>,
) {
    desktop::install_action(
        window,
        "toggle-fullscreen",
        clone!(
            #[weak]
            window,
            #[weak]
            toolbar,
            #[strong]
            is_fullscreen,
            move || {
                if is_fullscreen.get() {
                    is_fullscreen.set(false);
                    toolbar.set_reveal_top_bars(true);
                    window.unfullscreen();
                } else {
                    is_fullscreen.set(true);
                    toolbar.set_reveal_top_bars(false);
                    window.fullscreen();
                }
            }
        ),
    );
    app.set_accels_for_action("win.toggle-fullscreen", &["F11"]);

    desktop::install_action(
        window,
        "exit-fullscreen",
        clone!(
            #[weak]
            window,
            #[weak]
            toolbar,
            #[strong]
            is_fullscreen,
            move || {
                if is_fullscreen.get() {
                    is_fullscreen.set(false);
                    toolbar.set_reveal_top_bars(true);
                    window.unfullscreen();
                }
            }
        ),
    );
    app.set_accels_for_action("win.exit-fullscreen", &["Escape"]);
}

pub(super) fn register_close_action(window: &adw::ApplicationWindow, app: &gtk::Application) {
    desktop::install_action(
        window,
        "close-window",
        clone!(
            #[weak]
            window,
            move || {
                window.close();
            }
        ),
    );
    app.set_accels_for_action("win.close-window", &["<Ctrl>w"]);
}

pub(super) fn register_url_focus_action(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    url_bar: &gtk::Revealer,
    url_entry: &gtk::Entry,
    webview: &webkit::WebView,
) {
    desktop::install_action(
        window,
        "focus-url",
        clone!(
            #[weak]
            url_bar,
            #[weak]
            url_entry,
            #[weak]
            webview,
            move || {
                url_bar.set_reveal_child(true);
                if let Some(uri) = webview.uri() {
                    url_entry.set_text(&uri);
                }
                url_entry.grab_focus();
                url_entry.select_region(0, -1);
            }
        ),
    );
    app.set_accels_for_action("win.focus-url", &["<Ctrl>l"]);
}
