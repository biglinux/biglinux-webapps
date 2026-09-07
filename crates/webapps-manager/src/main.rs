//! Binary entrypoint for the WebApps Manager: builds the libadwaita
//! `Application`, installs styles, and shows the manager window.

use libadwaita as adw;

use adw::prelude::*;
use webapps_core::config;
use webapps_manager::{style, window};

fn main() {
    // SAFETY: No threads or signal handlers have been started.
    unsafe { webapps_core::i18n::init() };
    init_logger();

    let app = adw::Application::builder()
        .application_id(config::APP_ID)
        .build();

    app.connect_startup(|_| {
        style::install_manager_css();
    });

    app.connect_activate(|app| {
        window::build(app);
    });

    app.run();
}

fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}
