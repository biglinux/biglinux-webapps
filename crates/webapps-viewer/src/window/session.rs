use std::path::PathBuf;

use gdk4 as gdk;
use webkit6 as webkit;
use webkit6::prelude::*;

use webapps_core::config;

use super::settings;

const WEB_PROCESS_MEMORY_LIMIT_MB: u32 = 1024;
const NETWORK_PROCESS_MEMORY_LIMIT_MB: u32 = 512;
const MEMORY_PRESSURE_CONSERVATIVE_THRESHOLD: f64 = 0.50;
const MEMORY_PRESSURE_STRICT_THRESHOLD: f64 = 0.75;
const MEMORY_PRESSURE_POLL_INTERVAL_SECONDS: f64 = 30.0;

pub(super) struct ViewerSession {
    pub session: webkit::NetworkSession,
    pub webview: webkit::WebView,
    pub data_dir: PathBuf,
}

pub(super) fn build_viewer_session(app_id: &str) -> ViewerSession {
    let data_dir = config::data_dir().join(app_id);
    let cache_dir = config::cache_dir().join(app_id);
    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(&cache_dir).ok();

    let mut network_pressure = memory_pressure_settings(NETWORK_PROCESS_MEMORY_LIMIT_MB);
    webkit::NetworkSession::set_memory_pressure_settings(&mut network_pressure);

    let web_pressure = memory_pressure_settings(WEB_PROCESS_MEMORY_LIMIT_MB);
    let web_context = webkit::WebContext::builder()
        .memory_pressure_settings(&web_pressure)
        .build();
    web_context.set_cache_model(webkit::CacheModel::DocumentBrowser);

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

    let webview = webkit::WebView::builder()
        .web_context(&web_context)
        .network_session(&session)
        .build();
    let background = gdk::RGBA::new(0.012, 0.014, 0.030, 1.0);
    webview.set_background_color(&background);
    settings::configure_settings(&webview);
    settings::inject_resize_block(&webview);
    webview.set_vexpand(true);
    webview.set_hexpand(true);

    ViewerSession {
        session,
        webview,
        data_dir,
    }
}

fn memory_pressure_settings(limit_mb: u32) -> webkit::MemoryPressureSettings {
    let mut pressure = webkit::MemoryPressureSettings::new();
    pressure.set_memory_limit(limit_mb);
    // Order matters: `set_conservative_threshold` asserts the new value is below
    // the *current* strict threshold, so strict must be raised to its target
    // (0.75) before conservative (0.50) is applied — otherwise the conservative
    // value is compared against webkit's lower default strict fraction and a
    // GLib CRITICAL fires on every launch.
    pressure.set_strict_threshold(MEMORY_PRESSURE_STRICT_THRESHOLD);
    pressure.set_conservative_threshold(MEMORY_PRESSURE_CONSERVATIVE_THRESHOLD);
    pressure.set_kill_threshold(0.0);
    pressure.set_poll_interval(MEMORY_PRESSURE_POLL_INTERVAL_SECONDS);
    pressure
}
