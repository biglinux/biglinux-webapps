use std::cell::Cell;
use std::rc::Rc;

#[allow(unused_imports)]
use adw::prelude::*;
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
    use gtk::gio;

    let action_fs = gio::SimpleAction::new("toggle-fullscreen", None);
    action_fs.connect_activate(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_, _| {
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
    ));
    window.add_action(&action_fs);
    app.set_accels_for_action("win.toggle-fullscreen", &["F11"]);

    let action_esc = gio::SimpleAction::new("exit-fullscreen", None);
    action_esc.connect_activate(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_, _| {
            if is_fullscreen.get() {
                is_fullscreen.set(false);
                toolbar.set_reveal_top_bars(true);
                window.unfullscreen();
            }
        }
    ));
    window.add_action(&action_esc);
    app.set_accels_for_action("win.exit-fullscreen", &["Escape"]);
}

pub(super) fn register_close_action(window: &adw::ApplicationWindow, app: &gtk::Application) {
    use gtk::gio;

    let action_close = gio::SimpleAction::new("close-window", None);
    action_close.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            window.close();
        }
    ));
    window.add_action(&action_close);
    app.set_accels_for_action("win.close-window", &["<Ctrl>w"]);
}

pub(super) fn register_url_focus_action(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    url_bar: &gtk::Revealer,
    url_entry: &gtk::Entry,
    webview: &webkit::WebView,
) {
    use gtk::gio;

    let action_url = gio::SimpleAction::new("focus-url", None);
    action_url.connect_activate(clone!(
        #[weak]
        url_bar,
        #[weak]
        url_entry,
        #[weak]
        webview,
        move |_, _| {
            url_bar.set_reveal_child(true);
            if let Some(uri) = webview.uri() {
                url_entry.set_text(&uri);
            }
            url_entry.grab_focus();
            url_entry.select_region(0, -1);
        }
    ));
    window.add_action(&action_url);
    app.set_accels_for_action("win.focus-url", &["<Ctrl>l"]);
}
