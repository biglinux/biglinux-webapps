//! WebView settings and JS injection for webapp-mode behaviour.
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) const DEVELOPER_TOOLS_ENABLED: bool = cfg!(debug_assertions);
const NEXTCLOUD_STRICT_COOKIE_SUFFIX: &str = "nc_sameSiteCookiestrict";

/// Chrome UA spoof: some sites (Spotify, Teams, YouTube) reject non-Chrome browsers.
/// Update when Chrome reaches a version ≥2 years older than the current stable.
/// Last updated: 2026-06 (Chrome 149).
pub(super) const SPOOFED_UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36";

/// Configure WebView settings for webapp usage (JS, media, UA spoof, DevTools).
///
/// Clipboard write access is granted because most webapps (Slack, WhatsApp Web,
/// Notion) rely on `navigator.clipboard.writeText`. Read access is gated by the
/// regular WebKit permission flow handled in `permissions/`.
pub(super) fn configure_settings(webview: &webkit::WebView) {
    if let Some(s) = WebViewExt::settings(webview) {
        s.set_enable_javascript(true);
        // Clipboard write only — read is requested via clipboard PermissionRequest.
        s.set_javascript_can_access_clipboard(true);
        s.set_javascript_can_open_windows_automatically(false);
        s.set_enable_developer_extras(DEVELOPER_TOOLS_ENABLED);
        // Auto-play permitted (Spotify, YouTube, Music) but the navigator.mediaDevices
        // API still requires user-gesture-driven prompts handled by permissions/.
        s.set_enable_media(true);
        s.set_media_playback_requires_user_gesture(false);
        s.set_media_playback_allows_inline(true);
        s.set_enable_webaudio(true);
        s.set_enable_media_capabilities(true);
        s.set_enable_media_stream(true);
        s.set_enable_mediasource(true);
        s.set_enable_encrypted_media(true);
        s.set_enable_site_specific_quirks(true);
        s.set_enable_html5_local_storage(true);
        s.set_enable_page_cache(false);
        s.set_enable_smooth_scrolling(true);
        s.set_enable_back_forward_navigation_gestures(true);
        s.set_hardware_acceleration_policy(webkit::HardwareAccelerationPolicy::Always);
        // spoof Chrome UA → sites like Spotify/Teams reject unknown browsers
        s.set_user_agent(Some(SPOOFED_UA));
    }
}

/// Marker written into the webapp's profile directory once the site has been
/// identified as needing WebKit's native user agent.
///
/// The decision has to *persist*. Detection can only happen after a page has
/// loaded, so a purely in-memory flag means every launch starts by requesting
/// the login page under the wrong UA and then switching mid-session — the
/// document is fetched with one UA while its subresources use another, which is
/// exactly the inconsistency the switch is supposed to remove. With the marker,
/// detection happens once, ever, and every later launch sends the right UA on
/// its very first request.
const NATIVE_UA_MARKER: &str = "native-user-agent";

/// How many page loads may be inspected before giving up on detection.
///
/// Nextcloud sets its sentinel cookies on the first response from the instance,
/// so a handful of loads is generous. The budget exists so a webapp that will
/// never be Nextcloud stops dumping the whole cookie jar on every navigation for
/// the rest of the session.
const DETECTION_LOAD_BUDGET: u8 = 5;

/// Apply the persisted user-agent decision for this profile.
///
/// Called straight after [`configure_settings`], before the webview loads
/// anything, so a profile already known to need the native UA never issues a
/// single request under the spoofed one.
pub(super) fn apply_persisted_user_agent(webview: &webkit::WebView, data_dir: &Path) {
    if !prefers_native_user_agent(data_dir) {
        return;
    }
    if let Some(settings) = WebViewExt::settings(webview) {
        settings.set_user_agent(None);
        log::debug!("Profile is pinned to the native WebKit user agent");
    }
}

/// Watch for Nextcloud's strict CSRF sentinel and pin this profile to the native
/// user agent when it shows up.
///
/// # What the sentinel actually tells us
///
/// It identifies the instance as Nextcloud — nothing more. Checked against a
/// live instance, `GET /login` sets both `__Host-nc_sameSiteCookielax` and
/// `__Host-nc_sameSiteCookiestrict` identically under a Chromium UA and under
/// WebKit's own, so the cookie's *presence* says nothing about whether the
/// same-site check is passing.
///
/// That distinction matters because it is easy to write this the other way
/// round. An earlier comment here claimed a Chromium UA makes Nextcloud *omit*
/// the strict sentinel, which would make this function self-defeating: the
/// trigger it waits for would be the very thing the spoofed UA suppressed, so it
/// could never fire. The server sends the sentinel regardless, so detection is
/// reliable — but the signal must be read as "this is Nextcloud", not as "the
/// CSRF check just failed".
///
/// # Why Nextcloud gets the native UA
///
/// Empirically, Nextcloud's login POST fails in a loop when WebKitGTK claims to
/// be Chrome, and completes under WebKit's own UA. Nextcloud varies its session
/// and CSRF handling by user agent, so announcing an engine we are not lands the
/// request on a code path WebKitGTK does not satisfy. Because a Nextcloud
/// instance is self-hosted there is no domain to special-case, which is why this
/// is detected from the response rather than configured up front.
///
/// # Why this hooks page loads rather than cookie changes
///
/// The previous version listened on `CookieManager::changed`, which fires for
/// *every* cookie mutation by *any* site and dumped the entire jar each time.
/// On a busy webapp with dozens of cookies that is continuous churn for a signal
/// that can only appear on a page load. Worse, it never persisted the outcome
/// and never reloaded, so the detection could only help the *next* navigation —
/// the login page the user was already looking at kept the wrong UA.
pub(super) fn watch_for_native_ua_site(
    cookie_manager: &webkit::CookieManager,
    webview: &webkit::WebView,
    data_dir: &Path,
) {
    if prefers_native_user_agent(data_dir) {
        // Already decided and applied by `apply_persisted_user_agent`.
        return;
    }

    let budget = Rc::new(Cell::new(DETECTION_LOAD_BUDGET));
    let cookie_manager = cookie_manager.clone();
    let data_dir = data_dir.to_path_buf();

    webview.connect_load_changed(move |webview, event| {
        if event != webkit::LoadEvent::Finished {
            return;
        }
        // Once the UA has been switched there is nothing left to detect, and a
        // profile that has used up its budget is not Nextcloud.
        if !uses_spoofed_user_agent(webview) || budget.get() == 0 {
            return;
        }
        budget.set(budget.get() - 1);

        let webview = webview.downgrade();
        let data_dir = data_dir.clone();
        cookie_manager.all_cookies(None::<&gio::Cancellable>, move |result| {
            let Ok(mut cookies) = result else {
                return;
            };
            let has_sentinel = cookies.iter_mut().any(|cookie| {
                cookie
                    .name()
                    .is_some_and(|name| is_nextcloud_strict_cookie(&name))
            });
            if !has_sentinel {
                return;
            }
            let Some(webview) = webview.upgrade() else {
                return;
            };
            // Re-check: the jar dump is async, so another callback may have
            // switched the UA while this one was in flight. Without this the
            // reload below could fire twice.
            if !uses_spoofed_user_agent(&webview) {
                return;
            }
            adopt_native_user_agent(&webview, &data_dir);
        });
    });
}

/// Switch to the native UA, remember the decision, and re-fetch the page.
///
/// The reload is what makes the fix apply to the login attempt in progress
/// instead of only to the next one: the document currently on screen was fetched
/// under the spoofed UA, so leaving it alone would still fail the CSRF check and
/// the user would see the loop once before it healed.
fn adopt_native_user_agent(webview: &webkit::WebView, data_dir: &Path) {
    let Some(settings) = WebViewExt::settings(webview) else {
        return;
    };
    settings.set_user_agent(None);
    remember_native_user_agent(data_dir);
    log::info!("Nextcloud detected — pinned to the native WebKit user agent and reloading");
    webview.reload();
}

fn marker_path(data_dir: &Path) -> PathBuf {
    data_dir.join(NATIVE_UA_MARKER)
}

fn prefers_native_user_agent(data_dir: &Path) -> bool {
    marker_path(data_dir).exists()
}

fn remember_native_user_agent(data_dir: &Path) {
    let path = marker_path(data_dir);
    if let Err(err) = std::fs::write(&path, b"") {
        // Not fatal: the UA is already switched for this session, the profile
        // just has to re-detect on the next launch.
        log::warn!("Persist native-UA marker {}: {err}", path.display());
    }
}

fn uses_spoofed_user_agent(webview: &webkit::WebView) -> bool {
    WebViewExt::settings(webview)
        .and_then(|settings| settings.user_agent())
        .as_deref()
        == Some(SPOOFED_UA)
}

fn is_nextcloud_strict_cookie(name: &str) -> bool {
    name.ends_with(NEXTCLOUD_STRICT_COOKIE_SUFFIX)
}

/// Inject JS to block web content from resizing or moving the window.
pub(super) fn inject_resize_block(webview: &webkit::WebView) {
    let ucm = webview
        .user_content_manager()
        .expect("WebView must have UserContentManager");
    let script = webkit::UserScript::new(
        concat!(
            "window.resizeTo=function(){};",
            "window.resizeBy=function(){};",
            "window.moveTo=function(){};",
            "window.moveBy=function(){};",
        ),
        webkit::UserContentInjectedFrames::AllFrames,
        webkit::UserScriptInjectionTime::Start,
        &[],
        &[],
    );
    ucm.add_script(&script);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_prefixed_nextcloud_strict_cookie() {
        assert!(is_nextcloud_strict_cookie("__Host-nc_sameSiteCookiestrict"));
        // Nextcloud drops the `__Host-` prefix over plain HTTP.
        assert!(is_nextcloud_strict_cookie("nc_sameSiteCookiestrict"));
    }

    #[test]
    fn ignores_unrelated_strict_cookie() {
        assert!(!is_nextcloud_strict_cookie("session_strict"));
        // The *lax* sentinel must not trigger the switch: it is present even
        // when the strict check is passing, so keying off it would pin every
        // Nextcloud profile to the native UA unconditionally.
        assert!(!is_nextcloud_strict_cookie("__Host-nc_sameSiteCookielax"));
    }

    #[test]
    fn marker_absent_means_the_spoofed_ua_is_kept() {
        let tmp = TempDir::new().unwrap();
        assert!(!prefers_native_user_agent(tmp.path()));
    }

    #[test]
    fn remembering_the_decision_survives_a_relaunch() {
        // The marker is the whole point of persisting: the next launch must send
        // the native UA on its first request instead of re-detecting.
        let tmp = TempDir::new().unwrap();
        remember_native_user_agent(tmp.path());

        assert!(prefers_native_user_agent(tmp.path()));
        assert!(marker_path(tmp.path()).is_file());
        // Idempotent — a second detection round must not error or duplicate.
        remember_native_user_agent(tmp.path());
        assert!(prefers_native_user_agent(tmp.path()));
    }

    #[test]
    fn marker_lives_inside_the_profile_directory() {
        // Per-profile, not global: one Nextcloud webapp must not pin an
        // unrelated Spotify webapp to the native UA.
        let tmp = TempDir::new().unwrap();
        let nextcloud = tmp.path().join("cloudexampleorg");
        let spotify = tmp.path().join("openspotifycom");
        std::fs::create_dir_all(&nextcloud).unwrap();
        std::fs::create_dir_all(&spotify).unwrap();

        remember_native_user_agent(&nextcloud);

        assert!(prefers_native_user_agent(&nextcloud));
        assert!(!prefers_native_user_agent(&spotify));
        assert_eq!(marker_path(&nextcloud), nextcloud.join(NATIVE_UA_MARKER));
    }

    #[test]
    fn unwritable_profile_dir_does_not_panic() {
        // A read-only or missing profile dir must degrade to re-detecting next
        // launch, never abort the viewer.
        remember_native_user_agent(Path::new("/nonexistent-profile-dir-xyz"));
        assert!(!prefers_native_user_agent(Path::new(
            "/nonexistent-profile-dir-xyz"
        )));
    }
}
