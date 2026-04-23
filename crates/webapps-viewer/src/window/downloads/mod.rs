mod connect;

use gettextrs::gettext;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) use connect::connect_download_handlers;

pub(super) fn handle_download(window: &adw::ApplicationWindow, download: &webkit::Download) {
    let suggested = download
        .response()
        .and_then(|r| r.suggested_filename())
        .map(|g| g.to_string())
        .unwrap_or_else(|| "download".into());

    download.connect_finished(clone!(
        #[weak]
        window,
        move |dl| {
            let dest = dl.destination().map(|g| g.to_string()).unwrap_or_default();
            let fname = std::path::Path::new(&dest)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "File".into());
            let notif = gtk::gio::Notification::new(&gettext("Download Complete"));
            notif.set_body(Some(&fname));
            if let Some(app) = window.application() {
                app.send_notification(None, &notif);
            }
        }
    ));

    let dialog = gtk::FileDialog::builder()
        .title(gettext("Save File"))
        .initial_name(&suggested)
        .build();

    dialog.save(
        Some(window),
        gtk::gio::Cancellable::NONE,
        clone!(
            #[strong]
            download,
            move |result: Result<gtk::gio::File, glib::Error>| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let uri = format!("file://{}", path.display());
                        download.set_destination(&uri);
                    }
                } else {
                    download.cancel();
                }
            }
        ),
    );
}

pub(super) fn show_notification(
    window: &adw::ApplicationWindow,
    notification: &webkit::Notification,
) {
    let title = notification
        .title()
        .map(|g| g.to_string())
        .unwrap_or_default();
    let body = notification
        .body()
        .map(|g| g.to_string())
        .unwrap_or_default();

    let notif = gtk::gio::Notification::new(&title);
    notif.set_body(Some(&body));

    if let Some(app) = window.application() {
        app.send_notification(None, &notif);
    }
}
