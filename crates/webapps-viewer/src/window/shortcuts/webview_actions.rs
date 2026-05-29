use big_app_kit::desktop;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) fn register_navigation_actions(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    webview: &webkit::WebView,
) {
    desktop::install_action(
        window,
        "reload",
        clone!(
            #[weak]
            webview,
            move || {
                webview.reload();
            }
        ),
    );
    app.set_accels_for_action("win.reload", &["<Ctrl>r", "F5"]);

    desktop::install_action(
        window,
        "go-back",
        clone!(
            #[weak]
            webview,
            move || {
                webview.go_back();
            }
        ),
    );
    app.set_accels_for_action("win.go-back", &["<Alt>Left"]);

    desktop::install_action(
        window,
        "go-forward",
        clone!(
            #[weak]
            webview,
            move || {
                webview.go_forward();
            }
        ),
    );
    app.set_accels_for_action("win.go-forward", &["<Alt>Right"]);
}

pub(super) fn register_devtools_action(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    webview: &webkit::WebView,
) {
    if !super::super::settings::DEVELOPER_TOOLS_ENABLED {
        return;
    }

    desktop::install_action(
        window,
        "devtools",
        clone!(
            #[weak]
            webview,
            move || {
                if let Some(inspector) = webview.inspector() {
                    inspector.show();
                }
            }
        ),
    );
    app.set_accels_for_action("win.devtools", &["<Ctrl><Shift>i"]);
}
