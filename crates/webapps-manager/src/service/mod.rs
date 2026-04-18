mod browser;
mod io;
mod migration;

pub use browser::detect_browsers;
pub use io::{export_webapps, import_webapps};
pub use migration::migrate_legacy_desktops;

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{WebApp, WebAppCollection};

/// Persistent storage file for webapps list
fn webapps_json_path() -> PathBuf {
    config::data_dir().join("webapps.json")
}

// -- webapp CRUD --

pub fn load_webapps() -> WebAppCollection {
    let path = webapps_json_path();
    if !path.exists() {
        return WebAppCollection::default();
    }
    match fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<Vec<serde_json::Value>>(&data) {
            Ok(vals) => WebAppCollection::load_from_json(&vals),
            Err(e) => {
                log::error!("Parse webapps.json: {e}");
                WebAppCollection::default()
            }
        },
        Err(e) => {
            log::error!("Read webapps.json: {e}");
            WebAppCollection::default()
        }
    }
}

pub fn save_webapps(collection: &WebAppCollection) -> Result<()> {
    let dir = config::data_dir();
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(&collection.webapps)?;
    // atomic write: tmp file + rename → prevent corruption on crash
    let path = webapps_json_path();
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn create_webapp(webapp: &WebApp) -> Result<()> {
    let mut app = webapp.clone();
    // ensure app_file is populated → needed for update/remove by file
    if app.app_file.is_empty() {
        app.app_file = format!(
            "biglinux-webapp-{}.desktop",
            desktop::desktop_file_id(&app.app_url)
        );
    }
    let mut col = load_webapps();
    col.add(app.clone());
    save_webapps(&col)?;
    desktop::install_desktop_entry(&app)?;
    Ok(())
}

pub fn update_webapp(webapp: &WebApp) -> Result<()> {
    let mut col = load_webapps();
    col.remove_by_file(&webapp.app_file);
    col.add(webapp.clone());
    save_webapps(&col)?;
    desktop::install_desktop_entry(webapp)?;
    Ok(())
}

pub fn delete_webapp(webapp: &WebApp, delete_profile: bool) -> Result<()> {
    let mut col = load_webapps();
    col.remove_by_file(&webapp.app_file);
    save_webapps(&col)?;
    desktop::remove_desktop_entry(webapp)?;

    if delete_profile {
        cleanup_profile(webapp);
    }
    // cleanup viewer data if app mode
    if webapp.app_mode == webapps_core::models::AppMode::App {
        cleanup_viewer_data(&webapp.app_url);
    }
    Ok(())
}

pub fn delete_all_webapps() -> Result<()> {
    let col = load_webapps();
    for app in &col.webapps {
        let _ = desktop::remove_desktop_entry(app);
        if app.app_mode == webapps_core::models::AppMode::App {
            cleanup_viewer_data(&app.app_url);
        }
    }
    save_webapps(&WebAppCollection::default())?;
    Ok(())
}

fn cleanup_viewer_data(url: &str) {
    let app_id = desktop::desktop_file_id(url);
    // geometry config
    let geom = config::config_dir().join(format!("{app_id}.json"));
    let _ = fs::remove_file(geom);
    // session data
    let data = config::data_dir().join(&app_id);
    let _ = fs::remove_dir_all(data);
    // cache
    let cache = config::cache_dir().join(&app_id);
    let _ = fs::remove_dir_all(cache);
}

fn cleanup_profile(webapp: &WebApp) {
    let profile_dir = config::profiles_dir()
        .join(&webapp.browser)
        .join(&webapp.app_profile);
    if profile_dir.exists() {
        let _ = fs::remove_dir_all(&profile_dir);
        log::info!("Removed profile: {}", profile_dir.display());
    }
}

/// Check if any other webapp shares same browser+profile
pub fn profile_shared(webapp: &WebApp) -> bool {
    let col = load_webapps();
    col.webapps.iter().any(|a| {
        a.app_file != webapp.app_file
            && a.browser == webapp.browser
            && a.app_profile == webapp.app_profile
    })
}

pub fn generate_app_file(browser: &str, url: &str) -> String {
    // short browser name — matches original big-webapps script
    let short = if browser == "__viewer__" {
        "viewer"
    } else {
        let b = browser.to_lowercase();
        if b.contains("chrom") {
            "chrome"
        } else if b.contains("brave") {
            "brave"
        } else if b.contains("edge") {
            "msedge"
        } else if b.contains("vivaldi") {
            "vivaldi"
        } else {
            browser
        }
    };

    // url → path component: strip scheme, strip query, / → __
    let cleaned = url.replace("https://", "").replace("http://", "");
    let cleaned = cleaned.split('?').next().unwrap_or(&cleaned);
    let cleaned = cleaned.replace('/', "__");

    // keep first __ occurrence, replace subsequent with _
    let mut filename = format!("{short}-{cleaned}-Default.desktop");
    if !filename.contains("__") {
        filename = filename.replace("-Default", "__-Default");
    }

    // dedup: check existing files
    let apps_dir = webapps_core::config::applications_dir();
    if apps_dir.join(&filename).exists() {
        let base = filename.clone();
        let mut i = 2;
        loop {
            filename = base.replace(".desktop", &format!("-BigWebApp{i}.desktop"));
            if !apps_dir.join(&filename).exists() {
                break;
            }
            i += 1;
        }
    }

    filename
}

// -- icon resolution --

/// Resolve icon to display path. Checks: absolute path → user icons → hicolor → system → theme name
pub fn resolve_icon_path(icon: &str) -> String {
    if icon.is_empty() {
        return "webapp-manager-generic".into();
    }
    // absolute path
    let p = Path::new(icon);
    if p.is_absolute() && p.exists() {
        return icon.to_string();
    }
    // user-local icons (flat)
    let local_icons = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("icons");
    for ext in &["png", "svg", "xpm"] {
        let candidate = local_icons.join(format!("{icon}.{ext}"));
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }
    // user-local hicolor — return icon name so GTK theme renders SVG at correct size
    let hicolor_user = local_icons.join("hicolor/scalable/apps");
    for ext in &["svg", "png"] {
        let candidate = hicolor_user.join(format!("{icon}.{ext}"));
        if candidate.exists() {
            return icon.to_string();
        }
    }
    // system hicolor — return icon name for GTK theme lookup
    let hicolor_sys = PathBuf::from("/usr/share/icons/hicolor/scalable/apps");
    for ext in &["svg", "png"] {
        let candidate = hicolor_sys.join(format!("{icon}.{ext}"));
        if candidate.exists() {
            return icon.to_string();
        }
    }
    // system icons dir (biglinux-specific)
    let sys = config::system_icons_dir();
    for ext in &["svg", "png"] {
        let candidate = sys.join(format!("{icon}.{ext}"));
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }
    // fallback: return as icon-name for GTK theme lookup
    icon.to_string()
}

/// Check if welcome dialog should show (first run)
pub fn should_show_welcome() -> bool {
    let flag = config::config_dir().join("welcome_shown.json");
    !flag.exists()
}

pub fn mark_welcome_shown() {
    let dir = config::config_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("welcome_shown.json"), "true");
}
