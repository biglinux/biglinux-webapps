use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{AppMode, ProfileKind, WebApp};

use super::profile_files::profile_dir_for;

pub(super) fn cleanup_viewer_data(
    webapp: &WebApp,
    collection: &webapps_core::models::WebAppCollection,
) {
    let app_ids = [
        desktop::viewer_app_id(webapp),
        desktop::desktop_file_id(&webapp.app_url),
        desktop::legacy_host_desktop_file_id(&webapp.app_url),
    ];

    for app_id in app_ids {
        cleanup_viewer_data_for_id(webapp, &app_id, collection);
    }
}

fn cleanup_viewer_data_for_id(
    webapp: &WebApp,
    app_id: &str,
    collection: &webapps_core::models::WebAppCollection,
) {
    let still_in_use = collection
        .webapps
        .iter()
        .any(|app| app.app_file != webapp.app_file && desktop::viewer_app_id(app) == app_id);
    if still_in_use {
        log::info!("Skipping cleanup of viewer data for {app_id}: shared with another webapp");
        return;
    }

    let geometry_path = config::config_dir().join(format!("{app_id}.json"));
    if let Err(err) = fs::remove_file(&geometry_path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::warn!(
                "Failed to remove geometry file {}: {err}",
                geometry_path.display()
            );
        }
    }
    let data_dir = config::data_dir().join(app_id);
    if let Err(err) = fs::remove_dir_all(&data_dir) {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to remove viewer data {}: {err}", data_dir.display());
        }
    }
    let cache_dir = config::cache_dir().join(app_id);
    if let Err(err) = fs::remove_dir_all(&cache_dir) {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::warn!(
                "Failed to remove viewer cache {}: {err}",
                cache_dir.display()
            );
        }
    }
}

pub(super) fn cleanup_profile(webapp: &WebApp) -> Result<()> {
    let Some(profile_dir) = profile_dir_for(webapp)? else {
        return Ok(());
    };

    remove_profile_dir(&profile_dir)
}

fn remove_profile_dir(profile_dir: &Path) -> Result<()> {
    if profile_dir.exists() {
        fs::remove_dir_all(profile_dir)
            .with_context(|| format!("Remove profile directory {}", profile_dir.display()))?;
        log::info!("Removed profile: {}", profile_dir.display());
    }

    Ok(())
}

pub(super) fn cleanup_deleted_app(
    webapp: &WebApp,
    delete_profile: bool,
    collection: &webapps_core::models::WebAppCollection,
) -> Result<()> {
    match webapp.app_mode {
        AppMode::App => cleanup_viewer_data(webapp, collection),
        AppMode::Browser => {
            let shared = collection.webapps.iter().any(|app| {
                app.browser == webapp.browser
                    && app.app_profile == webapp.app_profile
                    && webapp.has_custom_profile()
            });
            if !shared {
                cleanup_browser_profile(webapp, delete_profile)?;
            }
        }
    }
    cleanup_unused_icon(&webapp.app_icon, collection);

    Ok(())
}

fn cleanup_browser_profile(webapp: &WebApp, delete_profile: bool) -> Result<()> {
    match webapp.profile_kind() {
        ProfileKind::Custom(_) if delete_profile => cleanup_profile(webapp),
        ProfileKind::Custom(_) => Ok(()),
        ProfileKind::Default | ProfileKind::Browser => {
            webapp.browser_id().validate()?;
            let profile_dir = config::profiles_dir()
                .join(webapp.browser_id().as_str())
                .join(default_browser_profile_key(webapp));
            remove_profile_dir(&profile_dir)
        }
    }
}

fn default_browser_profile_key(webapp: &WebApp) -> String {
    webapp
        .desktop_file_name()
        .map(|file_name| file_name.as_str().trim_end_matches(".desktop").to_string())
        .unwrap_or_else(|| desktop::desktop_file_id(&webapp.app_url))
}

pub(super) fn cleanup_unused_icon(icon: &str, collection: &webapps_core::models::WebAppCollection) {
    if collection.webapps.iter().any(|app| app.app_icon == icon) {
        return;
    }
    let path = Path::new(icon);
    if path.parent() != Some(desktop::webapp_icons_dir().as_path()) {
        return;
    }
    if let Err(err) = fs::remove_file(path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Remove unused icon {}: {err}", path.display());
        }
    }
}
