use crate::config;
use crate::models::{AppMode, WebApp};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Generate .desktop file content for a webapp
pub fn generate_desktop_entry(webapp: &WebApp) -> String {
    let app_id = desktop_file_id(&webapp.app_url);
    let wm_class = derive_wm_class(webapp);
    let has_mime = !webapp.mime_types.is_empty();
    let file_arg = if has_mime { " %f" } else { "" };

    // sanitize fields for shell/desktop safety — strip quotes and control chars
    let safe_url = sanitize_desktop_field(&webapp.app_url);
    let safe_name = sanitize_desktop_field(&webapp.app_name);
    let safe_icon = sanitize_desktop_field(&webapp.app_icon);
    let safe_file = sanitize_desktop_field(&webapp.app_file);
    let safe_browser = sanitize_desktop_field(&webapp.browser);
    let safe_profile = sanitize_desktop_field(&webapp.app_profile);

    let exec = match webapp.app_mode {
        AppMode::App => format!(
            "big-webapps-viewer --url=\"{safe_url}\" --name=\"{safe_name}\" --icon=\"{safe_icon}\" --app-id=\"{app_id}\"{file_arg}",
        ),
        AppMode::Browser => {
            let class = derive_class_from_url(&webapp.app_url);
            format!(
                "big-webapps-exec filename=\"{safe_file}\" {safe_browser} --class=\"{class}\" --profile-directory={safe_profile} --app=\"{safe_url}\"{file_arg}",
            )
        }
    };

    let mut lines = vec![
        "[Desktop Entry]".to_string(),
        "Version=1.0".to_string(),
        "Terminal=false".to_string(),
        "Type=Application".to_string(),
        format!("Name={}", webapp.app_name),
        format!("Exec={}", exec),
        format!("Icon={}", webapp.app_icon),
        format!("StartupWMClass={}", wm_class),
        format!(
            "Categories={}",
            if webapp.app_categories.ends_with(';') {
                webapp.app_categories.clone()
            } else {
                format!("{};", webapp.app_categories)
            }
        ),
    ];

    if !webapp.mime_types.is_empty() {
        lines.push(format!("MimeType={}", webapp.mime_types));
    }
    if !webapp.comment.is_empty() {
        lines.push(format!("Comment={}", webapp.comment));
    }
    if !webapp.generic_name.is_empty() {
        lines.push(format!("GenericName={}", webapp.generic_name));
    }
    if !webapp.keywords.is_empty() {
        lines.push(format!("Keywords={}", webapp.keywords));
    }

    lines.push("StartupNotify=false".to_string());

    // SoftwareRender action — fallback for GPU issues
    lines.push(String::new());
    lines.push("Actions=SoftwareRender;".to_string());
    lines.push(String::new());
    lines.push("[Desktop Action SoftwareRender]".to_string());
    lines.push("Name=Software Render".to_string());
    lines.push(format!("Exec=SoftwareRender {exec}"));

    lines.join("\n") + "\n"
}

/// Derive WM class — must match what the browser/viewer actually sets.
/// App mode → custom Freedesktop reverse-DNS.
/// Browser mode → `{browser_prefix}-{url_class}-{profile}` (Chrome/Brave convention).
fn derive_wm_class(webapp: &WebApp) -> String {
    match webapp.app_mode {
        AppMode::App => {
            let app_id = webapp
                .app_url
                .replace("https://", "")
                .replace("http://", "")
                .replace('/', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>();
            format!("br.com.biglinux.webapp.{app_id}")
        }
        AppMode::Browser => {
            let url_class = browser_url_class(&webapp.app_url);
            let prefix = browser_wm_prefix(&webapp.browser);
            // --user-data-dir always creates "Default" profile internally
            format!("{prefix}-{url_class}-Default")
        }
    }
}

/// Map browser binary/id → WM_CLASS prefix that Chrome/Brave/Chromium actually use.
fn browser_wm_prefix(browser: &str) -> &str {
    let b = browser
        .strip_prefix("flatpak-")
        .unwrap_or(browser);
    match b {
        "brave" | "brave-browser" => "brave",
        "google-chrome" | "google-chrome-stable" => "google-chrome",
        "chromium" | "chromium-browser" => "chromium",
        "microsoft-edge" | "microsoft-edge-stable" => "microsoft-edge",
        "vivaldi" | "vivaldi-stable" => "vivaldi",
        other => other,
    }
}

/// URL → class matching Chrome/Brave convention: strip scheme, replace / → __.
/// Ensures trailing slash for root paths so `deezer.com` → `deezer.com__`.
fn browser_url_class(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();
        let path_class = path.replace('/', "__");
        format!("{host}{path_class}")
    } else {
        derive_class_from_url(url)
    }
}

/// URL → class: strip scheme, replace / with __
fn derive_class_from_url(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .replace('/', "__")
}

/// Derive desktop file ID from URL (hostname with dots removed)
pub fn desktop_file_id(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.replace('.', "")))
        .unwrap_or_else(|| "webapp".into())
}

/// Path for a webapp's .desktop file
pub fn desktop_file_path(webapp: &WebApp) -> PathBuf {
    let filename = if webapp.app_file.is_empty() {
        format!(
            "biglinux-webapp-{}.desktop",
            desktop_file_id(&webapp.app_url)
        )
    } else {
        webapp.app_file.clone()
    };
    config::applications_dir().join(filename)
}

/// Write .desktop file to disk
pub fn install_desktop_entry(webapp: &WebApp) -> Result<()> {
    let path = desktop_file_path(webapp);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = generate_desktop_entry(webapp);
    fs::write(&path, content)?;
    log::info!("Installed desktop entry: {}", path.display());
    refresh_desktop_database();
    Ok(())
}

/// Remove .desktop file
pub fn remove_desktop_entry(webapp: &WebApp) -> Result<()> {
    let path = desktop_file_path(webapp);
    if path.exists() {
        fs::remove_file(&path)?;
        log::info!("Removed desktop entry: {}", path.display());
        refresh_desktop_database();
    }
    Ok(())
}

/// Remove a desktop file by filename directly
pub fn remove_desktop_file(filename: &str) -> Result<()> {
    let path = config::applications_dir().join(filename);
    if path.exists() {
        fs::remove_file(&path)?;
        log::info!("Removed old desktop entry: {}", path.display());
    }
    Ok(())
}

/// Notify desktop environment of .desktop changes
fn refresh_desktop_database() {
    let apps_dir = config::applications_dir();
    let _ = std::process::Command::new("update-desktop-database")
        .arg(&apps_dir)
        .spawn();

    // GNOME Shell caches app positions in app-picker-layout.
    // Reset layout + ensure WebApps folder has correct category filter.
    if std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase()
        .contains("gnome")
    {
        let _ = std::process::Command::new("dconf")
            .args(["reset", "/org/gnome/shell/app-picker-layout"])
            .spawn();

        // ensure WebApps folder uses correct category (match .desktop Categories=Webapps;)
        let _ = std::process::Command::new("dconf")
            .args([
                "write",
                "/org/gnome/desktop/app-folders/folders/WebApps/categories",
                "['Webapps']",
            ])
            .spawn();
    }
}

/// Strip chars that could break desktop file Exec or shell parsing
fn sanitize_desktop_field(s: &str) -> String {
    s.chars()
        .filter(|c| {
            *c != '"'
                && *c != '\''
                && *c != '`'
                && *c != '\\'
                && *c != '\n'
                && *c != '\r'
                && *c != '$'
        })
        .collect()
}
