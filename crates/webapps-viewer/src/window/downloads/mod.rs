//! Download handling for the WebApp viewer window: wires WebKit download
//! signals to the UI.

mod connect;

use gettextrs::gettext;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

use crate::platform::file_dialogs::FilePicker;

pub(super) use connect::connect_download_handlers;

pub(super) fn handle_download(window: &adw::ApplicationWindow, download: &webkit::Download) {
    connect_download_completion(
        download,
        clone!(
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
        ),
    );

    let weak_window = window.downgrade();
    download.connect_decide_destination(move |download, suggested| {
        let Some(window) = weak_window.upgrade() else {
            download.cancel();
            return true;
        };
        let download = download.clone();
        FilePicker::save_file(gettext("Save File"))
            .initial_name(suggested)
            .save_result(&window, move |result| match result {
                Ok(Some(path)) => download.set_destination(&path.to_string_lossy()),
                _ => download.cancel(),
            });
        true
    });
}

fn connect_download_completion(
    download: &webkit::Download,
    on_success: impl Fn(&webkit::Download) + 'static,
) {
    let failed = std::rc::Rc::new(std::cell::Cell::new(false));
    let failure = failed.clone();
    download.connect_failed(move |_, _| failure.set(true));
    download.connect_finished(move |download| {
        if !failed.get() && download.destination().is_some() {
            on_success(download);
        }
    });
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        io::{Read, Write},
        net::TcpListener,
        rc::Rc,
        time::{Duration, Instant},
    };

    #[test]
    #[ignore = "Requires an isolated GTK display and WebKit network session"]
    fn downloads_wait_for_destination_and_cancel_without_success() {
        gtk::init().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/report", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut request = [0; 4096];
                let _ = socket.read(&mut request);
                let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=report.txt\r\nContent-Length: 7\r\nConnection: close\r\n\r\ncontent");
            }
        });
        let root = tempfile::tempdir().unwrap();
        let session = webkit::NetworkSession::new_ephemeral();
        let success = Rc::new(Cell::new(0));
        for cancel in [false, true] {
            let destination = root.path().join(if cancel {
                "cancel.txt"
            } else {
                "report with spaces.txt"
            });
            let download = session.download_uri(&url).unwrap();
            let succeeded = success.clone();
            connect_download_completion(&download, move |_| succeeded.set(succeeded.get() + 1));
            let finished = Rc::new(Cell::new(false));
            let done = finished.clone();
            download.connect_finished(move |_| done.set(true));
            let selected = Rc::new(Cell::new(false));
            let chooser = selected.clone();
            let path = destination.clone();
            download.connect_decide_destination(move |download, suggested| {
                assert_eq!(suggested, "report.txt");
                let download = download.clone();
                let path = path.clone();
                let chooser = chooser.clone();
                glib::timeout_add_local_once(Duration::from_millis(50), move || {
                    assert!(
                        !path.exists(),
                        "Transfer must wait for the destination chooser"
                    );
                    chooser.set(true);
                    if cancel {
                        download.cancel();
                    } else {
                        download.set_destination(&path.to_string_lossy());
                    }
                });
                true
            });
            let deadline = Instant::now() + Duration::from_secs(10);
            while !finished.get() {
                assert!(Instant::now() < deadline, "Download timed out");
                while glib::MainContext::default().pending() {
                    glib::MainContext::default().iteration(false);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            assert!(selected.get());
            if cancel {
                assert!(!destination.exists());
            } else {
                assert_eq!(std::fs::read(destination).unwrap(), b"content");
            }
        }
        assert_eq!(success.get(), 1, "Cancellation must not notify success");
        server.join().unwrap();
    }
}
