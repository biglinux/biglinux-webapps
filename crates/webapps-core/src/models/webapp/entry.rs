use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::types::{
    AppMode, BrowserId, CategoryList, DesktopFileName, ProfileKind, UrlValidationError, WebAppUrl,
};

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
    #[serde(default)]
    pub auto_hide_headerbar: bool,
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
            auto_hide_headerbar: false,
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
    pub fn browser_id(&self) -> BrowserId {
        BrowserId::from(self.browser.as_str())
    }

    pub fn desktop_file_name(&self) -> Option<DesktopFileName> {
        DesktopFileName::parse(&self.app_file)
    }

    pub fn profile_kind(&self) -> ProfileKind {
        ProfileKind::parse(&self.app_profile)
    }

    pub fn has_custom_profile(&self) -> bool {
        self.profile_kind().is_custom()
    }

    pub fn category_list(&self) -> CategoryList {
        CategoryList::parse(&self.app_categories)
    }

    pub fn main_category(&self) -> &str {
        self.app_categories
            .split(';')
            .next()
            .filter(|segment| !segment.is_empty())
            .unwrap_or("Webapps")
    }

    pub fn set_main_category(&mut self, category: &str) {
        self.app_categories = self.category_list().with_main(category).to_serialized();
    }

    pub fn derive_profile_name(&self) -> String {
        url::Url::parse(&self.app_url)
            .ok()
            .and_then(|url| url.host_str().map(|host| host.replace('.', "")))
            .unwrap_or_else(|| "Default".into())
    }

    pub fn apply_template(&mut self, tpl: &crate::templates::WebAppTemplate) {
        self.template_id = tpl.template_id.clone();
        self.app_name = tpl.name.clone();
        self.app_url = tpl.url.clone();
        self.app_icon = tpl.icon.clone();
        self.app_icon_url = tpl.icon.clone();
        self.app_categories = tpl.category.clone();

        if tpl.requires_drm {
            self.app_mode = AppMode::Browser;
        }

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

    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.app_name.to_lowercase().contains(&query)
            || self.app_url.to_lowercase().contains(&query)
            || self.app_file.to_lowercase().contains(&query)
    }

    pub fn normalized_url(&self) -> Result<WebAppUrl, UrlValidationError> {
        WebAppUrl::parse(&self.app_url)
    }

    pub fn validate_domain(&self) -> Result<()> {
        self.normalized_url()?;
        self.category_list().validate()?;
        if let Some(app_file) = self.desktop_file_name() {
            app_file.validate()?;
        }
        self.browser_id().validate()?;
        self.profile_kind().validate()
    }
}
