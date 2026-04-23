mod browser;
mod crud;
mod icons;
mod io;
mod migration;
mod repository;
mod welcome;

pub use browser::detect_browsers;
pub use crud::{
    create_webapp, delete_all_webapps, delete_webapp, generate_app_file, profile_shared,
    update_webapp, validate_custom_profile_name,
};
pub use icons::resolve_icon_path;
pub use io::{export_webapps, import_webapps};
pub use migration::migrate_legacy_desktops;
pub use repository::{load_webapps, save_webapps};
pub use welcome::{mark_welcome_shown, should_show_welcome};

pub(crate) use repository::webapps_json_path;
