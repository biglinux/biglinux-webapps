use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::config;

use super::super::context::WindowContext;

pub(super) fn install(context: &WindowContext) {
    let action = gtk::gio::SimpleAction::new("about", None);
    let window = context.window.clone();
    action.connect_activate(move |_, _| {
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
    context.window.add_action(&action);
}
