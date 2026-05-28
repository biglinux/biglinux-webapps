use glib::clone;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(crate) fn connect_url_entry(
    url_entry: &gtk::Entry,
    url_bar: &gtk::Revealer,
    webview: &webkit::WebView,
) {
    url_entry.connect_activate(clone!(
        #[weak]
        webview,
        #[weak]
        url_bar,
        move |entry| {
            if let Some(uri) = normalize_navigation_url(&entry.text()) {
                webview.load_uri(&uri);
            }
            url_bar.set_reveal_child(false);
        }
    ));

    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.connect_key_pressed(clone!(
        #[weak]
        url_bar,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                url_bar.set_reveal_child(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    ));
    url_entry.add_controller(key_ctrl);
}

fn normalize_navigation_url(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("file://")
        {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        },
    )
}

#[cfg(test)]
mod tests {
    use super::normalize_navigation_url;

    #[test]
    fn normalize_navigation_url_keeps_existing_scheme() {
        assert_eq!(
            normalize_navigation_url("https://example.com"),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn normalize_navigation_url_adds_https() {
        assert_eq!(
            normalize_navigation_url("example.com"),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn normalize_navigation_url_rejects_empty_input() {
        assert_eq!(normalize_navigation_url("   "), None);
    }
}
