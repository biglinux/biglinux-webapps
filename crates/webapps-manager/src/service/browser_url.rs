//! Resolve the URL Chromium-family browsers will actually navigate to once
//! redirects are followed, so the Wayland app_id we predict from the saved
//! URL matches the one the running browser ends up reporting.
//!
//! Chromium synthesizes its xdg-shell app_id from the URL the window is
//! currently showing, not the one passed to `--app`. Sites like Spotify
//! redirect `https://open.spotify.com/` to `https://open.spotify.com/intl-pt/`
//! based on `Accept-Language`, so the launched window's app_id picks up
//! `_intl-pt_` and stops matching our `.desktop` basename — GNOME Shell then
//! falls back to the host browser's icon. We pre-follow the redirect here
//! using the same locale Brave will use so the stored `app_url` already
//! reflects the post-redirect destination.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use big_os_kit::http_client::{http_get_stream_with_extra_headers, RequestHeaders};

static RESOLVED_URLS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

/// Resolve `url` to its post-redirect destination using the system locale's
/// `Accept-Language`. Returns the original `url` on any error (network down,
/// timeout, server refusing the request) — callers must tolerate that.
pub fn resolve_browser_url(url: &str) -> String {
    if !is_redirectable_url(url) {
        return url.to_string();
    }

    let accept_language = derive_accept_language(&std::env::var("LANG").unwrap_or_default());
    let cache_key = format!("{accept_language}\0{url}");
    if let Some(cached) = cached_resolved_url(&cache_key) {
        return cached;
    }

    let resolved = resolve_browser_url_uncached(url, &accept_language);
    store_resolved_url(cache_key, resolved.clone());
    resolved
}

fn resolve_browser_url_uncached(url: &str, accept_language: &str) -> String {
    let extra_headers = if accept_language.is_empty() {
        Vec::new()
    } else {
        vec![("Accept-Language", accept_language)]
    };

    match http_get_stream_with_extra_headers(
        url,
        &RequestHeaders::browser(),
        &extra_headers,
        Duration::from_secs(5),
    ) {
        Ok(response) => {
            if response.final_url != url {
                log::info!("Resolved {url} → {}", response.final_url);
            }
            response.final_url
        }
        Err(err) => {
            log::warn!("Resolve {url}: request failed: {err}");
            url.to_string()
        }
    }
}

fn is_redirectable_url(url: &str) -> bool {
    url::Url::parse(url)
        .map(|parsed| matches!(parsed.scheme(), "http" | "https"))
        .unwrap_or(false)
}

fn cached_resolved_url(cache_key: &str) -> Option<String> {
    let cache = RESOLVED_URLS.get_or_init(|| Mutex::new(HashMap::new()));
    match cache.lock() {
        Ok(entries) => entries.get(cache_key).cloned(),
        Err(err) => {
            log::warn!("Resolved URL cache unavailable: {err}");
            None
        }
    }
}

fn store_resolved_url(cache_key: String, url: String) {
    let cache = RESOLVED_URLS.get_or_init(|| Mutex::new(HashMap::new()));
    match cache.lock() {
        Ok(mut entries) => {
            entries.insert(cache_key, url);
        }
        Err(err) => log::warn!("Resolved URL cache unavailable: {err}"),
    }
}

/// Convert a POSIX `LANG` value (e.g. `pt_BR.UTF-8`) into an `Accept-Language`
/// header value that mirrors what Chromium sends. An empty string means "no
/// override" — callers should skip setting the header in that case.
fn derive_accept_language(lang: &str) -> String {
    let base = lang.split('.').next().unwrap_or("").replace('_', "-");
    if base.is_empty() || base.eq_ignore_ascii_case("C") || base.eq_ignore_ascii_case("POSIX") {
        return String::new();
    }
    let primary = base.split('-').next().unwrap_or(&base);
    if primary == base {
        format!("{base},en;q=0.8")
    } else {
        format!("{base},{primary};q=0.9,en;q=0.8")
    }
}

/// Compact, human-readable form of `url` for list-row subtitles.
///
/// Returns `host` + `path` only, dropping the scheme, the query string, the
/// fragment, a leading `www.`, and any trailing slash. The point is sign-in
/// flows: a captured Google OAuth URL such as
/// `https://accounts.google.com/v3/signin/identifier?continue=…` carries
/// hundreds of query characters that wrap a row over several lines and break
/// its layout — the bloat lives entirely in the query, so dropping it yields a
/// stable `accounts.google.com/v3/signin/identifier`. Falls back to the raw
/// string when the URL has no parseable host. Display-only: the stored launch
/// URL keeps its full form.
#[must_use]
pub fn display_url(url: &str) -> String {
    let Ok(parsed) = url::Url::parse(url) else {
        return url.to_string();
    };
    let Some(host) = parsed.host_str() else {
        return url.to_string();
    };
    let host = host.strip_prefix("www.").unwrap_or(host);
    let path = parsed.path().trim_end_matches('/');
    if path.is_empty() {
        host.to_string()
    } else {
        format!("{host}{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_url_drops_oauth_query_and_scheme() {
        assert_eq!(
            display_url(
                "https://accounts.google.com/v3/signin/identifier?continue=https://calendar.google.com/calendar&service=cl&flowName=GlifWebSignIn"
            ),
            "accounts.google.com/v3/signin/identifier"
        );
    }

    #[test]
    fn display_url_shows_bare_host_for_root() {
        assert_eq!(
            display_url("https://calendar.google.com/"),
            "calendar.google.com"
        );
        assert_eq!(display_url("https://www.notion.so/"), "notion.so");
    }

    #[test]
    fn display_url_keeps_meaningful_path() {
        assert_eq!(
            display_url("https://notion.so/myworkspace"),
            "notion.so/myworkspace"
        );
    }

    #[test]
    fn display_url_falls_back_on_unparseable() {
        assert_eq!(display_url("not-a-url"), "not-a-url");
    }

    #[test]
    fn accept_language_handles_pt_br() {
        assert_eq!(
            derive_accept_language("pt_BR.UTF-8"),
            "pt-BR,pt;q=0.9,en;q=0.8"
        );
    }

    #[test]
    fn accept_language_handles_primary_only() {
        assert_eq!(derive_accept_language("en.UTF-8"), "en,en;q=0.8");
    }

    #[test]
    fn accept_language_skips_c_locale() {
        assert!(derive_accept_language("C").is_empty());
        assert!(derive_accept_language("POSIX").is_empty());
        assert!(derive_accept_language("").is_empty());
    }

    #[test]
    fn redirectable_url_accepts_http_and_https_only() {
        assert!(is_redirectable_url("https://open.spotify.com/"));
        assert!(is_redirectable_url("http://127.0.0.1:8080/app"));
        assert!(!is_redirectable_url("file:///tmp/index.html"));
        assert!(!is_redirectable_url("spotify:track:1"));
        assert!(!is_redirectable_url("not-a-url"));
    }
}
