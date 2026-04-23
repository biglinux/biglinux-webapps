use libadwaita as adw;

use adw::prelude::*;
use webapps_core::config;
use webapps_manager::{style, window};

fn main() {
    init_logger();
    webapps_core::i18n::init();

    let app = adw::Application::builder()
        .application_id(config::APP_ID)
        .build();

    app.connect_startup(|_| {
        style::load_css();
    });

    app.connect_activate(|app| {
        window::build(app);
    });

    app.run();
}

fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}
