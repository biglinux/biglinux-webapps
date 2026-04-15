use crate::config;
use crate::models::{AppMode, WebApp};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Generate .desktop file content for a webapp
pub fn generate_desktop_entry(webapp: &WebApp) -> String {
    let app_id = desktop_file_id(&webapp.app_url);
    let wm_class = derive_wm_class(&webapp.app_url, &webapp.app_mode);
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
        format!("Categories={}", webapp.app_categories),
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

/// Derive WM class from URL + mode (matches original big-webapps script)
fn derive_wm_class(url: &str, mode: &AppMode) -> String {
    match mode {
        AppMode::App => {
            let app_id = url
                .replace("https://", "")
                .replace("http://", "")
                .replace('/', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>();
            format!("br.com.biglinux.webapp.{app_id}")
        }
        AppMode::Browser => derive_class_from_url(url),
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
        format!("biglinux-webapp-{}.desktop", desktop_file_id(&webapp.app_url))
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
    Ok(())
}

/// Remove .desktop file
pub fn remove_desktop_entry(webapp: &WebApp) -> Result<()> {
    let path = desktop_file_path(webapp);
    if path.exists() {
        fs::remove_file(&path)?;
        log::info!("Removed desktop entry: {}", path.display());
    }
    Ok(())
}

/// Strip chars that could break desktop file Exec or shell parsing
fn sanitize_desktop_field(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '"' && *c != '\'' && *c != '`' && *c != '\\' && *c != '\n' && *c != '\r' && *c != '$')
        .collect()
}
