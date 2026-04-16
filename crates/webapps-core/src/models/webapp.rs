use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Launch mode for webapp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    /// Open in system browser
    #[default]
    Browser,
    /// Open in built-in viewer (CSD webview)
    App,
}

impl AppMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::App => "app",
        }
    }
}

/// Single web application entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebApp {
    #[serde(default)]
    pub browser: String,
    #[serde(default)]
    pub app_file: String,
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub app_url: String,
    #[serde(default)]
    pub app_icon: String,
    #[serde(default = "default_profile")]
    pub app_profile: String,
    #[serde(default = "default_categories")]
    pub app_categories: String,
    #[serde(default)]
    pub app_icon_url: String,
    #[serde(default)]
    pub app_mode: AppMode,

    // template metadata
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub mime_types: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub generic_name: String,
    #[serde(default)]
    pub keywords: String,
    #[serde(default)]
    pub url_schemes: String,
}

fn default_profile() -> String {
    "Default".into()
}

fn default_categories() -> String {
    "Webapps".into()
}

impl Default for WebApp {
    fn default() -> Self {
        Self {
            browser: String::new(),
            app_file: String::new(),
            app_name: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_profile: default_profile(),
            app_categories: default_categories(),
            app_icon_url: String::new(),
            app_mode: AppMode::default(),
            template_id: String::new(),
            mime_types: String::new(),
            comment: String::new(),
            generic_name: String::new(),
            keywords: String::new(),
            url_schemes: String::new(),
        }
    }
}

impl WebApp {
    pub fn main_category(&self) -> &str {
        self.app_categories
            .split(';')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("Webapps")
    }

    pub fn set_main_category(&mut self, category: &str) {
        if category.is_empty() {
            return;
        }
        let others: Vec<&str> = self
            .app_categories
            .split(';')
            .skip(1)
            .filter(|c| !c.is_empty() && *c != category)
            .collect();
        if others.is_empty() {
            self.app_categories = category.to_string();
        } else {
            self.app_categories = format!("{};{}", category, others.join(";"));
        }
    }

    /// Derive profile name from URL hostname (dots removed)
    pub fn derive_profile_name(&self) -> String {
        url::Url::parse(&self.app_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.replace('.', "")))
            .unwrap_or_else(|| "Default".into())
    }

    /// Apply template preset to pre-fill fields
    pub fn apply_template(&mut self, tpl: &crate::templates::WebAppTemplate) {
        self.template_id = tpl.template_id.clone();
        self.app_name = tpl.name.clone();
        self.app_url = tpl.url.clone();
        self.app_icon = tpl.icon.clone();
        self.app_icon_url = tpl.icon.clone();
        self.app_categories = tpl.category.clone();

        if !tpl.mime_types.is_empty() {
            self.mime_types = format!("{};", tpl.mime_types.join(";"));
        }
        if !tpl.comment.is_empty() {
            self.comment = tpl.comment.clone();
        }
        if !tpl.generic_name.is_empty() {
            self.generic_name = tpl.generic_name.clone();
        }
        if !tpl.keywords.is_empty() {
            self.keywords = format!("{};", tpl.keywords.join(";"));
        }
        if !tpl.url_schemes.is_empty() {
            self.url_schemes = format!("{};", tpl.url_schemes.join(";"));
        }
        if !tpl.profile.is_empty() {
            self.app_profile = tpl.profile.clone();
        }
    }

    /// Check if text matches name, URL, or file
    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.app_name.to_lowercase().contains(&q)
            || self.app_url.to_lowercase().contains(&q)
            || self.app_file.to_lowercase().contains(&q)
    }
}

/// Collection of WebApp with filtering
#[derive(Debug, Clone, Default)]
pub struct WebAppCollection {
    pub webapps: Vec<WebApp>,
}

impl WebAppCollection {
    pub fn load_from_json(json_data: &[serde_json::Value]) -> Self {
        let webapps = json_data
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        Self { webapps }
    }

    pub fn filter_by_text(&self, query: &str) -> Vec<&WebApp> {
        if query.is_empty() {
            return self.webapps.iter().collect();
        }
        self.webapps
            .iter()
            .filter(|app| app.matches(query))
            .collect()
    }

    pub fn categorized(&self, query: Option<&str>) -> HashMap<String, Vec<&WebApp>> {
        let apps: Vec<&WebApp> = match query {
            Some(q) if !q.is_empty() => self.filter_by_text(q),
            _ => self.webapps.iter().collect(),
        };
        let mut map: HashMap<String, Vec<&WebApp>> = HashMap::new();
        for app in apps {
            for cat in app.app_categories.split(';').filter(|c| !c.is_empty()) {
                map.entry(cat.to_string()).or_default().push(app);
            }
        }
        map
    }

    pub fn add(&mut self, webapp: WebApp) {
        self.webapps.push(webapp);
    }

    pub fn remove_by_file(&mut self, app_file: &str) {
        self.webapps.retain(|app| app.app_file != app_file);
    }
}
