mod browser_dialog;
mod favicon;
mod service;
mod template_gallery;
mod webapp_dialog;
mod webapp_row;
mod welcome_dialog;
mod window;

use libadwaita as adw;

use adw::prelude::*;
use webapps_core::config;

fn main() {
    env_logger::init();
    webapps_core::i18n::init();

    let app = adw::Application::builder()
        .application_id(config::APP_ID)
        .build();

    app.connect_activate(|app| {
        window::build(app);
    });

    app.run();
}
