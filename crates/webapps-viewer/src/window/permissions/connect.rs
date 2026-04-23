use std::path::Path;

use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

use super::PermissionDecision;

pub(crate) fn connect_permission_requests(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    perm_path: &Path,
) {
    let weak_window = window.downgrade();
    let perm_path = perm_path.to_path_buf();

    webview.connect_permission_request(move |_wv, request| {
        let Some(window) = weak_window.upgrade() else {
            return false;
        };

        match super::classify_request(request) {
            PermissionDecision::Allow => request.allow(),
            PermissionDecision::Deny => request.deny(),
            PermissionDecision::Prompt(perm_key) => {
                match super::load_permissions(&perm_path).get(perm_key) {
                    Some(true) => request.allow(),
                    Some(false) => request.deny(),
                    None => {
                        let request = request.clone();
                        let perm_path = perm_path.clone();
                        let perm_key_owned = perm_key.to_string();
                        super::prompt_permission(&window, perm_key, move |granted| {
                            if granted {
                                request.allow();
                            } else {
                                request.deny();
                            }
                            let mut permissions = super::load_permissions(&perm_path);
                            permissions.insert(perm_key_owned, granted);
                            super::save_permissions(&perm_path, &permissions);
                        });
                    }
                }
            }
        }

        true
    });
}
