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
    pub fn matches_id(&self, id: &str) -> bool {
        self.browser_id == id
            || (id.starts_with("flatpak-")
                && crate::browsers::find_def(id).is_some_and(|definition| {
                    definition.flatpak_id.as_deref() == Some(self.browser_id.as_str())
                        || definition.legacy_flatpak_ids.contains(&self.browser_id)
                }))
    }

    pub fn display_name(&self) -> &str {
        display_name_for(&self.browser_id)
    }

    pub fn icon_name(&self) -> String {
        icon_name_for(&self.browser_id)
    }

    pub fn kind(&self) -> BrowserKind {
        let id = &self.browser_id;
        if id == crate::models::BrowserId::VIEWER {
            return BrowserKind::Viewer;
        }
        // TOML-driven: firefox_like covers Gecko browsers added in browsers.toml.
        // String-pattern fallback handles legacy IDs not present in the file.
        let is_gecko = crate::browsers::find_def(id).map_or_else(
            || id.contains("firefox") || id.contains("librewolf"),
            |d| d.firefox_like,
        );
        if is_gecko {
            BrowserKind::Firefox
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
            b.is_default = b.matches_id(browser_id);
        }
    }

    pub fn default_browser(&self) -> Option<&Browser> {
        self.browsers
            .iter()
            .find(|b| b.is_default)
            .or_else(|| self.browsers.first())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Browser> {
        self.browsers.iter().find(|b| b.matches_id(id))
    }
}

// ---------------------------------------------------------------------------
// Display name + icon resolution — driven by browsers.toml, with fallbacks
// ---------------------------------------------------------------------------

fn display_name_for(id: &str) -> &str {
    // OnceLock data is 'static, so &def.display_name coerces to &'_ str safely
    if let Some(def) = crate::browsers::find_def(id) {
        return &def.display_name;
    }
    match id {
        crate::models::BrowserId::VIEWER => "Built-in Viewer",
        other => other,
    }
}

fn icon_name_for(id: &str) -> String {
    // For known browsers the icon name is the native browser ID
    if let Some(def) = crate::browsers::find_def(id) {
        return def.id.clone();
    }
    // Fallback: strip flatpak- prefix and return remainder
    id.strip_prefix("flatpak-").unwrap_or(id).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_flatpak_aliases_resolve_without_rewriting_ids() {
        for (saved, detected) in [
            ("flatpak-brave", "flatpak-brave-browser"),
            ("flatpak-chrome", "flatpak-google-chrome-stable"),
            ("flatpak-edge", "flatpak-microsoft-edge-stable"),
        ] {
            let mut browsers = BrowserCollection {
                browsers: vec![Browser {
                    browser_id: detected.into(),
                    is_default: false,
                }],
                default_id: None,
            };
            assert_eq!(browsers.get_by_id(saved).unwrap().browser_id, detected);
            browsers.set_default(saved);
            assert!(browsers.default_browser().unwrap().is_default);
            assert_eq!(browsers.default_id.as_deref(), Some(saved));
        }
    }

    #[test]
    fn flatpak_alias_never_selects_a_native_browser() {
        let native = Browser {
            browser_id: "brave".into(),
            is_default: false,
        };
        let flatpak = Browser {
            browser_id: "flatpak-brave-browser".into(),
            is_default: false,
        };
        assert!(!native.matches_id("flatpak-brave"));
        assert!(!flatpak.matches_id("brave"));
        assert!(!flatpak.matches_id("flatpak-firefox"));
    }
}
