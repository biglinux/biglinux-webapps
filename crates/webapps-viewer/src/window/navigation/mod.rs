//! Viewer navigation wiring: fullscreen toggle, URL entry, and WebView
//! navigation controls and new-window requests.

mod fullscreen;
mod url_entry;
mod webview;

pub(super) use fullscreen::{connect_fullscreen, setup_fullscreen_reveal};
pub(super) use url_entry::connect_url_entry;
pub(super) use webview::{connect_navigation_controls, connect_new_window_requests};
