#[allow(unused_imports)]
use adw::prelude::*;
use big_app_kit::desktop;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::config;

use super::super::context::WindowContext;

pub(super) fn install(context: &WindowContext) {
    desktop::install_action(&*context.window, "browse-apps", || {
        let path = config::applications_dir();
        let _ = open::that(path);
    });

    desktop::install_action(&*context.window, "browse-profiles", || {
        let path = config::profiles_dir();
        let _ = std::fs::create_dir_all(&path);
        let _ = open::that(path);
    });
}
