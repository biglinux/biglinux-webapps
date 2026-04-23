mod parse;
mod shell;

use std::fs;

use webapps_core::config;
use webapps_core::models::{WebApp, WebAppCollection};

use super::{save_webapps, webapps_json_path};

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
