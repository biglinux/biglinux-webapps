use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::WebApp;

use crate::{service, ui_async};

use super::super::ui::DialogWidgets;
use super::super::validation::SaveValidationError;
use super::super::{validation, DialogResult};

pub(crate) fn setup_cancel_handler(widgets: &DialogWidgets) {
    let dialog = widgets.dialog.clone();
    widgets.cancel_button.connect_clicked(move |_| {
        dialog.close();
    });
}

pub(crate) fn setup_save_handler(
    widgets: &DialogWidgets,
    webapp_cell: Rc<RefCell<WebApp>>,
    is_new: bool,
    on_done: impl Fn(DialogResult) + 'static,
) {
    let dialog = widgets.dialog.clone();
    let url_row = widgets.url_row.clone();
    let name_row = widgets.name_row.clone();
    let profile_entry = widgets.profile_entry.clone();
    let save_button = widgets.save_button.clone();
    let cancel_button = widgets.cancel_button.clone();
    let spinner_box = widgets.spinner_box.clone();
    // on_done fires at most once; wrap it so it can be moved into the worker
    // callback without Fn/FnOnce conflicts.
    let on_done = Rc::new(on_done);
    widgets.save_button.connect_clicked(move |_| {
        clear_error_state(&url_row, &name_row, &profile_entry);

        let app = webapp_cell.borrow().clone();
        let app = match validation::validate_for_save(&app) {
            Ok(app) => app,
            Err(SaveValidationError::EmptyName) => {
                name_row.add_css_class("error");
                name_row.grab_focus();
                return;
            }
            Err(SaveValidationError::EmptyUrl | SaveValidationError::InvalidUrl) => {
                url_row.add_css_class("error");
                url_row.grab_focus();
                return;
            }
            Err(SaveValidationError::InvalidProfile(message)) => {
                profile_entry.add_css_class("error");
                profile_entry.set_tooltip_text(Some(&message));
                profile_entry.grab_focus();
                return;
            }
        };

        save_button.set_sensitive(false);
        cancel_button.set_sensitive(false);
        spinner_box.set_visible(true);

        let dialog = dialog.clone();
        let save_button = save_button.clone();
        let cancel_button = cancel_button.clone();
        let spinner_box = spinner_box.clone();
        let on_done = on_done.clone();
        let app_for_log = app.clone();
        ui_async::run_with_result(
            move || {
                if is_new {
                    service::create_webapp(&app)
                } else {
                    service::update_webapp(&app)
                }
            },
            move |result| {
                spinner_box.set_visible(false);
                match result {
                    Ok(()) => {
                        log::info!(
                            "Saved webapp '{}' mode={:?}",
                            app_for_log.app_name,
                            app_for_log.app_mode
                        );
                        dialog.close();
                        on_done(DialogResult { saved: true });
                    }
                    Err(err) => {
                        log::error!("Save webapp failed: {err}");
                        save_button.set_sensitive(true);
                        cancel_button.set_sensitive(true);
                        reveal_save_error(&dialog, &gettext("Failed to save webapp"));
                    }
                }
            },
        );
    });
}

pub(crate) fn setup_destroy_handler(
    dialog: &adw::Dialog,
    debounce_handle: Rc<RefCell<Option<glib::SourceId>>>,
) {
    // Cancel a pending detect-debounce when the dialog closes without saving —
    // otherwise the timeout fires after the widget tree is torn down and tries
    // to mutate already-dropped widgets.
    dialog.connect_closed(move |_| {
        if let Some(id) = debounce_handle.borrow_mut().take() {
            id.remove();
        }
    });
}

fn clear_error_state(
    url_row: &adw::EntryRow,
    name_row: &adw::EntryRow,
    profile_entry: &adw::EntryRow,
) {
    url_row.remove_css_class("error");
    name_row.remove_css_class("error");
    profile_entry.remove_css_class("error");
    profile_entry.set_tooltip_text(None);
}

fn reveal_save_error(dialog: &adw::Dialog, message: &str) {
    let Some(child) = dialog.child() else {
        return;
    };
    let Ok(container) = child.downcast::<gtk::Box>() else {
        return;
    };

    // The outer Box from shell.rs is [HeaderBar, Overlay]; insert the banner
    // between them. If an AdwBanner is already present (second child), reuse it.
    if let Some(banner) = container
        .first_child()
        .and_then(|first| first.next_sibling())
        .and_then(|second| second.downcast::<adw::Banner>().ok())
    {
        banner.set_title(message);
        banner.set_revealed(true);
        return;
    }

    let banner = adw::Banner::new(message);
    banner.set_revealed(true);
    let header = container.first_child();
    container.insert_child_after(&banner, header.as_ref());
}
