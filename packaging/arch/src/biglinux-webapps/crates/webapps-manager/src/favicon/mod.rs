//! Favicon detection for WebApps: fetch a target page, parse the HTML for
//! icon links, and download the chosen asset to a local path.

mod download;
mod html;

use anyhow::Result;
use std::path::PathBuf;

use webapps_core::config;

pub struct SiteInfo {
    pub title: String,
    pub icon_paths: Vec<PathBuf>,
}

pub fn fetch_site_info(url: &str) -> Result<SiteInfo> {
    let normalized_url = normalize_http_url(url)?;
    let parsed = url::Url::parse(&normalized_url)?;

    let client = build_http_client()?;
    let response = client.get(&normalized_url).send()?;
    let html_text = response.text()?;
    let document = scraper::Html::parse_document(&html_text);

    let title = derive_title(&document, &parsed);
    let mut icon_candidates = html::extract_icon_candidates(&document, &normalized_url);
    icon_candidates.extend(fetch_manifest_icons(&client, &document, &normalized_url));
    html::sort_icon_candidates(&mut icon_candidates);
    let cache_dir = ensure_favicon_cache()?;
    let icon_paths = download_icon_set(&client, &parsed, &cache_dir, &icon_candidates);

    Ok(SiteInfo { title, icon_paths })
}

fn normalize_http_url(raw_url: &str) -> Result<String> {
    let normalized = if !raw_url.contains("://") {
        format!("https://{raw_url}")
    } else {
        raw_url.to_string()
    };

    let parsed = url::Url::parse(&normalized)?;
    match parsed.scheme() {
        "http" | "https" => Ok(normalized),
        other => anyhow::bail!("Blocked scheme: {other}"),
    }
}

fn build_http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|error| {
            log::error!("TLS client init: {error:?}");
            error.into()
        })
}

fn derive_title(document: &scraper::Html, parsed_url: &url::Url) -> String {
    let title = html::extract_title(document).unwrap_or_default();
    if is_generic_title(&title) {
        fallback_title_from_host(parsed_url)
    } else {
        title
    }
}

fn is_generic_title(title: &str) -> bool {
    ["ok", "loading", "redirect", "please wait", ""]
        .iter()
        .any(|generic| title.trim().to_lowercase() == *generic)
        || title.len() < 3
}

fn fallback_title_from_host(parsed_url: &url::Url) -> String {
    let Some(host) = parsed_url.host_str() else {
        return String::new();
    };

    let clean = host.strip_prefix("www.").unwrap_or(host);
    let mut chars = clean.chars();
    match chars.next() {
        Some(ch) => ch.to_uppercase().to_string() + chars.as_str(),
        None => clean.to_string(),
    }
}

fn ensure_favicon_cache() -> Result<PathBuf> {
    let cache_dir = config::cache_dir().join("favicons");
    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

fn download_icon_set(
    client: &reqwest::blocking::Client,
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icon_candidates: &[html::IconCandidate],
) -> Vec<PathBuf> {
    let mut icon_paths = Vec::new();

    for (index, candidate) in icon_candidates.iter().enumerate() {
        match download::download_icon(client, &candidate.url, cache_dir, index) {
            Ok(path) => icon_paths.push(path),
            Err(error) => log::warn!("Download icon {}: {error}", candidate.url),
        }
    }

    if icon_paths.is_empty() {
        download_fallback_favicon(client, parsed_url, cache_dir, &mut icon_paths);
    }
    if icon_paths.is_empty() {
        download_google_favicon(client, parsed_url, cache_dir, &mut icon_paths);
    }

    icon_paths
}

#[derive(serde::Deserialize)]
struct WebManifest {
    icons: Option<Vec<ManifestIcon>>,
}

#[derive(serde::Deserialize)]
struct ManifestIcon {
    src: String,
    sizes: Option<String>,
    #[serde(rename = "type")]
    mime_type: Option<String>,
}

fn fetch_manifest_icons(
    client: &reqwest::blocking::Client,
    document: &scraper::Html,
    base_url: &str,
) -> Vec<html::IconCandidate> {
    let mut candidates = Vec::new();
    for manifest_url in html::extract_manifest_urls(document, base_url) {
        match fetch_manifest(client, &manifest_url) {
            Ok(manifest) => {
                let base = url::Url::parse(&manifest_url).ok();
                for icon in manifest.icons.unwrap_or_default() {
                    if let Some(abs) = html::resolve_url(&icon.src, &base) {
                        let sizes = icon.sizes.unwrap_or_default();
                        let mime_type = icon.mime_type.unwrap_or_default();
                        candidates.push(html::IconCandidate::new(
                            abs.clone(),
                            html::largest_declared_size(&sizes),
                            html::IconSource::Manifest,
                            sizes.eq_ignore_ascii_case("any")
                                || mime_type.eq_ignore_ascii_case("image/svg+xml")
                                || abs
                                    .split('?')
                                    .next()
                                    .unwrap_or(&abs)
                                    .to_lowercase()
                                    .ends_with(".svg"),
                        ));
                    }
                }
            }
            Err(err) => log::warn!("Fetch web manifest {manifest_url}: {err}"),
        }
    }
    candidates
}

fn fetch_manifest(client: &reqwest::blocking::Client, manifest_url: &str) -> Result<WebManifest> {
    let response = client
        .get(manifest_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()?;
    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }
    if let Some(content_length) = response.content_length() {
        if content_length > 512 * 1024 {
            anyhow::bail!("Manifest too large: {content_length} bytes");
        }
    }
    let text = response.text()?;
    if text.len() > 512 * 1024 {
        anyhow::bail!("Manifest too large: {} bytes", text.len());
    }
    Ok(serde_json::from_str(&text)?)
}

fn download_fallback_favicon(
    client: &reqwest::blocking::Client,
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icon_paths: &mut Vec<PathBuf>,
) {
    let favicon_url = format!(
        "{}://{}/favicon.ico",
        parsed_url.scheme(),
        parsed_url.host_str().unwrap_or("")
    );
    if let Ok(path) = download::download_icon(client, &favicon_url, cache_dir, 99) {
        icon_paths.push(path);
    }
}

fn download_google_favicon(
    client: &reqwest::blocking::Client,
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icon_paths: &mut Vec<PathBuf>,
) {
    let Some(host) = parsed_url.host_str() else {
        return;
    };

    let google_url = format!("https://www.google.com/s2/favicons?domain={host}&sz=256");
    if let Ok(path) = download::download_icon(client, &google_url, cache_dir, 100) {
        icon_paths.push(path);
    }
}
