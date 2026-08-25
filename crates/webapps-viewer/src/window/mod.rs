//! Viewer window: builds and wires the main application window.
//!
//! Helpers are split into focused submodules:
//! - [`geometry`]    — persist/restore window size
//! - [`permissions`] — camera/mic/geolocation consent
//! - [`downloads`]   — download prompt + system notifications
//! - [`shortcuts`]   — keyboard shortcuts via GActions
//! - [`settings`]    — WebView settings + JS injection
mod chrome;
mod component;
mod context_menu;
mod cookie_migration;
mod downloads;
mod geometry;
mod loading;
mod navigation;
mod permissions;
mod session;
mod settings;
mod shortcuts;
mod shortcuts_window;
mod startup;

use std::cell::RefCell;

use adw::prelude::*;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use relm4::{Component, ComponentController, Controller};

use self::component::{ViewerInit, ViewerWindow};

thread_local! {
    /// Keepalive for viewer window controllers: parked for the window's
    /// lifetime, dropped on window destroy (tears the component tree down) —
    /// no `unsafe` window `set_data`.
    static VIEWER_CONTROLLERS:
        RefCell<Vec<(glib::WeakRef<adw::ApplicationWindow>, Controller<ViewerWindow>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Build the viewer window: create the `adw::ApplicationWindow` and mount the
/// Relm4 [`ViewerWindow`] component (which owns the WebView, chrome, and all
/// navigation/permission/download/shortcut wiring).
pub fn build(
    app: &adw::Application,
    url: &str,
    name: &str,
    icon: &str,
    app_id: &str,
    auto_hide_headerbar: bool,
) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(name)
        .default_width(1024)
        .default_height(720)
        .build();

    let controller = ViewerWindow::builder()
        .launch(ViewerInit {
            window: window.clone(),
            url: url.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            app_id: app_id.to_string(),
            auto_hide_headerbar,
        })
        .detach();
    window.set_content(Some(controller.widget()));

    VIEWER_CONTROLLERS.with(|reg| reg.borrow_mut().push((window.downgrade(), controller)));
    {
        let window_weak = window.downgrade();
        window.connect_destroy(move |_| {
            let Some(closed) = window_weak.upgrade() else {
                return;
            };
            VIEWER_CONTROLLERS.with(|reg| {
                reg.borrow_mut()
                    .retain(|(w, _)| w.upgrade().is_some_and(|w| w != closed));
            });
        });
    }

    window
}
