use anyhow::Result;
use scraper::{Html, Selector};
use std::path::PathBuf;
use webapps_core::config;

/// Website metadata from HTML fetch
pub struct SiteInfo {
    pub title: String,
    pub icon_paths: Vec<PathBuf>,
}

/// Fetch title + icons from URL (blocking — call from thread)
pub fn fetch_site_info(url: &str) -> Result<SiteInfo> {
    // normalize: prepend https:// if no scheme
    let url = if !url.contains("://") {
        format!("https://{url}")
    } else {
        url.to_string()
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| { log::error!("TLS client init: {e:?}"); e })?;

    let resp = client.get(&url).send()?;
    let html_text = resp.text()?;
    let doc = Html::parse_document(&html_text);

    let title = extract_title(&doc).unwrap_or_default();
    let icon_urls = extract_icon_urls(&doc, &url);

    // download icons to cache
    let cache = config::cache_dir().join("favicons");
    std::fs::create_dir_all(&cache)?;

    let mut icon_paths = Vec::new();
    for (i, icon_url) in icon_urls.iter().enumerate() {
        match download_icon(&client, icon_url, &cache, i) {
            Ok(path) => icon_paths.push(path),
            Err(e) => log::warn!("Download icon {icon_url}: {e}"),
        }
    }

    // try /favicon.ico fallback
    if icon_paths.is_empty() {
        if let Ok(base) = url::Url::parse(&url) {
            let favicon_url = format!("{}://{}/favicon.ico", base.scheme(), base.host_str().unwrap_or(""));
            if let Ok(path) = download_icon(&client, &favicon_url, &cache, 99) {
                icon_paths.push(path);
            }
        }
    }

    Ok(SiteInfo { title, icon_paths })
}

fn extract_title(doc: &Html) -> Option<String> {
    let sel = Selector::parse("title").ok()?;
    doc.select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty())
}

fn extract_icon_urls(doc: &Html, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let base = url::Url::parse(base_url).ok();

    // <link rel="icon|shortcut icon|apple-touch-icon" href="...">
    if let Ok(sel) = Selector::parse("link[rel]") {
        for el in doc.select(&sel) {
            let rel = el.value().attr("rel").unwrap_or("").to_lowercase();
            if rel.contains("icon") {
                if let Some(href) = el.value().attr("href") {
                    if let Some(abs) = resolve_url(href, &base) {
                        urls.push(abs);
                    }
                }
            }
        }
    }

    // <meta property="og:image" content="...">
    if let Ok(sel) = Selector::parse("meta[property='og:image']") {
        for el in doc.select(&sel) {
            if let Some(content) = el.value().attr("content") {
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
    base.as_ref()?.join(href).ok().map(|u| u.to_string())
}

/// Max download size per icon: 5 MB
const MAX_ICON_BYTES: usize = 5 * 1024 * 1024;

fn download_icon(
    client: &reqwest::blocking::Client,
    url: &str,
    cache_dir: &std::path::Path,
    index: usize,
) -> Result<PathBuf> {
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(5))
        .send()?;

    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }

    // check content-length before downloading
    if let Some(cl) = resp.content_length() {
        if cl as usize > MAX_ICON_BYTES {
            anyhow::bail!("Icon too large: {cl} bytes");
        }
    }

    let bytes = resp.bytes()?;
    if bytes.is_empty() {
        anyhow::bail!("Empty response");
    }
    if bytes.len() > MAX_ICON_BYTES {
        anyhow::bail!("Icon too large: {} bytes", bytes.len());
    }

    // determine extension from content or URL
    let ext = guess_extension(url, &bytes);
    let filename = format!("icon_{index}.{ext}");
    let path = cache_dir.join(&filename);

    // convert ICO → PNG if needed
    if ext == "ico" {
        if let Ok(img) = image::load_from_memory(&bytes) {
            let png_path = cache_dir.join(format!("icon_{index}.png"));
            if img.save(&png_path).is_ok() {
                return Ok(png_path);
            }
        }
    }

    std::fs::write(&path, &bytes)?;
    Ok(path)
}

fn guess_extension(url: &str, bytes: &[u8]) -> &'static str {
    // check magic bytes
    if bytes.starts_with(b"\x89PNG") {
        return "png";
    }
    if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        return "svg";
    }
    if bytes.starts_with(&[0, 0, 1, 0]) || bytes.starts_with(&[0, 0, 2, 0]) {
        return "ico";
    }
    // fallback: URL extension
    if url.contains(".svg") {
        return "svg";
    }
    if url.contains(".ico") {
        return "ico";
    }
    "png"
}
