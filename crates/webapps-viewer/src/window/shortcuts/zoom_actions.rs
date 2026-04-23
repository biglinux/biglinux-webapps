#[allow(unused_imports)]
use adw::prelude::*;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) fn register_zoom_actions(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    webview: &webkit::WebView,
) {
    use gtk::gio;

    let action_zin = gio::SimpleAction::new("zoom-in", None);
    action_zin.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            let level = webview.zoom_level();
            webview.set_zoom_level(level + 0.1);
        }
    ));
    window.add_action(&action_zin);
    app.set_accels_for_action("win.zoom-in", &["<Ctrl>plus", "<Ctrl>equal"]);

    let action_zout = gio::SimpleAction::new("zoom-out", None);
    action_zout.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            let level = webview.zoom_level();
            webview.set_zoom_level((level - 0.1).max(0.3));
        }
    ));
    window.add_action(&action_zout);
    app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus"]);

    let action_zreset = gio::SimpleAction::new("zoom-reset", None);
    action_zreset.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.set_zoom_level(1.0);
        }
    ));
    window.add_action(&action_zreset);
    app.set_accels_for_action("win.zoom-reset", &["<Ctrl>0"]);
}
