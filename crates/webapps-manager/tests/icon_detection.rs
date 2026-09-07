use serial_test::serial;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[path = "icon_detection/fallback.rs"]
mod fallback;

struct Site {
    address: String,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Site {
    fn new(routes: HashMap<&'static str, (String, Vec<u8>)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let thread = std::thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                            .unwrap();
                        let mut request = [0; 4096];
                        let count = stream.read(&mut request).unwrap_or(0);
                        let request = String::from_utf8_lossy(&request[..count]);
                        let path = request.split_whitespace().nth(1).unwrap_or("/");
                        let (headers, bytes) = routes.get(path).cloned().unwrap_or((
                            "404 Not Found\r\nContent-Type: text/plain".into(),
                            Vec::new(),
                        ));
                        let response = format!(
                            "HTTP/1.1 {headers}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            bytes.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.write_all(&bytes);
                    }
                    Err(_) => std::thread::sleep(std::time::Duration::from_millis(2)),
                }
            }
        });
        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for Site {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().unwrap();
    }
}
fn png(side: i32, color: u32) -> Vec<u8> {
    let icon = gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, side, side).unwrap();
    icon.fill(color);
    icon.save_to_bufferv("png", &[]).unwrap()
}
fn response(mime: &str, bytes: impl Into<Vec<u8>>) -> (String, Vec<u8>) {
    (format!("200 OK\r\nContent-Type: {mime}"), bytes.into())
}

#[test]
#[serial]
fn redirects_manifest_and_real_dimensions_select_native_high_resolution() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CACHE_HOME", cache.path());
    let site = Site::new(HashMap::from([
        ("/", ("302 Found\r\nLocation: /web/index.html".into(), vec![])),
        ("/web/index.html", response("text/html", br#"<title>Spotify fixture</title><base href="/assets/"><link rel="manifest" href="manifest"><link rel="icon" sizes="2048x2048" href="tiny.png">"#.to_vec())),
        ("/assets/manifest", ("302 Found\r\nLocation: /cdn/app.json".into(), vec![])),
        ("/cdn/app.json", response("application/json", br#"{"icons":[{"src":"large.png","sizes":"32x32"},{"src":"medium.png","sizes":"512x512"}]}"#.to_vec())),
        ("/assets/tiny.png", response("image/png", png(48, 0x0000ffff))),
        ("/cdn/large.png", response("image/png", png(1024, 0x00ff00ff))),
        ("/cdn/medium.png", response("image/png", png(512, 0xff0000ff))),
    ]));
    let info = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
    assert_eq!(info.title, "Spotify fixture");
    assert_eq!(info.icons.len(), 2);
    let best = gdk_pixbuf::Pixbuf::from_file(&info.icons[0].path).unwrap();
    assert_eq!(best.width(), 1024);
    assert_eq!(best.height(), 1024);
    assert!(!info.icons[0].path.to_string_lossy().contains("hires"));
    let second = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
    assert_ne!(info.icons[0].path, second.icons[0].path);
    assert!(info.icons[0].path.is_file());
}

#[test]
#[serial]
fn insufficient_or_corrupt_icons_produce_no_candidates() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CACHE_HOME", cache.path());
    let site = Site::new(HashMap::from([
        ("/", response("text/html", br#"<title>Only tiny</title><link rel="icon" sizes="512x512" href="/tiny.png"><link rel="icon" href="/broken.png">"#.to_vec())),
        ("/tiny.png", response("image/png", png(48, 0x00ff00ff))),
        ("/broken.png", response("image/png", b"broken image".to_vec())),
    ]));
    let info = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
    assert_eq!(info.title, "Only tiny");
    assert!(info.icons.is_empty());
}

#[test]
#[ignore = "Requires an isolated GTK display"]
fn gtk_detection_preserves_edits_filters_candidates_and_saves_the_selected_icon() {
    use gtk4::prelude::*;
    use libadwaita::prelude::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    use webapps_core::models::{AppMode, BrowserCollection, BrowserId, WebApp};
    let sandbox = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_DATA_HOME", sandbox.path().join("data"));
    std::env::set_var("XDG_CONFIG_HOME", sandbox.path().join("config"));
    std::env::set_var("XDG_CACHE_HOME", sandbox.path().join("cache"));
    libadwaita::init().unwrap();
    let site = Site::new(HashMap::from([
        ("/", response("text/html", br#"<title>Detected title</title><link rel="icon" href="/large.png"><link rel="icon" href="/tiny.png">"#.to_vec())),
        ("/large.png", response("image/png", png(1024, 0x11bb33ff))),
        ("/tiny.png", response("image/png", png(48, 0x0000ffff))),
        ("/fallback.png", response("image/png", png(64, 0xff1122ff))),
        ("/fallback", response("text/html", br#"<title>Fallback title</title><link rel="icon" href="/fallback.png">"#.to_vec())),
        ("/tiny-only", response("text/html", br#"<title>Tiny title</title><link rel="icon" href="/tiny.png">"#.to_vec())),
    ]));
    let window = libadwaita::Window::new();
    window.set_default_size(1000, 900);
    window.present();
    let saved = Rc::new(Cell::new(false));
    let completed = saved.clone();
    webapps_manager::webapp_dialog::show(
        &window,
        WebApp {
            app_name: "My custom title".into(),
            app_url: site.address.clone(),
            browser: BrowserId::VIEWER.into(),
            app_mode: AppMode::App,
            ..WebApp::default()
        },
        Rc::new(RefCell::new(BrowserCollection::default())),
        true,
        move |result| completed.set(result.saved),
    );
    let dialog = window.visible_dialog().unwrap();
    let all = widgets(dialog.upcast_ref());
    let name = all
        .iter()
        .find_map(|widget| {
            widget
                .clone()
                .downcast::<libadwaita::EntryRow>()
                .ok()
                .filter(|row| row.title() == "Name")
        })
        .unwrap();
    let url = all
        .iter()
        .find_map(|widget| {
            widget
                .clone()
                .downcast::<libadwaita::EntryRow>()
                .ok()
                .filter(|row| row.title() == "URL")
        })
        .unwrap();
    let detect = all
        .iter()
        .find_map(|widget| {
            widget
                .clone()
                .downcast::<gtk4::Button>()
                .ok()
                .filter(|button| button.label().as_deref() == Some("Detect"))
        })
        .unwrap();
    let save = all
        .iter()
        .find_map(|widget| {
            widget
                .clone()
                .downcast::<gtk4::Button>()
                .ok()
                .filter(|button| {
                    matches!(button.label().as_deref(), Some("Save" | "Create" | "Add"))
                })
        })
        .unwrap();
    let flow = all
        .iter()
        .find_map(|widget| widget.clone().downcast::<gtk4::FlowBox>().ok())
        .unwrap();
    detect.emit_clicked();
    assert!(!save.is_sensitive());
    name.set_text("Edited while searching");
    spin_until(|| save.is_sensitive());
    assert_eq!(name.text(), "Edited while searching");
    assert!(flow.child_at_index(0).is_some());
    assert!(
        flow.child_at_index(1).is_none(),
        "48 px icon must not be displayed"
    );
    let image = flow
        .child_at_index(0)
        .unwrap()
        .child()
        .unwrap()
        .downcast::<gtk4::Image>()
        .unwrap();
    assert!(image.tooltip_text().unwrap().contains("1024 × 1024"));
    let candidate = image.file().unwrap();
    url.set_text(&format!("{}/fallback", site.address));
    detect.emit_clicked();
    spin_until(|| save.is_sensitive());
    let fallback = flow
        .child_at_index(0)
        .unwrap()
        .child()
        .unwrap()
        .downcast::<gtk4::Image>()
        .unwrap();
    assert!(fallback.tooltip_text().unwrap().contains("64 × 64"));
    let stored = gdk_pixbuf::Pixbuf::from_file(fallback.file().unwrap()).unwrap();
    assert_eq!((stored.width(), stored.height()), (64, 64));
    url.set_text(&format!("{}/tiny-only", site.address));
    detect.emit_clicked();
    spin_until(|| save.is_sensitive());
    let label = flow
        .child_at_index(0)
        .unwrap()
        .child()
        .unwrap()
        .downcast::<gtk4::Label>()
        .unwrap();
    assert!(label.text().contains("minimum 64"));
    assert_eq!(name.text(), "Edited while searching");
    url.set_text(&site.address);
    detect.emit_clicked();
    spin_until(|| save.is_sensitive());
    save.emit_clicked();
    spin_until(|| saved.get());
    let collection = webapps_manager::service::try_load_webapps().unwrap();
    assert_eq!(collection.webapps.len(), 1);
    assert_eq!(collection.webapps[0].app_name, "Edited while searching");
    let stored = gdk_pixbuf::Pixbuf::from_file(&collection.webapps[0].app_icon).unwrap();
    assert_eq!((stored.width(), stored.height()), (1024, 1024));
    assert_ne!(collection.webapps[0].app_icon, candidate);
    assert!(std::path::Path::new(&collection.webapps[0].app_icon).is_file());
    window.close();
}

fn widgets(root: &gtk4::Widget) -> Vec<gtk4::Widget> {
    use gtk4::prelude::*;
    let mut result = vec![root.clone()];
    let mut child = root.first_child();
    while let Some(widget) = child {
        result.extend(widgets(&widget));
        child = widget.next_sibling();
    }
    result
}

fn spin_until(condition: impl Fn() -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !condition() {
        assert!(
            std::time::Instant::now() < deadline,
            "GTK operation timed out"
        );
        while glib::MainContext::default().pending() {
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
