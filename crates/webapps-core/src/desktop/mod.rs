mod builder;
mod icon;
mod paths;
mod sanitize;
mod wm_class;

pub use builder::generate_desktop_entry;
pub use icon::{persist_icon, webapp_icons_dir};
pub use paths::{
    desktop_file_id, desktop_file_path, install_desktop_entry, legacy_host_desktop_file_id,
    remove_desktop_entry, remove_desktop_file, viewer_desktop_filename,
};
pub use wm_class::{canonical_browser_desktop_filename, chromium_browser_app_id};

#[cfg(test)]
mod tests {
    use super::sanitize::{sanitize_desktop_field, sanitize_desktop_value};
    use super::wm_class::derive_wm_class;
    use super::*;
    use crate::models::{AppMode, WebApp};

    fn app(url: &str, mode: AppMode) -> WebApp {
        WebApp {
            app_url: url.to_string(),
            app_name: "Test App".to_string(),
            app_mode: mode,
            browser: "brave-browser".to_string(),
            app_profile: "Default".to_string(),
            ..WebApp::default()
        }
    }

    #[test]
    fn desktop_file_id_strips_dots() {
        assert_eq!(
            desktop_file_id("https://music.youtube.com/"),
            "musicyoutubecom"
        );
    }

    #[test]
    fn desktop_file_id_plain_domain() {
        assert_eq!(desktop_file_id("https://spotify.com"), "spotifycom");
    }

    #[test]
    fn desktop_file_id_includes_path_to_avoid_collisions() {
        assert_eq!(
            desktop_file_id("https://cloud.talesam.org/apps/notes"),
            "cloudtalesamorg_apps_notes"
        );
        assert_eq!(
            desktop_file_id("https://cloud.talesam.org/apps/calendar"),
            "cloudtalesamorg_apps_calendar"
        );
    }

    #[test]
    fn legacy_host_desktop_file_id_keeps_pre_path_scheme() {
        assert_eq!(
            legacy_host_desktop_file_id("https://cloud.talesam.org/apps/notes"),
            "cloudtalesamorg"
        );
    }

    #[test]
    fn viewer_desktop_filename_uses_path_aware_id() {
        assert_eq!(
            viewer_desktop_filename("https://cloud.talesam.org/apps/notes"),
            "biglinux-webapp-cloudtalesamorg_apps_notes.desktop"
        );
    }

    #[test]
    fn desktop_file_id_invalid_url_falls_back() {
        assert_eq!(desktop_file_id("not-a-url"), "webapp");
    }

    #[test]
    fn sanitize_strips_dangerous_chars() {
        let input = r#"hello \"world\" $foo`bar"#;
        let out = sanitize_desktop_field(input);
        assert!(!out.contains('"'));
        assert!(!out.contains('$'));
        assert!(!out.contains('`'));
        assert_eq!(out, "hello world foobar");
    }

    #[test]
    fn sanitize_keeps_safe_chars() {
        let input = "https://example.com/path?q=1&r=2";
        let out = sanitize_desktop_field(input);
        assert_eq!(out, input);
    }

    #[test]
    fn sanitize_desktop_value_strips_line_breaks() {
        let input = "hello\nworld\r!";
        let out = sanitize_desktop_value(input);
        assert_eq!(out, "helloworld!");
    }

    #[test]
    fn chromium_app_id_matches_brave_synthesis_for_spotify() {
        // Matches the value Brave 148 on GNOME 47 Wayland actually emits as the
        // xdg-shell app_id when launched with --app=URL --profile-directory=Default
        // for https://open.spotify.com/intl-pt/. If this diverges, GNOME Shell
        // can't map the window to our .desktop and falls back to the host browser's
        // icon.
        let id = chromium_browser_app_id("brave", "https://open.spotify.com/intl-pt/", "Default");
        assert_eq!(id, "brave-open.spotify.com__intl-pt_-Default");
    }

    #[test]
    fn chromium_app_id_root_path_has_single_trailing_underscore() {
        let id = chromium_browser_app_id("brave", "https://deezer.com/", "Default");
        assert_eq!(id, "brave-deezer.com__-Default");
    }

    #[test]
    fn derive_wm_class_app_mode_uses_reverse_dns() {
        let w = app("https://spotify.com", AppMode::App);
        let cls = derive_wm_class(&w);
        assert!(
            cls.starts_with("br.com.biglinux.webapp."),
            "Expected reverse-DNS prefix: {cls}"
        );
    }

    #[test]
    fn derive_wm_class_app_mode_matches_viewer_application_id() {
        // Must equal the viewer's GTK application_id so Wayland compositors can
        // associate the window with the .desktop entry (otherwise the taskbar
        // shows the raw app_id and a generic icon).
        let w = app("https://cloud.talesam.org/apps/notes", AppMode::App);
        let cls = derive_wm_class(&w);
        let expected = format!("br.com.biglinux.webapp.{}", desktop_file_id(&w.app_url));
        assert_eq!(cls, expected);
    }

    #[test]
    fn derive_wm_class_browser_mode_matches_chromium_app_id() {
        // Browser-mode StartupWMClass must equal what Chromium synthesizes for
        // the running window's Wayland app_id, otherwise GNOME Shell can't fall
        // back to the StartupWMClass lookup either.
        let w = app("https://web.whatsapp.com/", AppMode::Browser);
        let cls = derive_wm_class(&w);
        assert_eq!(cls, "brave-web.whatsapp.com__-Default");
    }

    #[test]
    fn canonical_browser_desktop_filename_matches_app_id_plus_extension() {
        // GNOME's primary window→.desktop lookup matches the running Wayland
        // app_id against the .desktop basename — these must agree exactly.
        let w = app("https://open.spotify.com/intl-pt/", AppMode::Browser);
        assert_eq!(
            canonical_browser_desktop_filename(&w),
            "brave-open.spotify.com__intl-pt_-Default.desktop"
        );
    }

    #[test]
    fn generate_desktop_entry_strips_line_injection_from_entry_fields() {
        let mut w = app("https://spotify.com/$HOME", AppMode::Browser);
        w.app_name = "Music\nX-Evil=1".to_string();
        w.comment = "Hello\nY-Evil=1".to_string();
        w.keywords = "music;\nZ-Evil=1".to_string();

        let entry = generate_desktop_entry(&w);
        let lines = entry.lines().collect::<Vec<_>>();

        assert!(!entry.contains("$HOME"));
        assert!(!lines.iter().any(|line| line == &"X-Evil=1"));
        assert!(!lines.iter().any(|line| line == &"Y-Evil=1"));
        assert!(!lines.iter().any(|line| line == &"Z-Evil=1"));
        assert!(lines
            .iter()
            .any(|line| line.starts_with("Name=MusicX-Evil=1")));
        assert!(lines
            .iter()
            .any(|line| line.starts_with("Comment=HelloY-Evil=1")));
        assert!(lines
            .iter()
            .any(|line| line.starts_with("Keywords=music;Z-Evil=1")));
    }
}
