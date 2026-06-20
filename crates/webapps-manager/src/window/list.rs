//! Bridge between the (still-shared) `WindowContext` and the Relm4
//! [`WebAppListController`].
//!
//! After the onda6 migration, list rendering is driven by Relm4; this module
//! is just a thin shim that converts `state::sections_snapshot` into a
//! `WebAppListInput::Refresh` message. Row-action handlers (`handle_edit`,
//! `handle_browser_change`, `handle_delete`) are unchanged and still target
//! the existing dialogs.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use big_app_kit::dialogs;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use relm4::ComponentController;
use webapps_core::models::{AppMode, BrowserId, WebApp, WebAppCollection};
use webapps_core::templates::default_registry;

use crate::relm4_window::list::{WebAppListInput, WebAppSection as Relm4Section};
use crate::{browser_dialog, service, ui_async, webapp_dialog};

use super::context::WindowContext;
use super::state;

/// Reload the webapp set after a mutation and repaint the list.
///
/// `service::load_webapps` does blocking file I/O (read + advisory lock), so it
/// runs on a worker thread; the result is applied on the main thread via
/// `state::apply_webapps` (no disk hit). A `reload_generation` guard drops a
/// result that finished after a newer `refresh_and_render` was issued, so rapid
/// save/delete sequences completing out of order can't repaint stale data.
pub(super) fn refresh_and_render(context: &WindowContext) {
    let generation = context.reload_generation.get().wrapping_add(1);
    context.reload_generation.set(generation);

    let context = context.clone();
    ui_async::run_with_result(service::load_webapps, move |webapps: WebAppCollection| {
        if context.reload_generation.get() != generation {
            return;
        }
        state::apply_webapps(&context.state, webapps);
        populate_list(&context);
    });
}

/// Push the current state snapshot into the Relm4 list controller.
pub(super) fn populate_list(context: &WindowContext) {
    let sections = state::sections_snapshot(&context.state);
    let relm4_sections: Vec<Relm4Section> = sections
        .into_iter()
        .map(|s| Relm4Section {
            title: s.title,
            apps: s.apps,
        })
        .collect();

    let has_filter = state::has_active_filter(&context.state);
    let result_count = state::result_count(&context.state);

    context.list.sender().emit(WebAppListInput::Refresh {
        sections: relm4_sections,
        has_filter,
        result_count,
    });
}

pub(super) fn open_add_dialog(context: &WindowContext) {
    let mut new_app = WebApp::default();
    new_app.app_file = service::generate_app_file(&new_app.browser, &new_app.app_url);
    if let Some(default_browser) = context.browsers.borrow().default_browser() {
        new_app.browser = default_browser.browser_id.clone();
    }

    let after_save = context.clone();
    webapp_dialog::show(
        &*context.window,
        new_app,
        context.browsers.clone(),
        true,
        move |result| {
            if result.saved {
                refresh_and_render(&after_save);
                after_save.show_toast(&gettext("WebApp created successfully"));
            }
        },
    );
}

/// Open the curated template gallery; the chosen template seeds a fresh
/// webapp that is then opened in the standard create dialog for review/edit.
pub(super) fn open_template_gallery(context: &WindowContext) {
    let parent = context.window.clone();
    let context = context.clone();
    crate::template_gallery::show(&*parent, move |template_id| {
        let mut new_app = WebApp::default();
        if let Some(tpl) = default_registry().get(&template_id) {
            new_app.apply_template(tpl);
        }
        if let Some(default_browser) = context.browsers.borrow().default_browser() {
            new_app.browser = default_browser.browser_id.clone();
        }
        new_app.app_file = service::generate_app_file(&new_app.browser, &new_app.app_url);

        let after_save = context.clone();
        webapp_dialog::show(
            &*context.window,
            new_app,
            context.browsers.clone(),
            true,
            move |result| {
                if result.saved {
                    refresh_and_render(&after_save);
                    after_save.show_toast(&gettext("WebApp created successfully"));
                }
            },
        );
    });
}

pub(super) fn handle_edit(context: WindowContext, app: &WebApp) {
    let browsers = context.browsers.clone();
    let after_save = context.clone();
    webapp_dialog::show(
        &*context.window,
        app.clone(),
        browsers,
        false,
        move |result| {
            if result.saved {
                refresh_and_render(&after_save);
                after_save.show_toast(&gettext("WebApp updated successfully"));
            }
        },
    );
}

pub(super) fn handle_browser_change(context: WindowContext, app: &WebApp) {
    let browsers = context.browsers.borrow().clone();
    let app_cell = Rc::new(RefCell::new(app.clone()));
    let after_change = context.clone();
    let allow_viewer = !default_registry().requires_drm(&app.template_id, &app.app_url);
    browser_dialog::show(
        &*context.window,
        &browsers,
        &app.browser,
        app.auto_hide_headerbar,
        allow_viewer,
        move |selection| {
            {
                let mut app = app_cell.borrow_mut();
                app.auto_hide_headerbar = selection.auto_hide_headerbar;
                if selection.browser_id == BrowserId::VIEWER {
                    app.browser = BrowserId::VIEWER.to_string();
                    app.app_mode = AppMode::App;
                } else {
                    app.browser = selection.browser_id;
                    app.app_mode = AppMode::Browser;
                }
            }
            let updated = app_cell.borrow().clone();
            let after_change = after_change.clone();
            ui_async::run_with_result(
                move || service::update_webapp(&updated),
                move |result| match result {
                    Ok(()) => {
                        refresh_and_render(&after_change);
                        after_change.show_toast(&gettext("Browser changed"));
                    }
                    Err(err) => {
                        after_change
                            .show_toast(&format!("{}: {err}", gettext("Browser change failed")));
                    }
                },
            );
        },
    );
}

pub(super) fn handle_delete(context: WindowContext, app: &WebApp) {
    let dialog = dialogs::confirm_dialog(
        &gettext("Remove WebApp?"),
        &format!("{}\n{}", app.app_name, app.app_url),
        &gettext("Cancel"),
        "delete",
        &gettext("Remove"),
        true,
    );

    let delete_profile = Rc::new(RefCell::new(false));
    let has_profile = app.has_custom_profile();
    let owns_profile = !service::profile_shared(app);
    if has_profile && owns_profile {
        let check = gtk::CheckButton::with_label(&gettext("Also delete configuration folder"));
        let delete_profile_ref = delete_profile.clone();
        check.connect_toggled(move |button| {
            *delete_profile_ref.borrow_mut() = button.is_active();
        });
        dialog.set_extra_child(Some(&check));
    }

    let app = app.clone();
    let after_delete = context.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            let should_delete_profile = *delete_profile.borrow();
            let app_for_worker = app.clone();
            let after_delete = after_delete.clone();
            ui_async::run_with_result(
                move || service::delete_webapp(&app_for_worker, should_delete_profile),
                move |result| match result {
                    Ok(()) => {
                        refresh_and_render(&after_delete);
                        after_delete.show_toast(&gettext("WebApp removed"));
                    }
                    Err(err) => {
                        after_delete.show_toast(&format!("{}: {err}", gettext("Remove failed")));
                    }
                },
            );
        }
    });
    dialog.present(Some(&*context.window));
}
