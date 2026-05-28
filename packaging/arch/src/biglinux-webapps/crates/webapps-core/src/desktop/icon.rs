//! Stable per-webapp icon storage.
//!
//! When the user picks an icon in the manager (favicon flow, drag-in, or file
//! picker), `webapp.app_icon` ends up holding either:
//!
//!   1. A volatile path under `~/.cache/biglinux-webapps/favicons/icon_N.<ext>`
//!      whose filename is reused on the next site fetch — saving another
//!      webapp would clobber this one's icon on disk.
//!   2. A user-chosen path somewhere arbitrary on the filesystem that could
//!      move, be renamed, or disappear.
//!
//! Either way, writing that path verbatim into the `.desktop`'s `Icon=` field
//! leaves the entry tied to mutable state outside our control. We copy the
//! icon into our own data directory under a name keyed off the webapp URL so
//! the entry references a path we own and can keep stable.
//!
//! Absolute paths in `Icon=` are valid per the freedesktop entry spec and
//! GNOME Shell renders them directly — no icon-theme indexing required.
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;
use crate::models::WebApp;

use super::paths::desktop_file_id;

const ALLOWED_EXTS: &[&str] = &["png", "svg", "ico", "webp", "jpg", "jpeg"];

/// Copy `webapp.app_icon` into the webapp icons directory and return the new
/// absolute path. Returns `None` when the source is empty, isn't an absolute
/// path on disk, or the copy fails — callers should keep the original value
/// in that case.
pub fn persist_icon(webapp: &WebApp) -> Option<String> {
    let source = webapp.app_icon.trim();
    if source.is_empty() {
        return None;
    }

    let source_path = Path::new(source);
    if !source_path.is_absolute() || !source_path.is_file() {
        return None;
    }

    let target_dir = webapp_icons_dir();
    if let Err(err) = fs::create_dir_all(&target_dir) {
        log::warn!("Create webapp icons dir {}: {err}", target_dir.display());
        return None;
    }

    let ext = extension_for(source_path);
    let stem = format!("webapp-{}", desktop_file_id(&webapp.app_url));
    let target = target_dir.join(format!("{stem}.{ext}"));

    if source_path == target {
        return Some(target.to_string_lossy().into_owned());
    }

    remove_sibling_extensions(&target_dir, &stem, &target);

    if let Err(err) = fs::copy(source_path, &target) {
        log::warn!(
            "Copy webapp icon {} → {}: {err}",
            source_path.display(),
            target.display()
        );
        return None;
    }

    Some(target.to_string_lossy().into_owned())
}

/// `~/.local/share/biglinux-webapps/icons` — under our own data dir so we
/// don't pollute the user's icon theme tree and can clean up on delete.
pub fn webapp_icons_dir() -> PathBuf {
    config::data_dir().join("icons")
}

fn extension_for(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .filter(|s| ALLOWED_EXTS.contains(&s.as_str()))
        .unwrap_or_else(|| "png".to_string())
}

/// When a webapp is re-saved with a different image format (e.g. SVG → PNG),
/// leave only the new file so theme caches and previews can't pick up the
/// stale one with the same stem.
fn remove_sibling_extensions(dir: &Path, stem: &str, keep: &Path) {
    for ext in ALLOWED_EXTS {
        let candidate = dir.join(format!("{stem}.{ext}"));
        if candidate == keep {
            continue;
        }
        let _ = fs::remove_file(&candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WebApp;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn webapp_with(icon: &str, url: &str) -> WebApp {
        WebApp {
            app_url: url.to_string(),
            app_icon: icon.to_string(),
            ..WebApp::default()
        }
    }

    #[test]
    fn persist_returns_none_for_empty_icon() {
        let app = webapp_with("", "https://spotify.com");
        assert!(persist_icon(&app).is_none());
    }

    #[test]
    fn persist_returns_none_for_theme_name() {
        let app = webapp_with("firefox", "https://spotify.com");
        assert!(persist_icon(&app).is_none());
    }

    #[test]
    #[serial]
    fn persist_copies_to_data_dir_and_returns_absolute_path() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("XDG_DATA_HOME", tmp.path());

        let src = tmp.path().join("source.png");
        fs::write(&src, b"\x89PNG\r\n").unwrap();

        let app = webapp_with(src.to_str().unwrap(), "https://open.spotify.com/");
        let result = persist_icon(&app);

        std::env::remove_var("XDG_DATA_HOME");

        let new_path = result.expect("expected persisted path");
        assert!(new_path.ends_with("biglinux-webapps/icons/webapp-openspotifycom.png"));
        assert!(Path::new(&new_path).is_file());
    }

    #[test]
    #[serial]
    fn persist_removes_sibling_extension_when_format_changes() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("XDG_DATA_HOME", tmp.path());

        let icons_dir = webapp_icons_dir();
        fs::create_dir_all(&icons_dir).unwrap();
        let stale = icons_dir.join("webapp-openspotifycom.svg");
        fs::write(&stale, b"<svg/>").unwrap();

        let src = tmp.path().join("new.png");
        fs::write(&src, b"\x89PNG\r\n").unwrap();
        let app = webapp_with(src.to_str().unwrap(), "https://open.spotify.com/");

        let _ = persist_icon(&app);

        std::env::remove_var("XDG_DATA_HOME");

        assert!(
            !stale.exists(),
            "stale sibling icon should have been removed"
        );
    }
}
