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
use webapps_core::desktop;

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

/// Below this side length an icon is treated as too small to hand to a launcher
/// as-is, which triggers both well-known-path probing and a final upscale.
/// Launchers draw at 96–128 px, doubled again on HiDPI.
const MIN_ACCEPTABLE_SIDE: u32 = 256;

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

pub struct SiteInfo {
    pub title: String,
    pub icon_paths: Vec<PathBuf>,
}

pub fn fetch_site_info(url: &str) -> Result<SiteInfo> {
    let normalized_url = normalize_http_url(url)?;
    let parsed = url::Url::parse(&normalized_url)?;

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
    let html_bytes = html_response.bytes;
    let html_text = String::from_utf8_lossy(&html_bytes);
    let document = scraper::Html::parse_document(html_text.as_ref());

    let title = derive_title(&document, &parsed);
    let mut icon_candidates = html::extract_icon_candidates(&document, &normalized_url);
    html::merge_candidates(
        &mut icon_candidates,
        fetch_manifest_icons(&document, &normalized_url),
    );
    html::sort_icon_candidates(&mut icon_candidates);
    let cache_dir = ensure_favicon_cache(&normalized_url)?;
    let icon_paths = download_icon_set(&parsed, &cache_dir, &icon_candidates);

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

/// Per-site favicon cache directory.
///
/// Each detection writes its candidate icons as `icon_<index>.<ext>` with the
/// index restarting at 0 for every site. A single flat directory shared across
/// all sites meant a second site's detection overwrote the first site's
/// `icon_0.png` — and since the dialog keeps that volatile cache path in
/// `app_icon` until the user saves, the icon copied into the stable per-app path
/// at save time could be the *other* site's image. That is a time-of-check /
/// time-of-use race between "detect" and "save": persisting a stable copy didn't
/// help because the source bytes were already clobbered. Namespacing the cache by
/// the site's stable id (the same id used for the persisted icon and `.desktop`
/// name) keeps each site's candidates in their own directory, so concurrent or
/// interleaved detections can never clobber each other.
fn favicon_cache_dir(url: &str) -> PathBuf {
    config::cache_dir()
        .join("favicons")
        .join(desktop::desktop_file_id(url))
}

fn ensure_favicon_cache(url: &str) -> Result<PathBuf> {
    let cache_dir = favicon_cache_dir(url);
    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Fetch the candidate set, then rank it by **measured** resolution.
///
/// The declared ordering from `sort_icon_candidates` only decides fetch order.
/// Final ranking is redone on the bytes, because `sizes=""` in the page and
/// `"sizes"` in the manifest are author hints that are frequently absent or
/// simply wrong — trusting them is what put a 32 px favicon ahead of the
/// site's real 512 px app icon.
fn download_icon_set(
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icon_candidates: &[html::IconCandidate],
) -> Vec<PathBuf> {
    let mut icons = Vec::new();

    for (index, candidate) in icon_candidates
        .iter()
        .take(MAX_CANDIDATE_DOWNLOADS)
        .enumerate()
    {
        match download::download_icon(&candidate.url, cache_dir, index, candidate.source) {
            Ok(icon) => icons.push(icon),
            Err(error) => log::warn!("Download icon {}: {error}", candidate.url),
        }
    }

    // Probing runs whenever the declared set is missing *or* uniformly small:
    // a page that only links a 32 px favicon may still serve a 180 px
    // apple-touch-icon at its conventional path without ever mentioning it.
    if best_side(&icons) < MIN_ACCEPTABLE_SIDE {
        probe_well_known_paths(parsed_url, cache_dir, &mut icons);
    }
    if icons.is_empty() {
        download_fallback_favicon(parsed_url, cache_dir, &mut icons);
    }
    if icons.is_empty() {
        download_favicon_services(parsed_url, cache_dir, &mut icons);
    }

    rank_by_measured_resolution(&mut icons);
    normalize_best(&mut icons, cache_dir);

    icons.into_iter().map(|icon| icon.path).collect()
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

/// Replace the winner with a `TARGET_SIDE` render when it is too small, so the
/// `.desktop` never points at an asset the shell has to upscale itself.
///
/// Only the winner is touched: the rest stay at native resolution for the
/// dialog's candidate picker, where the user is choosing between *images*, not
/// resolutions. Vectors are left alone — they already scale losslessly.
fn normalize_best(icons: &mut Vec<download::DownloadedIcon>, cache_dir: &std::path::Path) {
    let Some(best) = icons.first() else {
        return;
    };
    if best.is_vector || best.effective_side() >= download::TARGET_SIDE {
        return;
    }
    let source = best.source;
    let Some(upscaled) = download::upscale_to_target(&best.path, cache_dir) else {
        return;
    };
    let dimensions = image::measure_file(&upscaled);
    icons.insert(
        0,
        download::DownloadedIcon {
            path: upscaled,
            dimensions,
            is_vector: false,
            source,
        },
    );
}

fn probe_well_known_paths(
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icons: &mut Vec<download::DownloadedIcon>,
) {
    for (offset, path) in WELL_KNOWN_ICON_PATHS.iter().enumerate() {
        let Ok(probe_url) = parsed_url.join(path) else {
            continue;
        };
        // Index base sits above MAX_CANDIDATE_DOWNLOADS so probe results can
        // never overwrite a declared candidate's `icon_<index>` file.
        let index = MAX_CANDIDATE_DOWNLOADS + offset;
        // A conventional path is an undeclared `apple-touch-icon`, so it earns the
        // same top rank tier as a declared one.
        match download::download_icon(
            probe_url.as_str(),
            cache_dir,
            index,
            html::IconSource::AppleTouch,
        ) {
            Ok(icon) => icons.push(icon),
            Err(error) => log::debug!("Probe {probe_url}: {error}"),
        }
    }
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

fn fetch_manifest_icons(document: &scraper::Html, base_url: &str) -> Vec<html::IconCandidate> {
    let mut candidates = Vec::new();
    for manifest_url in html::extract_manifest_urls(document, base_url) {
        match fetch_manifest(&manifest_url) {
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

fn fetch_manifest(manifest_url: &str) -> Result<WebManifest> {
    let response = http_get_bytes_capped(
        manifest_url,
        &RequestHeaders::browser(),
        MAX_MANIFEST_BYTES as u64,
        Duration::from_secs(5),
    )
    .map_err(|error| anyhow::anyhow!(error))?;
    if !(200..300).contains(&response.status) {
        anyhow::bail!("HTTP {}", response.status);
    }
    Ok(serde_json::from_slice(&response.bytes)?)
}

/// `/favicon.ico` at the site root. Built with `Url::join` rather than string
/// formatting so a non-default port survives — the old
/// `format!("{scheme}://{host}/favicon.ico")` dropped `:8080`, sending the
/// request to the wrong service on hosts that run one.
fn download_fallback_favicon(
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icons: &mut Vec<download::DownloadedIcon>,
) {
    let Ok(favicon_url) = parsed_url.join("/favicon.ico") else {
        return;
    };
    if let Ok(icon) =
        download::download_icon(favicon_url.as_str(), cache_dir, 99, html::IconSource::Icon)
    {
        icons.push(icon);
    }
}

/// Last resort: third-party favicon proxies. Both are tried and the larger
/// result wins — each falls back to a low-res cached copy for some domains, so
/// whichever happens to hold the better one varies by site.
fn download_favicon_services(
    parsed_url: &url::Url,
    cache_dir: &std::path::Path,
    icons: &mut Vec<download::DownloadedIcon>,
) {
    let Some(host) = parsed_url.host_str() else {
        return;
    };

    let services = [
        format!("https://www.google.com/s2/favicons?domain={host}&sz=256"),
        format!("https://icons.duckduckgo.com/ip3/{host}.ico"),
    ];
    for (offset, service_url) in services.iter().enumerate() {
        match download::download_icon(service_url, cache_dir, 100 + offset, html::IconSource::Icon)
        {
            Ok(icon) => icons.push(icon),
            Err(error) => log::debug!("Favicon service {service_url}: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favicon_cache_dir_is_namespaced_per_site() {
        // Two different sites must resolve to different cache directories so a
        // detection for one can never overwrite the other's `icon_<index>` files.
        let reddit = favicon_cache_dir("https://www.reddit.com/");
        let loy = favicon_cache_dir("https://example.com/");

        assert_ne!(reddit, loy);
        assert!(reddit.ends_with("favicons/wwwredditcom"));
        assert!(loy.ends_with("favicons/examplecom"));
    }

    #[test]
    fn favicon_cache_dir_is_stable_for_same_site() {
        assert_eq!(
            favicon_cache_dir("https://example.com/"),
            favicon_cache_dir("https://example.com/")
        );
    }

    fn icon(path: &str, side: Option<u32>, is_vector: bool) -> download::DownloadedIcon {
        sourced_icon(path, side, side, is_vector, html::IconSource::Icon)
    }

    fn sourced_icon(
        path: &str,
        width: Option<u32>,
        height: Option<u32>,
        is_vector: bool,
        source: html::IconSource,
    ) -> download::DownloadedIcon {
        download::DownloadedIcon {
            path: PathBuf::from(path),
            dimensions: width
                .zip(height)
                .map(|(width, height)| image::Dimensions { width, height }),
            is_vector,
            source,
        }
    }

    #[test]
    fn ranking_puts_highest_measured_resolution_first() {
        let mut icons = vec![
            icon("small.png", Some(32), false),
            icon("huge.png", Some(1024), false),
            icon("medium.png", Some(180), false),
        ];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(
            icons
                .iter()
                .map(|icon| icon.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["huge.png", "medium.png", "small.png"]
        );
    }

    #[test]
    fn ranking_sinks_unmeasurable_icons_below_measured_ones() {
        // An asset whose header we could not parse might be anything; a known
        // 16 px PNG is at least a known quantity.
        let mut icons = vec![
            icon("mystery.png", None, false),
            icon("tiny.png", Some(16), false),
        ];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(icons[0].path, PathBuf::from("tiny.png"));
    }

    #[test]
    fn ranking_prefers_vector_when_effective_sides_tie() {
        let mut icons = vec![
            icon("raster.png", Some(image::VECTOR_SIDE), false),
            icon("vector.svg", Some(image::VECTOR_SIDE), true),
        ];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(icons[0].path, PathBuf::from("vector.svg"));
    }

    #[test]
    fn ranking_rejects_og_image_banner_in_favour_of_smaller_square_icon() {
        // Regression pin for Discord/Slack: a 1200x630 share card is the biggest
        // image on the page and the worst possible launcher icon.
        let mut icons = vec![
            sourced_icon(
                "banner.png",
                Some(1200),
                Some(630),
                false,
                html::IconSource::OgImage,
            ),
            sourced_icon(
                "icon-512.png",
                Some(512),
                Some(512),
                false,
                html::IconSource::Manifest,
            ),
        ];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(icons[0].path, PathBuf::from("icon-512.png"));
    }

    #[test]
    fn ranking_rejects_square_og_image_in_favour_of_smaller_real_icon() {
        // Even a perfectly square og:image is a share card, not a logo, so the
        // shape gate alone is not enough — provenance has to outrank pixels.
        let mut icons = vec![
            sourced_icon(
                "square-banner.png",
                Some(1024),
                Some(1024),
                false,
                html::IconSource::OgImage,
            ),
            sourced_icon(
                "icon-64.png",
                Some(64),
                Some(64),
                false,
                html::IconSource::Icon,
            ),
        ];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(icons[0].path, PathBuf::from("icon-64.png"));
    }

    #[test]
    fn ranking_sinks_monochrome_mask_icon_svg_below_a_small_colour_icon() {
        // A mask-icon SVG scores unlimited resolution; its tier must still lose
        // to any real full-colour icon.
        let mut icons = vec![
            sourced_icon(
                "mask.svg",
                Some(image::VECTOR_SIDE),
                Some(image::VECTOR_SIDE),
                true,
                html::IconSource::MaskIcon,
            ),
            sourced_icon(
                "icon-32.png",
                Some(32),
                Some(32),
                false,
                html::IconSource::Icon,
            ),
        ];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(icons[0].path, PathBuf::from("icon-32.png"));
    }

    #[test]
    fn ranking_still_accepts_a_banner_when_it_is_all_there_is() {
        // Demotion must not become exclusion: a share card beats no icon.
        let mut icons = vec![sourced_icon(
            "banner.png",
            Some(1200),
            Some(630),
            false,
            html::IconSource::OgImage,
        )];
        rank_by_measured_resolution(&mut icons);
        assert_eq!(icons[0].path, PathBuf::from("banner.png"));
    }

    #[test]
    fn best_side_ignores_banners_so_probing_still_runs() {
        assert_eq!(best_side(&[]), 0);
        assert_eq!(best_side(&[icon("x.png", None, false)]), 0);
        // A page offering only a 32 px favicon must still be probed for an
        // unlisted apple-touch-icon.
        assert!(best_side(&[icon("x.png", Some(32), false)]) < MIN_ACCEPTABLE_SIDE);
        assert!(best_side(&[icon("x.png", Some(512), false)]) >= MIN_ACCEPTABLE_SIDE);
        // A huge og:image must not count as "we already have a good icon".
        let banner = sourced_icon(
            "banner.png",
            Some(1200),
            Some(630),
            false,
            html::IconSource::OgImage,
        );
        assert_eq!(best_side(&[banner]), 0);
    }

    #[test]
    fn normalize_best_leaves_vectors_and_large_rasters_untouched() {
        let cache = std::path::Path::new("/nonexistent-cache");

        let mut vector = vec![icon("vector.svg", Some(image::VECTOR_SIDE), true)];
        normalize_best(&mut vector, cache);
        assert_eq!(vector.len(), 1, "an SVG already scales losslessly");

        let mut large = vec![icon("big.png", Some(download::TARGET_SIDE), false)];
        normalize_best(&mut large, cache);
        assert_eq!(large.len(), 1, "a 512 px source needs no upscale");

        let mut empty: Vec<download::DownloadedIcon> = Vec::new();
        normalize_best(&mut empty, cache);
        assert!(empty.is_empty(), "no candidates must not panic");
    }

    #[test]
    fn probe_indexes_cannot_collide_with_declared_candidate_indexes() {
        // Declared candidates occupy 0..MAX_CANDIDATE_DOWNLOADS; probes and the
        // two fallbacks must sit above that, or a probe would overwrite a
        // declared candidate's `icon_<index>` file on disk.
        let probe_range =
            MAX_CANDIDATE_DOWNLOADS..MAX_CANDIDATE_DOWNLOADS + WELL_KNOWN_ICON_PATHS.len();
        assert!(probe_range.start >= MAX_CANDIDATE_DOWNLOADS);
        assert!(probe_range.end <= 99, "99..=101 are the fallback indexes");
    }
}
