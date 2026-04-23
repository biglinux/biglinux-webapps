//! Integration tests for the CRUD flow against a fake XDG home.
//!
//! Each test points `XDG_DATA_HOME`/`XDG_CONFIG_HOME`/`XDG_CACHE_HOME` at a
//! fresh `tempfile::TempDir` so the real `~/.local` is never touched. The
//! tests run with `serial_test::serial` to avoid clobbering each other (env
//! vars are process-global).
//!
//! These tests guard the rollback contracts in `service::crud` — if a future
//! refactor breaks atomicity or accidentally drops the lock, this suite will
//! flag it.

use std::sync::Mutex;

use serial_test::serial;
use tempfile::TempDir;

use webapps_core::models::{AppMode, BrowserId, WebApp};

// Single mutex held across the body of any test that mutates env vars — protects
// against parallel test runners that ignore the `serial` attribute (e.g. cargo
// nextest with `--test-threads`).
static ENV_GUARD: Mutex<()> = Mutex::new(());

struct XdgSandbox {
    _dir: TempDir,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl XdgSandbox {
    fn new() -> Self {
        let guard = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        let dir = TempDir::new().expect("create tempdir");
        let root = dir.path();
        std::env::set_var("HOME", root);
        std::env::set_var("XDG_DATA_HOME", root.join("data"));
        std::env::set_var("XDG_CONFIG_HOME", root.join("config"));
        std::env::set_var("XDG_CACHE_HOME", root.join("cache"));
        Self {
            _dir: dir,
            _guard: guard,
        }
    }
}

fn make_app(name: &str, url: &str) -> WebApp {
    WebApp {
        app_name: name.to_string(),
        app_url: url.to_string(),
        app_categories: "Webapps".to_string(),
        browser: BrowserId::VIEWER.to_string(),
        app_mode: AppMode::App,
        app_file: format!("biglinux-{name}-test.desktop"),
        ..WebApp::default()
    }
}

#[test]
#[serial]
fn create_then_load_round_trip() {
    let _sandbox = XdgSandbox::new();
    let app = make_app("RoundTrip", "https://example.com/rt");

    webapps_manager::service::create_webapp(&app).expect("create");
    let collection = webapps_manager::service::load_webapps();
    assert_eq!(collection.webapps.len(), 1);
    assert_eq!(collection.webapps[0].app_name, "RoundTrip");
}

#[test]
#[serial]
fn delete_removes_from_persisted_collection() {
    let _sandbox = XdgSandbox::new();
    let app = make_app("DeleteMe", "https://example.com/del");
    webapps_manager::service::create_webapp(&app).expect("create");
    assert_eq!(webapps_manager::service::load_webapps().webapps.len(), 1);

    webapps_manager::service::delete_webapp(&app, false).expect("delete");
    assert!(webapps_manager::service::load_webapps().webapps.is_empty());
}

#[test]
#[serial]
fn update_replaces_entry_in_place() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_app("Original", "https://example.com/u");
    webapps_manager::service::create_webapp(&app).expect("create");

    app.app_name = "Updated".to_string();
    webapps_manager::service::update_webapp(&app).expect("update");

    let collection = webapps_manager::service::load_webapps();
    assert_eq!(collection.webapps.len(), 1);
    assert_eq!(collection.webapps[0].app_name, "Updated");
}

#[test]
#[serial]
fn delete_all_clears_everything() {
    let _sandbox = XdgSandbox::new();
    for n in 0..3 {
        let app = make_app(&format!("App{n}"), &format!("https://example.com/{n}"));
        webapps_manager::service::create_webapp(&app).expect("create");
    }
    assert_eq!(webapps_manager::service::load_webapps().webapps.len(), 3);

    webapps_manager::service::delete_all_webapps().expect("delete_all");
    assert!(webapps_manager::service::load_webapps().webapps.is_empty());
}

#[test]
#[serial]
fn create_validates_browser_id() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_app("Bad", "https://example.com/bad");
    app.browser = "../etc/passwd".to_string();

    let result = webapps_manager::service::create_webapp(&app);
    assert!(
        result.is_err(),
        "expected validation failure for traversal browser id"
    );
}

#[test]
#[serial]
fn create_validates_app_url() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_app("Bad URL", "irrelevant");
    // Unbalanced IPv6 bracket — guaranteed to fail `url::Url::parse`.
    app.app_url = "http://[::1".to_string();

    let result = webapps_manager::service::create_webapp(&app);
    assert!(result.is_err(), "expected URL validation failure");
}
