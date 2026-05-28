use adw::prelude::*;
use big_app_kit::desktop;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::config;

use super::super::context::WindowContext;

pub(super) fn install(context: &WindowContext) {
    let window = context.window.clone();
    desktop::install_action(&*context.window, "about", move || {
        let about = adw::AboutDialog::builder()
            .application_name(gettext("WebApps Manager"))
            .application_icon("big-webapps")
            .developer_name("BigLinux")
            .version(config::APP_VERSION)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/biglinux/biglinux-webapps")
            .build();
        about.present(Some(&*window));
    });
}
