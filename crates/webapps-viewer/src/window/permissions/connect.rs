use super::PermissionDecision;
use libadwaita as adw;
use std::path::Path;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(crate) fn connect_permission_requests(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    perm_path: &Path,
) {
    let weak_window = window.downgrade();
    let perm_path = perm_path.to_path_buf();
    webview.connect_permission_request(move |wv, request| {
        let Some(window) = weak_window.upgrade() else {
            request.deny();
            return true;
        };
        match super::classify_request(request) {
            PermissionDecision::Allow => request.allow(),
            PermissionDecision::Deny => request.deny(),
            PermissionDecision::Prompt(key) => {
                prompt_keys(&window, wv, request, &perm_path, vec![key])
            }
            PermissionDecision::CameraAndMicrophone => prompt_keys(
                &window,
                wv,
                request,
                &perm_path,
                vec!["camera", "microphone"],
            ),
        }
        true
    });
}

fn scoped_key(uri: &str, permission: &str) -> Option<String> {
    let url = url::Url::parse(uri).ok()?;
    if !matches!(url.scheme(), "https" | "http") {
        return None;
    }
    Some(format!(
        "{}|{permission}",
        url.origin().ascii_serialization()
    ))
}

fn prompt_keys(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    request: &webkit::PermissionRequest,
    path: &Path,
    mut keys: Vec<&'static str>,
) {
    let Some(key) = keys.pop() else {
        request.allow();
        return;
    };
    let uri = webview.uri().unwrap_or_default().to_string();
    let Some(scoped) = scoped_key(&uri, key) else {
        request.deny();
        return;
    };
    if super::load_permissions(path).get(&scoped) == Some(&false) {
        request.deny();
        return;
    }
    // WebKit does not expose the requesting frame origin; never reuse an allow for another frame.
    let request = request.clone();
    let window = window.clone();
    let webview = webview.downgrade();
    let path = path.to_path_buf();
    super::prompt_permission(&window.clone(), key, move |granted| {
        let Some(webview) = webview.upgrade() else {
            request.deny();
            return;
        };
        if webview.uri().as_deref() != Some(&uri) {
            request.deny();
            return;
        }
        let mut permissions = super::load_permissions(&path);
        permissions.insert(scoped, granted);
        super::save_permissions(&path, &permissions);
        if granted {
            prompt_keys(&window, &webview, &request, &path, keys);
        } else {
            request.deny();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permission_scope_includes_scheme_host_and_port() {
        let key = scoped_key("https://example.com/a", "camera");
        assert_eq!(key, scoped_key("https://example.com/b", "camera"));
        for uri in [
            "http://example.com",
            "https://other.com",
            "https://example.com:8443",
        ] {
            assert_ne!(key, scoped_key(uri, "camera"));
        }
        assert_ne!(key, scoped_key("https://example.com", "microphone"));
        assert!(scoped_key("file:///tmp/index.html", "camera").is_none());
    }
}
