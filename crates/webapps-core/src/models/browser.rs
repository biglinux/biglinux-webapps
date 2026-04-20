use serde::{Deserialize, Serialize};

/// Browser engine family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserKind {
    Chromium,
    Firefox,
    Viewer,
}

/// Installed browser entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Browser {
    #[serde(default, alias = "browser")]
    pub browser_id: String,
    #[serde(default)]
    pub is_default: bool,
}

impl Browser {
    pub fn display_name(&self) -> &str {
        display_name_for(&self.browser_id)
    }

    pub fn icon_name(&self) -> String {
        icon_name_for(&self.browser_id)
    }

    pub fn kind(&self) -> BrowserKind {
        let id = self.browser_id.to_lowercase();
        if id.contains("firefox") || id.contains("librewolf") {
            BrowserKind::Firefox
        } else if id == "__viewer__" {
            BrowserKind::Viewer
        } else {
            BrowserKind::Chromium
        }
    }
}

/// Collection of installed browsers
#[derive(Debug, Clone, Default)]
pub struct BrowserCollection {
    pub browsers: Vec<Browser>,
    pub default_id: Option<String>,
}

impl BrowserCollection {
    pub fn load_from_json(json_data: &[serde_json::Value]) -> Self {
        let browsers: Vec<Browser> = json_data
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        let default_id = browsers
            .iter()
            .find(|b| b.is_default)
            .map(|b| b.browser_id.clone());
        Self {
            browsers,
            default_id,
        }
    }

    pub fn set_default(&mut self, browser_id: &str) {
        self.default_id = Some(browser_id.to_string());
        for b in &mut self.browsers {
            b.is_default = b.browser_id == browser_id;
        }
    }

    pub fn default_browser(&self) -> Option<&Browser> {
        self.browsers
            .iter()
            .find(|b| b.is_default)
            .or_else(|| self.browsers.first())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Browser> {
        self.browsers.iter().find(|b| b.browser_id == id)
    }
}

// -- display name mapping --

fn display_name_for(id: &str) -> &str {
    match id {
        "google-chrome-stable" => "Google Chrome",
        "google-chrome-beta" => "Google Chrome Beta",
        "google-chrome-unstable" => "Google Chrome Dev",
        "chromium" => "Chromium",
        "chromium-dev" => "Chromium Dev",
        "microsoft-edge-stable" => "Microsoft Edge",
        "microsoft-edge-beta" => "Microsoft Edge Beta",
        "microsoft-edge-dev" => "Microsoft Edge Dev",
        "brave-browser" | "brave" => "Brave",
        "brave-browser-beta" => "Brave Beta",
        "brave-browser-nightly" => "Brave Nightly",
        "vivaldi-stable" | "vivaldi" => "Vivaldi",
        "vivaldi-beta" => "Vivaldi Beta",
        "vivaldi-snapshot" => "Vivaldi Snapshot",
        "firefox" => "Firefox",
        "firefox-developer-edition" => "Firefox Developer",
        "firefox-nightly" => "Firefox Nightly",
        "librewolf" => "LibreWolf",
        "ungoogled-chromium" => "Ungoogled Chromium",
        "__viewer__" => "Built-in Viewer",
        other => other,
    }
}

fn icon_name_for(id: &str) -> String {
    // strip flatpak prefix if present
    let base = id
        .strip_prefix("com.google.")
        .or_else(|| id.strip_prefix("org.chromium."))
        .or_else(|| id.strip_prefix("org.mozilla."))
        .unwrap_or(id);

    // handle flatpak-style IDs → map to icon filename
    let icon = match base {
        "Chrome" | "google-chrome-stable" => "google-chrome-stable",
        "Chromium" | "chromium" => "chromium",
        _ => base,
    };
    icon.to_string()
}
