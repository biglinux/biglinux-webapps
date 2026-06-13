//! Manager service layer: browser detection, CRUD on webapps, icon handling,
//! IO persistence, migrations, repository access, and the welcome flow.

mod browser;
mod browser_url;
mod crud;
mod icons;
mod io;
mod migration;
mod repository;
mod welcome;

pub use browser::detect_browsers;
pub use browser_url::display_url;
pub use crud::{
    create_webapp, delete_all_webapps, delete_webapp, generate_app_file, profile_shared,
    update_webapp, validate_custom_profile_name,
};
pub use icons::resolve_icon_path;
pub use io::{export_webapps, import_webapps};
pub use migration::{
    migrate_browser_desktop_filenames, migrate_legacy_desktops, persist_existing_icons,
    regenerate_app_mode_desktops, regenerate_browser_mode_desktops,
};
pub use repository::{load_webapps, save_webapps};
pub use welcome::{mark_welcome_shown, should_show_welcome};

pub(crate) use repository::webapps_json_path;
