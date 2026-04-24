mod parse;
mod shell;

use std::fs;

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{AppMode, WebApp, WebAppCollection};

use super::{load_webapps, save_webapps, webapps_json_path};

/// Marker indicating the viewer-mode `StartupWMClass` realignment migration ran.
///
/// Pre-v4.1 desktop entries for `AppMode::App` set `StartupWMClass` to a value
/// that included the URL path, while the viewer's GTK `application_id` only
/// uses the host. The mismatch prevented Wayland compositors from associating
/// viewer windows with their `.desktop` file, so the taskbar fell back to the
/// raw `app_id` and a generic icon. This marker records that the one-shot
/// regeneration has run so we only do it once per user.
const WMCLASS_MIGRATION_MARKER: &str = ".desktop-wmclass-aligned-v1";

pub fn migrate_legacy_desktops() -> usize {
    let json_path = webapps_json_path();
    if json_path.exists() {
        return 0;
    }

    let apps_dir = config::applications_dir();
    let entries = match fs::read_dir(&apps_dir) {
        Ok(entries) => entries,
        Err(_) => return 0,
    };

    let webapps = collect_legacy_webapps(entries);
    persist_migrated_webapps(webapps)
}

/// Regenerate `AppMode::App` desktop entries once, so existing installs pick
/// up the corrected `StartupWMClass` without the user having to re-save each
/// webapp in the manager.
pub fn regenerate_app_mode_desktops() -> usize {
    let marker = config::data_dir().join(WMCLASS_MIGRATION_MARKER);
    if marker.exists() {
        return 0;
    }

    let collection = load_webapps();
    let mut regenerated = 0;
    for app in &collection.webapps {
        if app.app_mode != AppMode::App {
            continue;
        }
        match desktop::install_desktop_entry(app) {
            Ok(()) => regenerated += 1,
            Err(err) => log::warn!(
                "Regenerate desktop entry for {}: {err}",
                app.app_name
            ),
        }
    }

    if let Err(err) = fs::create_dir_all(config::data_dir())
        .and_then(|()| fs::write(&marker, ""))
    {
        log::warn!(
            "Write StartupWMClass migration marker {}: {err}",
            marker.display()
        );
    }

    regenerated
}

fn collect_legacy_webapps(entries: fs::ReadDir) -> Vec<WebApp> {
    let mut webapps = Vec::new();

    for entry in entries.flatten() {
        let filename = entry.file_name().to_string_lossy().to_string();
        if !filename.ends_with(".desktop") {
            continue;
        }

        let content = match fs::read_to_string(entry.path()) {
            Ok(content) => content,
            Err(_) => continue,
        };

        if !is_legacy_big_webapps_entry(&content) {
            continue;
        }

        if let Some(app) = parse::parse_legacy_desktop(&filename, &content) {
            webapps.push(app);
        }
    }

    webapps
}

fn is_legacy_big_webapps_entry(content: &str) -> bool {
    content.contains("big-webapps-exec") || content.contains("big-webapps-viewer")
}

fn persist_migrated_webapps(webapps: Vec<WebApp>) -> usize {
    let count = webapps.len();
    if count == 0 {
        return 0;
    }

    let collection = WebAppCollection { webapps };
    if let Err(error) = save_webapps(&collection) {
        log::error!("Save migrated webapps: {error}");
        return 0;
    }

    log::info!("Migrated {count} legacy webapps");
    count
}
