//! Browser definitions loaded from `browsers.toml`.
//!
//! The file at `/usr/share/biglinux-webapps/browsers.toml` is the live source
//! of truth. Distro maintainers can add or remove browser entries there without
//! recompiling. A copy of the same file is embedded in the binary as a fallback
//! for development environments where the system file is absent.
use std::sync::OnceLock;

use serde::Deserialize;

/// Definition of one supported browser, loaded from `browsers.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct BrowserDef {
    /// Stable ID stored in `webapps.json` (e.g. `"brave"`, `"flatpak-brave-browser"`).
    /// Must not change after webapps have been created with it.
    pub id: String,
    /// Human-readable label shown in the manager UI.
    pub display_name: String,
    /// Candidate binary paths for native detection; first existing path wins.
    #[serde(default)]
    pub native_paths: Vec<String>,
    /// WM_CLASS prefix set by Chromium-family browsers.
    /// Empty for Gecko-based browsers (Firefox, LibreWolf).
    #[serde(default)]
    pub wm_class_prefix: String,
    /// Substring matched against `xdg-settings get default-web-browser` output
    /// for default-browser detection.
    #[serde(default)]
    pub desktop_pattern: String,
    /// Extra substrings that also resolve to this browser when found in the
    /// system default (e.g. `google-chrome.desktop` → `google-chrome-stable`).
    /// Longer matches (more specific) always win over shorter ones.
    #[serde(default)]
    pub desktop_aliases: Vec<String>,
    /// `true` for Gecko-based browsers → `big-webapps-exec` uses different launch flags.
    #[serde(default)]
    pub firefox_like: bool,
    /// Flatpak application ID (e.g. `"com.brave.Browser"`).
    /// Absent means this browser has no tracked Flatpak variant.
    pub flatpak_app_id: Option<String>,
    /// `browser_id` assigned when detected via Flatpak (e.g. `"flatpak-brave-browser"`).
    /// Required when `flatpak_app_id` is set.
    pub flatpak_id: Option<String>,
    /// Legacy `browser_id` aliases still present in older `webapps.json` files.
    /// Matched by the launcher so existing entries keep resolving after renames.
    #[serde(default)]
    pub legacy_flatpak_ids: Vec<String>,
}

/// Embedded default — used when the system file is absent (dev builds, CI, etc.).
/// Path is relative to this source file: 3 levels up → workspace root.
const DEFAULT_TOML: &str =
    include_str!("../../../biglinux-webapps/usr/share/biglinux-webapps/browsers.toml");

/// System-installed path. Distro maintainers edit this file to add browsers.
const SYSTEM_PATH: &str = "/usr/share/biglinux-webapps/browsers.toml";

static BROWSER_DEFS: OnceLock<Vec<BrowserDef>> = OnceLock::new();

/// Return the loaded browser definitions (loaded once per process via [`OnceLock`]).
///
/// Reads from [`SYSTEM_PATH`] first; falls back to the embedded default on any
/// read or parse error, and logs a warning.
pub fn browser_defs() -> &'static [BrowserDef] {
    BROWSER_DEFS.get_or_init(load_browser_defs)
}

/// Find a definition by native `id`, `flatpak_id`, or any `legacy_flatpak_ids` entry.
pub fn find_def(browser_id: &str) -> Option<&'static BrowserDef> {
    browser_defs().iter().find(|d| {
        d.id == browser_id
            || d.flatpak_id.as_deref() == Some(browser_id)
            || d.legacy_flatpak_ids.iter().any(|lid| lid == browser_id)
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Root {
    browser: Vec<BrowserDef>,
}

fn parse_defs(src: &str) -> Result<Vec<BrowserDef>, toml::de::Error> {
    toml::from_str::<Root>(src).map(|r| r.browser)
}

fn load_browser_defs() -> Vec<BrowserDef> {
    let content = std::fs::read_to_string(SYSTEM_PATH).unwrap_or_default();
    if !content.is_empty() {
        match parse_defs(&content) {
            Ok(defs) => return defs,
            Err(e) => log::warn!("Failed to parse {SYSTEM_PATH}: {e}; using embedded defaults"),
        }
    }
    // Embedded copy is guaranteed valid — panic only if a developer breaks the file
    parse_defs(DEFAULT_TOML).expect("embedded browsers.toml must be valid TOML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_toml_parses_successfully() {
        let defs = parse_defs(DEFAULT_TOML).expect("embedded TOML must be valid");
        assert!(!defs.is_empty(), "Expected at least one browser definition");
    }

    #[test]
    fn brave_has_flatpak_entry() {
        let defs = parse_defs(DEFAULT_TOML).unwrap();
        let brave = defs
            .iter()
            .find(|d| d.id == "brave")
            .expect("brave must be defined");
        assert_eq!(brave.flatpak_app_id.as_deref(), Some("com.brave.Browser"));
        assert_eq!(brave.flatpak_id.as_deref(), Some("flatpak-brave-browser"));
    }

    #[test]
    fn firefox_is_gecko() {
        let defs = parse_defs(DEFAULT_TOML).unwrap();
        let ff = defs
            .iter()
            .find(|d| d.id == "firefox")
            .expect("firefox must be defined");
        assert!(ff.firefox_like);
        assert!(
            ff.wm_class_prefix.is_empty(),
            "Firefox has no WM_CLASS prefix"
        );
    }

    #[test]
    fn chromium_family_has_wm_prefix() {
        let defs = parse_defs(DEFAULT_TOML).unwrap();
        for id in &[
            "brave",
            "chromium",
            "google-chrome-stable",
            "vivaldi-stable",
        ] {
            let def = defs
                .iter()
                .find(|d| d.id == *id)
                .unwrap_or_else(|| panic!("{id} must be defined"));
            assert!(
                !def.wm_class_prefix.is_empty(),
                "{id} must have wm_class_prefix"
            );
            assert!(!def.firefox_like, "{id} must not be firefox_like");
        }
    }

    #[test]
    fn find_def_by_flatpak_id() {
        let defs = parse_defs(DEFAULT_TOML).unwrap();
        let found = defs
            .iter()
            .find(|d| d.flatpak_id.as_deref() == Some("flatpak-firefox"));
        assert!(
            found.is_some(),
            "flatpak-firefox must resolve to a definition"
        );
        assert_eq!(found.unwrap().id, "firefox");
    }
}
