use glib::clone;
use gtk4 as gtk;
use gtk4::gio;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(crate) fn connect_navigation_controls(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    title_widget: &adw::WindowTitle,
    back_btn: &gtk::Button,
    forward_btn: &gtk::Button,
    reload_btn: &gtk::Button,
) {
    back_btn.connect_clicked(clone!(
        #[weak]
        webview,
        move |_| {
            webview.go_back();
        }
    ));
    forward_btn.connect_clicked(clone!(
        #[weak]
        webview,
        move |_| {
            webview.go_forward();
        }
    ));
    reload_btn.connect_clicked(clone!(
        #[weak]
        webview,
        move |_| {
            webview.reload();
        }
    ));

    webview.connect_title_notify(clone!(
        #[weak]
        title_widget,
        #[weak]
        window,
        move |wv| {
            if let Some(title) = wv.title() {
                let title = title.to_string();
                if !title.is_empty() {
                    title_widget.set_title(&title);
                    window.set_title(Some(&title));
                }
            }
        }
    ));

    webview.connect_uri_notify(clone!(
        #[weak]
        title_widget,
        #[weak]
        back_btn,
        #[weak]
        forward_btn,
        move |wv| {
            if let Some(uri) = wv.uri() {
                title_widget.set_subtitle(&uri);
            }
            back_btn.set_sensitive(wv.can_go_back());
            forward_btn.set_sensitive(wv.can_go_forward());
        }
    ));

    webview.connect_load_changed(clone!(
        #[weak]
        back_btn,
        #[weak]
        forward_btn,
        move |wv, _event| {
            back_btn.set_sensitive(wv.can_go_back());
            forward_btn.set_sensitive(wv.can_go_forward());
        }
    ));
}

pub(crate) fn connect_new_window_requests(webview: &webkit::WebView) {
    webview.connect_create(|wv, action| {
        let request = action.request()?;
        let uri = request.uri().map(|g| g.to_string())?;

        let current = wv.uri().map(|g| g.to_string()).unwrap_or_default();
        if is_same_origin(&current, &uri) {
            wv.load_uri(&uri);
        } else {
            // Cross-origin window.open() / target=_blank → hand off to system default browser
            // to avoid leaking webapp session cookies into untrusted content.
            if let Err(err) =
                gio::AppInfo::launch_default_for_uri(&uri, gio::AppLaunchContext::NONE)
            {
                log::warn!(
                    "Failed to open {uri} in default handler: {err}; falling back to internal load"
                );
                wv.load_uri(&uri);
            }
        }
        None
    });
}

fn is_same_origin(current: &str, target: &str) -> bool {
    let Ok(current) = url::Url::parse(current) else {
        // No origin context → treat as same-origin to preserve current behaviour for
        // about:blank, data: URIs and pre-load callbacks.
        return true;
    };
    let Ok(target) = url::Url::parse(target) else {
        return true;
    };

    current.scheme() == target.scheme()
        && current.host_str() == target.host_str()
        && current.port_or_known_default() == target.port_or_known_default()
}

#[cfg(test)]
mod tests {
    use super::is_same_origin;

    #[test]
    fn same_host_and_scheme_is_same_origin() {
        assert!(is_same_origin(
            "https://example.com/page",
            "https://example.com/other",
        ));
    }

    #[test]
    fn different_host_is_cross_origin() {
        assert!(!is_same_origin(
            "https://example.com/page",
            "https://attacker.com/page",
        ));
    }

    #[test]
    fn different_scheme_is_cross_origin() {
        assert!(!is_same_origin(
            "https://example.com/page",
            "http://example.com/page",
        ));
    }

    #[test]
    fn different_port_is_cross_origin() {
        assert!(!is_same_origin(
            "https://example.com/page",
            "https://example.com:8443/page",
        ));
    }

    #[test]
    fn missing_current_origin_treated_as_same() {
        // about:blank / first load callbacks must not be hijacked
        assert!(is_same_origin("", "https://example.com/page"));
    }
}
