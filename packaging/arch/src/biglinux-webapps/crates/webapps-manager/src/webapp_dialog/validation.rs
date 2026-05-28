use webapps_core::models::{UrlValidationError, WebApp, WebAppUrl};

use crate::service;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SaveValidationError {
    EmptyName,
    EmptyUrl,
    InvalidUrl,
    InvalidProfile(String),
}

pub(super) fn validate_for_save(webapp: &WebApp) -> Result<WebApp, SaveValidationError> {
    if webapp.app_name.trim().is_empty() {
        return Err(SaveValidationError::EmptyName);
    }

    let mut normalized = webapp.clone();
    normalized.app_url = WebAppUrl::parse(&webapp.app_url)
        .map_err(map_url_error)?
        .into_string();

    service::validate_custom_profile_name(&normalized.app_profile)
        .map_err(|err| SaveValidationError::InvalidProfile(err.to_string()))?;

    Ok(normalized)
}

pub(super) fn should_auto_detect(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < 4 {
        return false;
    }
    // Schemed URLs always count even without a dot — covers `http://localhost:8080`
    // and similar internal addresses that broke the previous "must contain a dot"
    // heuristic.
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
    {
        return true;
    }
    trimmed.contains('.')
}

fn map_url_error(error: UrlValidationError) -> SaveValidationError {
    match error {
        UrlValidationError::Empty => SaveValidationError::EmptyUrl,
        UrlValidationError::Invalid => SaveValidationError::InvalidUrl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use webapps_core::models::WebApp;

    #[test]
    fn validate_for_save_rejects_empty_name() {
        let app = WebApp {
            app_url: "https://example.com".into(),
            ..WebApp::default()
        };
        assert!(matches!(
            validate_for_save(&app),
            Err(SaveValidationError::EmptyName)
        ));
    }

    #[test]
    fn validate_for_save_normalizes_scheme() {
        let app = WebApp {
            app_name: "Example".into(),
            app_url: "example.com".into(),
            ..WebApp::default()
        };

        let validated = validate_for_save(&app).expect("validation should pass");
        assert_eq!(validated.app_url, "https://example.com");
    }

    #[test]
    fn validate_for_save_rejects_invalid_url() {
        let app = WebApp {
            app_name: "Example".into(),
            app_url: "http://[::1".into(),
            ..WebApp::default()
        };
        assert!(matches!(
            validate_for_save(&app),
            Err(SaveValidationError::InvalidUrl)
        ));
    }

    #[test]
    fn should_auto_detect_requires_hostname_like_text() {
        assert!(should_auto_detect("example.com"));
        assert!(!should_auto_detect("ab"));
        // Bare hostnames without a scheme still don't trigger detection — too noisy.
        assert!(!should_auto_detect("localhost"));
    }

    #[test]
    fn should_auto_detect_accepts_schemed_local_urls() {
        assert!(should_auto_detect("http://localhost:8080"));
        assert!(should_auto_detect("https://intranet/dash"));
        assert!(should_auto_detect("file:///srv/app"));
    }

    #[test]
    fn should_auto_detect_ignores_leading_whitespace() {
        assert!(should_auto_detect("  example.com  "));
        assert!(!should_auto_detect("    "));
    }
}
