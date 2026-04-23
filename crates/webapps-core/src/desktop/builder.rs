use crate::models::{AppMode, WebApp};

use super::paths::desktop_file_id;
use super::sanitize::{sanitize_desktop_field, sanitize_desktop_value};
use super::wm_class::{derive_class_from_url, derive_wm_class};

pub fn generate_desktop_entry(webapp: &WebApp) -> String {
    let app_id = desktop_file_id(&webapp.app_url);
    let exec = build_exec_command(webapp, &app_id);
    let wm_class = sanitize_desktop_field(&derive_wm_class(webapp));

    let mut lines = vec![
        "[Desktop Entry]".to_string(),
        "Version=1.0".to_string(),
        "Terminal=false".to_string(),
        "Type=Application".to_string(),
        format!("Name={}", sanitize_desktop_value(&webapp.app_name)),
        format!("Exec={exec}"),
        format!("Icon={}", sanitize_desktop_value(&webapp.app_icon)),
        format!("StartupWMClass={wm_class}"),
        format!(
            "Categories={}",
            sanitize_desktop_value(&webapp.category_list().to_desktop_string())
        ),
    ];

    push_optional_entry(&mut lines, "MimeType", &webapp.mime_types);
    push_optional_entry(&mut lines, "Comment", &webapp.comment);
    push_optional_entry(&mut lines, "GenericName", &webapp.generic_name);
    push_optional_entry(&mut lines, "Keywords", &webapp.keywords);

    lines.push("StartupNotify=false".to_string());
    lines.push(String::new());
    lines.push("Actions=SoftwareRender;".to_string());
    lines.push(String::new());
    lines.push("[Desktop Action SoftwareRender]".to_string());
    lines.push("Name=Software Render".to_string());
    lines.push(format!("Exec=SoftwareRender {exec}"));

    lines.join("\n") + "\n"
}

fn build_exec_command(webapp: &WebApp, app_id: &str) -> String {
    let file_arg = if webapp.mime_types.is_empty() {
        ""
    } else {
        " %f"
    };
    let safe_url = sanitize_desktop_field(&webapp.app_url);
    let safe_name = sanitize_desktop_field(&webapp.app_name);
    let safe_icon = sanitize_desktop_field(&webapp.app_icon);
    let safe_file = webapp
        .desktop_file_name()
        .map(|file_name| sanitize_desktop_field(file_name.as_str()))
        .unwrap_or_default();
    let safe_browser = sanitize_desktop_field(webapp.browser_id().as_str());
    let safe_profile = sanitize_desktop_field(webapp.profile_kind().as_str());

    match webapp.app_mode {
        AppMode::App => {
            let auto_hide = if webapp.auto_hide_headerbar {
                " --auto-hide-headerbar"
            } else {
                ""
            };
            format!(
                "big-webapps-viewer --url=\"{safe_url}\" --name=\"{safe_name}\" --icon=\"{safe_icon}\" --app-id=\"{app_id}\"{auto_hide}{file_arg}",
            )
        }
        AppMode::Browser => {
            let class = sanitize_desktop_field(&derive_class_from_url(&webapp.app_url));
            format!(
                "big-webapps-exec filename=\"{safe_file}\" {safe_browser} --class=\"{class}\" --profile-directory={safe_profile} --app=\"{safe_url}\"{file_arg}",
            )
        }
    }
}

fn push_optional_entry(lines: &mut Vec<String>, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }

    lines.push(format!("{key}={}", sanitize_desktop_value(value)));
}
