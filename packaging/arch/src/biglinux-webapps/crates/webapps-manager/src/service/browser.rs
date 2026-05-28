use std::path::Path;

use webapps_core::browsers::browser_defs;
use webapps_core::models::{Browser, BrowserCollection};

/// Detect all installed browsers (native + Flatpak) and identify the system default.
///
/// Browser support is driven by `/usr/share/biglinux-webapps/browsers.toml` (or the
/// version embedded in the binary as a fallback). Add new browsers there — no
/// recompilation needed.
pub fn detect_browsers() -> BrowserCollection {
    let defs = browser_defs();
    let mut browsers: Vec<Browser> = Vec::new();

    // Native: first existing candidate path wins
    for def in defs {
        if def.native_paths.iter().any(|p| Path::new(p).exists()) {
            browsers.push(Browser {
                browser_id: def.id.clone(),
                is_default: false,
            });
        }
    }

    // Flatpak: entries with flatpak_app_id + flatpak_id
    if let Ok(output) = std::process::Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for def in defs {
            if let (Some(app_id), Some(fid)) = (&def.flatpak_app_id, &def.flatpak_id) {
                if stdout.lines().any(|l| l.trim() == app_id.as_str()) {
                    browsers.push(Browser {
                        browser_id: fid.clone(),
                        is_default: false,
                    });
                }
            }
        }
    }

    let default_id = detect_default_browser(defs);
    let mut col = BrowserCollection {
        browsers,
        default_id: None,
    };
    if let Some(id) = default_id {
        col.set_default(&id);
    }
    col
}

fn detect_default_browser(defs: &[webapps_core::browsers::BrowserDef]) -> Option<String> {
    let desktop = query_default_browser()?;
    resolve_default_id(defs, &desktop)
}

fn query_default_browser() -> Option<String> {
    // xdg-settings is the canonical source; xdg-mime is the fallback for
    // distros/desktops where xdg-settings isn't configured.
    let primary = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase())
        .filter(|s| !s.is_empty());
    if primary.is_some() {
        return primary;
    }
    std::process::Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/http"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase())
        .filter(|s| !s.is_empty())
}

/// Map a `.desktop` filename to a `browser_id`. Collects every pattern, alias,
/// or Flatpak app-id that appears in the desktop name and keeps the longest —
/// so `brave-beta.desktop` picks `brave-beta` over `brave`, and
/// `google-chrome.desktop` picks `google-chrome-stable` via its alias even
/// though the full `google-chrome-stable` pattern isn't present.
fn resolve_default_id(
    defs: &[webapps_core::browsers::BrowserDef],
    desktop: &str,
) -> Option<String> {
    let mut candidates: Vec<(usize, String)> = Vec::new();
    for def in defs {
        if !def.desktop_pattern.is_empty() && desktop.contains(&def.desktop_pattern) {
            candidates.push((def.desktop_pattern.len(), def.id.clone()));
        }
        for alias in &def.desktop_aliases {
            if !alias.is_empty() && desktop.contains(alias.as_str()) {
                candidates.push((alias.len(), def.id.clone()));
            }
        }
        // Flatpak default: xdg-settings returns the reverse-DNS app-id; in
        // that case we hand back the `flatpak_id`, not the native `id`.
        if let (Some(app_id), Some(flatpak_id)) = (&def.flatpak_app_id, &def.flatpak_id) {
            let lowered = app_id.to_lowercase();
            if !lowered.is_empty() && desktop.contains(&lowered) {
                candidates.push((lowered.len(), flatpak_id.clone()));
            }
        }
    }
    candidates.sort_by_key(|(len, _)| std::cmp::Reverse(*len));
    candidates.into_iter().next().map(|(_, id)| id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use webapps_core::browsers::browser_defs;

    #[test]
    fn chrome_desktop_resolves_to_stable_via_alias() {
        let id = resolve_default_id(browser_defs(), "google-chrome.desktop");
        assert_eq!(id.as_deref(), Some("google-chrome-stable"));
    }

    #[test]
    fn chrome_beta_desktop_prefers_specific_pattern() {
        let id = resolve_default_id(browser_defs(), "google-chrome-beta.desktop");
        assert_eq!(id.as_deref(), Some("google-chrome-beta"));
    }

    #[test]
    fn edge_desktop_resolves_to_stable_via_alias() {
        let id = resolve_default_id(browser_defs(), "microsoft-edge.desktop");
        assert_eq!(id.as_deref(), Some("microsoft-edge-stable"));
    }

    #[test]
    fn vivaldi_desktop_resolves_to_stable_via_alias() {
        let id = resolve_default_id(browser_defs(), "vivaldi.desktop");
        assert_eq!(id.as_deref(), Some("vivaldi-stable"));
    }

    #[test]
    fn brave_beta_desktop_prefers_specific_pattern() {
        let id = resolve_default_id(browser_defs(), "brave-browser-beta.desktop");
        assert_eq!(id.as_deref(), Some("brave-beta"));
    }

    #[test]
    fn flatpak_firefox_desktop_returns_flatpak_id() {
        let id = resolve_default_id(browser_defs(), "org.mozilla.firefox.desktop");
        assert_eq!(id.as_deref(), Some("flatpak-firefox"));
    }

    #[test]
    fn unknown_desktop_returns_none() {
        let id = resolve_default_id(browser_defs(), "some-other-browser.desktop");
        assert!(id.is_none());
    }
}
