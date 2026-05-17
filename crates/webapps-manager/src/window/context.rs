use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use libadwaita as adw;

use relm4::component::Controller;
use webapps_core::models::BrowserCollection;

use crate::relm4_window::list::WebAppListController;

use super::state::SharedState;

/// Window-scoped context shared between handlers + actions.
///
/// `list` is the Relm4 controller that owns row rendering; calling
/// [`WebAppListController`] inputs replaces the old `populate_list` flow.
/// `content`/`status` are retained for backward compat with legacy paths but
/// are unused by the Relm4 list controller.
#[derive(Clone)]
pub(super) struct WindowContext {
    pub state: SharedState,
    pub browsers: Rc<RefCell<BrowserCollection>>,
    #[allow(dead_code)]
    pub content: Rc<gtk::Box>,
    pub window: Rc<adw::ApplicationWindow>,
    pub toast: Rc<adw::ToastOverlay>,
    #[allow(dead_code)]
    pub status: Rc<gtk::Label>,
    pub list: Rc<Controller<WebAppListController>>,
}

impl WindowContext {
    pub fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(3);
        self.toast.add_toast(toast);
    }
}
