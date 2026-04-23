use scraper::{Html, Selector};

pub(super) fn extract_title(doc: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    doc.select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|title| !title.is_empty())
}

pub(super) fn extract_icon_urls(doc: &Html, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let base = url::Url::parse(base_url).ok();

    if let Ok(selector) = Selector::parse("link[rel]") {
        for element in doc.select(&selector) {
            let rel = element.value().attr("rel").unwrap_or("").to_lowercase();
            if rel.contains("icon") {
                if let Some(href) = element.value().attr("href") {
                    if let Some(abs) = resolve_url(href, &base) {
                        urls.push(abs);
                    }
                }
            }
        }
    }

    if let Ok(selector) = Selector::parse("meta[property='og:image']") {
        for element in doc.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                if let Some(abs) = resolve_url(content, &base) {
                    urls.push(abs);
                }
            }
        }
    }

    urls
}

fn resolve_url(href: &str, base: &Option<url::Url>) -> Option<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_string());
    }

    base.as_ref()?.join(href).ok().map(|url| url.to_string())
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
