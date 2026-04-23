mod actions;
mod context;
mod list;
mod shortcuts;
mod shortcuts_window;
mod state;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

#[allow(unused_imports)]
use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::{BrowserCollection, WebApp, WebAppCollection};

use crate::{geometry, service, ui_async, webapp_dialog, welcome_dialog};

use self::context::WindowContext;

const MAIN_WINDOW_GEOMETRY: &str = "manager-window.json";
const MAIN_DEFAULT_WIDTH: i32 = 800;
const MAIN_DEFAULT_HEIGHT: i32 = 650;

pub fn build(app: &adw::Application) {
    // migrate + load happen on a worker thread so a slow home (NFS, etc.)
    // can't delay the first paint. The window opens with an empty collection
    // and the real data is swapped in via `apply_webapps` once the worker
    // finishes.
    let state = state::new_empty_state();
    let browsers = Rc::new(RefCell::new(BrowserCollection::default()));

    let ui = ui::build_window(app);

    // Block "Add" until browser detection completes — clicking before would create
    // a webapp with browser="", silently producing a broken .desktop file.
    ui.add_btn.set_sensitive(false);
    ui.add_btn
        .set_tooltip_text(Some(&gettext("Detecting installed browsers…")));
    {
        let browsers = browsers.clone();
        let add_btn = ui.add_btn.clone();
        ui_async::run_with_result(service::detect_browsers, move |detected| {
            let has_any = !detected.browsers.is_empty();
            *browsers.borrow_mut() = detected;
            add_btn.set_sensitive(has_any);
            add_btn.set_tooltip_text(Some(&gettext(if has_any {
                "Add WebApp"
            } else {
                "No supported browser is installed"
            })));
        });
    }

    let context = WindowContext {
        state,
        browsers,
        content: Rc::new(ui.content_box),
        window: Rc::new(ui.window),
        toast: Rc::new(ui.toast_overlay),
        status: Rc::new(ui.status_label),
    };

    list::populate_list(&context);
    actions::install_window_actions(&context);

    // Kick off migration + initial load on a worker thread.
    {
        let context_for_load = context.clone();
        ui_async::run_with_result(
            || {
                let migrated = service::migrate_legacy_desktops();
                let webapps = service::load_webapps();
                (migrated, webapps)
            },
            move |(migrated, webapps): (usize, WebAppCollection)| {
                if migrated > 0 {
                    log::info!("Migrated {migrated} legacy webapps from .desktop files");
                }
                state::apply_webapps(&context_for_load.state, webapps);
                list::populate_list(&context_for_load);
            },
        );
    }

    {
        let context = context.clone();
        ui.search_entry.connect_search_changed(move |entry| {
            state::set_filter_text(&context.state, entry.text().to_string());
            list::populate_list(&context);
        });
    }

    {
        let context = context.clone();
        ui.add_btn.connect_clicked(move |_| {
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
                        list::refresh_and_render(&after_save);
                        after_save.show_toast(&gettext("WebApp created successfully"));
                    }
                },
            );
        });
    }

    shortcuts::install_shortcuts(app, &context.window, &ui.add_btn, &ui.search_btn);

    let geometry_path = geometry::geometry_path(MAIN_WINDOW_GEOMETRY);
    geometry::load_geometry(
        &*context.window,
        &geometry_path,
        MAIN_DEFAULT_WIDTH,
        MAIN_DEFAULT_HEIGHT,
    );
    context.window.connect_close_request(move |win| {
        geometry::save_geometry(win, &geometry_path);
        glib::Propagation::Proceed
    });

    context.window.present();
    welcome_dialog::show_if_needed(&context.window);
}
