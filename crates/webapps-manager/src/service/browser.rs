use std::path::Path;

use webapps_core::models::{Browser, BrowserCollection};

pub fn detect_browsers() -> BrowserCollection {
    // (browser_id, [candidate_paths]) — first existing path wins
    let known_browsers: &[(&str, &[&str])] = &[
        ("firefox", &["/usr/bin/firefox"]),
        (
            "firefox-developer-edition",
            &["/usr/bin/firefox-developer-edition"],
        ),
        ("librewolf", &["/usr/bin/librewolf"]),
        ("google-chrome-stable", &["/usr/bin/google-chrome-stable"]),
        ("google-chrome-beta", &["/usr/bin/google-chrome-beta"]),
        ("google-chrome-unstable", &["/usr/bin/google-chrome-unstable"]),
        ("chromium", &["/usr/bin/chromium"]),
        (
            "brave",
            &[
                "/usr/bin/brave",
                "/usr/bin/brave-browser",
                "/usr/bin/brave-browser-stable",
            ],
        ),
        (
            "brave-beta",
            &["/usr/bin/brave-browser-beta", "/usr/bin/brave-beta"],
        ),
        (
            "brave-nightly",
            &["/usr/bin/brave-browser-nightly", "/usr/bin/brave-nightly"],
        ),
        ("microsoft-edge-stable", &["/usr/bin/microsoft-edge-stable"]),
        ("microsoft-edge-beta", &["/usr/bin/microsoft-edge-beta"]),
        ("vivaldi-stable", &["/usr/bin/vivaldi-stable"]),
        ("vivaldi-beta", &["/usr/bin/vivaldi-beta"]),
        ("vivaldi-snapshot", &["/usr/bin/vivaldi-snapshot"]),
        ("ungoogled-chromium", &["/usr/bin/ungoogled-chromium"]),
    ];

    let mut browsers: Vec<Browser> = Vec::new();

    for (id, paths) in known_browsers {
        if paths.iter().any(|p| Path::new(p).exists()) {
            browsers.push(Browser {
                browser_id: id.to_string(),
                is_default: false,
            });
        }
    }

    // detect flatpak browsers
    if let Ok(output) = std::process::Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let flatpak_map = [
            ("org.mozilla.firefox", "flatpak-firefox"),
            ("com.google.Chrome", "flatpak-google-chrome-stable"),
            ("org.chromium.Chromium", "flatpak-chromium"),
            ("com.brave.Browser", "flatpak-brave-browser"),
            ("com.microsoft.Edge", "flatpak-microsoft-edge-stable"),
            ("com.vivaldi.Vivaldi", "flatpak-vivaldi-stable"),
            ("io.gitlab.librewolf-community", "flatpak-librewolf"),
        ];
        for (flatpak_id, browser_id) in &flatpak_map {
            if stdout.lines().any(|l| l.trim() == *flatpak_id) {
                browsers.push(Browser {
                    browser_id: browser_id.to_string(),
                    is_default: false,
                });
            }
        }
    }

    // detect system default
    let default_id = detect_default_browser();

    let mut col = BrowserCollection {
        browsers,
        default_id: None,
    };
    if let Some(id) = default_id {
        col.set_default(&id);
    }
    col
}

fn detect_default_browser() -> Option<String> {
    let output = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;
    let desktop_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if desktop_name.is_empty() {
        return None;
    }
    match_desktop_to_browser(&desktop_name)
}

fn match_desktop_to_browser(desktop: &str) -> Option<String> {
    let d = desktop.to_lowercase();
    let mappings = [
        ("firefox", "firefox"),
        ("firefox-developer", "firefox-developer-edition"),
        ("librewolf", "librewolf"),
        ("google-chrome-stable", "google-chrome-stable"),
        ("google-chrome-beta", "google-chrome-beta"),
        ("google-chrome-unstable", "google-chrome-unstable"),
        ("chromium", "chromium"),
        ("brave", "brave"),
        ("microsoft-edge-stable", "microsoft-edge-stable"),
        ("microsoft-edge-beta", "microsoft-edge-beta"),
        ("vivaldi-stable", "vivaldi-stable"),
        ("vivaldi-beta", "vivaldi-beta"),
        ("vivaldi-snapshot", "vivaldi-snapshot"),
    ];
    for (pattern, id) in &mappings {
        if d.contains(pattern) {
            return Some(id.to_string());
        }
    }
    None
}
