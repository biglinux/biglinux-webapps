use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn legacy_flatpak_launchers_keep_application_ids_and_profile_paths() {
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let stub = bin.join("flatpak");
    fs::write(
        &stub,
        "#!/bin/sh\nif [ \"$1\" = run ]; then printf '%s\\n' \"$@\" > \"$WEBAPPS_TEST_ARGV\"; fi\n",
    )
    .unwrap();
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
    let prefix = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../biglinux-webapps/usr");
    for (id, app_id) in [
        ("flatpak-brave", "com.brave.Browser"),
        ("flatpak-chrome", "com.google.Chrome"),
        ("flatpak-chrome-unstable", "com.google.ChromeDev"),
        ("flatpak-chromium", "org.chromium.Chromium"),
        ("flatpak-edge", "com.microsoft.Edge"),
        (
            "flatpak-ungoogled-chromium",
            "com.github.Eloston.UngoogledChromium",
        ),
        ("flatpak-firefox", "org.mozilla.firefox"),
        ("flatpak-librewolf", "io.gitlab.librewolf-community"),
    ] {
        let home = root.path().join(id);
        let gecko = id == "flatpak-firefox" || id == "flatpak-librewolf";
        let profile =
            home.join(".bigwebapps")
                .join(id)
                .join(if gecko { "Existing" } else { "Work" });
        fs::create_dir_all(profile.join("chrome")).unwrap();
        fs::write(profile.join("Cookies"), b"existing session").unwrap();
        let recorded = home.join("argv");
        let status = Command::new(env!("CARGO_BIN_EXE_big-webapps-exec"))
            .env("HOME", &home)
            .env("XDG_DATA_HOME", home.join("data"))
            .env("PATH", &bin)
            .env("BIGLINUX_WEBAPPS_PREFIX", &prefix)
            .env("WEBAPPS_TEST_ARGV", &recorded)
            .args([
                "filename=Existing.desktop",
                id,
                "--class=Existing",
                if gecko {
                    "--profile-directory=Default"
                } else {
                    "--profile-directory=Work"
                },
                "--app=https://example.com/",
            ])
            .status()
            .unwrap();
        assert!(status.success(), "{id}");
        let deadline = Instant::now() + Duration::from_secs(5);
        while fs::read_to_string(&recorded)
            .unwrap_or_default()
            .lines()
            .count()
            < 4
        {
            assert!(Instant::now() < deadline, "{id} never launched");
            std::thread::sleep(Duration::from_millis(5));
        }
        let argv = fs::read_to_string(recorded).unwrap();
        assert_eq!(argv.lines().nth(1), Some(app_id));
        assert!(
            argv.lines().any(|arg| arg == profile.to_string_lossy()
                || arg == format!("--user-data-dir={}", profile.display())),
            "{id}: {argv}"
        );
        assert_eq!(
            fs::read(profile.join("Cookies")).unwrap(),
            b"existing session"
        );
    }
}
