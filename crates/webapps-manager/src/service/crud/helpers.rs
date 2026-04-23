use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

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
        "viewer"
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

    // `desktop_file_id` strips dots from the host, so distinct hosts can collide
    // (e.g. `docs.google.com` and `docsgoogle.com` both yield "docsgooglecom").
    // Skip cleanup when any other persisted webapp resolves to the same app_id —
    // wiping its profile/cache by accident would surprise the user.
    let collection = super::super::repository::load_webapps();
    let still_in_use = collection
        .webapps
        .iter()
        .any(|app| app.app_url != url && desktop::desktop_file_id(&app.app_url) == app_id);
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
    let data_dir = config::data_dir().join(&app_id);
    if let Err(err) = fs::remove_dir_all(&data_dir) {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to remove viewer data {}: {err}", data_dir.display());
        }
    }
    let cache_dir = config::cache_dir().join(&app_id);
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

    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir)
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
    if delete_profile {
        cleanup_profile(webapp)?;
    }
    if webapp.app_mode == AppMode::App {
        cleanup_viewer_data(&webapp.app_url);
    }

    Ok(())
}
