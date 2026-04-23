use std::path::PathBuf;

use webkit6 as webkit;
use webkit6::prelude::*;

use webapps_core::config;

use super::settings;

pub(super) struct ViewerSession {
    pub session: webkit::NetworkSession,
    pub webview: webkit::WebView,
    pub data_dir: PathBuf,
}

pub(super) fn build_viewer_session(app_id: &str, url: &str) -> ViewerSession {
    let data_dir = config::data_dir().join(app_id);
    let cache_dir = config::cache_dir().join(app_id);
    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(&cache_dir).ok();

    let session = webkit::NetworkSession::new(
        Some(data_dir.to_str().unwrap_or_default()),
        Some(cache_dir.to_str().unwrap_or_default()),
    );

    session.set_itp_enabled(false);
    if let Some(cookie_manager) = session.cookie_manager() {
        let cookie_db = data_dir.join("webkit-cookies.db");
        cookie_manager.set_persistent_storage(
            cookie_db.to_str().unwrap_or_default(),
            webkit::CookiePersistentStorage::Sqlite,
        );
        cookie_manager.set_accept_policy(webkit::CookieAcceptPolicy::Always);
    }

    let webview = webkit::WebView::builder().network_session(&session).build();
    settings::configure_settings(&webview);
    settings::inject_resize_block(&webview);
    webview.load_uri(url);
    webview.set_vexpand(true);
    webview.set_hexpand(true);

    ViewerSession {
        session,
        webview,
        data_dir,
    }
}
