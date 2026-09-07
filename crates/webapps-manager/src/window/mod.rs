//! Manager main window.
//!
//! The window content is the Relm4 `ManagerWindow` component (actions, list,
//! search, toasts, shortcuts live there). This module's [`build`] is the
//! launcher: it creates the `adw::ApplicationWindow`, mounts the component, and
//! owns geometry + presentation.

mod actions;
mod component;
mod context;
mod list;
mod shortcuts;
mod shortcuts_window;
mod state;
mod ui;

use std::cell::RefCell;

use adw::prelude::*;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use relm4::{Component, ComponentController, Controller};

use crate::{geometry, welcome_dialog};

use self::component::{ManagerInit, ManagerWindow};

const MAIN_WINDOW_GEOMETRY: &str = "manager-window.json";

thread_local! {
    /// Keepalive for the manager window controller: parked for the window's
    /// lifetime and dropped (tearing the component tree down) on window destroy
    /// — no `unsafe` window `set_data`, lifetime tied through this registry.
    static MANAGER_CONTROLLERS:
        RefCell<Vec<(glib::WeakRef<adw::ApplicationWindow>, Controller<ManagerWindow>)>> =
        const { RefCell::new(Vec::new()) };
}

pub fn build(app: &adw::Application) {
    let shell = crate::relm4_window::shell_spec::build();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(&shell.title)
        .css_classes(["biglinux-webapps"])
        .default_width(shell.default_width)
        .default_height(shell.default_height)
        .build();

    // The window content is the Relm4 ManagerWindow component (chrome + list +
    // actions + search + shortcuts). It owns the WindowContext; we own the
    // window, geometry, and presentation here.
    let controller = ManagerWindow::builder()
        .launch(ManagerInit {
            app: app.clone(),
            window: window.clone(),
        })
        .detach();
    window.set_content(Some(controller.widget()));

    let geometry_path = geometry::geometry_path(MAIN_WINDOW_GEOMETRY);
    geometry::load_geometry(
        &window,
        &geometry_path,
        shell.default_width,
        shell.default_height,
    );
    {
        let geometry_path = geometry_path.clone();
        window.connect_close_request(move |win| {
            geometry::save_geometry(win, &geometry_path);
            glib::Propagation::Proceed
        });
    }

    // Park the controller for the window's lifetime; drop it on destroy so the
    // component tree tears down with the window.
    MANAGER_CONTROLLERS.with(|reg| reg.borrow_mut().push((window.downgrade(), controller)));
    {
        let window_weak = window.downgrade();
        window.connect_destroy(move |_| {
            let Some(closed) = window_weak.upgrade() else {
                return;
            };
            MANAGER_CONTROLLERS.with(|reg| {
                reg.borrow_mut()
                    .retain(|(w, _)| w.upgrade().is_some_and(|w| w != closed));
            });
        });
    }

    window.present();
    welcome_dialog::show_if_needed(&window);
}
