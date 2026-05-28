use webapps_core::models::{BrowserId, WebApp};

use super::shell::shell_split;

pub(super) fn parse_legacy_desktop(filename: &str, content: &str) -> Option<WebApp> {
    let mut app = WebApp {
        app_file: filename.to_string(),
        ..Default::default()
    };

    for line in content.lines() {
        let line = line.trim();
        if should_stop_parsing(line, &app) {
            if !app.app_name.is_empty() {
                break;
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("Name=") {
            app.app_name = value.to_string();
        } else if let Some(value) = line.strip_prefix("Icon=") {
            app.app_icon = value.to_string();
        } else if let Some(value) = line.strip_prefix("Categories=") {
            app.app_categories = value.to_string();
        } else if let Some(value) = line.strip_prefix("MimeType=") {
            app.mime_types = value.to_string();
        } else if let Some(value) = line.strip_prefix("Comment=") {
            app.comment = value.to_string();
        } else if let Some(value) = line.strip_prefix("Exec=") {
            parse_exec_line(value, &mut app);
        }
    }

    if app.app_name.is_empty() || app.app_url.is_empty() {
        return None;
    }

    Some(app)
}

fn should_stop_parsing(line: &str, app: &WebApp) -> bool {
    line.starts_with("[Desktop Action")
        || (line.starts_with('[') && line != "[Desktop Entry]" && !line.starts_with('#'))
            && !app.app_name.is_empty()
}

pub(super) fn parse_exec_line(exec: &str, app: &mut WebApp) {
    if exec.starts_with("big-webapps-viewer") {
        parse_viewer_exec(exec, app);
    } else if exec.starts_with("big-webapps-exec") {
        parse_browser_exec(exec, app);
    }
}

fn parse_viewer_exec(exec: &str, app: &mut WebApp) {
    app.app_mode = webapps_core::models::AppMode::App;
    app.browser = BrowserId::VIEWER.to_string();

    for part in shell_split(exec) {
        if let Some(value) = part.strip_prefix("--url=") {
            app.app_url = value.trim_matches('"').to_string();
        } else if let Some(value) = part.strip_prefix("--icon=") {
            let icon = value.trim_matches('"');
            if !icon.is_empty() {
                app.app_icon = icon.to_string();
            }
        }
    }
}

fn parse_browser_exec(exec: &str, app: &mut WebApp) {
    app.app_mode = webapps_core::models::AppMode::Browser;

    for (index, part) in shell_split(exec).iter().enumerate() {
        if let Some(value) = part.strip_prefix("filename=") {
            app.app_file = value.trim_matches('"').to_string();
        } else if let Some(value) = part.strip_prefix("--app=") {
            app.app_url = value.trim_matches('"').to_string();
        } else if let Some(value) = part.strip_prefix("--profile-directory=") {
            app.app_profile = value.trim_matches('"').to_string();
        } else if index == 2 && !part.starts_with('-') && !part.contains('=') {
            app.browser = part.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exec_viewer_mode() {
        let exec = r#"big-webapps-viewer --url="https://youtube.com" --name="YouTube" --icon="/path/icon.png" --app-id="yt""#;
        let mut app = WebApp::default();
        parse_exec_line(exec, &mut app);
        assert_eq!(app.app_mode, webapps_core::models::AppMode::App);
        assert_eq!(app.browser, BrowserId::VIEWER);
        assert_eq!(app.app_url, "https://youtube.com");
        assert_eq!(app.app_icon, "/path/icon.png");
    }

    #[test]
    fn parse_exec_browser_mode() {
        let exec = r#"big-webapps-exec filename="test.desktop" google-chrome --class="WebApp" --profile-directory=Profile1 --app="https://gmail.com""#;
        let mut app = WebApp::default();
        parse_exec_line(exec, &mut app);
        assert_eq!(app.app_mode, webapps_core::models::AppMode::Browser);
        assert_eq!(app.browser, "google-chrome");
        assert_eq!(app.app_url, "https://gmail.com");
        assert_eq!(app.app_profile, "Profile1");
        assert_eq!(app.app_file, "test.desktop");
    }

    #[test]
    fn parse_exec_unknown_prefix() {
        let exec = "some-other-command --url=test";
        let mut app = WebApp::default();
        parse_exec_line(exec, &mut app);
        assert_eq!(app.app_url, "");
        assert_eq!(app.browser, "");
    }

    #[test]
    fn parse_legacy_desktop_basic() {
        let content = "[Desktop Entry]\nName=Test App\nIcon=test-icon\nCategories=Network;\nExec=big-webapps-viewer --url=\"https://example.com\"\n";
        let app = parse_legacy_desktop("test.desktop", content).unwrap();
        assert_eq!(app.app_name, "Test App");
        assert_eq!(app.app_icon, "test-icon");
        assert_eq!(app.app_categories, "Network;");
        assert_eq!(app.app_url, "https://example.com");
        assert_eq!(app.app_file, "test.desktop");
    }

    #[test]
    fn parse_legacy_desktop_missing_name() {
        let content = "[Desktop Entry]\nIcon=test-icon\nExec=big-webapps-viewer --url=\"https://example.com\"\n";
        let result = parse_legacy_desktop("test.desktop", content);
        assert!(result.is_none() || result.unwrap().app_name.is_empty());
    }
}
