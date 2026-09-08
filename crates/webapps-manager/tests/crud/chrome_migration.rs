use super::*;

#[test]
#[serial]
fn chrome_wmclass_migration_preserves_existing_profiles_and_icons() {
    let _sandbox = XdgSandbox::new();
    let mut app = make_browser_app("Spotify", "https://open.spotify.com/intl-pt/");
    app.browser = "google-chrome-stable".into();
    app.app_profile = "Default".into();
    app.app_file = "google-chrome-open.spotify.com__intl-pt_-Default.desktop".into();
    let icon = config::data_dir().join("spotify.png");
    app.app_icon = icon.to_string_lossy().into_owned();
    let profile = config::profiles_dir()
        .join(&app.browser)
        .join(app.app_file.trim_end_matches(".desktop"));
    fs::create_dir_all(&profile).unwrap();
    fs::write(profile.join("Cookies"), b"existing session").unwrap();
    fs::create_dir_all(config::data_dir()).unwrap();
    fs::write(&icon, b"existing icon").unwrap();
    fs::write(config::data_dir().join(".desktop-wmclass-browser-v1"), b"").unwrap();
    webapps_manager::service::save_webapps(&WebAppCollection {
        webapps: vec![app.clone()],
    })
    .unwrap();

    assert_eq!(
        webapps_manager::service::regenerate_browser_mode_desktops(),
        1
    );
    let saved = webapps_manager::service::load_webapps().webapps.remove(0);
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        serde_json::to_value(&app).unwrap()
    );
    let desktop = fs::read_to_string(config::applications_dir().join(&app.app_file)).unwrap();
    assert!(desktop.contains("StartupWMClass=chrome-open.spotify.com__intl-pt_-Default\n"));
    assert!(desktop.contains(&format!("filename=\"{}\"", app.app_file)));
    assert!(desktop.contains(&format!("Icon={}\n", app.app_icon)));
    assert_eq!(
        fs::read(profile.join("Cookies")).unwrap(),
        b"existing session"
    );
    assert_eq!(fs::read(&icon).unwrap(), b"existing icon");
    assert!(config::data_dir()
        .join(".desktop-wmclass-browser-v2")
        .exists());
    assert_eq!(
        webapps_manager::service::regenerate_browser_mode_desktops(),
        0
    );
}
