//! Migration of legacy shell-based webapps into the current `WebApp` model:
//! parses the old shell wrappers and converts entries on import.

mod parse;
mod shell;

use std::fs;

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{AppMode, WebApp, WebAppCollection};

use super::browser_url::resolve_browser_url;
use super::transaction::RegistryTransaction;
use super::{save_webapps, webapps_json_path};

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
    migrate_once(
        BROWSER_FILENAME_MIGRATION_MARKER,
        Some(AppMode::Browser),
        true,
        false,
    )
}

pub fn persist_existing_icons() -> usize {
    migrate_once(ICON_PERSIST_MIGRATION_MARKER, None, false, true)
}

fn regenerate_desktops_once(mode: AppMode, marker: &str) -> usize {
    migrate_once(marker, Some(mode), false, false)
}

fn migrate_once(
    marker_name: &str,
    mode: Option<AppMode>,
    rename_browser: bool,
    icons: bool,
) -> usize {
    let result = (|| -> anyhow::Result<usize> {
        let mut transaction = RegistryTransaction::begin()?;
        let marker = config::data_dir().join(marker_name);
        if marker.exists() {
            return Ok(0);
        }
        let snapshot = transaction.collection.webapps.clone();
        let mut count = 0;
        for app in &snapshot {
            if mode.is_some_and(|mode| app.app_mode != mode) {
                continue;
            }
            let mut updated = app.clone();
            if rename_browser {
                updated.app_url = resolve_browser_url(&app.app_url);
                updated.app_file = desktop::canonical_browser_desktop_filename(&updated);
            } else if updated.desktop_file_name().is_none() {
                updated.app_file = desktop::viewer_desktop_filename(&updated.app_url);
            }
            if updated.app_file != app.app_file {
                anyhow::ensure!(
                    !transaction
                        .collection
                        .webapps
                        .iter()
                        .any(|other| other.app_file == updated.app_file)
                        && !config::applications_dir().join(&updated.app_file).exists(),
                    "Desktop migration collision: {}",
                    updated.app_file
                );
            }
            if icons {
                if let Some(destination) = desktop::icon_destination(&updated) {
                    let bytes = fs::read(&updated.app_icon)?;
                    transaction.write(&destination, &bytes)?;
                    updated.app_icon = destination.to_string_lossy().into_owned();
                    updated.app_icon_url = updated.app_icon.clone();
                }
            }
            transaction.write(
                &config::applications_dir().join(&updated.app_file),
                desktop::generate_desktop_entry(&updated).as_bytes(),
            )?;
            if app.app_file != updated.app_file && !app.app_file.is_empty() {
                transaction.remove(&config::applications_dir().join(&app.app_file))?;
            }
            if mode == Some(AppMode::App) {
                migrate_viewer_storage(&updated, &snapshot, &mut transaction)?;
            }
            transaction.collection.remove_by_file(&app.app_file);
            transaction.collection.add(updated);
            count += 1;
        }
        transaction.write(&marker, b"")?;
        transaction.commit()?;
        desktop::refresh_desktop_database();
        Ok(count)
    })();
    match result {
        Ok(count) => count,
        Err(error) => {
            log::warn!("Migration {marker_name}: {error:#}");
            0
        }
    }
}

fn migrate_viewer_storage(
    app: &WebApp,
    apps: &[WebApp],
    transaction: &mut RegistryTransaction,
) -> anyhow::Result<()> {
    let old_id = desktop::legacy_host_desktop_file_id(&app.app_url);
    let new_id = desktop::viewer_app_id(app);
    if old_id == new_id
        || apps
            .iter()
            .filter(|other| {
                other.app_mode == AppMode::App
                    && desktop::legacy_host_desktop_file_id(&other.app_url) == old_id
            })
            .count()
            > 1
    {
        return Ok(());
    }
    transaction.rename_unclaimed(
        &config::config_dir().join(format!("{old_id}.json")),
        &config::config_dir().join(format!("{new_id}.json")),
    )?;
    transaction.rename_unclaimed(
        &config::data_dir().join(&old_id),
        &config::data_dir().join(&new_id),
    )?;
    transaction.rename_unclaimed(
        &config::cache_dir().join(&old_id),
        &config::cache_dir().join(&new_id),
    )
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
