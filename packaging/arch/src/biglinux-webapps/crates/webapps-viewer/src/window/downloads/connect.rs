use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(crate) fn connect_download_handlers(
    window: &adw::ApplicationWindow,
    session: &webkit::NetworkSession,
    webview: &webkit::WebView,
) {
    let weak_window = window.downgrade();
    session.connect_download_started(move |_session, download| {
        if let Some(window) = weak_window.upgrade() {
            super::handle_download(&window, download);
        }
    });

    let weak_window = window.downgrade();
    webview.connect_show_notification(move |_wv, notification| {
        if let Some(window) = weak_window.upgrade() {
            super::show_notification(&window, notification);
            return true;
        }
        false
    });
}
