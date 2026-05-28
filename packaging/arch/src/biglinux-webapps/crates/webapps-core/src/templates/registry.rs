use std::collections::HashMap;
use std::sync::OnceLock;

/// File-handling strategy for webapp template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileHandler {
    #[default]
    None,
    Upload,
    Url,
}

/// Immutable preset for a known web service
#[derive(Debug, Clone)]
pub struct WebAppTemplate {
    pub template_id: String,
    pub name: String,
    pub url: String,
    pub icon: String,
    pub category: String,
    pub mime_types: Vec<String>,
    pub url_schemes: Vec<String>,
    pub features: Vec<String>,
    pub profile: String,
    pub comment: String,
    pub generic_name: String,
    pub keywords: Vec<String>,
    pub file_handler: FileHandler,
    /// Site needs DRM (Widevine) → force Browser mode
    pub requires_drm: bool,
}

impl Default for WebAppTemplate {
    fn default() -> Self {
        Self {
            template_id: String::new(),
            name: String::new(),
            url: String::new(),
            icon: String::new(),
            category: String::new(),
            mime_types: Vec::new(),
            url_schemes: Vec::new(),
            features: Vec::new(),
            profile: String::new(),
            comment: String::new(),
            generic_name: String::new(),
            keywords: Vec::new(),
            file_handler: FileHandler::None,
            requires_drm: false,
        }
    }
}

impl WebAppTemplate {
    /// Domain extracted from URL for matching
    pub fn domain(&self) -> Option<String> {
        url::Url::parse(&self.url).ok().and_then(|u| {
            u.host_str().map(|h| {
                let h = h.strip_prefix("www.").unwrap_or(h);
                h.to_lowercase()
            })
        })
    }
}

/// Central store for webapp templates with lookup helpers
#[derive(Debug, Clone, Default)]
pub struct TemplateRegistry {
    templates: HashMap<String, WebAppTemplate>,
    by_category: HashMap<String, Vec<String>>,
}

impl TemplateRegistry {
    pub fn register(&mut self, tpl: WebAppTemplate) {
        let id = tpl.template_id.clone();
        let cat = tpl.category.clone();
        self.templates.insert(id.clone(), tpl);
        self.by_category.entry(cat).or_default().push(id);
    }

    pub fn register_many(&mut self, templates: Vec<WebAppTemplate>) {
        for t in templates {
            self.register(t);
        }
    }

    pub fn get(&self, id: &str) -> Option<&WebAppTemplate> {
        self.templates.get(id)
    }

    pub fn get_all(&self) -> Vec<&WebAppTemplate> {
        self.templates.values().collect()
    }

    pub fn get_by_category(&self, category: &str) -> Vec<&WebAppTemplate> {
        self.by_category
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.templates.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.by_category.keys().cloned().collect();
        cats.sort();
        cats
    }

    pub fn match_url(&self, url: &str) -> Option<&WebAppTemplate> {
        let url_lower = url.to_lowercase();
        self.templates.values().find(|tpl| {
            tpl.domain()
                .map(|d| url_lower.contains(&d))
                .unwrap_or(false)
        })
    }

    pub fn search(&self, query: &str) -> Vec<&WebAppTemplate> {
        let q = query.to_lowercase();
        self.templates
            .values()
            .filter(|tpl| {
                tpl.name.to_lowercase().contains(&q)
                    || tpl.category.to_lowercase().contains(&q)
                    || tpl.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Check if a webapp needs DRM — match by template_id or URL domain
    pub fn requires_drm(&self, template_id: &str, url: &str) -> bool {
        if let Some(tpl) = self.templates.get(template_id) {
            return tpl.requires_drm;
        }
        self.match_url(url)
            .map(|tpl| tpl.requires_drm)
            .unwrap_or(false)
    }
}

/// Build registry with all bundled templates.
///
/// Cheap callers (UI handlers fired on every dialog open) should use
/// [`default_registry`] instead — this function rebuilds the registry from
/// scratch each call and the templates are immutable.
pub fn build_default_registry() -> TemplateRegistry {
    let mut reg = TemplateRegistry::default();
    reg.register_many(super::office365::templates());
    reg.register_many(super::google::templates());
    reg.register_many(super::communication::templates());
    reg.register_many(super::media::templates());
    reg.register_many(super::productivity::templates());
    reg
}

/// Process-wide cached registry. Built once on first call.
pub fn default_registry() -> &'static TemplateRegistry {
    static REGISTRY: OnceLock<TemplateRegistry> = OnceLock::new();
    REGISTRY.get_or_init(build_default_registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_template(id: &str, name: &str, url: &str, category: &str) -> WebAppTemplate {
        WebAppTemplate {
            template_id: id.into(),
            name: name.into(),
            url: url.into(),
            category: category.into(),
            keywords: vec![name.to_lowercase()],
            ..Default::default()
        }
    }

    #[test]
    fn register_and_get() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "gmail",
            "Gmail",
            "https://mail.google.com",
            "Communication",
        ));
        assert!(reg.get("gmail").is_some());
        assert_eq!(reg.get("gmail").unwrap().name, "Gmail");
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn categories_sorted() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template("c", "C", "https://c.com", "Zebra"));
        reg.register(sample_template("a", "A", "https://a.com", "Alpha"));
        let cats = reg.categories();
        assert_eq!(cats, vec!["Alpha", "Zebra"]);
    }

    #[test]
    fn get_by_category() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "g",
            "Gmail",
            "https://mail.google.com",
            "Communication",
        ));
        reg.register(sample_template(
            "s",
            "Spotify",
            "https://spotify.com",
            "Media",
        ));
        let comms = reg.get_by_category("Communication");
        assert_eq!(comms.len(), 1);
        assert_eq!(comms[0].name, "Gmail");
        assert!(reg.get_by_category("Nonexistent").is_empty());
    }

    #[test]
    fn match_url_finds_template() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "yt",
            "YouTube",
            "https://www.youtube.com",
            "Media",
        ));
        let found = reg.match_url("https://youtube.com/watch?v=123");
        assert!(found.is_some());
        assert_eq!(found.unwrap().template_id, "yt");
    }

    #[test]
    fn match_url_no_match() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "yt",
            "YouTube",
            "https://www.youtube.com",
            "Media",
        ));
        assert!(reg.match_url("https://example.com").is_none());
    }

    #[test]
    fn search_by_name() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "g",
            "Gmail",
            "https://mail.google.com",
            "Communication",
        ));
        reg.register(sample_template(
            "s",
            "Spotify",
            "https://spotify.com",
            "Media",
        ));
        let results = reg.search("gmail");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Gmail");
    }

    #[test]
    fn search_by_category() {
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "g",
            "Gmail",
            "https://mail.google.com",
            "Communication",
        ));
        let results = reg.search("communication");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_empty_query() {
        let reg = build_default_registry();
        let results = reg.search("");
        // empty query matches everything
        assert!(!results.is_empty());
    }

    #[test]
    fn default_registry_has_templates() {
        let reg = build_default_registry();
        assert!(reg.get_all().len() > 30);
        assert!(!reg.categories().is_empty());
    }

    #[test]
    fn domain_extraction() {
        let tpl = sample_template("t", "Test", "https://www.example.com/path", "X");
        assert_eq!(tpl.domain(), Some("example.com".into()));
    }
}
