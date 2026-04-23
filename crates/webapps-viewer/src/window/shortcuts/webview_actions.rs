#[allow(unused_imports)]
use adw::prelude::*;
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
    use gtk::gio;

    let action_reload = gio::SimpleAction::new("reload", None);
    action_reload.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.reload();
        }
    ));
    window.add_action(&action_reload);
    app.set_accels_for_action("win.reload", &["<Ctrl>r", "F5"]);

    let action_back = gio::SimpleAction::new("go-back", None);
    action_back.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.go_back();
        }
    ));
    window.add_action(&action_back);
    app.set_accels_for_action("win.go-back", &["<Alt>Left"]);

    let action_fwd = gio::SimpleAction::new("go-forward", None);
    action_fwd.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.go_forward();
        }
    ));
    window.add_action(&action_fwd);
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

    use gtk::gio;

    let action_dev = gio::SimpleAction::new("devtools", None);
    action_dev.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            if let Some(inspector) = webview.inspector() {
                inspector.show();
            }
        }
    ));
    window.add_action(&action_dev);
    app.set_accels_for_action("win.devtools", &["<Ctrl><Shift>i"]);
}
