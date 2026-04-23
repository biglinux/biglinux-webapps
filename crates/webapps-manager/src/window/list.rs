use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::{AppMode, BrowserId, WebApp};
use webapps_core::templates::default_registry;

use crate::{browser_dialog, service, ui_async, webapp_dialog, webapp_row};

use super::context::WindowContext;
use super::state;

pub(super) fn refresh_and_render(context: &WindowContext) {
    state::refresh_state(&context.state);
    populate_list(context);
}

pub(super) fn populate_list(context: &WindowContext) {
    clear_container(&context.content);

    let sections = state::sections_snapshot(&context.state);
    if sections.is_empty() {
        context.content.append(&build_empty_state(context));
    } else {
        for section in sections {
            context
                .content
                .append(&build_section_group(&section, context));
        }
    }

    if state::has_active_filter(&context.state) {
        let label = gettext("{} results");
        context
            .status
            .set_label(&label.replace("{}", &state::result_count(&context.state).to_string()));
        context.status.set_visible(true);
    } else {
        context.status.set_label("");
        context.status.set_visible(false);
    }
}

fn clear_container(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn build_empty_state(context: &WindowContext) -> adw::StatusPage {
    let status_page = adw::StatusPage::builder()
        .icon_name("big-webapps")
        .title(gettext("No WebApps yet"))
        .description(gettext(
            "Turn any website into a desktop app. Press Add to get started.",
        ))
        .vexpand(true)
        .build();
    status_page.add_css_class("empty-state-icon");

    let cta = gtk::Button::with_label(&gettext("Add WebApp"));
    cta.add_css_class("pill");
    cta.add_css_class("suggested-action");
    cta.set_halign(gtk::Align::Center);
    {
        let context = context.clone();
        cta.connect_clicked(move |_| open_add_dialog(&context));
    }
    status_page.set_child(Some(&cta));
    status_page
}

fn build_section_group(
    section: &state::WebAppSection,
    context: &WindowContext,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();
    group.set_title(&section.title);

    let callbacks = row_callbacks(context);
    for app in &section.apps {
        group.add(&webapp_row::build_row(app, &callbacks));
    }
    group
}

fn row_callbacks(context: &WindowContext) -> Rc<webapp_row::RowCallbacks> {
    Rc::new(webapp_row::RowCallbacks {
        on_edit: {
            let context = context.clone();
            Box::new(move |app| handle_edit(context.clone(), app))
        },
        on_browser: {
            let context = context.clone();
            Box::new(move |app| handle_browser_change(context.clone(), app))
        },
        on_delete: {
            let context = context.clone();
            Box::new(move |app| handle_delete(context.clone(), app))
        },
    })
}

fn open_add_dialog(context: &WindowContext) {
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

fn handle_edit(context: WindowContext, app: &WebApp) {
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

fn handle_browser_change(context: WindowContext, app: &WebApp) {
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
                        after_change.show_toast(&format!("Failed: {err}"));
                    }
                },
            );
        },
    );
}

fn handle_delete(context: WindowContext, app: &WebApp) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettext("Remove WebApp?"))
        .body(format!("{}\n{}", app.app_name, app.app_url))
        .build();
    dialog.add_response("cancel", &gettext("Cancel"));
    dialog.add_response("delete", &gettext("Remove"));
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

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
                        after_delete.show_toast(&format!("Failed: {err}"));
                    }
                },
            );
        }
    });
    dialog.present(Some(&*context.window));
}
