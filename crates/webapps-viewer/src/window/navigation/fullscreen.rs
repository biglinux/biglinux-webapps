use std::cell::Cell;
use std::rc::Rc;

#[allow(unused_imports)]
use adw::prelude::*;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(crate) fn connect_fullscreen(
    window: &adw::ApplicationWindow,
    toolbar: &adw::ToolbarView,
    webview: &webkit::WebView,
    fullscreen_btn: &gtk::Button,
    auto_hide_headerbar: bool,
) -> Rc<Cell<bool>> {
    let is_fullscreen = Rc::new(Cell::new(false));

    webview.connect_enter_fullscreen(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        #[upgrade_or]
        false,
        move |_| {
            is_fullscreen.set(true);
            toolbar.set_reveal_top_bars(false);
            window.fullscreen();
            true
        }
    ));

    webview.connect_leave_fullscreen(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        #[upgrade_or]
        false,
        move |_| {
            is_fullscreen.set(false);
            toolbar.set_reveal_top_bars(!auto_hide_headerbar);
            window.unfullscreen();
            true
        }
    ));

    fullscreen_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_| {
            if is_fullscreen.get() {
                is_fullscreen.set(false);
                toolbar.set_reveal_top_bars(!auto_hide_headerbar);
                window.unfullscreen();
            } else {
                is_fullscreen.set(true);
                toolbar.set_reveal_top_bars(false);
                window.fullscreen();
            }
        }
    ));

    is_fullscreen
}

pub(crate) fn setup_fullscreen_reveal(
    toolbar: &adw::ToolbarView,
    is_fullscreen: &Rc<Cell<bool>>,
    auto_hide_headerbar: bool,
) {
    toolbar.set_top_bar_style(adw::ToolbarStyle::Raised);

    let motion = gtk::EventControllerMotion::new();
    motion.connect_motion(clone!(
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_, _x, y| {
            if is_fullscreen.get() || auto_hide_headerbar {
                toolbar.set_reveal_top_bars(y < 50.0);
            }
        }
    ));
    toolbar.add_controller(motion);
}
