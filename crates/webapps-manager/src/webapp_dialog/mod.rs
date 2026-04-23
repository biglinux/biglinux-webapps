mod handlers;
mod tasks;
mod ui;
mod validation;

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use webapps_core::models::{BrowserCollection, WebApp};
use webapps_core::templates::default_registry;

use self::ui::build_dialog;

pub struct DialogResult {
    pub saved: bool,
}

pub fn show(
    parent: &impl IsA<gtk::Widget>,
    webapp: WebApp,
    browsers: Rc<RefCell<BrowserCollection>>,
    is_new: bool,
    on_done: impl Fn(DialogResult) + 'static,
) {
    let webapp_cell = Rc::new(RefCell::new(webapp));
    let widgets = build_dialog(&webapp_cell.borrow(), is_new, browsers.clone());

    let skip_auto_detect = Rc::new(Cell::new(false));
    let debounce_handle: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let drm_required = {
        let data = webapp_cell.borrow();
        Rc::new(Cell::new(
            default_registry().requires_drm(&data.template_id, &data.app_url),
        ))
    };

    handlers::setup_url_handler(
        &widgets,
        webapp_cell.clone(),
        skip_auto_detect.clone(),
        debounce_handle.clone(),
    );
    handlers::setup_name_handler(&widgets, webapp_cell.clone());
    handlers::setup_category_handler(&widgets, webapp_cell.clone());
    handlers::setup_browser_handler(
        &widgets,
        webapp_cell.clone(),
        browsers.clone(),
        drm_required.clone(),
    );
    handlers::setup_profile_handlers(&widgets, webapp_cell.clone());
    handlers::setup_detection_handler(&widgets, webapp_cell.clone());
    handlers::setup_favicon_picker(&widgets, webapp_cell.clone());
    handlers::setup_icon_picker(&widgets, webapp_cell.clone());
    handlers::setup_cancel_handler(&widgets);
    handlers::setup_save_handler(&widgets, webapp_cell.clone(), is_new, on_done);
    handlers::setup_destroy_handler(&widgets.dialog, debounce_handle);

    widgets.dialog.present(Some(parent));
    if is_new {
        widgets.url_row.grab_focus();
    } else {
        widgets.name_row.grab_focus();
    }
}
