//! Viewer window: builds and wires the main application window.
//!
//! Helpers are split into focused submodules:
//! - [`geometry`]    — persist/restore window size
//! - [`permissions`] — camera/mic/geolocation consent
//! - [`downloads`]   — download prompt + system notifications
//! - [`shortcuts`]   — keyboard shortcuts via GActions
//! - [`settings`]    — WebView settings + JS injection
mod chrome;
mod context_menu;
mod downloads;
mod geometry;
mod navigation;
mod permissions;
mod session;
mod settings;
mod shortcuts;
mod shortcuts_window;

#[allow(unused_imports)]
use adw::prelude::*;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::config;

/// Build and wire up the viewer window.
pub fn build(
    app: &adw::Application,
    url: &str,
    name: &str,
    icon: &str,
    app_id: &str,
    auto_hide_headerbar: bool,
) -> adw::ApplicationWindow {
    let viewer_session = session::build_viewer_session(app_id, url);
    let chrome = chrome::build_chrome(name, url, &viewer_session.webview);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(name)
        .default_width(1024)
        .default_height(720)
        .content(&chrome.toolbar)
        .build();

    if !icon.is_empty() {
        gtk::Window::set_default_icon_name(icon);
    }

    let config_path = config::config_dir().join(format!("{app_id}.json"));
    geometry::load_geometry(&window, &config_path);
    navigation::connect_url_entry(&chrome.url_entry, &chrome.url_bar, &viewer_session.webview);
    navigation::connect_navigation_controls(
        &window,
        &viewer_session.webview,
        &chrome.title_widget,
        &chrome.back_btn,
        &chrome.forward_btn,
        &chrome.reload_btn,
    );
    let is_fullscreen = navigation::connect_fullscreen(
        &window,
        &chrome.toolbar,
        &viewer_session.webview,
        &chrome.fullscreen_btn,
        auto_hide_headerbar,
    );
    downloads::connect_download_handlers(&window, &viewer_session.session, &viewer_session.webview);
    permissions::connect_permission_requests(
        &window,
        &viewer_session.webview,
        &viewer_session.data_dir.join("permissions.json"),
    );
    navigation::connect_new_window_requests(&viewer_session.webview);
    context_menu::setup_context_menu(&viewer_session.webview);

    shortcuts::setup_shortcuts(
        &window,
        &viewer_session.webview,
        &chrome.toolbar,
        &is_fullscreen,
        &chrome.url_bar,
        &chrome.url_entry,
    );

    window.connect_close_request(clone!(
        #[strong]
        config_path,
        move |win| {
            geometry::save_geometry(win, &config_path);
            glib::Propagation::Proceed
        }
    ));

    navigation::setup_fullscreen_reveal(&chrome.toolbar, &is_fullscreen, auto_hide_headerbar);
    if auto_hide_headerbar {
        chrome.toolbar.set_reveal_top_bars(false);
    }

    window
}
