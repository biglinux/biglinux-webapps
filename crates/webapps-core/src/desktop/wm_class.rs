use crate::models::{AppMode, BrowserId, WebApp};

pub(super) fn derive_wm_class(webapp: &WebApp) -> String {
    match webapp.app_mode {
        AppMode::App => {
            let app_id = webapp
                .app_url
                .replace("https://", "")
                .replace("http://", "")
                .replace('/', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>();
            format!("br.com.biglinux.webapp.{app_id}")
        }
        AppMode::Browser => {
            let url_class = browser_url_class(&webapp.app_url);
            let prefix = browser_wm_prefix(&webapp.browser_id());
            format!("{prefix}-{url_class}-Default")
        }
    }
}

fn browser_wm_prefix(browser: &BrowserId) -> String {
    use crate::browsers::find_def;

    if let Some(definition) = find_def(browser.as_str()) {
        if !definition.wm_class_prefix.is_empty() {
            return definition.wm_class_prefix.clone();
        }
    }

    browser
        .as_str()
        .strip_prefix("flatpak-")
        .unwrap_or(browser.as_str())
        .to_string()
}

pub(super) fn browser_url_class(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();
        let path_class = path.replace('/', "__");
        format!("{host}{path_class}")
    } else {
        derive_class_from_url(url)
    }
}

pub(super) fn derive_class_from_url(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .replace('/', "__")
}
