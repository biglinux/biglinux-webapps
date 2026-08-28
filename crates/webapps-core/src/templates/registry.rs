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

    /// Find the template whose domain best describes `url`.
    ///
    /// Matching is on the parsed **host**, not on the URL string, and the most
    /// specific domain wins. The previous implementation did
    /// `url.contains(template_domain)` over `HashMap::values()`, which was wrong
    /// twice over:
    ///
    ///  * **Substring false positives.** `"https://www.netflix.com"` contains
    ///    `"x.com"` (inside "netfli**x.com**"), so the DRM-free Twitter/X
    ///    template matched Netflix.
    ///  * **Non-deterministic order.** `music.youtube.com` matches both the
    ///    `youtube-music` domain and the plain `youtube.com` one, and
    ///    `HashMap` iteration order varies per process, so `find` returned
    ///    either one at random.
    ///
    /// Together those made [`Self::requires_drm`] flaky: the internal-browser
    /// block for DRM sites fired on some launches and not others, letting the
    /// user create a WebKit-backed Netflix webapp that plays no video. Ranking by
    /// domain length makes `music.youtube.com` beat `youtube.com`, and the
    /// `template_id` tiebreak keeps the outcome stable across processes for the
    /// four templates that legitimately share `office.com`.
    pub fn match_url(&self, url: &str) -> Option<&WebAppTemplate> {
        let host = host_of(url)?;
        self.templates
            .values()
            .filter(|tpl| {
                tpl.domain()
                    .is_some_and(|domain| host_matches_domain(&host, &domain))
            })
            .max_by(|left, right| {
                let specificity = |tpl: &WebAppTemplate| tpl.domain().map_or(0, |d| d.len());
                specificity(left)
                    .cmp(&specificity(right))
                    // Deterministic tiebreak: `HashMap` order is not stable
                    // across processes, and callers compare results between runs.
                    .then_with(|| right.template_id.cmp(&left.template_id))
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

    /// Check if a webapp needs DRM — match by template_id or URL domain.
    ///
    /// When the host matches no template exactly, fall back to comparing
    /// registrable domains against the DRM-requiring templates only. Several DRM
    /// sites are registered under an app subdomain (`open.spotify.com`,
    /// `listen.tidal.com`), so a user who types the bare `spotify.com` would
    /// otherwise get the internal browser and silent playback. The asymmetry is
    /// deliberate: a false positive here costs a webapp that opens in the
    /// external browser, a false negative costs one that cannot play anything.
    pub fn requires_drm(&self, template_id: &str, url: &str) -> bool {
        if let Some(tpl) = self.templates.get(template_id) {
            return tpl.requires_drm;
        }
        if let Some(tpl) = self.match_url(url) {
            return tpl.requires_drm;
        }
        let Some(host) = host_of(url) else {
            return false;
        };
        let Some(base) = registrable_domain(&host) else {
            return false;
        };
        self.templates.values().any(|tpl| {
            tpl.requires_drm
                && tpl
                    .domain()
                    .as_deref()
                    .and_then(registrable_domain)
                    .is_some_and(|tpl_base| tpl_base == base)
        })
    }
}

/// Lowercased host of `url`, tolerating input with no scheme (the manager stores
/// user-typed URLs before normalisation).
fn host_of(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let parsed = url::Url::parse(trimmed)
        .or_else(|_| url::Url::parse(&format!("https://{trimmed}")))
        .ok()?;
    let host = parsed.host_str()?.to_lowercase();
    Some(host.strip_prefix("www.").unwrap_or(&host).to_string())
}

/// Whether `host` is `domain` itself or a subdomain of it.
///
/// The leading dot is what makes this safe: without it, `"notnetflix.com"`
/// would match `"netflix.com"` by plain suffix.
fn host_matches_domain(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Last two labels of a host — an approximation of the registrable domain.
///
/// Not a Public Suffix List lookup, so a multi-label suffix like `co.uk` would
/// collapse to `co.uk`. That is acceptable because this is only consulted to
/// compare a host against the bundled templates, all of which sit on
/// single-label TLDs; a wrong answer there can only fail to match.
fn registrable_domain(host: &str) -> Option<String> {
    let labels: Vec<&str> = host.split('.').filter(|part| !part.is_empty()).collect();
    match labels[..] {
        [.., second, top] => Some(format!("{second}.{top}")),
        _ => None,
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

    fn drm_template(id: &str, url: &str) -> WebAppTemplate {
        WebAppTemplate {
            template_id: id.into(),
            name: id.into(),
            url: url.into(),
            requires_drm: true,
            ..Default::default()
        }
    }

    #[test]
    fn host_matches_domain_requires_a_label_boundary() {
        assert!(host_matches_domain("netflix.com", "netflix.com"));
        assert!(host_matches_domain("www2.netflix.com", "netflix.com"));
        // The dot is what stops a look-alike domain from matching.
        assert!(!host_matches_domain("notnetflix.com", "netflix.com"));
        assert!(!host_matches_domain("netflix.com.evil.test", "netflix.com"));
    }

    #[test]
    fn registrable_domain_takes_last_two_labels() {
        assert_eq!(
            registrable_domain("open.spotify.com").unwrap(),
            "spotify.com"
        );
        assert_eq!(registrable_domain("spotify.com").unwrap(), "spotify.com");
        assert!(registrable_domain("localhost").is_none());
    }

    #[test]
    fn host_of_tolerates_missing_scheme_and_www() {
        assert_eq!(host_of("https://www.example.com/x").unwrap(), "example.com");
        assert_eq!(host_of("example.com/x").unwrap(), "example.com");
        assert_eq!(
            host_of("  HTTPS://WWW.Example.COM  ").unwrap(),
            "example.com"
        );
        assert!(host_of("").is_none());
    }

    #[test]
    fn match_url_does_not_substring_match_across_domains() {
        // Regression pin: `"https://www.netflix.com".contains("x.com")` is true,
        // so the DRM-free X template used to match Netflix.
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template("twitter", "X", "https://x.com", "Social"));
        reg.register(drm_template("netflix", "https://www.netflix.com"));

        let matched = reg.match_url("https://www.netflix.com").expect("match");
        assert_eq!(matched.template_id, "netflix");
        assert!(reg.requires_drm("", "https://www.netflix.com"));

        // And X itself still matches its own domain.
        assert_eq!(
            reg.match_url("https://x.com/home").unwrap().template_id,
            "twitter"
        );
        assert!(!reg.requires_drm("", "https://x.com/home"));
    }

    #[test]
    fn match_url_prefers_the_most_specific_domain() {
        // `music.youtube.com` matches both templates; the subdomain one must win
        // so its `requires_drm` is the one that counts.
        let mut reg = TemplateRegistry::default();
        reg.register(sample_template(
            "youtube",
            "YouTube",
            "https://www.youtube.com",
            "Media",
        ));
        reg.register(drm_template("youtube-music", "https://music.youtube.com"));

        assert_eq!(
            reg.match_url("https://music.youtube.com/playlist")
                .unwrap()
                .template_id,
            "youtube-music"
        );
        assert!(reg.requires_drm("", "https://music.youtube.com/playlist"));

        // Plain YouTube keeps the less specific, non-DRM template.
        assert_eq!(
            reg.match_url("https://www.youtube.com/watch?v=1")
                .unwrap()
                .template_id,
            "youtube"
        );
        assert!(!reg.requires_drm("", "https://www.youtube.com/watch?v=1"));
    }

    #[test]
    fn match_url_is_deterministic_for_templates_sharing_a_domain() {
        // Four bundled Office templates share `office.com`. `HashMap` order is
        // randomised per process, so the same query must not return a different
        // template on the next launch.
        let reg = build_default_registry();
        let first = reg
            .match_url("https://www.office.com/launch/word")
            .map(|tpl| tpl.template_id.clone());
        for _ in 0..25 {
            let again = build_default_registry()
                .match_url("https://www.office.com/launch/word")
                .map(|tpl| tpl.template_id.clone());
            assert_eq!(first, again, "match_url must be stable across registries");
        }
    }

    #[test]
    fn requires_drm_falls_back_to_the_registrable_domain() {
        // `open.spotify.com` is the template; a user typing the bare
        // `spotify.com` must still be treated as needing DRM, or the webapp
        // opens in WebKit and plays nothing.
        let mut reg = TemplateRegistry::default();
        reg.register(drm_template("spotify", "https://open.spotify.com"));

        assert!(reg.match_url("https://spotify.com").is_none());
        assert!(reg.requires_drm("", "https://spotify.com"));
        assert!(reg.requires_drm("", "spotify.com/browse"));
        // An unrelated host must not inherit it.
        assert!(!reg.requires_drm("", "https://example.com"));
    }

    #[test]
    fn requires_drm_prefers_an_explicit_template_id() {
        let mut reg = TemplateRegistry::default();
        reg.register(drm_template("netflix", "https://www.netflix.com"));
        reg.register(sample_template("blog", "Blog", "https://blog.test", "X"));

        assert!(reg.requires_drm("netflix", "https://unrelated.test"));
        assert!(!reg.requires_drm("blog", "https://www.netflix.com"));
    }

    #[test]
    fn requires_drm_on_the_bundled_registry() {
        let reg = default_registry();
        for url in [
            "https://www.netflix.com/browse",
            "https://www.primevideo.com",
            "https://www.disneyplus.com",
            "https://open.spotify.com",
            "https://music.youtube.com",
            "https://listen.tidal.com",
            "https://www.deezer.com",
            // Bare registrable domains of the subdomain-registered DRM sites.
            "https://spotify.com",
            "https://tidal.com",
        ] {
            assert!(reg.requires_drm("", url), "{url} must require DRM");
        }
        for url in [
            "https://github.com",
            "https://mail.google.com",
            "https://x.com",
            "https://www.youtube.com",
            "https://example.com",
        ] {
            assert!(!reg.requires_drm("", url), "{url} must not require DRM");
        }
    }
}
