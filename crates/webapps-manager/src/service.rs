use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use webapps_core::config;
use webapps_core::desktop;
use webapps_core::models::{Browser, BrowserCollection, WebApp, WebAppCollection};

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
    fs::write(webapps_json_path(), json)?;
    Ok(())
}

pub fn create_webapp(webapp: &WebApp) -> Result<()> {
    let mut col = load_webapps();
    col.add(webapp.clone());
    save_webapps(&col)?;
    desktop::install_desktop_entry(webapp)?;
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

// -- browser detection --

pub fn detect_browsers() -> BrowserCollection {
    let known_browsers = [
        ("firefox", "/usr/bin/firefox"),
        ("firefox-developer-edition", "/usr/bin/firefox-developer-edition"),
        ("librewolf", "/usr/bin/librewolf"),
        ("google-chrome-stable", "/usr/bin/google-chrome-stable"),
        ("google-chrome-beta", "/usr/bin/google-chrome-beta"),
        ("google-chrome-unstable", "/usr/bin/google-chrome-unstable"),
        ("chromium", "/usr/bin/chromium"),
        ("brave-browser", "/usr/bin/brave-browser-stable"),
        ("brave-browser-beta", "/usr/bin/brave-browser-beta"),
        ("brave-browser-nightly", "/usr/bin/brave-browser-nightly"),
        ("microsoft-edge-stable", "/usr/bin/microsoft-edge-stable"),
        ("microsoft-edge-beta", "/usr/bin/microsoft-edge-beta"),
        ("vivaldi-stable", "/usr/bin/vivaldi-stable"),
        ("vivaldi-beta", "/usr/bin/vivaldi-beta"),
        ("vivaldi-snapshot", "/usr/bin/vivaldi-snapshot"),
        ("ungoogled-chromium", "/usr/bin/ungoogled-chromium"),
    ];

    let mut browsers: Vec<Browser> = Vec::new();

    for (id, path) in &known_browsers {
        if Path::new(path).exists() {
            browsers.push(Browser {
                browser_id: id.to_string(),
                is_default: false,
            });
        }
    }

    // detect flatpak browsers
    if let Ok(output) = std::process::Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let flatpak_map = [
            ("org.mozilla.firefox", "flatpak-firefox"),
            ("com.google.Chrome", "flatpak-google-chrome-stable"),
            ("org.chromium.Chromium", "flatpak-chromium"),
            ("com.brave.Browser", "flatpak-brave-browser"),
            ("com.microsoft.Edge", "flatpak-microsoft-edge-stable"),
            ("com.vivaldi.Vivaldi", "flatpak-vivaldi-stable"),
            ("io.gitlab.librewolf-community", "flatpak-librewolf"),
        ];
        for (flatpak_id, browser_id) in &flatpak_map {
            if stdout.lines().any(|l| l.trim() == *flatpak_id) {
                browsers.push(Browser {
                    browser_id: browser_id.to_string(),
                    is_default: false,
                });
            }
        }
    }

    // detect system default
    let default_id = detect_default_browser();

    let mut col = BrowserCollection {
        browsers,
        default_id: None,
    };
    if let Some(id) = default_id {
        col.set_default(&id);
    }
    col
}

fn detect_default_browser() -> Option<String> {
    let output = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;
    let desktop_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if desktop_name.is_empty() {
        return None;
    }
    match_desktop_to_browser(&desktop_name)
}

fn match_desktop_to_browser(desktop: &str) -> Option<String> {
    let d = desktop.to_lowercase();
    let mappings = [
        ("firefox", "firefox"),
        ("firefox-developer", "firefox-developer-edition"),
        ("librewolf", "librewolf"),
        ("google-chrome-stable", "google-chrome-stable"),
        ("google-chrome-beta", "google-chrome-beta"),
        ("google-chrome-unstable", "google-chrome-unstable"),
        ("chromium", "chromium"),
        ("brave-browser-stable", "brave-browser"),
        ("brave-browser-beta", "brave-browser-beta"),
        ("brave-browser-nightly", "brave-browser-nightly"),
        ("microsoft-edge-stable", "microsoft-edge-stable"),
        ("microsoft-edge-beta", "microsoft-edge-beta"),
        ("vivaldi-stable", "vivaldi-stable"),
        ("vivaldi-beta", "vivaldi-beta"),
        ("vivaldi-snapshot", "vivaldi-snapshot"),
    ];
    for (pattern, id) in &mappings {
        if d.contains(pattern) {
            return Some(id.to_string());
        }
    }
    None
}

// -- export / import --

pub fn export_webapps(zip_path: &Path) -> Result<String> {
    let col = load_webapps();
    if col.webapps.is_empty() {
        return Ok("no_webapps".into());
    }

    let file = fs::File::create(zip_path).context("Create zip file")?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // write manifest
    let manifest = serde_json::to_string_pretty(&col.webapps)?;
    zip.start_file("webapps.json", options)?;
    zip.write_all(manifest.as_bytes())?;

    // copy icons
    for app in &col.webapps {
        if app.app_icon_url.is_empty() {
            continue;
        }
        let icon_path = Path::new(&app.app_icon_url);
        if icon_path.is_file() {
            let fname = icon_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if !fname.is_empty() {
                zip.start_file(format!("icons/{fname}"), options)?;
                let mut f = fs::File::open(icon_path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            }
        }
    }

    zip.finish()?;
    Ok("ok".into())
}

pub fn import_webapps(zip_path: &Path) -> Result<(usize, usize)> {
    let file = fs::File::open(zip_path).context("Open zip file")?;
    let mut archive = zip::ZipArchive::new(file)?;

    // read manifest
    let manifest = {
        let mut entry = archive.by_name("webapps.json")?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        buf
    };
    let imported_apps: Vec<WebApp> = serde_json::from_str(&manifest)?;

    // extract icons
    let icons_dir = config::data_dir().join("icons");
    fs::create_dir_all(&icons_dir)?;
    let icons_canonical = icons_dir.canonicalize()?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.starts_with("icons/") {
            let fname = name.strip_prefix("icons/").unwrap_or(&name);
            // strict filename: must be non-empty, no path separators, no ..
            if fname.is_empty()
                || fname.contains('/')
                || fname.contains('\\')
                || fname.contains("..")
            {
                continue;
            }
            let dest = icons_dir.join(fname);
            // verify dest stays within icons_dir
            if let Ok(canonical) = dest.parent().map(|p| p.canonicalize()).transpose() {
                if canonical.as_deref() != Some(icons_canonical.as_path()) {
                    continue;
                }
            }
            let mut out = fs::File::create(&dest)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }

    // import webapps, skip duplicates
    let existing = load_webapps();
    let mut imported = 0usize;
    let mut duplicates = 0usize;

    for app in imported_apps {
        let is_dup = existing.webapps.iter().any(|e| {
            e.app_name == app.app_name && e.app_url == app.app_url
        });
        if is_dup {
            duplicates += 1;
            continue;
        }
        // generate new app_file
        let mut new_app = app;
        new_app.app_file = generate_app_file(&new_app.browser, &new_app.app_url);
        if let Err(e) = create_webapp(&new_app) {
            log::error!("Import webapp {}: {e}", new_app.app_name);
        } else {
            imported += 1;
        }
    }

    Ok((imported, duplicates))
}

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
    let mut app = WebApp::default();
    app.app_file = filename.to_string();

    for line in content.lines() {
        let line = line.trim();
        // stop at Desktop Action sections — only parse [Desktop Entry]
        if line.starts_with("[Desktop Action") || (line.starts_with('[') && line != "[Desktop Entry]" && !line.starts_with("#")) {
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
    let cleaned = url
        .replace("https://", "")
        .replace("http://", "");
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
    // user-local hicolor
    let hicolor_user = local_icons.join("hicolor/scalable/apps");
    for ext in &["svg", "png"] {
        let candidate = hicolor_user.join(format!("{icon}.{ext}"));
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }
    // system hicolor
    let hicolor_sys = PathBuf::from("/usr/share/icons/hicolor/scalable/apps");
    for ext in &["svg", "png"] {
        let candidate = hicolor_sys.join(format!("{icon}.{ext}"));
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
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
