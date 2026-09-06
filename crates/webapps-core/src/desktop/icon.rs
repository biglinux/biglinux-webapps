use crate::{config, models::WebApp, storage::write_atomic};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn icon_destination(webapp: &WebApp) -> Option<PathBuf> {
    let source = Path::new(webapp.app_icon.trim());
    if !source.is_absolute() {
        return None;
    }
    let ext = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    let ext = if ["png", "svg", "ico", "webp", "jpg", "jpeg", "gif"].contains(&ext.as_str()) {
        ext.as_str()
    } else {
        "png"
    };
    let identity = webapp
        .desktop_file_name()
        .map(|name| name.as_str().trim_end_matches(".desktop").to_string())
        .unwrap_or_else(|| {
            super::paths::viewer_desktop_filename(&webapp.app_url)
                .trim_end_matches(".desktop")
                .to_string()
        });
    Some(webapp_icons_dir().join(format!("webapp-{identity}.{ext}")))
}

pub fn persist_icon(webapp: &WebApp) -> Option<String> {
    let target = icon_destination(webapp)?;
    let source = Path::new(webapp.app_icon.trim());
    if source == target {
        return source
            .is_file()
            .then(|| target.to_string_lossy().into_owned());
    }
    let result = fs::read(source)
        .map_err(anyhow::Error::from)
        .and_then(|bytes| write_atomic(&target, &bytes));
    if let Err(err) = result {
        log::warn!("Persist icon {}: {err:#}", source.display());
        return None;
    }
    Some(target.to_string_lossy().into_owned())
}

pub fn webapp_icons_dir() -> PathBuf {
    config::data_dir().join("icons")
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
        assert!(new_path.contains("biglinux-webapps/icons/webapp-biglinux-webapp-openspotifycom-"));
        assert!(Path::new(&new_path).is_file());
    }

    #[test]
    #[serial]
    fn persist_preserves_other_icon_files() {
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
            stale.exists(),
            "unreferenced files are cleaned only after the registry commits"
        );
    }
}
