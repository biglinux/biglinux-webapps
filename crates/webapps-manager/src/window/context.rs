use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::BrowserCollection;

use super::state::SharedState;

#[derive(Clone)]
pub(super) struct WindowContext {
    pub state: SharedState,
    pub browsers: Rc<RefCell<BrowserCollection>>,
    pub content: Rc<gtk::Box>,
    pub window: Rc<adw::ApplicationWindow>,
    pub toast: Rc<adw::ToastOverlay>,
    pub status: Rc<gtk::Label>,
}

impl WindowContext {
    pub fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(3);
        self.toast.add_toast(toast);
    }
}
