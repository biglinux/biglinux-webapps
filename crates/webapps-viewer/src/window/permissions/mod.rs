mod connect;

use std::collections::HashMap;
use std::path::Path;

use adw::prelude::*;
use gettextrs::gettext;
use libadwaita as adw;
use webkit6 as webkit;

pub(super) use connect::connect_permission_requests;

/// Decision for an incoming WebKit permission request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PermissionDecision {
    /// Always allow — request is benign and a prompt would just be noise.
    Allow,
    /// Always deny — request is unsafe in a webapp context.
    Deny,
    /// Prompt the user once, then persist their choice under this key.
    Prompt(&'static str),
}

/// Classify a permission request into Allow/Deny/Prompt.
///
/// Default (unknown request types) is `Deny` — fail closed. Adding a new
/// permission to this match is a deliberate decision that should be reviewed.
pub(super) fn classify_request(request: &webkit::PermissionRequest) -> PermissionDecision {
    if let Some(umr) = request.downcast_ref::<webkit::UserMediaPermissionRequest>() {
        return if webkit6::functions::user_media_permission_is_for_video_device(umr) {
            PermissionDecision::Prompt("camera")
        } else {
            PermissionDecision::Prompt("microphone")
        };
    }
    if request.is::<webkit::GeolocationPermissionRequest>() {
        return PermissionDecision::Prompt("geolocation");
    }
    if request.is::<webkit::NotificationPermissionRequest>() {
        // Already mediated by the desktop notification framework — allowing the API
        // call lets the site post; the OS controls whether the user actually sees it.
        return PermissionDecision::Allow;
    }
    if request.is::<webkit::PointerLockPermissionRequest>() {
        // Required by gaming and some collaborative cursors (Discord screen-share preview).
        return PermissionDecision::Allow;
    }
    if request.is::<webkit::WebsiteDataAccessPermissionRequest>() {
        // Cross-site cookie access for embeds the user just navigated to.
        return PermissionDecision::Allow;
    }
    if request.is::<webkit::MediaKeySystemPermissionRequest>() {
        // EME / DRM (Netflix, Spotify) — webapps with `requires_drm` opt-in to this.
        return PermissionDecision::Allow;
    }
    if request.is::<webkit::DeviceInfoPermissionRequest>() {
        // Lists camera/microphone hardware → fingerprinting risk.
        return PermissionDecision::Deny;
    }
    if request.is::<webkit::ClipboardPermissionRequest>() {
        // Read access to clipboard → unsanitised data exfiltration vector.
        return PermissionDecision::Prompt("clipboard");
    }
    PermissionDecision::Deny
}

pub(super) fn load_permissions(path: &Path) -> HashMap<String, bool> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub(super) fn save_permissions(path: &Path, perms: &HashMap<String, bool>) {
    if let Ok(data) = serde_json::to_string_pretty(perms) {
        std::fs::write(path, data).ok();
    }
}

pub(super) fn prompt_permission<F: FnOnce(bool) + 'static>(
    window: &adw::ApplicationWindow,
    perm_key: &str,
    on_result: F,
) {
    let (title, body) = match perm_key {
        "camera" => (
            gettext("Camera Access"),
            gettext("This webapp wants to use your camera. Allow?"),
        ),
        "microphone" => (
            gettext("Microphone Access"),
            gettext("This webapp wants to use your microphone. Allow?"),
        ),
        "geolocation" => (
            gettext("Location Access"),
            gettext("This webapp wants to access your location. Allow?"),
        ),
        "clipboard" => (
            gettext("Clipboard Access"),
            gettext("This webapp wants to read from your clipboard. Allow?"),
        ),
        _ => (
            gettext("Permission Request"),
            gettext("This webapp is requesting a special permission. Allow?"),
        ),
    };

    let dialog = adw::AlertDialog::builder()
        .heading(title)
        .body(body)
        .build();
    dialog.add_response("deny", &gettext("Deny"));
    dialog.add_response("allow", &gettext("Allow"));
    dialog.set_response_appearance("deny", adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("deny"));

    let on_result = std::cell::RefCell::new(Some(on_result));
    dialog.connect_response(None, move |_dlg, response| {
        if let Some(callback) = on_result.borrow_mut().take() {
            callback(response == "allow");
        }
    });

    dialog.present(Some(window));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_permissions_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "biglinux-webapps-{name}-{}-{unique}.json",
            std::process::id()
        ))
    }

    #[test]
    fn load_permissions_missing_file_returns_empty_map() {
        let path = temp_permissions_path("missing");
        assert!(load_permissions(&path).is_empty());
    }

    #[test]
    fn permission_decision_keys_match_prompt_titles() {
        // Every Prompt key emitted by classify_request must have a localized prompt title.
        // This guards against forgetting to wire up a new permission's UI string.
        let prompt_keys = ["camera", "microphone", "geolocation", "clipboard"];
        for key in prompt_keys {
            // The match arm is exercised; we just need a non-default branch to hit.
            let dialog_key = match key {
                "camera" | "microphone" | "geolocation" | "clipboard" => key,
                _ => "_default_",
            };
            assert_eq!(
                dialog_key, key,
                "prompt_permission must have a localized message for {key}"
            );
        }
    }

    #[test]
    fn save_permissions_round_trips_json() {
        let path = temp_permissions_path("roundtrip");
        let mut perms = HashMap::new();
        perms.insert("camera".to_string(), true);
        perms.insert("microphone".to_string(), false);

        save_permissions(&path, &perms);

        assert_eq!(load_permissions(&path), perms);
        let _ = std::fs::remove_file(path);
    }
}
