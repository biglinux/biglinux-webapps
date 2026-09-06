use super::*;

#[test]
#[serial]
fn corrupt_registry_blocks_every_mutation_without_replacing_bytes() {
    let _sandbox = XdgSandbox::new();
    let path = config::data_dir().join("webapps.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original = b"[{broken json";
    fs::write(&path, original).unwrap();
    let app = make_app("Protected", "https://example.com/protected");
    assert!(webapps_manager::service::create_webapp(&app).is_err());
    assert!(webapps_manager::service::update_webapp(&app).is_err());
    assert!(webapps_manager::service::delete_webapp(&app, false).is_err());
    assert!(webapps_manager::service::delete_all_webapps().is_err());
    assert!(webapps_manager::service::save_webapps(&WebAppCollection::default()).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(!config::applications_dir().exists());
}

#[test]
#[serial]
fn viewer_url_variants_do_not_overwrite_each_other() {
    let _sandbox = XdgSandbox::new();
    let urls = [
        "https://example.com/app?a=1",
        "https://example.com/app?a=2",
        "https://example.com/App?a=1",
        "https://example.com:8443/app?a=1",
        "https://example.com/app?a=1#inbox",
    ];
    for (index, url) in urls.iter().enumerate() {
        webapps_manager::service::create_webapp(&make_app(&format!("App{index}"), url)).unwrap();
    }
    let saved = webapps_manager::service::try_load_webapps().unwrap();
    assert_eq!(saved.webapps.len(), urls.len());
    let names: std::collections::HashSet<_> =
        saved.webapps.iter().map(|app| &app.app_file).collect();
    assert_eq!(names.len(), urls.len());
    assert!(webapps_manager::service::create_webapp(&make_app("Duplicate", urls[0])).is_err());
    assert_eq!(
        webapps_manager::service::try_load_webapps()
            .unwrap()
            .webapps
            .len(),
        urls.len()
    );
}

#[test]
#[serial]
fn failed_desktop_write_restores_overwritten_icon() {
    let _sandbox = XdgSandbox::new();
    let source = config::data_dir().join("source.png");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"original icon").unwrap();
    let mut app = make_app("Rollback", "https://example.com/rollback");
    app.app_icon = source.to_string_lossy().into_owned();
    webapps_manager::service::create_webapp(&app).unwrap();
    let mut saved = webapps_manager::service::try_load_webapps()
        .unwrap()
        .webapps
        .remove(0);
    let icon = saved.app_icon.clone();
    let desktop = config::applications_dir().join(&saved.app_file);
    fs::remove_file(&desktop).unwrap();
    fs::create_dir(&desktop).unwrap();
    fs::write(&source, b"replacement icon").unwrap();
    saved.app_icon = source.to_string_lossy().into_owned();
    assert!(webapps_manager::service::update_webapp(&saved).is_err());
    assert_eq!(fs::read(icon).unwrap(), b"original icon");
    assert_eq!(
        webapps_manager::service::try_load_webapps()
            .unwrap()
            .webapps[0]
            .app_name,
        "Rollback"
    );
}

#[test]
#[serial]
fn concurrent_creates_preserve_all_entries() {
    let _sandbox = XdgSandbox::new();
    std::thread::scope(|scope| {
        for index in 0..8 {
            scope.spawn(move || {
                webapps_manager::service::create_webapp(&make_app(
                    &format!("Concurrent{index}"),
                    &format!("https://example.com/{index}"),
                ))
                .unwrap()
            });
        }
    });
    assert_eq!(
        webapps_manager::service::try_load_webapps()
            .unwrap()
            .webapps
            .len(),
        8
    );
}

#[test]
#[serial]
fn deleting_one_app_preserves_shared_icon_until_last_reference() {
    let _sandbox = XdgSandbox::new();
    for index in 0..2 {
        webapps_manager::service::create_webapp(&make_app(
            &format!("Shared{index}"),
            &format!("https://example.com/shared/{index}"),
        ))
        .unwrap();
    }
    let icon = config::data_dir().join("icons/shared.png");
    fs::create_dir_all(icon.parent().unwrap()).unwrap();
    fs::write(&icon, b"shared icon").unwrap();
    let mut col = webapps_manager::service::try_load_webapps().unwrap();
    for app in &mut col.webapps {
        app.app_icon = icon.to_string_lossy().into_owned();
    }
    webapps_manager::service::save_webapps(&col).unwrap();
    webapps_manager::service::delete_webapp(&col.webapps[0], false).unwrap();
    assert!(icon.is_file());
    webapps_manager::service::delete_webapp(&col.webapps[1], false).unwrap();
    assert!(!icon.exists());
}

#[test]
#[serial]
fn archive_round_trip_restores_actual_icons_and_replaces_existing_archive() {
    let _sandbox = XdgSandbox::new();
    let mut originals = Vec::new();
    for index in 0..2 {
        let source = config::data_dir().join(format!("source{index}/same.png"));
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        let bytes = format!("icon {index}").into_bytes();
        fs::write(&source, &bytes).unwrap();
        let mut app = make_app(
            &format!("Archive{index}"),
            &format!("https://example.com/archive/{index}"),
        );
        app.app_icon = source.to_string_lossy().into_owned();
        app.app_icon_url.clear();
        webapps_manager::service::create_webapp(&app).unwrap();
        originals.push(bytes);
    }
    let archive = config::cache_dir().join("backup.zip");
    webapps_manager::service::export_webapps(&archive).unwrap();
    webapps_manager::service::export_webapps(&archive).unwrap();
    webapps_manager::service::delete_all_webapps().unwrap();
    assert_eq!(
        webapps_manager::service::import_webapps(&archive).unwrap(),
        (2, 0)
    );
    let saved = webapps_manager::service::try_load_webapps().unwrap();
    for (app, bytes) in saved.webapps.iter().zip(originals) {
        assert_eq!(fs::read(&app.app_icon).unwrap(), bytes);
        assert!(
            fs::read_to_string(config::applications_dir().join(&app.app_file))
                .unwrap()
                .contains(&format!("Icon={}", app.app_icon))
        );
    }
    assert_eq!(
        webapps_manager::service::import_webapps(&archive).unwrap(),
        (0, 2)
    );
}

#[test]
#[serial]
fn failed_migration_restores_desktops_and_remains_retryable() {
    let _sandbox = XdgSandbox::new();
    let mut apps = Vec::new();
    for index in 0..2 {
        let app = make_app(
            &format!("Migration{index}"),
            &format!("https://example.com/migration/{index}"),
        );
        webapps_manager::service::create_webapp(&app).unwrap();
    }
    apps.extend(
        webapps_manager::service::try_load_webapps()
            .unwrap()
            .webapps,
    );
    let first = config::applications_dir().join(&apps[0].app_file);
    fs::write(&first, b"original desktop bytes").unwrap();
    let second = config::applications_dir().join(&apps[1].app_file);
    fs::remove_file(&second).unwrap();
    fs::create_dir(&second).unwrap();
    assert_eq!(webapps_manager::service::regenerate_app_mode_desktops(), 0);
    assert_eq!(fs::read(&first).unwrap(), b"original desktop bytes");
    assert!(!config::data_dir()
        .join(".desktop-wmclass-aligned-v3")
        .exists());
    fs::remove_dir(&second).unwrap();
    assert_eq!(webapps_manager::service::regenerate_app_mode_desktops(), 2);
    assert!(config::data_dir()
        .join(".desktop-wmclass-aligned-v3")
        .is_file());
    assert_eq!(webapps_manager::service::regenerate_app_mode_desktops(), 0);
}

#[test]
#[serial]
fn failed_delete_all_restores_previous_desktop_bytes() {
    let _sandbox = XdgSandbox::new();
    for index in 0..2 {
        webapps_manager::service::create_webapp(&make_app(
            &format!("Remove{index}"),
            &format!("https://example.com/remove/{index}"),
        ))
        .unwrap();
    }
    let collection = webapps_manager::service::try_load_webapps().unwrap();
    let first = config::applications_dir().join(&collection.webapps[0].app_file);
    let original = fs::read(&first).unwrap();
    let second = config::applications_dir().join(&collection.webapps[1].app_file);
    fs::remove_file(&second).unwrap();
    fs::create_dir(&second).unwrap();
    assert!(webapps_manager::service::delete_all_webapps().is_err());
    assert_eq!(fs::read(first).unwrap(), original);
    assert_eq!(
        webapps_manager::service::try_load_webapps()
            .unwrap()
            .webapps,
        collection.webapps
    );
}

#[test]
#[serial]
fn simultaneous_duplicate_creates_have_exactly_one_winner() {
    let _sandbox = XdgSandbox::new();
    let barrier = std::sync::Barrier::new(4);
    let successes = std::thread::scope(|scope| {
        let attempts: Vec<_> = (0..4)
            .map(|index| {
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    webapps_manager::service::create_webapp(&make_app(
                        &format!("Winner{index}"),
                        "https://example.com/duplicate",
                    ))
                    .is_ok()
                })
            })
            .collect();
        attempts
            .into_iter()
            .map(|attempt| usize::from(attempt.join().unwrap()))
            .sum::<usize>()
    });
    assert_eq!(successes, 1);
    let collection = webapps_manager::service::try_load_webapps().unwrap();
    assert_eq!(collection.webapps.len(), 1);
    let desktop =
        fs::read_to_string(config::applications_dir().join(&collection.webapps[0].app_file))
            .unwrap();
    assert!(desktop.contains(&format!("Name={}\n", collection.webapps[0].app_name)));
}

#[test]
#[serial]
fn desktop_launch_preserves_profile_spaces_and_literal_url_percentages() {
    use std::os::unix::fs::PermissionsExt;
    let _sandbox = XdgSandbox::new();
    let root = config::data_dir();
    fs::create_dir_all(&root).unwrap();
    let captured = root.join("arguments.txt");
    let launcher = root.join("capture-arguments");
    fs::write(
        &launcher,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            captured.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let mut app = make_browser_app("Arguments", "https://example.com/a%20b?q=%25");
    app.app_profile = "Work Profile".into();
    let entry = webapps_core::desktop::generate_desktop_entry(&app)
        .replace("big-webapps-exec", &launcher.to_string_lossy());
    let desktop = root.join("argument-test.desktop");
    fs::write(&desktop, entry).unwrap();
    assert!(std::process::Command::new("gio")
        .arg("launch")
        .arg(&desktop)
        .status()
        .unwrap()
        .success());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while !captured.exists() {
        assert!(std::time::Instant::now() < deadline, "Launcher did not run");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let arguments = fs::read_to_string(captured).unwrap();
    assert!(arguments
        .lines()
        .any(|arg| arg == "--profile-directory=Work Profile"));
    assert!(arguments
        .lines()
        .any(|arg| arg == "--app=https://example.com/a%20b?q=%25"));
}
