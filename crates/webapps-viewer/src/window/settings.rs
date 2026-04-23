//! WebView settings and JS injection for webapp-mode behaviour.
use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) const DEVELOPER_TOOLS_ENABLED: bool = cfg!(debug_assertions);

/// Chrome UA spoof: some sites (Spotify, Teams, YouTube) reject non-Chrome browsers.
/// Update when Chrome reaches a version ≥2 years older than the current stable.
/// Last updated: 2024-11 (Chrome 131).
pub(super) const SPOOFED_UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

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
        s.set_media_playback_requires_user_gesture(false);
        s.set_enable_media_stream(true);
        s.set_enable_mediasource(true);
        s.set_enable_encrypted_media(true);
        s.set_enable_smooth_scrolling(true);
        s.set_enable_back_forward_navigation_gestures(true);
        // spoof Chrome UA → sites like Spotify/Teams reject unknown browsers
        s.set_user_agent(Some(SPOOFED_UA));
    }
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
