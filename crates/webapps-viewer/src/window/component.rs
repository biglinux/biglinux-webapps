//! `ViewerWindow` — the WebApp viewer window as a Relm4 [`Component`].
//!
//! ADR-D14 mountable content: the component `Root` is the `adw::ToolbarView`
//! chrome (header + URL bar + the WebKit `WebView` wrapped in a loading
//! overlay), never the window. The launcher [`super::build`] creates the
//! `adw::ApplicationWindow` and mounts `controller.widget()`.
//!
//! Display-only shell: navigation (back/forward/reload/url/fullscreen),
//! downloads, permissions, context menu, and shortcuts are wired directly onto
//! the model-owned `WebView` (in-component view wiring — the WebView is a raw
//! gtk widget with no Relm4 equivalent, embedded like vte4/GLArea). There are
//! no user-action messages, so [`ViewerInput`] is uninhabited.

use adw::prelude::*;
use gtk::glib::{self, clone};
use gtk4 as gtk;
use libadwaita as adw;
use relm4::{Component, ComponentParts, ComponentSender};

use webapps_core::config;

use super::session::ViewerSession;
use super::{
    chrome, context_menu, downloads, geometry, navigation, permissions, session, shortcuts, startup,
};

/// What the launcher hands the component: the window it mounts into plus the
/// CLI-derived view parameters.
pub(super) struct ViewerInit {
    pub window: adw::ApplicationWindow,
    pub url: String,
    pub name: String,
    pub icon: String,
    pub app_id: String,
    pub auto_hide_headerbar: bool,
}

/// No user-action messages — navigation is wired onto the WebView directly.
#[derive(Debug)]
pub(super) enum ViewerInput {}

pub(super) struct ViewerWindow {
    /// Keepalive: the WebKit `NetworkSession` + data dir must outlive `init`
    /// (the `WebView` itself is owned by the widget tree under the root).
    _session: ViewerSession,
}

impl Component for ViewerWindow {
    type Init = ViewerInit;
    type Input = ViewerInput;
    type Output = ();
    type CommandOutput = ();
    /// Mountable content (ADR-D14): the toolbar chrome, never the window.
    type Root = adw::ToolbarView;
    type Widgets = ();

    fn init_root() -> Self::Root {
        adw::ToolbarView::new()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ViewerInit {
            window,
            url,
            name,
            icon,
            app_id,
            auto_hide_headerbar,
        } = init;

        let viewer_session = session::build_viewer_session(&app_id);
        let chrome = chrome::build_chrome(&root, &name, &url, &viewer_session.webview);

        if !icon.is_empty() {
            gtk::Window::set_default_icon_name(&icon);
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
            &root,
            &viewer_session.webview,
            &chrome.fullscreen_btn,
            auto_hide_headerbar,
        );
        downloads::connect_download_handlers(
            &window,
            &viewer_session.session,
            &viewer_session.webview,
        );
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
            &root,
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

        navigation::setup_fullscreen_reveal(&root, &is_fullscreen, auto_hide_headerbar);
        if auto_hide_headerbar {
            root.set_reveal_top_bars(false);
        }

        startup::connect_initial_load(&window, &viewer_session.webview, &url);

        ComponentParts {
            model: ViewerWindow {
                _session: viewer_session,
            },
            widgets: (),
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {}
    }
}
