use crate::models::{AppMode, WebApp};

use super::paths::desktop_file_id;

pub(super) fn derive_wm_class(webapp: &WebApp) -> String {
    match webapp.app_mode {
        AppMode::App => {
            // Must match the GTK application_id set by the viewer
            // (`br.com.biglinux.webapp.{desktop_file_id}`), so Wayland compositors
            // can associate the window with this desktop entry — otherwise the
            // taskbar falls back to displaying the raw app_id and a generic icon.
            format!(
                "br.com.biglinux.webapp.{}",
                desktop_file_id(&webapp.app_url)
            )
        }
        // Chromium-family browsers (Brave/Chrome/Edge/Vivaldi/Chromium) ignore
        // `--class` when picking the xdg-shell `app_id` on Wayland — they
        // synthesize their own from the binary name, URL, and profile dir.
        // We mirror that synthesis here so `StartupWMClass` and the `.desktop`
        // filename both match what GNOME Shell actually sees, letting it
        // resolve the window to this entry (and its `Icon=`) without falling
        // back to the host browser's generic taskbar group.
        AppMode::Browser => chromium_browser_app_id(
            &chromium_short_name(webapp.browser_id().as_str()),
            &webapp.app_url,
            webapp.profile_kind().as_str(),
        ),
    }
}

/// Canonical `.desktop` filename for a browser-mode webapp — the same string
/// as the synthesized Wayland `app_id`, suffixed with `.desktop`. Matching the
/// filename to the running window's `app_id` lets GNOME Shell resolve the
/// window to this entry directly (its primary lookup), bypassing the
/// `StartupWMClass` fallback path.
pub fn canonical_browser_desktop_filename(webapp: &WebApp) -> String {
    let id = chromium_browser_app_id(
        &chromium_short_name(webapp.browser_id().as_str()),
        &webapp.app_url,
        webapp.profile_kind().as_str(),
    );
    format!("{id}.desktop")
}

/// Predict the Wayland `app_id` Chromium-family browsers set when launched
/// with `--app=URL --profile-directory=PROFILE`.
///
/// Observed pattern (Brave 148 on GNOME 47 Wayland):
/// `{short}-{host}_{path-with-/-as-_}-{profile}`. For
/// `https://open.spotify.com/intl-pt/`, browser `brave`, profile `Default`
/// → `brave-open.spotify.com__intl-pt_-Default`.
pub fn chromium_browser_app_id(short: &str, url: &str, profile: &str) -> String {
    let (host, path) = parsed_host_and_path(url);
    let path_part = path.replace('/', "_");
    let path_separator = if path.is_empty() { "" } else { "_" };
    format!("{short}-{host}{path_separator}{path_part}-{profile}")
}

/// Browser-id prefix used by each Chromium fork. Mirrors `argv[0]` of the
/// binary the user runs — the value Chromium injects as the leading segment
/// of its synthesized Wayland `app_id`.
pub(super) fn chromium_short_name(browser_id: &str) -> String {
    if let Some(definition) = crate::browsers::find_def(browser_id) {
        if !definition.wm_class_prefix.is_empty() {
            return definition.wm_class_prefix.clone();
        }
    }

    let lower = browser_id.to_lowercase();
    if lower.contains("brave") {
        "brave".to_string()
    } else if lower.contains("vivaldi") {
        "vivaldi".to_string()
    } else if lower.contains("edge") {
        "microsoft-edge".to_string()
    } else if lower.contains("opera") {
        "opera".to_string()
    } else if lower.contains("chromium") {
        "chromium".to_string()
    } else if lower.contains("chrom") {
        "google-chrome".to_string()
    } else {
        // Fall back to the raw browser id; unknown forks will still produce a
        // unique-but-app_id-mismatched value, which is no worse than the
        // pre-fix behavior.
        browser_id.to_string()
    }
}

fn parsed_host_and_path(url: &str) -> (String, String) {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("").to_string();
        let path = parsed.path().to_string();
        return (host, path);
    }
    let stripped = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    match stripped.split_once('/') {
        Some((host, rest)) => (host.to_string(), format!("/{rest}")),
        None => (stripped.to_string(), String::new()),
    }
}
