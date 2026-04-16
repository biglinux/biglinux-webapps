use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

#[allow(unused_imports)]
use adw::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;
use webkit6::prelude::*;

use webapps_core::config;

/// Build and wire up the viewer window
pub fn build(
    app: &adw::Application,
    url: &str,
    name: &str,
    icon: &str,
    app_id: &str,
) -> adw::ApplicationWindow {
    // --- profile isolation via NetworkSession ---
    let data_dir = config::data_dir().join(app_id);
    let cache_dir = config::cache_dir().join(app_id);
    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(&cache_dir).ok();

    let session = webkit::NetworkSession::new(
        Some(data_dir.to_str().unwrap_or_default()),
        Some(cache_dir.to_str().unwrap_or_default()),
    );

    // persist cookies in WebKitGTK format + allow third-party (auth flows)
    // NOTE: existing "Cookies" file = legacy Chromium format, not used by WebKitGTK6
    // ITP disabled → allow cross-site cookies for login persistence
    // (YouTube, Spotify etc use third-party auth flows that break with ITP)
    session.set_itp_enabled(false);
    if let Some(cm) = session.cookie_manager() {
        let cookie_db = data_dir.join("webkit-cookies.db");
        cm.set_persistent_storage(
            cookie_db.to_str().unwrap_or_default(),
            webkit::CookiePersistentStorage::Sqlite,
        );
        cm.set_accept_policy(webkit::CookieAcceptPolicy::Always);
    }

    // --- webview ---
    let webview = webkit::WebView::builder().network_session(&session).build();

    configure_settings(&webview);
    inject_resize_block(&webview);
    webview.load_uri(url);
    webview.set_vexpand(true);
    webview.set_hexpand(true);

    // --- headerbar ---
    let title_widget = adw::WindowTitle::new(name, url);

    let back_btn = gtk::Button::from_icon_name("go-previous-symbolic");
    back_btn.set_sensitive(false);
    back_btn.set_tooltip_text(Some(&gettext("Back")));
    back_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Back"))]);

    let fwd_btn = gtk::Button::from_icon_name("go-next-symbolic");
    fwd_btn.set_sensitive(false);
    fwd_btn.set_tooltip_text(Some(&gettext("Forward")));
    fwd_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Forward"))]);

    let reload_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
    reload_btn.set_tooltip_text(Some(&gettext("Reload")));
    reload_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Reload"))]);

    let fullscreen_btn = gtk::Button::from_icon_name("view-fullscreen-symbolic");
    fullscreen_btn.set_tooltip_text(Some(&gettext("Fullscreen")));
    fullscreen_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Fullscreen"))]);

    let menu_btn = build_menu_button();

    let header = adw::HeaderBar::builder()
        .title_widget(&title_widget)
        .build();
    header.pack_start(&back_btn);
    header.pack_start(&fwd_btn);
    header.pack_start(&reload_btn);
    header.pack_end(&menu_btn);
    header.pack_end(&fullscreen_btn);

    // --- URL bar (hidden by default, toggled via Ctrl+L) ---
    let url_entry = gtk::Entry::builder()
        .placeholder_text(gettext("Enter URL…"))
        .hexpand(true)
        .build();
    url_entry.set_text(url);

    let url_bar = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .reveal_child(false)
        .child(&url_entry)
        .build();

    // Enter → navigate + hide bar
    url_entry.connect_activate(clone!(
        #[weak]
        webview,
        #[weak]
        url_bar,
        move |entry| {
            let mut text = entry.text().to_string();
            if !text.is_empty() {
                if !text.starts_with("http://")
                    && !text.starts_with("https://")
                    && !text.starts_with("file://")
                {
                    text = format!("https://{text}");
                }
                webview.load_uri(&text);
            }
            url_bar.set_reveal_child(false);
        }
    ));

    // Escape while focused → hide bar
    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.connect_key_pressed(clone!(
        #[weak]
        url_bar,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, key, _, _| {
            if key == gdk4::Key::Escape {
                url_bar.set_reveal_child(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    ));
    url_entry.add_controller(key_ctrl);

    // --- ToolbarView: revealable header in fullscreen ---
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.add_top_bar(&url_bar);
    toolbar.set_content(Some(&webview));
    toolbar.set_reveal_top_bars(true);

    // --- window ---
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(name)
        .default_width(1024)
        .default_height(720)
        .content(&toolbar)
        .build();

    // set window icon
    if !icon.is_empty() {
        // GTK4: set default icon name for the window
        gtk::Window::set_default_icon_name(icon);
    }

    let config_path = config::config_dir().join(format!("{app_id}.json"));

    // load saved geometry
    load_geometry(&window, &config_path);

    // --- wire navigation ---
    back_btn.connect_clicked(clone!(
        #[weak]
        webview,
        move |_| {
            webview.go_back();
        }
    ));
    fwd_btn.connect_clicked(clone!(
        #[weak]
        webview,
        move |_| {
            webview.go_forward();
        }
    ));
    reload_btn.connect_clicked(clone!(
        #[weak]
        webview,
        move |_| {
            webview.reload();
        }
    ));

    // --- title changed ---
    webview.connect_title_notify(clone!(
        #[weak]
        title_widget,
        #[weak]
        window,
        move |wv| {
            if let Some(title) = wv.title() {
                let t = title.to_string();
                if !t.is_empty() {
                    title_widget.set_title(&t);
                    window.set_title(Some(&t));
                }
            }
        }
    ));

    // --- URI changed → update subtitle + nav buttons ---
    webview.connect_uri_notify(clone!(
        #[weak]
        title_widget,
        #[weak]
        back_btn,
        #[weak]
        fwd_btn,
        move |wv| {
            if let Some(uri) = wv.uri() {
                title_widget.set_subtitle(&uri);
            }
            back_btn.set_sensitive(wv.can_go_back());
            fwd_btn.set_sensitive(wv.can_go_forward());
        }
    ));

    // --- load changed → update nav state ---
    webview.connect_load_changed(clone!(
        #[weak]
        back_btn,
        #[weak]
        fwd_btn,
        move |wv, _event| {
            back_btn.set_sensitive(wv.can_go_back());
            fwd_btn.set_sensitive(wv.can_go_forward());
        }
    ));

    // --- fullscreen ---
    let is_fullscreen = Rc::new(Cell::new(false));

    webview.connect_enter_fullscreen(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        #[upgrade_or]
        false,
        move |_| {
            is_fullscreen.set(true);
            toolbar.set_reveal_top_bars(false);
            window.fullscreen();
            true
        }
    ));

    webview.connect_leave_fullscreen(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        #[upgrade_or]
        false,
        move |_| {
            is_fullscreen.set(false);
            toolbar.set_reveal_top_bars(true);
            window.unfullscreen();
            true
        }
    ));

    fullscreen_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_| {
            if is_fullscreen.get() {
                is_fullscreen.set(false);
                toolbar.set_reveal_top_bars(true);
                window.unfullscreen();
            } else {
                is_fullscreen.set(true);
                toolbar.set_reveal_top_bars(false);
                window.fullscreen();
            }
        }
    ));

    // --- downloads ---
    session.connect_download_started(clone!(
        #[weak]
        window,
        move |_session, download| {
            handle_download(&window, download);
        }
    ));

    // --- notifications ---
    webview.connect_show_notification(clone!(
        #[weak]
        window,
        #[upgrade_or]
        false,
        move |_wv, notification| {
            show_notification(&window, notification);
            true
        }
    ));

    // --- permission requests ---
    // auto-grant all → webapp UX expectation; webapps are user-trusted apps
    // TODO: consider per-webapp permission preferences for untrusted sites
    webview.connect_permission_request(|_wv, request| {
        log::info!("Permission auto-granted: {:?}", request.type_());
        request.allow();
        true
    });

    // --- new window requests → open in same view ---
    webview.connect_create(|wv, action| {
        if let Some(request) = action.request() {
            if let Some(uri) = request.uri() {
                wv.load_uri(&uri);
            }
        }
        None
    });

    // --- context menu ---
    setup_context_menu(&webview);

    // --- keyboard shortcuts ---
    setup_shortcuts(
        &window,
        &webview,
        &toolbar,
        &is_fullscreen,
        &url_bar,
        &url_entry,
    );

    // --- save geometry on close ---
    window.connect_close_request(clone!(
        #[strong]
        config_path,
        move |win| {
            save_geometry(win, &config_path);
            glib::Propagation::Proceed
        }
    ));

    // --- fullscreen headerbar reveal on mouse hover ---
    setup_fullscreen_reveal(&toolbar, &is_fullscreen);

    window
}

/// Configure WebView settings for webapp usage
fn configure_settings(webview: &webkit::WebView) {
    if let Some(s) = WebViewExt::settings(webview) {
        s.set_enable_javascript(true);
        s.set_javascript_can_access_clipboard(true);
        s.set_enable_developer_extras(true);
        s.set_media_playback_requires_user_gesture(false);
        s.set_enable_media_stream(true);
        s.set_enable_mediasource(true);
        s.set_enable_encrypted_media(true);
        s.set_enable_smooth_scrolling(true);
        s.set_enable_back_forward_navigation_gestures(true);
        // spoof Chrome UA → sites like Spotify/Teams reject unknown browsers
        s.set_user_agent(Some(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        ));
    }
}

/// Inject JS to block web content from resizing/moving the window
fn inject_resize_block(webview: &webkit::WebView) {
    let ucm = webview
        .user_content_manager()
        .expect("WebView must have UserContentManager");
    let script = webkit::UserScript::new(
        concat!(
            "window.resizeTo=function(){};",
            "window.resizeBy=function(){};",
            "window.moveTo=function(){};",
            "window.moveBy=function(){};",
        ),
        webkit::UserContentInjectedFrames::AllFrames,
        webkit::UserScriptInjectionTime::Start,
        &[],
        &[],
    );
    ucm.add_script(&script);
}

/// Build hamburger menu
fn build_menu_button() -> gtk::MenuButton {
    let menu = gio::Menu::new();
    menu.append(Some(&gettext("Zoom In")), Some("win.zoom-in"));
    menu.append(Some(&gettext("Zoom Out")), Some("win.zoom-out"));
    menu.append(Some(&gettext("Reset Zoom")), Some("win.zoom-reset"));
    menu.append(Some(&gettext("Developer Tools")), Some("win.devtools"));

    let btn = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text(gettext("Menu"))
        .build();
    btn.update_property(&[gtk::accessible::Property::Label(&gettext("Menu"))]);
    btn
}

/// Custom context menu — add "Open in Browser" action for links
fn setup_context_menu(webview: &webkit::WebView) {
    webview.connect_context_menu(|wv, menu, hit| {
        // add "Open in Browser" for links
        if let Some(uri) = hit.link_uri() {
            let uri_str = uri.to_string();
            // add separator + custom item
            menu.append(&webkit::ContextMenuItem::new_separator());

            let action = gio::SimpleAction::new("open-link-browser", None);
            let u = uri_str.clone();
            action.connect_activate(move |_, _| {
                let _ = gio::AppInfo::launch_default_for_uri(&u, gio::AppLaunchContext::NONE);
            });
            wv.insert_action_group(
                "ctx",
                Some(&{
                    let group = gio::SimpleActionGroup::new();
                    group.add_action(&action);
                    group
                }),
            );
            let item = webkit::ContextMenuItem::from_gaction(
                &action,
                &gettext("Open Link in Browser"),
                None,
            );
            menu.append(&item);
        }

        false
    });
}

/// Handle download: prompt user for save location
fn handle_download(window: &adw::ApplicationWindow, download: &webkit::Download) {
    let suggested = download
        .response()
        .and_then(|r| r.suggested_filename())
        .map(|g| g.to_string())
        .unwrap_or_else(|| "download".into());

    // notify on completion
    download.connect_finished(clone!(
        #[weak]
        window,
        move |dl| {
            let dest = dl.destination().map(|g| g.to_string()).unwrap_or_default();
            let fname = std::path::Path::new(&dest)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "File".into());
            let notif = gio::Notification::new(&gettext("Download Complete"));
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
        gio::Cancellable::NONE,
        clone!(
            #[strong]
            download,
            move |result: Result<gio::File, glib::Error>| {
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

/// Bridge webkit notification → system notification
fn show_notification(window: &adw::ApplicationWindow, notification: &webkit::Notification) {
    let title = notification
        .title()
        .map(|g| g.to_string())
        .unwrap_or_default();
    let body = notification
        .body()
        .map(|g| g.to_string())
        .unwrap_or_default();

    let notif = gio::Notification::new(&title);
    notif.set_body(Some(&body));

    if let Some(app) = window.application() {
        app.send_notification(None, &notif);
    }
}

/// Setup keyboard shortcuts via GActions
fn setup_shortcuts(
    window: &adw::ApplicationWindow,
    webview: &webkit::WebView,
    toolbar: &adw::ToolbarView,
    is_fullscreen: &Rc<Cell<bool>>,
    url_bar: &gtk::Revealer,
    url_entry: &gtk::Entry,
) {
    let app = window
        .application()
        .expect("Window must belong to an Application");

    // F11 → fullscreen toggle
    let action_fs = gio::SimpleAction::new("toggle-fullscreen", None);
    action_fs.connect_activate(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_, _| {
            if is_fullscreen.get() {
                is_fullscreen.set(false);
                toolbar.set_reveal_top_bars(true);
                window.unfullscreen();
            } else {
                is_fullscreen.set(true);
                toolbar.set_reveal_top_bars(false);
                window.fullscreen();
            }
        }
    ));
    window.add_action(&action_fs);
    app.set_accels_for_action("win.toggle-fullscreen", &["F11"]);

    // Escape → exit fullscreen
    let action_esc = gio::SimpleAction::new("exit-fullscreen", None);
    action_esc.connect_activate(clone!(
        #[weak]
        window,
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_, _| {
            if is_fullscreen.get() {
                is_fullscreen.set(false);
                toolbar.set_reveal_top_bars(true);
                window.unfullscreen();
            }
        }
    ));
    window.add_action(&action_esc);
    app.set_accels_for_action("win.exit-fullscreen", &["Escape"]);

    // Ctrl+R / F5 → reload
    let action_reload = gio::SimpleAction::new("reload", None);
    action_reload.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.reload();
        }
    ));
    window.add_action(&action_reload);
    app.set_accels_for_action("win.reload", &["<Ctrl>r", "F5"]);

    // Alt+Left → back
    let action_back = gio::SimpleAction::new("go-back", None);
    action_back.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.go_back();
        }
    ));
    window.add_action(&action_back);
    app.set_accels_for_action("win.go-back", &["<Alt>Left"]);

    // Alt+Right → forward
    let action_fwd = gio::SimpleAction::new("go-forward", None);
    action_fwd.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.go_forward();
        }
    ));
    window.add_action(&action_fwd);
    app.set_accels_for_action("win.go-forward", &["<Alt>Right"]);

    // Ctrl+W → close
    let action_close = gio::SimpleAction::new("close-window", None);
    action_close.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            window.close();
        }
    ));
    window.add_action(&action_close);
    app.set_accels_for_action("win.close-window", &["<Ctrl>w"]);

    // Zoom in/out/reset
    let action_zin = gio::SimpleAction::new("zoom-in", None);
    action_zin.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            let level = webview.zoom_level();
            webview.set_zoom_level(level + 0.1);
        }
    ));
    window.add_action(&action_zin);
    app.set_accels_for_action("win.zoom-in", &["<Ctrl>plus", "<Ctrl>equal"]);

    let action_zout = gio::SimpleAction::new("zoom-out", None);
    action_zout.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            let level = webview.zoom_level();
            webview.set_zoom_level((level - 0.1).max(0.3));
        }
    ));
    window.add_action(&action_zout);
    app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus"]);

    let action_zreset = gio::SimpleAction::new("zoom-reset", None);
    action_zreset.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            webview.set_zoom_level(1.0);
        }
    ));
    window.add_action(&action_zreset);
    app.set_accels_for_action("win.zoom-reset", &["<Ctrl>0"]);

    // Ctrl+Shift+I → devtools
    let action_dev = gio::SimpleAction::new("devtools", None);
    action_dev.connect_activate(clone!(
        #[weak]
        webview,
        move |_, _| {
            if let Some(inspector) = webview.inspector() {
                inspector.show();
            }
        }
    ));
    window.add_action(&action_dev);
    app.set_accels_for_action("win.devtools", &["<Ctrl><Shift>i"]);

    // Ctrl+L → focus URL bar
    let action_url = gio::SimpleAction::new("focus-url", None);
    action_url.connect_activate(clone!(
        #[weak]
        url_bar,
        #[weak]
        url_entry,
        #[weak]
        webview,
        move |_, _| {
            url_bar.set_reveal_child(true);
            if let Some(uri) = webview.uri() {
                url_entry.set_text(&uri);
            }
            url_entry.grab_focus();
            url_entry.select_region(0, -1);
        }
    ));
    window.add_action(&action_url);
    app.set_accels_for_action("win.focus-url", &["<Ctrl>l"]);
}

/// Reveal headerbar when mouse near top edge in fullscreen
fn setup_fullscreen_reveal(toolbar: &adw::ToolbarView, is_fullscreen: &Rc<Cell<bool>>) {
    // ToolbarView with Adw handles reveal via its own policy
    // set top-bar-style to raised for visual separation
    toolbar.set_top_bar_style(adw::ToolbarStyle::Raised);

    // use motion controller to reveal header on hover in fullscreen
    let motion = gtk::EventControllerMotion::new();
    motion.connect_motion(clone!(
        #[weak]
        toolbar,
        #[strong]
        is_fullscreen,
        move |_, _x, y| {
            if is_fullscreen.get() {
                // reveal when mouse within 50px of top edge
                toolbar.set_reveal_top_bars(y < 50.0);
            }
        }
    ));
    toolbar.add_controller(motion);
}

/// Load window geometry from JSON config
fn load_geometry(window: &adw::ApplicationWindow, config_path: &PathBuf) {
    let data = match std::fs::read_to_string(config_path) {
        Ok(d) => d,
        Err(_) => return, // no config yet → use defaults
    };
    match serde_json::from_str::<serde_json::Value>(&data) {
        Ok(geo) => {
            let w = geo.get("width").and_then(|v| v.as_i64()).unwrap_or(1024) as i32;
            let h = geo.get("height").and_then(|v| v.as_i64()).unwrap_or(720) as i32;
            window.set_default_size(w, h);

            if geo
                .get("maximized")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                window.maximize();
            }
        }
        Err(e) => log::warn!("Geometry parse fail: {e}"),
    }
}

/// Save window geometry to JSON config
fn save_geometry(window: &adw::ApplicationWindow, config_path: &PathBuf) {
    if window.is_fullscreen() {
        return;
    }

    let (w, h) = window.default_size();
    let geo = serde_json::json!({
        "width": if w > 0 { w } else { 1024 },
        "height": if h > 0 { h } else { 720 },
        "maximized": window.is_maximized(),
    });

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Err(e) = std::fs::write(config_path, geo.to_string()) {
        log::error!("Failed to save geometry: {e}");
    }
}
