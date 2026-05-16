use std::{cell::Cell, rc::Rc, time::Duration};

use glib::clone;
use libadwaita as adw;
use url::Url;
use webkit6 as webkit;
use webkit6::prelude::*;

const INITIAL_LOAD_RECOVERY_DELAY: Duration = Duration::from_secs(1);
const STALLED_INITIAL_PROGRESS: f64 = 0.35;

pub(super) fn connect_initial_load(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    url: &str,
) {
    let initial_url = url.to_string();
    let initial_load_started = Rc::new(Cell::new(false));

    window.connect_map(clone!(
        #[weak]
        webview,
        #[strong]
        initial_url,
        #[strong]
        initial_load_started,
        move |_| {
            if initial_load_started.replace(true) {
                return;
            }
            webview.load_uri(&initial_url);
            schedule_initial_load_recovery(&webview, &initial_url);
        }
    ));
}

fn schedule_initial_load_recovery(webview: &webkit::WebView, initial_url: &str) {
    let initial_url = initial_url.to_string();
    glib::timeout_add_local_once(
        INITIAL_LOAD_RECOVERY_DELAY,
        clone!(
            #[weak]
            webview,
            #[strong]
            initial_url,
            move || {
                if should_reload_initial_load(&webview, &initial_url) {
                    log::info!("Reloading stalled initial viewer load: {initial_url}");
                    webview.reload();
                }
            }
        ),
    );
}

fn should_reload_initial_load(webview: &webkit::WebView, initial_url: &str) -> bool {
    let current_url = webview.uri().map(|uri| uri.to_string()).unwrap_or_default();
    if !same_initial_site(initial_url, &current_url) {
        return false;
    }

    title_is_missing(webview) || webview.estimated_load_progress() < STALLED_INITIAL_PROGRESS
}

fn title_is_missing(webview: &webkit::WebView) -> bool {
    webview
        .title()
        .map(|title| title.trim().is_empty())
        .unwrap_or(true)
}

fn same_initial_site(initial_url: &str, current_url: &str) -> bool {
    if current_url.is_empty() {
        return true;
    }
    let Ok(initial) = Url::parse(initial_url) else {
        return true;
    };
    let Ok(current) = Url::parse(current_url) else {
        return true;
    };

    initial.scheme() == current.scheme() && comparable_host(&initial) == comparable_host(&current)
}

fn comparable_host(url: &Url) -> Option<&str> {
    url.host_str()
        .map(|host| host.strip_prefix("www.").unwrap_or(host))
}

#[cfg(test)]
mod tests {
    use super::same_initial_site;

    #[test]
    fn empty_current_url_is_initial_site() {
        assert!(same_initial_site("https://youtube.com", ""));
    }

    #[test]
    fn same_host_is_initial_site() {
        assert!(same_initial_site(
            "https://youtube.com",
            "https://youtube.com/feed/subscriptions"
        ));
    }

    #[test]
    fn www_redirect_is_initial_site() {
        assert!(same_initial_site(
            "https://youtube.com",
            "https://www.youtube.com/feed/subscriptions"
        ));
    }

    #[test]
    fn different_host_is_not_initial_site() {
        assert!(!same_initial_site(
            "https://youtube.com",
            "https://accounts.google.com"
        ));
    }
}
