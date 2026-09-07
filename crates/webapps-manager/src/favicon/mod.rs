//! Favicon detection for WebApps: fetch a target page, parse the HTML for
//! icon links, and download the chosen asset to a local path.

mod download;
mod html;
mod image;

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

use crate::http_client::{http_get_bytes_capped, RequestHeaders};
use webapps_core::config;

/// Hard cap on a fetched HTML page. Real pages are well under this; the cap
/// stops a hostile server from streaming an unbounded body into memory.
const MAX_PAGE_BYTES: usize = 4 * 1024 * 1024;
/// Hard cap on a fetched web-app manifest (`manifest.json`).
const MAX_MANIFEST_BYTES: usize = 512 * 1024;

/// Ceiling on how many declared candidates we actually fetch.
///
/// Ranking needs the real pixel dimensions, and those only exist once the bytes
/// are in hand — so candidates have to be downloaded to be judged. Sites that
/// list a dozen `apple-touch-icon` sizes would otherwise turn one "Detect" click
/// into a dozen round trips. Candidates arrive pre-sorted by their declared
/// quality, so truncating the tail drops the least promising ones.
const MAX_CANDIDATE_DOWNLOADS: usize = 10;

const PREFERRED_SIDE: u32 = 256;
const MIN_ACCEPTABLE_SIDE: u32 = 64;

/// Conventional icon locations to probe when nothing declared in the page is
/// big enough. These are unlisted on plenty of sites that still serve them, and
/// an `apple-touch-icon` is 180 px at minimum — far better than a 32 px favicon.
const WELL_KNOWN_ICON_PATHS: &[&str] = &[
    "/apple-touch-icon.png",
    "/apple-touch-icon-precomposed.png",
    "/apple-touch-icon-180x180.png",
    "/icon.png",
    "/logo.png",
];

pub struct SiteIcon {
    pub path: PathBuf,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub scalable: bool,
}

pub struct SiteInfo {
    pub title: String,
    pub icons: Vec<SiteIcon>,
    pub cache: Option<tempfile::TempDir>,
}

pub fn fetch_site_info(url: &str) -> Result<SiteInfo> {
    let normalized_url = normalize_http_url(url)?;
    let deadline = std::time::Instant::now() + Duration::from_secs(20);

    let html_response = http_get_bytes_capped(
        &normalized_url,
        &RequestHeaders::browser(),
        MAX_PAGE_BYTES as u64,
        Duration::from_secs(10),
    )
    .map_err(|error| anyhow::anyhow!(error))?;
    if !(200..300).contains(&html_response.status) {
        anyhow::bail!("HTTP {}", html_response.status);
    }
    let parsed = url::Url::parse(&html_response.final_url)?;
    let html_bytes = html_response.bytes;
    let html_text = String::from_utf8_lossy(&html_bytes);
    let document = scraper::Html::parse_document(html_text.as_ref());

    let base_url = document
        .select(&scraper::Selector::parse("base[href]").expect("static selector"))
        .next()
        .and_then(|element| element.value().attr("href"))
        .and_then(|href| parsed.join(href).ok())
        .unwrap_or_else(|| parsed.clone());
    let title = derive_title(&document, &parsed);
    let mut icon_candidates = html::extract_icon_candidates(&document, base_url.as_str());
    html::merge_candidates(
        &mut icon_candidates,
        fetch_manifest_icons(&document, base_url.as_str(), deadline),
    );
    html::sort_icon_candidates(&mut icon_candidates);
    let cache_dir = ensure_favicon_cache(&normalized_url)?;
    let icons = download_icon_set(&parsed, cache_dir.path(), &icon_candidates, deadline);

    Ok(SiteInfo {
        title,
        icons,
        cache: Some(cache_dir),
    })
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

fn ensure_favicon_cache(_url: &str) -> Result<tempfile::TempDir> {
    let parent = config::cache_dir().join("favicons");
    std::fs::create_dir_all(&parent)?;
    Ok(tempfile::Builder::new()
        .prefix("search-")
        .tempdir_in(parent)?)
}

fn download_icon_set(
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    candidates: &[html::IconCandidate],
    deadline: std::time::Instant,
) -> Vec<SiteIcon> {
    let mut icons = Vec::new();
    let candidates: Vec<_> = candidates
        .iter()
        .filter(|icon| icon.source.rank_tier() == 2)
        .take(MAX_CANDIDATE_DOWNLOADS)
        .collect();
    for (batch, candidates) in candidates.chunks(3).enumerate() {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = candidates
                .iter()
                .enumerate()
                .map(|(index, candidate)| {
                    scope.spawn(move || {
                        download::download_icon(
                            &candidate.url,
                            cache_dir,
                            batch * 3 + index,
                            candidate.source,
                            remaining.min(Duration::from_secs(5)),
                        )
                    })
                })
                .collect();
            for handle in handles {
                if let Ok(Ok(icon)) = handle.join() {
                    icons.push(icon);
                }
            }
        });
    }
    if best_side(&icons) < PREFERRED_SIDE {
        for (index, path) in WELL_KNOWN_ICON_PATHS
            .iter()
            .chain(["/favicon.ico"].iter())
            .enumerate()
        {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            let Ok(url) = parsed_url.join(path) else {
                continue;
            };
            if let Ok(icon) = download::download_icon(
                url.as_str(),
                cache_dir,
                20 + index,
                html::IconSource::Icon,
                remaining.min(Duration::from_secs(2)),
            ) {
                icons.push(icon);
            }
        }
    }
    if best_side(&icons) >= PREFERRED_SIDE {
        icons.retain(|icon| icon.effective_side() >= PREFERRED_SIDE);
    }
    rank_by_measured_resolution(&mut icons);
    let mut fingerprints = Vec::new();
    icons
        .into_iter()
        .filter(|icon| {
            let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_file_at_scale(&icon.path, 16, 16, false)
            else {
                return false;
            };
            let fingerprint = pixbuf.read_pixel_bytes();
            if fingerprints.iter().any(|previous: &glib::Bytes| {
                similar_pixels(previous.as_ref(), fingerprint.as_ref())
            }) {
                return false;
            }
            fingerprints.push(fingerprint);
            true
        })
        .map(|icon| SiteIcon {
            width: icon.dimensions.map_or(0, |size| size.width),
            height: icon.dimensions.map_or(0, |size| size.height),
            scalable: icon.is_vector,
            path: icon.path,
            url: icon.url,
        })
        .collect()
}

fn similar_pixels(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && !left.is_empty()
        && left
            .iter()
            .zip(right)
            .map(|(a, b)| u64::from(a.abs_diff(*b)))
            .sum::<u64>()
            <= left.len() as u64 * 6
}

/// Best resolution among candidates that are actually usable as icons.
///
/// Banners are excluded deliberately: a page whose only large image is a
/// 1200×630 `og:image` still needs the well-known-path probe, and counting the
/// banner here would suppress it and leave the 32 px favicon as the real winner.
fn best_side(icons: &[download::DownloadedIcon]) -> u32 {
    icons
        .iter()
        .filter(|icon| icon.is_icon_shaped() && icon.source.rank_tier() > 0)
        .map(download::DownloadedIcon::effective_side)
        .max()
        .unwrap_or(0)
}

/// Final ranking, most-preferred first.
///
/// Resolution alone is not enough, because the two biggest images a page offers
/// are often the two least suitable:
///
///  * `og:image` is a 1200×630 social share card — a wide screenshot, not a
///    logo. Ranked purely on pixel count it beat Discord's and Slack's real
///    512 px app icons.
///  * `mask-icon` is Safari's pinned-tab silhouette: a flat single-colour path
///    which, being an SVG, scores unlimited resolution.
///
/// So shape and provenance gate the comparison, and resolution only decides
/// among candidates that already look like icons.
fn rank_by_measured_resolution(icons: &mut [download::DownloadedIcon]) {
    icons.sort_by_key(|icon| {
        std::cmp::Reverse((
            u8::from(icon.is_icon_shaped()),
            icon.source.rank_tier(),
            icon.effective_side(),
            u8::from(icon.is_vector),
        ))
    });
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
    document: &scraper::Html,
    base_url: &str,
    deadline: std::time::Instant,
) -> Vec<html::IconCandidate> {
    let mut candidates = Vec::new();
    for manifest_url in html::extract_manifest_urls(document, base_url)
        .into_iter()
        .take(2)
    {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match fetch_manifest(&manifest_url, remaining.min(Duration::from_secs(5))) {
            Ok((manifest, final_url)) => {
                let base = url::Url::parse(&final_url).ok();
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

fn fetch_manifest(manifest_url: &str, timeout: Duration) -> Result<(WebManifest, String)> {
    let response = http_get_bytes_capped(
        manifest_url,
        &RequestHeaders::browser(),
        MAX_MANIFEST_BYTES as u64,
        timeout,
    )
    .map_err(|error| anyhow::anyhow!(error))?;
    if !(200..300).contains(&response.status) {
        anyhow::bail!("HTTP {}", response.status);
    }
    Ok((serde_json::from_slice(&response.bytes)?, response.final_url))
}

#[cfg(test)]
mod tests;
