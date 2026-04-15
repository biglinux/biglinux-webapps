mod window;

use clap::Parser;
use libadwaita as adw;

use adw::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "big-webapps-viewer", about = "BigLinux WebApp Viewer")]
struct Cli {
    /// URL to load
    #[arg(long)]
    url: String,

    /// Window title
    #[arg(long, default_value = "WebApp")]
    name: String,

    /// Icon name or path
    #[arg(long, default_value = "")]
    icon: String,

    /// Unique application ID for profile isolation
    #[arg(long)]
    app_id: String,

    /// Files to open via upload
    files: Vec<String>,
}

fn main() -> glib::ExitCode {
    env_logger::init();
    webapps_core::i18n::init();
    let cli = Cli::parse();

    let mut url = cli.url.clone();
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
        url = format!("https://{url}");
    }

    let app = adw::Application::builder()
        .application_id(&format!("br.com.biglinux.webapp.{}", cli.app_id))
        .build();

    let name = cli.name.clone();
    let icon = cli.icon.clone();
    let app_id = cli.app_id.clone();

    app.connect_activate(move |app| {
        let win = window::build(app, &url, &name, &icon, &app_id);
        win.present();
    });

    // run with empty args — clap already consumed CLI args, prevent GLib re-parsing
    app.run_with_args::<&str>(&[])
}
