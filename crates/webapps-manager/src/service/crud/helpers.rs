use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{AppMode, BrowserId, ProfileKind, WebApp};

use super::super::repository::load_webapps;

pub fn validate_custom_profile_name(profile_name: &str) -> Result<()> {
    ProfileKind::parse(profile_name).validate()
}

pub fn profile_shared(webapp: &WebApp) -> bool {
    let collection = load_webapps();
    collection.webapps.iter().any(|app| {
        app.app_file != webapp.app_file
            && app.browser_id() == webapp.browser_id()
            && app.profile_kind() == webapp.profile_kind()
    })
}

pub fn generate_app_file(browser: &str, url: &str) -> String {
    let browser_id = BrowserId::from(browser);
    let short = if browser_id.is_viewer() {
        return desktop::viewer_desktop_filename(url);
    } else {
        let browser_lower = browser_id.as_str().to_lowercase();
        if browser_lower.contains("chrom") {
            "chrome"
        } else if browser_lower.contains("brave") {
            "brave"
        } else if browser_lower.contains("edge") {
            "msedge"
        } else if browser_lower.contains("vivaldi") {
            "vivaldi"
        } else {
            browser_id.as_str()
        }
    };

    let cleaned = url.replace("https://", "").replace("http://", "");
    let cleaned = cleaned.split('?').next().unwrap_or(&cleaned);
    let cleaned = cleaned.replace('/', "__");

    let mut filename = format!("{short}-{cleaned}-Default.desktop");
    if !filename.contains("__") {
        filename = filename.replace("-Default", "__-Default");
    }

    let apps_dir = config::applications_dir();
    if apps_dir.join(&filename).exists() {
        let base = filename.clone();
        let mut index = 2;
        loop {
            filename = base.replace(".desktop", &format!("-BigWebApp{index}.desktop"));
            if !apps_dir.join(&filename).exists() {
                break;
            }
            index += 1;
        }
    }

    filename
}

pub(super) fn cleanup_viewer_data(url: &str) {
    let app_id = desktop::desktop_file_id(url);
    let legacy_app_id = desktop::legacy_host_desktop_file_id(url);
    let collection = super::super::repository::load_webapps();

    cleanup_viewer_data_for_id(url, &app_id, &collection, desktop::desktop_file_id);
    if legacy_app_id != app_id {
        cleanup_viewer_data_for_id(
            url,
            &legacy_app_id,
            &collection,
            desktop::legacy_host_desktop_file_id,
        );
    }
}

fn cleanup_viewer_data_for_id(
    url: &str,
    app_id: &str,
    collection: &webapps_core::models::WebAppCollection,
    id_for_url: fn(&str) -> String,
) {
    let still_in_use = collection
        .webapps
        .iter()
        .any(|app| app.app_url != url && id_for_url(&app.app_url) == app_id);
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

pub(super) fn validate_webapp(webapp: &WebApp) -> Result<()> {
    webapp.validate_domain()
}

pub(super) fn profile_dir_for(webapp: &WebApp) -> Result<Option<PathBuf>> {
    let ProfileKind::Custom(profile_name) = webapp.profile_kind() else {
        return Ok(None);
    };

    webapp.browser_id().validate()?;
    ProfileKind::Custom(profile_name.clone()).validate()?;

    Ok(Some(
        config::profiles_dir()
            .join(webapp.browser_id().as_str())
            .join(profile_name),
    ))
}

pub(super) fn cleanup_deleted_app(webapp: &WebApp, delete_profile: bool) -> Result<()> {
    match webapp.app_mode {
        AppMode::App => cleanup_viewer_data(&webapp.app_url),
        AppMode::Browser => cleanup_browser_profile(webapp, delete_profile)?,
    }
    cleanup_persisted_icon(webapp);

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

/// Remove the per-webapp icon we copied into our data dir at save time so
/// stale icons don't accumulate after deletes. Only touches files that match
/// the stem `webapp-<desktop_file_id>` — leaves anything else alone.
fn cleanup_persisted_icon(webapp: &WebApp) {
    let current_stem = format!("webapp-{}", desktop::desktop_file_id(&webapp.app_url));
    remove_persisted_icon_stem(&current_stem);

    let legacy_stem = format!(
        "webapp-{}",
        desktop::legacy_host_desktop_file_id(&webapp.app_url)
    );
    if legacy_stem != current_stem {
        remove_persisted_icon_stem(&legacy_stem);
    }
}

fn remove_persisted_icon_stem(stem: &str) {
    let icons_dir = desktop::webapp_icons_dir();
    for ext in ["png", "svg", "ico", "webp", "jpg", "jpeg"] {
        let candidate = icons_dir.join(format!("{stem}.{ext}"));
        if let Err(err) = fs::remove_file(&candidate) {
            if err.kind() != std::io::ErrorKind::NotFound {
                log::warn!("Remove persisted icon {}: {err}", candidate.display());
            }
        }
    }
}
