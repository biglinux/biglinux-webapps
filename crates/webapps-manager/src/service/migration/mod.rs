mod parse;
mod shell;

use std::fs;
use std::path::Path;

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{AppMode, WebApp, WebAppCollection};

use super::browser_url::resolve_browser_url;
use super::repository::mutate_webapps;
use super::{load_webapps, save_webapps, webapps_json_path};

/// Marker indicating the viewer-mode `StartupWMClass` realignment migration ran.
///
/// v1 aligned `StartupWMClass` with the viewer's host-only GTK application ID.
/// v2 moved viewer IDs to host+path. v3 keeps existing viewer identities stable
/// and only uses path-aware IDs for newly created entries, so WebKit cookies do
/// not move just because the URL derivation changed.
const WMCLASS_MIGRATION_MARKER: &str = ".desktop-wmclass-aligned-v3";

/// Marker for the browser-mode realignment migration.
///
/// Pre-fix browser-mode `.desktop` entries set `StartupWMClass` to a value
/// (`<browser-prefix>-<url>-Default`) that did not match what Chromium
/// reports — and `big-webapps-exec` never forwarded `--class` to the browser,
/// so the launched window's WM_CLASS came from the URL alone. The mismatch
/// caused taskbars to group webapp windows under the host browser's own
/// `.desktop` entry. This marker records that the one-shot regeneration with
/// the corrected derivation has run.
const BROWSER_WMCLASS_MIGRATION_MARKER: &str = ".desktop-wmclass-browser-v1";

/// Marker for the icon-persistence migration.
///
/// Pre-fix entries kept `Icon=` pointing at a volatile favicon cache path or
/// at a bare stem in `~/.local/share/icons/` that no icon theme indexed, so
/// GNOME Shell taskbars fell back to a generic glyph. This marker records
/// that the one-shot pass re-persisted each webapp's icon into our data dir
/// and rewrote `Icon=` to the stable absolute path.
const ICON_PERSIST_MIGRATION_MARKER: &str = ".desktop-icon-persisted-v1";

/// Marker for the Browser-mode `.desktop` filename realignment.
///
/// Chromium-family browsers ignore `--class` when picking the xdg-shell
/// `app_id` on Wayland — they synthesize one from the binary name, host,
/// path, and profile dir (e.g. `brave-open.spotify.com__intl-pt_-Default`).
/// GNOME Shell's primary window→.desktop lookup matches that `app_id`
/// against the `.desktop` basename, so pre-fix entries (whose filenames used
/// `__` separators and no browser prefix) lost the icon mapping and fell back
/// to the host browser's taskbar group. v2 additionally pre-follows redirects
/// (Spotify → `/intl-pt/` on pt-BR locale) so the prediction matches the
/// post-redirect app_id Brave actually emits. This marker records that the
/// one-shot rename to the canonical scheme has run.
const BROWSER_FILENAME_MIGRATION_MARKER: &str = ".desktop-filename-chromium-v3";

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
/// up the corrected viewer app ID, desktop filename, and storage layout.
pub fn regenerate_app_mode_desktops() -> usize {
    regenerate_desktops_once(AppMode::App, WMCLASS_MIGRATION_MARKER)
}

/// Regenerate `AppMode::Browser` desktop entries once, so existing installs
/// pick up the corrected `StartupWMClass` and `--class` argument.
pub fn regenerate_browser_mode_desktops() -> usize {
    regenerate_desktops_once(AppMode::Browser, BROWSER_WMCLASS_MIGRATION_MARKER)
}

/// One-shot rename of every Browser-mode `.desktop` so its basename matches
/// the Wayland `app_id` Chromium synthesizes at launch — without this, GNOME
/// Shell can't map the running window to the entry and falls back to the
/// generic host-browser icon. Returns the number of entries actually renamed.
pub fn migrate_browser_desktop_filenames() -> usize {
    let marker = config::data_dir().join(BROWSER_FILENAME_MIGRATION_MARKER);
    if marker.exists() {
        return 0;
    }

    let collection = load_webapps();
    let mut renames: Vec<(String, WebApp)> = Vec::new();

    for app in &collection.webapps {
        if app.app_mode != AppMode::Browser {
            continue;
        }

        // Resolve redirects first — Chromium derives the runtime app_id from
        // the post-redirect URL, so the saved URL must already reflect that
        // for the predicted filename to stick.
        let resolved_url = resolve_browser_url(&app.app_url);
        let mut updated = app.clone();
        updated.app_url = resolved_url;
        let canonical = desktop::canonical_browser_desktop_filename(&updated);
        if canonical == app.app_file && updated.app_url == app.app_url {
            continue;
        }

        let old_file = app.app_file.clone();
        updated.app_file = canonical;

        match desktop::install_desktop_entry(&updated) {
            Ok(()) => {
                if !old_file.is_empty() && old_file != updated.app_file {
                    if let Err(err) = desktop::remove_desktop_file(&old_file) {
                        log::warn!(
                            "Remove stale desktop entry {old_file} during browser-filename migration: {err}"
                        );
                    }
                }
                renames.push((old_file, updated));
            }
            Err(err) => log::warn!(
                "Install canonical desktop entry for {}: {err}",
                app.app_name
            ),
        }
    }

    let count = renames.len();
    if count > 0 {
        let updates = renames.clone();
        if let Err(err) = mutate_webapps(move |collection| {
            for (old_file, updated) in &updates {
                collection.remove_by_file(old_file);
                collection.add(updated.clone());
            }
            Ok(())
        }) {
            log::warn!("Browser-filename migration: save webapps.json failed: {err}");
        }
    }

    if let Err(err) = fs::create_dir_all(config::data_dir()).and_then(|()| fs::write(&marker, "")) {
        log::warn!(
            "Write browser-filename migration marker {}: {err}",
            marker.display()
        );
    }

    count
}

/// One-shot pass over every saved webapp: re-persist the icon into our data
/// directory under a stable stem and rewrite the `.desktop` so `Icon=` points
/// at the new absolute path. Returns the number of entries whose icon was
/// actually rewritten.
pub fn persist_existing_icons() -> usize {
    let marker = config::data_dir().join(ICON_PERSIST_MIGRATION_MARKER);
    if marker.exists() {
        return 0;
    }

    let collection = load_webapps();
    let mut rewrites: Vec<(String, String)> = Vec::new();

    for app in &collection.webapps {
        let Some(new_icon) = desktop::persist_icon(app) else {
            continue;
        };
        if new_icon == app.app_icon {
            continue;
        }
        let mut updated = app.clone();
        updated.app_icon = new_icon.clone();
        match desktop::install_desktop_entry(&updated) {
            Ok(()) => rewrites.push((app.app_file.clone(), new_icon)),
            Err(err) => log::warn!(
                "Persist icon for {}: rewrite desktop entry failed: {err}",
                app.app_name
            ),
        }
    }

    let count = rewrites.len();
    if count > 0 {
        let updates = rewrites.clone();
        if let Err(err) = mutate_webapps(move |collection| {
            for (app_file, new_icon) in &updates {
                if let Some(app) = collection
                    .webapps
                    .iter_mut()
                    .find(|app| &app.app_file == app_file)
                {
                    app.app_icon = new_icon.clone();
                }
            }
            Ok(())
        }) {
            log::warn!("Persist icon migration: save webapps.json failed: {err}");
        }
    }

    if let Err(err) = fs::create_dir_all(config::data_dir()).and_then(|()| fs::write(&marker, "")) {
        log::warn!(
            "Write icon-persist migration marker {}: {err}",
            marker.display()
        );
    }

    count
}

fn regenerate_desktops_once(mode: AppMode, marker_name: &str) -> usize {
    let marker = config::data_dir().join(marker_name);
    if marker.exists() {
        return 0;
    }

    let collection = load_webapps();
    let mut regenerated = 0;
    let mut updates: Vec<(String, WebApp)> = Vec::new();
    for app in &collection.webapps {
        if app.app_mode != mode {
            continue;
        }
        let mut updated = app.clone();
        if mode == AppMode::App {
            migrate_viewer_storage(&updated, &collection);
            if updated.desktop_file_name().is_none() {
                updated.app_file = desktop::viewer_desktop_filename(&updated.app_url);
            }
        }
        match desktop::install_desktop_entry(&updated) {
            Ok(()) => {
                regenerated += 1;
                if updated.app_file != app.app_file {
                    if !app.app_file.is_empty() {
                        if let Err(err) = desktop::remove_desktop_file(&app.app_file) {
                            log::warn!(
                                "Remove stale desktop entry {} during app-mode migration: {err}",
                                app.app_file
                            );
                        }
                    }
                    updates.push((app.app_file.clone(), updated));
                }
            }
            Err(err) => log::warn!("Regenerate desktop entry for {}: {err}", app.app_name),
        }
    }

    if !updates.is_empty() {
        let updates_for_save = updates.clone();
        if let Err(err) = mutate_webapps(move |collection| {
            for (old_file, updated) in &updates_for_save {
                collection.remove_by_file(old_file);
                collection.add(updated.clone());
            }
            Ok(())
        }) {
            log::warn!("App-mode desktop migration: save webapps.json failed: {err}");
        }
    }

    if let Err(err) = fs::create_dir_all(config::data_dir()).and_then(|()| fs::write(&marker, "")) {
        log::warn!(
            "Write StartupWMClass migration marker {}: {err}",
            marker.display()
        );
    }

    regenerated
}

fn migrate_viewer_storage(app: &WebApp, collection: &WebAppCollection) {
    let old_id = desktop::legacy_host_desktop_file_id(&app.app_url);
    let new_id = desktop::viewer_app_id(app);
    if old_id == new_id {
        return;
    }

    let shared_old_id = collection
        .webapps
        .iter()
        .filter(|candidate| {
            candidate.app_mode == AppMode::App
                && desktop::legacy_host_desktop_file_id(&candidate.app_url) == old_id
        })
        .take(2)
        .count()
        > 1;
    if shared_old_id {
        log::info!(
            "Skipping viewer storage migration for {old_id}: legacy ID is shared by multiple webapps"
        );
        return;
    }

    rename_unclaimed_path(
        &config::config_dir().join(format!("{old_id}.json")),
        &config::config_dir().join(format!("{new_id}.json")),
    );
    rename_unclaimed_path(
        &config::data_dir().join(&old_id),
        &config::data_dir().join(&new_id),
    );
    rename_unclaimed_path(
        &config::cache_dir().join(&old_id),
        &config::cache_dir().join(&new_id),
    );
}

fn rename_unclaimed_path(old_path: &Path, new_path: &Path) {
    if !old_path.exists() {
        return;
    }
    if new_path.exists() {
        log::info!(
            "Keeping legacy viewer path {} because {} already exists",
            old_path.display(),
            new_path.display()
        );
        return;
    }

    if let Some(parent) = new_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            log::warn!("Create parent directory {}: {err}", parent.display());
            return;
        }
    }

    if let Err(err) = fs::rename(old_path, new_path) {
        log::warn!(
            "Move viewer path {} to {}: {err}",
            old_path.display(),
            new_path.display()
        );
    }
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
