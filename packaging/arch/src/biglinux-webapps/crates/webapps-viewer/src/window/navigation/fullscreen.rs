use std::cell::Cell;
use std::rc::Rc;

#[allow(unused_imports)]
use adw::prelude::*;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

const HIDDEN_REVEAL_EDGE_PX: f64 = 6.0;
const REVEALED_KEEP_PX: f64 = 72.0;

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
                toolbar.set_reveal_top_bars(should_reveal_top_bars(toolbar.reveals_top_bars(), y));
            }
        }
    ));
    toolbar.add_controller(motion);
}

fn should_reveal_top_bars(currently_revealed: bool, y: f64) -> bool {
    if currently_revealed {
        y < REVEALED_KEEP_PX
    } else {
        y <= HIDDEN_REVEAL_EDGE_PX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_header_reveals_only_at_top_edge() {
        assert!(should_reveal_top_bars(false, 0.0));
        assert!(should_reveal_top_bars(false, HIDDEN_REVEAL_EDGE_PX));
        assert!(!should_reveal_top_bars(false, HIDDEN_REVEAL_EDGE_PX + 1.0));
    }

    #[test]
    fn revealed_header_stays_open_while_pointer_is_over_header_area() {
        assert!(should_reveal_top_bars(true, 40.0));
        assert!(!should_reveal_top_bars(true, REVEALED_KEEP_PX + 1.0));
    }
}
