//! CRUD operations for webapps: create/update/delete plus shared routines for
//! profile-file generation, cleanup, and custom-profile-name validation.

mod cleanup;
mod operations;
mod profile_files;
mod validation;

pub use operations::{create_webapp, delete_all_webapps, delete_webapp, update_webapp};
pub use profile_files::{generate_app_file, profile_shared};
pub use validation::validate_custom_profile_name;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_custom_profile_name_accepts_single_component() {
        assert!(validate_custom_profile_name("Work Profile").is_ok());
    }

    #[test]
    fn validate_custom_profile_name_rejects_parent_dir() {
        assert!(validate_custom_profile_name("../Documents").is_err());
    }

    #[test]
    fn validate_custom_profile_name_rejects_absolute_path() {
        assert!(validate_custom_profile_name("/tmp/profile").is_err());
    }

    #[test]
    fn validate_app_file_rejects_nested_path() {
        assert!(
            webapps_core::models::DesktopFileName::parse("../evil.desktop")
                .expect("desktop filename should parse")
                .validate()
                .is_err()
        );
    }
}
