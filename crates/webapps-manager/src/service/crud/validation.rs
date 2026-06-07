use anyhow::Result;

use webapps_core::models::{ProfileKind, WebApp};

pub fn validate_custom_profile_name(profile_name: &str) -> Result<()> {
    ProfileKind::parse(profile_name).validate()
}

pub(super) fn validate_webapp(webapp: &WebApp) -> Result<()> {
    webapp.validate_domain()
}
