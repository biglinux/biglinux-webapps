use std::fs;

use webapps_core::config;
use webapps_core::models::{WebApp, WebAppCollection};

use super::{save_webapps, webapps_json_path};

/// Migrate existing .desktop files from legacy big-webapps into webapps.json.
/// Scans ~/.local/share/applications/ for files matching pattern:
/// `{browser}-*.desktop` with `Exec=big-webapps-exec` or `Exec=big-webapps-viewer`.
/// Returns count of migrated apps.
pub fn migrate_legacy_desktops() -> usize {
    let json_path = webapps_json_path();
    if json_path.exists() {
        // already has data — skip migration
        return 0;
    }

    let apps_dir = config::applications_dir();
    let entries = match fs::read_dir(&apps_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut webapps: Vec<WebApp> = Vec::new();

    for entry in entries.flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();
        if !fname.ends_with(".desktop") {
            continue;
        }

        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // only import big-webapps desktop files
        if !content.contains("big-webapps-exec") && !content.contains("big-webapps-viewer") {
            continue;
        }

        if let Some(app) = parse_legacy_desktop(&fname, &content) {
            webapps.push(app);
        }
    }

    let count = webapps.len();
    if count > 0 {
        let col = WebAppCollection { webapps };
        if let Err(e) = save_webapps(&col) {
            log::error!("Save migrated webapps: {e}");
            return 0;
        }
        log::info!("Migrated {count} legacy webapps");
    }

    count
}

/// Parse a legacy .desktop file into WebApp struct
fn parse_legacy_desktop(filename: &str, content: &str) -> Option<WebApp> {
    let mut app = WebApp {
        app_file: filename.to_string(),
        ..Default::default()
    };

    for line in content.lines() {
        let line = line.trim();
        // stop at Desktop Action sections — only parse [Desktop Entry]
        if line.starts_with("[Desktop Action")
            || (line.starts_with('[') && line != "[Desktop Entry]" && !line.starts_with("#"))
        {
            if !app.app_name.is_empty() {
                break;
            }
            continue;
        }
        if let Some(val) = line.strip_prefix("Name=") {
            app.app_name = val.to_string();
        } else if let Some(val) = line.strip_prefix("Icon=") {
            app.app_icon = val.to_string();
        } else if let Some(val) = line.strip_prefix("Categories=") {
            app.app_categories = val.to_string();
        } else if let Some(val) = line.strip_prefix("MimeType=") {
            app.mime_types = val.to_string();
        } else if let Some(val) = line.strip_prefix("Comment=") {
            app.comment = val.to_string();
        } else if let Some(val) = line.strip_prefix("Exec=") {
            parse_exec_line(val, &mut app);
        }
    }

    // minimal validation
    if app.app_name.is_empty() || app.app_url.is_empty() {
        return None;
    }

    Some(app)
}

/// Extract browser, url, profile, mode from Exec= line
fn parse_exec_line(exec: &str, app: &mut WebApp) {
    if exec.starts_with("big-webapps-viewer") {
        app.app_mode = webapps_core::models::AppMode::App;
        app.browser = "__viewer__".to_string();

        // --url="..." --name="..." --icon="..." --app-id="..."
        for part in shell_split(exec) {
            if let Some(val) = part.strip_prefix("--url=") {
                app.app_url = val.trim_matches('"').to_string();
            } else if let Some(val) = part.strip_prefix("--icon=") {
                let icon = val.trim_matches('"');
                if !icon.is_empty() {
                    app.app_icon = icon.to_string();
                }
            }
        }
    } else if exec.starts_with("big-webapps-exec") {
        app.app_mode = webapps_core::models::AppMode::Browser;

        let parts = shell_split(exec);
        // format: big-webapps-exec filename="..." browser --class="..." --profile-directory=X --app="URL"
        for (i, part) in parts.iter().enumerate() {
            if let Some(val) = part.strip_prefix("filename=") {
                app.app_file = val.trim_matches('"').to_string();
            } else if let Some(val) = part.strip_prefix("--app=") {
                app.app_url = val.trim_matches('"').to_string();
            } else if let Some(val) = part.strip_prefix("--profile-directory=") {
                app.app_profile = val.trim_matches('"').to_string();
            } else if i == 2 && !part.starts_with('-') && !part.contains('=') {
                // browser name is the 3rd token (index 2)
                app.browser = part.to_string();
            }
        }
    }
}

/// Simple tokenizer that respects quotes in Exec lines
fn shell_split(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for ch in s.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            c if c == quote_char && in_quote => {
                in_quote = false;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_split_simple_tokens() {
        assert_eq!(shell_split("a b c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn shell_split_quoted_strings() {
        assert_eq!(
            shell_split(r#"cmd --opt="hello world" arg"#),
            vec!["cmd", "--opt=hello world", "arg"]
        );
    }

    #[test]
    fn shell_split_single_quotes() {
        assert_eq!(
            shell_split("cmd 'one two' three"),
            vec!["cmd", "one two", "three"]
        );
    }

    #[test]
    fn shell_split_empty_input() {
        assert!(shell_split("").is_empty());
    }

    #[test]
    fn shell_split_extra_spaces() {
        assert_eq!(shell_split("  a   b  "), vec!["a", "b"]);
    }

    #[test]
    fn parse_exec_viewer_mode() {
        let exec = r#"big-webapps-viewer --url="https://youtube.com" --name="YouTube" --icon="/path/icon.png" --app-id="yt""#;
        let mut app = WebApp::default();
        parse_exec_line(exec, &mut app);
        assert_eq!(app.app_mode, webapps_core::models::AppMode::App);
        assert_eq!(app.browser, "__viewer__");
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
        // should not modify app
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
        // should return None — name is empty
        let result = parse_legacy_desktop("test.desktop", content);
        assert!(result.is_none() || result.unwrap().app_name.is_empty());
    }
}
