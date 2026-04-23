mod collection;
mod entry;
mod types;

pub use collection::WebAppCollection;
pub use entry::WebApp;
pub use types::{
    AppCategory, AppMode, BrowserId, CategoryList, DesktopFileName, ProfileKind,
    UrlValidationError, WebAppUrl,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_category_returns_first_segment() {
        let app = WebApp {
            app_categories: "Network;Office".to_string(),
            ..WebApp::default()
        };
        assert_eq!(app.main_category(), "Network");
    }

    #[test]
    fn main_category_falls_back_when_empty() {
        let app = WebApp {
            app_categories: String::new(),
            ..WebApp::default()
        };
        assert_eq!(app.main_category(), "Webapps");
    }

    #[test]
    fn main_category_single_without_trailing_semi() {
        let app = WebApp {
            app_categories: "Office".to_string(),
            ..WebApp::default()
        };
        assert_eq!(app.main_category(), "Office");
    }

    #[test]
    fn set_main_category_replaces_first() {
        let mut app = WebApp {
            app_categories: "Network;Office".to_string(),
            ..WebApp::default()
        };
        app.set_main_category("Graphics");
        assert!(
            app.app_categories.starts_with("Graphics"),
            "Expected Graphics as first category, got: {}",
            app.app_categories
        );
        assert!(
            app.app_categories.contains("Office"),
            "Office should be preserved"
        );
    }

    #[test]
    fn set_main_category_single() {
        let mut app = WebApp::default();
        app.set_main_category("Development");
        assert_eq!(app.main_category(), "Development");
    }

    #[test]
    fn set_main_category_ignores_empty() {
        let mut app = WebApp::default();
        let original = app.app_categories.clone();
        app.set_main_category("");
        assert_eq!(app.app_categories, original);
    }

    #[test]
    fn categorized_groups_by_category() {
        let col = WebAppCollection {
            webapps: vec![
                WebApp {
                    app_name: "A".into(),
                    app_categories: "Network".into(),
                    ..WebApp::default()
                },
                WebApp {
                    app_name: "B".into(),
                    app_categories: "Network;Office".into(),
                    ..WebApp::default()
                },
                WebApp {
                    app_name: "C".into(),
                    app_categories: "Office".into(),
                    ..WebApp::default()
                },
            ],
        };
        let cats = col.categorized(None);
        assert_eq!(cats["Network"].len(), 2);
        assert_eq!(cats["Office"].len(), 2);
    }

    #[test]
    fn categorized_filters_by_name() {
        let col = WebAppCollection {
            webapps: vec![
                WebApp {
                    app_name: "Spotify".into(),
                    app_url: "https://spotify.com".into(),
                    app_categories: "Network".into(),
                    ..WebApp::default()
                },
                WebApp {
                    app_name: "YouTube".into(),
                    app_url: "https://youtube.com".into(),
                    app_categories: "Network".into(),
                    ..WebApp::default()
                },
            ],
        };
        let cats = col.categorized(Some("spot"));
        assert_eq!(cats["Network"].len(), 1);
        assert_eq!(cats["Network"][0].app_name, "Spotify");
    }

    #[test]
    fn profile_kind_detects_custom_profiles() {
        let app = WebApp {
            app_profile: "Work".to_string(),
            ..WebApp::default()
        };

        assert_eq!(app.profile_kind(), ProfileKind::Custom("Work".to_string()));
        assert!(app.has_custom_profile());
    }

    #[test]
    fn browser_id_detects_viewer_mode() {
        let app = WebApp {
            browser: BrowserId::VIEWER.to_string(),
            ..WebApp::default()
        };

        assert!(app.browser_id().is_viewer());
    }

    #[test]
    fn category_list_serializes_for_desktop_entries() {
        let categories = CategoryList::parse("Network;Office");
        assert_eq!(categories.to_desktop_string(), "Network;Office;");
    }

    #[test]
    fn category_list_uses_typed_categories() {
        let categories = CategoryList::parse("Network;WeirdCategory");
        assert_eq!(
            categories.iter().collect::<Vec<_>>(),
            vec!["Network", "WeirdCategory"]
        );
        assert!(categories.validate().is_ok());
    }

    #[test]
    fn browser_id_validation_rejects_nested_paths() {
        assert!(BrowserId::from("../evil").validate().is_err());
    }

    #[test]
    fn desktop_file_name_validation_requires_extension() {
        assert!(DesktopFileName::parse("test.txt")
            .expect("desktop file should parse")
            .validate()
            .is_err());
    }

    #[test]
    fn profile_kind_validation_rejects_absolute_paths() {
        assert!(ProfileKind::parse("/tmp/profile").validate().is_err());
    }

    #[test]
    fn webapp_domain_validation_uses_typed_fields() {
        let app = WebApp {
            browser: "../bad".to_string(),
            ..WebApp::default()
        };

        assert!(app.validate_domain().is_err());
    }

    #[test]
    fn webapp_url_normalizes_missing_scheme() {
        let url = WebAppUrl::parse("example.com").expect("url should normalize");
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn webapp_url_rejects_invalid_urls() {
        assert_eq!(
            WebAppUrl::parse("http://[::1").unwrap_err(),
            UrlValidationError::Invalid
        );
    }
}
