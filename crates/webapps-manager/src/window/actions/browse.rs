#[allow(unused_imports)]
use adw::prelude::*;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::config;

use super::super::context::WindowContext;

pub(super) fn install(context: &WindowContext) {
    let browse_apps = gtk::gio::SimpleAction::new("browse-apps", None);
    browse_apps.connect_activate(|_, _| {
        let path = config::applications_dir();
        let _ = open::that(path);
    });
    context.window.add_action(&browse_apps);

    let browse_profiles = gtk::gio::SimpleAction::new("browse-profiles", None);
    browse_profiles.connect_activate(|_, _| {
        let path = config::profiles_dir();
        let _ = std::fs::create_dir_all(&path);
        let _ = open::that(path);
    });
    context.window.add_action(&browse_profiles);
}
