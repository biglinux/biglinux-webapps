use anyhow::Result;
use std::path::PathBuf;

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{BrowserId, ProfileKind, WebApp};

use super::super::repository::load_webapps;

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
