use scraper::{Html, Selector};

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
            Self::MaskIcon => 4,
            Self::Icon => 3,
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

    fn quality_key(&self) -> (u8, u32, u32) {
        (
            u8::from(self.vector_hint),
            self.declared_size.unwrap_or(0),
            u32::from(self.source.priority()),
        )
    }
}

pub(super) fn extract_title(doc: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    doc.select(&selector)
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

    if let Ok(selector) = Selector::parse("link[rel]") {
        for element in doc.select(&selector) {
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
    }

    if let Ok(selector) = Selector::parse("meta[property='og:image']") {
        for element in doc.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                if let Some(abs) = resolve_url(content, &base) {
                    push_best_candidate(
                        &mut urls,
                        IconCandidate::new(abs, None, IconSource::OgImage, false),
                    );
                }
            }
        }
    }

    sort_icon_candidates(&mut urls);
    urls
}

pub(super) fn extract_manifest_urls(doc: &Html, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let base = url::Url::parse(base_url).ok();

    if let Ok(selector) = Selector::parse("link[rel]") {
        for element in doc.select(&selector) {
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
