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

use std::{fs, sync::Mutex};

use serial_test::serial;
use tempfile::TempDir;

use webapps_core::config;
use webapps_core::models::{AppMode, BrowserId, WebApp, WebAppCollection};

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

fn make_browser_app(name: &str, url: &str) -> WebApp {
    WebApp {
        app_name: name.to_string(),
        app_url: url.to_string(),
        app_categories: "Webapps".to_string(),
        browser: "brave".to_string(),
        app_mode: AppMode::Browser,
        app_file: "old-browser-entry.desktop".to_string(),
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
fn app_mode_create_uses_path_aware_desktop_name() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_app("Notes", "https://cloud.talesam.org/apps/notes");
    app.app_file.clear();

    webapps_manager::service::create_webapp(&app).expect("create");

    let collection = webapps_manager::service::load_webapps();
    assert_eq!(collection.webapps.len(), 1);
    let saved = &collection.webapps[0];
    assert_eq!(
        saved.app_file,
        webapps_core::desktop::viewer_desktop_filename("https://cloud.talesam.org/apps/notes")
    );
}

#[test]
#[serial]
fn delete_removes_from_persisted_collection() {
    let _sandbox = XdgSandbox::new();
    let app = make_app("DeleteMe", "https://example.com/del");
    webapps_manager::service::create_webapp(&app).expect("create");
    assert_eq!(webapps_manager::service::load_webapps().webapps.len(), 1);
    let saved = webapps_manager::service::load_webapps()
        .webapps
        .into_iter()
        .next()
        .expect("saved app");

    webapps_manager::service::delete_webapp(&saved, false).expect("delete");
    assert!(webapps_manager::service::load_webapps().webapps.is_empty());
}

#[test]
#[serial]
fn update_replaces_entry_in_place() {
    let _sandbox = XdgSandbox::new();
    let app = make_app("Original", "https://example.com/u");
    webapps_manager::service::create_webapp(&app).expect("create");
    let mut app = webapps_manager::service::load_webapps()
        .webapps
        .into_iter()
        .next()
        .expect("saved app");

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

#[test]
#[serial]
fn browser_create_uses_chromium_app_id_desktop_name() {
    let _sandbox = XdgSandbox::new();
    let app = make_browser_app("Browser", "http://127.0.0.1:9/browser");

    webapps_manager::service::create_webapp(&app).expect("create");

    let collection = webapps_manager::service::load_webapps();
    assert_eq!(collection.webapps.len(), 1);
    let saved = &collection.webapps[0];
    assert_eq!(saved.app_file, "brave-127.0.0.1__browser-Default.desktop");

    let desktop = config::applications_dir().join(&saved.app_file);
    let content = std::fs::read_to_string(desktop).expect("desktop entry");
    assert!(content.contains("StartupWMClass=brave-127.0.0.1__browser-Default"));
    assert!(content.contains("--class=\"brave-127.0.0.1__browser-Default\""));
}

#[test]
#[serial]
fn delete_browser_default_profile_removes_isolated_dir() {
    let _sandbox = XdgSandbox::new();
    let app = make_browser_app("Browser", "http://127.0.0.1:9/delete-profile");

    webapps_manager::service::create_webapp(&app).expect("create");
    let saved = webapps_manager::service::load_webapps()
        .webapps
        .into_iter()
        .next()
        .expect("saved app");

    let profile_key = saved.app_file.trim_end_matches(".desktop");
    let profile_dir = config::profiles_dir()
        .join(&saved.browser)
        .join(profile_key);
    std::fs::create_dir_all(&profile_dir).expect("profile dir");

    webapps_manager::service::delete_webapp(&saved, false).expect("delete");

    assert!(!profile_dir.exists(), "profile dir should be removed");
}

#[test]
#[serial]
fn delete_all_removes_browser_default_profiles() {
    let _sandbox = XdgSandbox::new();
    let app = make_browser_app("Browser", "http://127.0.0.1:9/delete-all-profile");

    webapps_manager::service::create_webapp(&app).expect("create");
    let saved = webapps_manager::service::load_webapps()
        .webapps
        .into_iter()
        .next()
        .expect("saved app");

    let profile_key = saved.app_file.trim_end_matches(".desktop");
    let profile_dir = config::profiles_dir()
        .join(&saved.browser)
        .join(profile_key);
    std::fs::create_dir_all(&profile_dir).expect("profile dir");

    webapps_manager::service::delete_all_webapps().expect("delete all");

    assert!(!profile_dir.exists(), "profile dir should be removed");
}

#[test]
#[serial]
fn app_mode_migration_preserves_existing_viewer_identity() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_app("Notes", "https://cloud.talesam.org/apps/notes");
    app.app_file = "biglinux-webapp-cloudtalesamorg.desktop".to_string();
    webapps_manager::service::save_webapps(&WebAppCollection {
        webapps: vec![app.clone()],
    })
    .expect("save collection");

    let legacy_id = webapps_core::desktop::legacy_host_desktop_file_id(&app.app_url);
    let legacy_geometry = config::config_dir().join(format!("{legacy_id}.json"));
    let legacy_data = config::data_dir().join(&legacy_id);
    let legacy_cache = config::cache_dir().join(&legacy_id);
    fs::create_dir_all(config::config_dir()).expect("config dir");
    fs::write(&legacy_geometry, "{}").expect("legacy geometry");
    fs::create_dir_all(&legacy_data).expect("legacy data");
    fs::create_dir_all(&legacy_cache).expect("legacy cache");

    let count = webapps_manager::service::regenerate_app_mode_desktops();

    assert_eq!(count, 1);
    assert!(legacy_geometry.exists());
    assert!(legacy_data.exists());
    assert!(legacy_cache.exists());

    let saved = webapps_manager::service::load_webapps()
        .webapps
        .into_iter()
        .next()
        .expect("saved app");
    assert_eq!(saved.app_file, "biglinux-webapp-cloudtalesamorg.desktop");

    let desktop = config::applications_dir().join(&saved.app_file);
    let content = fs::read_to_string(desktop).expect("desktop entry");
    assert!(content.contains("--app-id=\"cloudtalesamorg\""));
    assert!(content.contains("StartupWMClass=br.com.biglinux.webapp.cloudtalesamorg"));
}

#[test]
#[serial]
fn app_mode_migration_moves_storage_when_identity_already_changed() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_app("Notes", "https://cloud.talesam.org/apps/notes");
    app.app_file = "biglinux-webapp-cloudtalesamorg_apps_notes.desktop".to_string();
    webapps_manager::service::save_webapps(&WebAppCollection {
        webapps: vec![app.clone()],
    })
    .expect("save collection");

    let legacy_id = webapps_core::desktop::legacy_host_desktop_file_id(&app.app_url);
    let new_id = webapps_core::desktop::viewer_app_id(&app);
    let legacy_data = config::data_dir().join(&legacy_id);
    let legacy_cache = config::cache_dir().join(&legacy_id);
    fs::create_dir_all(&legacy_data).expect("legacy data");
    fs::create_dir_all(&legacy_cache).expect("legacy cache");

    let count = webapps_manager::service::regenerate_app_mode_desktops();

    assert_eq!(count, 1);
    assert!(!legacy_data.exists());
    assert!(!legacy_cache.exists());
    assert!(config::data_dir().join(&new_id).exists());
    assert!(config::cache_dir().join(&new_id).exists());
}

#[test]
#[serial]
fn app_mode_migration_keeps_shared_legacy_viewer_storage() {
    let _sandbox = XdgSandbox::new();
    let notes = make_app("Notes", "https://cloud.talesam.org/apps/notes");
    let calendar = make_app("Calendar", "https://cloud.talesam.org/apps/calendar");
    webapps_manager::service::save_webapps(&WebAppCollection {
        webapps: vec![notes.clone(), calendar],
    })
    .expect("save collection");

    let legacy_id = webapps_core::desktop::legacy_host_desktop_file_id(&notes.app_url);
    let legacy_data = config::data_dir().join(&legacy_id);
    fs::create_dir_all(&legacy_data).expect("legacy data");

    let count = webapps_manager::service::regenerate_app_mode_desktops();

    assert_eq!(count, 2);
    assert!(legacy_data.exists());
    assert!(!config::data_dir()
        .join(webapps_core::desktop::desktop_file_id(&notes.app_url))
        .exists());
}

#[path = "crud_integration/regressions.rs"]
mod regressions;
