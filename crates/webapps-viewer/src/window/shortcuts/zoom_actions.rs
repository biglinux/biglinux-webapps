use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

use crate::platform::desktop;

pub(super) fn register_zoom_actions(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    webview: &webkit::WebView,
) {
    desktop::install_action(
        window,
        "zoom-in",
        clone!(
            #[weak]
            webview,
            move || {
                let level = webview.zoom_level();
                webview.set_zoom_level(level + 0.1);
            }
        ),
    );
    app.set_accels_for_action("win.zoom-in", &["<Ctrl>plus", "<Ctrl>equal"]);

    desktop::install_action(
        window,
        "zoom-out",
        clone!(
            #[weak]
            webview,
            move || {
                let level = webview.zoom_level();
                webview.set_zoom_level((level - 0.1).max(0.3));
            }
        ),
    );
    app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus"]);

    desktop::install_action(
        window,
        "zoom-reset",
        clone!(
            #[weak]
            webview,
            move || {
                webview.set_zoom_level(1.0);
            }
        ),
    );
    app.set_accels_for_action("win.zoom-reset", &["<Ctrl>0"]);
}
