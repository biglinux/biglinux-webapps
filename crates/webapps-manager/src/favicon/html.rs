use std::sync::LazyLock;

use scraper::{Html, Selector};

static TITLE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("title").expect("static selector"));
static LINK_REL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("link[rel]").expect("static selector"));
static META_OG_IMAGE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='og:image']").expect("static selector"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IconSource {
    Manifest,
    AppleTouch,
    MaskIcon,
    Icon,
    OgImage,
}

impl IconSource {
    fn priority(self) -> u8 {
        match self {
            Self::Manifest => 6,
            Self::AppleTouch => 5,
            Self::Icon => 3,
            // `mask-icon` is Safari's pinned-tab asset: a single-colour
            // silhouette with no brand colour, meant to be recoloured by the
            // browser chrome. It used to sit above `Icon` *and* score the vector
            // bonus, so a monochrome outline beat the site's real full-colour
            // icon. It stays in the list only as a last resort before og:image.
            Self::MaskIcon => 1,
            Self::OgImage => 0,
        }
    }

    /// Whether being an SVG should promote this candidate. Resolution
    /// independence is worth a lot — except for `mask-icon`, where the vector is
    /// a flat silhouette and winning on that bonus is precisely the wrong
    /// outcome.
    fn honours_vector_bonus(self) -> bool {
        self != Self::MaskIcon
    }

    /// Coarse band applied *after* download, ahead of measured resolution.
    ///
    /// Everything a site publishes as an app icon shares the top tier, so
    /// resolution decides between them. The two sources that are not really app
    /// icons get their own tiers below, which no pixel count can overcome: a
    /// monochrome silhouette or a share banner is the wrong picture, not merely
    /// a smaller one.
    pub(super) fn rank_tier(self) -> u8 {
        match self {
            Self::Manifest | Self::AppleTouch | Self::Icon => 2,
            Self::MaskIcon => 1,
            Self::OgImage => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IconCandidate {
    pub url: String,
    pub declared_size: Option<u32>,
    pub source: IconSource,
    pub vector_hint: bool,
}

impl IconCandidate {
    pub fn new(
        url: String,
        declared_size: Option<u32>,
        source: IconSource,
        vector_hint: bool,
    ) -> Self {
        Self {
            url,
            declared_size,
            source,
            vector_hint,
        }
    }

    /// Declared quality, used only to order *fetch* attempts — the final
    /// ranking is redone in `mod.rs` on the measured pixel dimensions.
    fn quality_key(&self) -> (u8, u32, u32) {
        (
            u8::from(self.vector_hint && self.source.honours_vector_bonus()),
            self.declared_size.unwrap_or(0),
            u32::from(self.source.priority()),
        )
    }
}

pub(super) fn extract_title(doc: &Html) -> Option<String> {
    doc.select(&TITLE)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|title| !title.is_empty())
}

#[cfg(test)]
fn extract_icon_urls(doc: &Html, base_url: &str) -> Vec<String> {
    extract_icon_candidates(doc, base_url)
        .into_iter()
        .map(|candidate| candidate.url)
        .collect()
}

pub(super) fn extract_icon_candidates(doc: &Html, base_url: &str) -> Vec<IconCandidate> {
    let mut urls = Vec::new();
    let base = url::Url::parse(base_url).ok();

    for element in doc.select(&LINK_REL) {
        let rel = element.value().attr("rel").unwrap_or("").to_lowercase();
        let Some(source) = icon_source_from_rel(&rel) else {
            continue;
        };
        if let Some(href) = element.value().attr("href") {
            if let Some(abs) = resolve_url(href, &base) {
                let sizes = element.value().attr("sizes").unwrap_or("");
                let mime_type = element.value().attr("type").unwrap_or("");
                push_best_candidate(
                    &mut urls,
                    IconCandidate::new(
                        abs,
                        largest_declared_size(sizes),
                        source,
                        is_vector_hint(href, sizes, mime_type),
                    ),
                );
            }
        }
    }

    for element in doc.select(&META_OG_IMAGE) {
        if let Some(content) = element.value().attr("content") {
            if let Some(abs) = resolve_url(content, &base) {
                push_best_candidate(
                    &mut urls,
                    IconCandidate::new(abs, None, IconSource::OgImage, false),
                );
            }
        }
    }

    sort_icon_candidates(&mut urls);
    urls
}

pub(super) fn extract_manifest_urls(doc: &Html, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let base = url::Url::parse(base_url).ok();

    for element in doc.select(&LINK_REL) {
        let rel = element.value().attr("rel").unwrap_or("").to_lowercase();
        if !rel.split_whitespace().any(|token| token == "manifest") {
            continue;
        }
        if let Some(href) = element.value().attr("href") {
            if let Some(abs) = resolve_url(href, &base) {
                urls.push(abs);
            }
        }
    }

    urls
}

pub(super) fn sort_icon_candidates(candidates: &mut [IconCandidate]) {
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.quality_key()));
}

pub(super) fn largest_declared_size(sizes: &str) -> Option<u32> {
    if sizes.eq_ignore_ascii_case("any") {
        return Some(1024);
    }

    sizes
        .split_whitespace()
        .filter_map(|size| size.split_once('x'))
        .filter_map(|(width, height)| {
            let width = width.parse::<u32>().ok()?;
            let height = height.parse::<u32>().ok()?;
            Some(width.min(height))
        })
        .max()
}

pub(super) fn resolve_url(href: &str, base: &Option<url::Url>) -> Option<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_string());
    }

    base.as_ref()?.join(href).ok().map(|url| url.to_string())
}

fn icon_source_from_rel(rel: &str) -> Option<IconSource> {
    let tokens: Vec<&str> = rel.split_whitespace().collect();
    if tokens.contains(&"apple-touch-icon") || tokens.contains(&"apple-touch-icon-precomposed") {
        return Some(IconSource::AppleTouch);
    }
    if tokens.contains(&"mask-icon") {
        return Some(IconSource::MaskIcon);
    }
    tokens
        .iter()
        .any(|token| token.contains("icon"))
        .then_some(IconSource::Icon)
}

fn is_vector_hint(href: &str, sizes: &str, mime_type: &str) -> bool {
    sizes.eq_ignore_ascii_case("any")
        || mime_type.eq_ignore_ascii_case("image/svg+xml")
        || href
            .split('?')
            .next()
            .unwrap_or(href)
            .to_lowercase()
            .ends_with(".svg")
}

/// Fold `incoming` into `candidates`, keeping one entry per URL.
///
/// The same asset is routinely listed both as a `<link rel="icon">` and in the
/// manifest. Concatenating the two lists meant downloading those URLs twice and
/// showing the identical image twice in the dialog's candidate picker.
pub(super) fn merge_candidates(
    candidates: &mut Vec<IconCandidate>,
    incoming: impl IntoIterator<Item = IconCandidate>,
) {
    for candidate in incoming {
        push_best_candidate(candidates, candidate);
    }
}

fn push_best_candidate(candidates: &mut Vec<IconCandidate>, candidate: IconCandidate) {
    if let Some(existing) = candidates
        .iter_mut()
        .find(|existing| existing.url == candidate.url)
    {
        if candidate.quality_key() > existing.quality_key() {
            *existing = candidate;
        }
        return;
    }
    candidates.push(candidate);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_title_basic() {
        let html = Html::parse_document("<html><head><title>My Site</title></head></html>");
        assert_eq!(extract_title(&html), Some("My Site".into()));
    }

    #[test]
    fn extract_title_empty() {
        let html = Html::parse_document("<html><head><title></title></head></html>");
        assert_eq!(extract_title(&html), None);
    }

    #[test]
    fn extract_title_missing() {
        let html = Html::parse_document("<html><head></head></html>");
        assert_eq!(extract_title(&html), None);
    }

    #[test]
    fn extract_title_whitespace() {
        let html = Html::parse_document("<html><head><title>  Hello World  </title></head></html>");
        assert_eq!(extract_title(&html), Some("Hello World".into()));
    }

    #[test]
    fn icon_source_priority_orders_manifest_high_ogimage_zero() {
        use IconSource::{AppleTouch, Icon, Manifest, MaskIcon, OgImage};
        assert!(Manifest.priority() > AppleTouch.priority());
        assert!(AppleTouch.priority() > Icon.priority());
        // `mask-icon` sits *below* `icon`: it is a monochrome Safari pinned-tab
        // silhouette, so it must never outrank the site's full-colour icon.
        assert!(Icon.priority() > MaskIcon.priority());
        assert!(MaskIcon.priority() > OgImage.priority());
        assert_eq!(OgImage.priority(), 0);
    }

    #[test]
    fn mask_icon_svg_does_not_outrank_full_colour_png() {
        // Regression pin: the vector bonus used to be unconditional, so a
        // monochrome `mask-icon` SVG sorted ahead of a 512 px colour PNG and
        // became the launcher icon.
        let mut candidates = vec![
            IconCandidate::new("mask.svg".into(), None, IconSource::MaskIcon, true),
            IconCandidate::new("icon-512.png".into(), Some(512), IconSource::Icon, false),
        ];
        sort_icon_candidates(&mut candidates);
        assert_eq!(candidates[0].url, "icon-512.png");
    }

    #[test]
    fn merge_candidates_dedupes_by_url_keeping_best() {
        // The same asset commonly appears in both the HTML links and the
        // manifest; merging must not queue it for download twice.
        let mut candidates = vec![IconCandidate::new(
            "same.png".into(),
            Some(32),
            IconSource::Icon,
            false,
        )];
        merge_candidates(
            &mut candidates,
            vec![
                IconCandidate::new("same.png".into(), Some(512), IconSource::Manifest, false),
                IconCandidate::new("other.png".into(), Some(64), IconSource::Manifest, false),
            ],
        );
        assert_eq!(candidates.len(), 2);
        let same = candidates
            .iter()
            .find(|candidate| candidate.url == "same.png")
            .expect("merged entry");
        assert_eq!(same.declared_size, Some(512));
    }

    #[test]
    fn resolve_url_absolute_and_relative() {
        let base = Some(url::Url::parse("https://example.com/page/").unwrap());
        assert_eq!(
            resolve_url("http://cdn.test/i.png", &base).as_deref(),
            Some("http://cdn.test/i.png")
        );
        assert_eq!(
            resolve_url("/icon.png", &base).as_deref(),
            Some("https://example.com/icon.png")
        );
        assert_eq!(
            resolve_url("rel.png", &base).as_deref(),
            Some("https://example.com/page/rel.png")
        );
        // Absolute href with NO base must still pass through (pins the
        // `http || https` scheme guard against `&&`, which would hit `base?`).
        assert_eq!(
            resolve_url("https://abs.test/i.png", &None).as_deref(),
            Some("https://abs.test/i.png")
        );
        assert_eq!(resolve_url("/rel.png", &None), None);
    }

    #[test]
    fn icon_source_from_rel_classifies_each_kind() {
        assert_eq!(
            icon_source_from_rel("apple-touch-icon"),
            Some(IconSource::AppleTouch)
        );
        assert_eq!(
            icon_source_from_rel("apple-touch-icon-precomposed"),
            Some(IconSource::AppleTouch)
        );
        assert_eq!(
            icon_source_from_rel("mask-icon"),
            Some(IconSource::MaskIcon)
        );
        assert_eq!(icon_source_from_rel("icon"), Some(IconSource::Icon));
        assert_eq!(
            icon_source_from_rel("shortcut icon"),
            Some(IconSource::Icon)
        );
        assert_eq!(icon_source_from_rel("stylesheet"), None);
    }

    #[test]
    fn is_vector_hint_detects_svg_any_and_mime() {
        assert!(is_vector_hint("/i.svg", "", ""));
        assert!(is_vector_hint("/i.svg?v=2", "", "")); // query stripped
        assert!(is_vector_hint("/i.png", "any", "")); // sizes any
        assert!(is_vector_hint("/i.png", "", "image/svg+xml")); // mime
        assert!(!is_vector_hint("/i.png", "32x32", "image/png"));
    }

    #[test]
    fn push_best_candidate_keeps_highest_quality_per_url() {
        let mut candidates = Vec::new();
        push_best_candidate(
            &mut candidates,
            IconCandidate::new("u".into(), Some(32), IconSource::Icon, false),
        );
        // Higher declared size for the same url → replaces.
        push_best_candidate(
            &mut candidates,
            IconCandidate::new("u".into(), Some(128), IconSource::Icon, false),
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].declared_size, Some(128));
        // Lower quality for the same url → kept.
        push_best_candidate(
            &mut candidates,
            IconCandidate::new("u".into(), Some(16), IconSource::Icon, false),
        );
        assert_eq!(candidates[0].declared_size, Some(128));
    }

    #[test]
    fn extract_icon_urls_link_rel() {
        let html = Html::parse_document(
            r#"<html><head><link rel="icon" href="/favicon.png"></head></html>"#,
        );
        let urls = extract_icon_urls(&html, "https://example.com");
        assert_eq!(urls, vec!["https://example.com/favicon.png"]);
    }

    #[test]
    fn extract_icon_urls_absolute() {
        let html = Html::parse_document(
            r#"<html><head><link rel="shortcut icon" href="https://cdn.example.com/icon.png"></head></html>"#,
        );
        let urls = extract_icon_urls(&html, "https://example.com");
        assert_eq!(urls, vec!["https://cdn.example.com/icon.png"]);
    }

    #[test]
    fn extract_icon_urls_og_image() {
        let html = Html::parse_document(
            r#"<html><head><meta property="og:image" content="https://example.com/og.png"></head></html>"#,
        );
        let urls = extract_icon_urls(&html, "https://example.com");
        assert_eq!(urls, vec!["https://example.com/og.png"]);
    }

    #[test]
    fn extract_icon_candidates_prioritizes_large_icons() {
        let html = Html::parse_document(
            r#"<html><head>
                <link rel="icon" sizes="16x16" href="/favicon-16.png">
                <link rel="icon" sizes="512x512" href="/icon-512.png">
                <link rel="apple-touch-icon" sizes="180x180" href="/apple.png">
            </head></html>"#,
        );
        let urls = extract_icon_urls(&html, "https://example.com");
        assert_eq!(
            urls,
            vec![
                "https://example.com/icon-512.png",
                "https://example.com/apple.png",
                "https://example.com/favicon-16.png"
            ]
        );
    }

    #[test]
    fn extract_manifest_urls_finds_webmanifest() {
        let html = Html::parse_document(
            r#"<html><head><link rel="manifest" href="/site.webmanifest"></head></html>"#,
        );
        assert_eq!(
            extract_manifest_urls(&html, "https://example.com/app/"),
            vec!["https://example.com/site.webmanifest"]
        );
    }

    #[test]
    fn largest_declared_size_uses_biggest_square_side() {
        assert_eq!(largest_declared_size("16x16 192x192 512x512"), Some(512));
        assert_eq!(largest_declared_size("1024x512"), Some(512));
        assert_eq!(largest_declared_size("any"), Some(1024));
    }

    #[test]
    fn resolve_url_absolute() {
        let base = url::Url::parse("https://example.com").ok();
        assert_eq!(
            resolve_url("https://cdn.example.com/icon.png", &base),
            Some("https://cdn.example.com/icon.png".into())
        );
    }

    #[test]
    fn resolve_url_relative() {
        let base = url::Url::parse("https://example.com/page/").ok();
        assert_eq!(
            resolve_url("../favicon.ico", &base),
            Some("https://example.com/favicon.ico".into())
        );
    }

    #[test]
    fn resolve_url_no_base() {
        assert_eq!(resolve_url("/favicon.ico", &None), None);
    }
}
