use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::gio;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use webapps_core::models::{AppMode, BrowserCollection, WebApp};
use webapps_core::templates::build_default_registry;

use crate::{browser_dialog, favicon, service, template_gallery};

const CATEGORIES: &[&str] = &[
    "Webapps",
    "Network",
    "Office",
    "Development",
    "Graphics",
    "AudioVideo",
    "Game",
    "Utility",
    "System",
    "Education",
    "Science",
];

#[allow(dead_code)]
pub struct DialogResult {
    pub saved: bool,
    pub webapp: WebApp,
}

/// Show the create/edit webapp dialog
pub fn show(
    parent: &impl IsA<gtk::Window>,
    webapp: WebApp,
    browsers: Rc<RefCell<BrowserCollection>>,
    is_new: bool,
    on_done: impl Fn(DialogResult) + 'static,
) {
    let webapp_cell = Rc::new(RefCell::new(webapp));

    let dialog_title = if is_new {
        gettext("New WebApp")
    } else {
        gettext("Edit WebApp")
    };
    let win = adw::Window::builder()
        .title(&dialog_title)
        .default_width(680)
        .default_height(600)
        .modal(true)
        .transient_for(parent)
        .build();

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // headerbar
    let header = adw::HeaderBar::new();
    // placeholder for template button — will be wired after widgets exist
    let tmpl_btn = if is_new {
        let btn = gtk::Button::with_label(&gettext("Templates"));
        btn.set_tooltip_text(Some(&gettext("Choose from templates")));
        btn.add_css_class("suggested-action");
        header.pack_start(&btn);
        Some(btn)
    } else {
        None
    };
    outer.append(&header);

    // overlay for loading spinner
    let overlay = gtk::Overlay::new();

    let spinner_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    spinner_box.set_halign(gtk::Align::Center);
    spinner_box.set_valign(gtk::Align::Center);
    let spinner = gtk::Spinner::new();
    spinner.set_spinning(true);
    spinner.set_width_request(32);
    spinner.set_height_request(32);
    spinner_box.append(&spinner);
    let spin_label = gtk::Label::new(Some(&gettext("Loading...")));
    spinner_box.append(&spin_label);
    spinner_box.set_visible(false);
    overlay.add_overlay(&spinner_box);

    // scrollable form
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(600);

    let form = gtk::Box::new(gtk::Orientation::Vertical, 16);
    form.set_margin_top(24);
    form.set_margin_bottom(24);
    form.set_margin_start(24);
    form.set_margin_end(24);

    // ── Card 1: Website ──────────────────────────────────────
    let group_website = adw::PreferencesGroup::new();
    group_website.set_title(&gettext("Website"));

    let url_row = adw::EntryRow::builder()
        .title(gettext("URL"))
        .text(&webapp_cell.borrow().app_url)
        .build();
    let detect_btn = gtk::Button::with_label(&gettext("Detect"));
    detect_btn.set_tooltip_text(Some(&gettext("Detect name and icon from website")));
    detect_btn.set_valign(gtk::Align::Center);
    url_row.add_suffix(&detect_btn);
    group_website.add(&url_row);

    let name_row = adw::EntryRow::builder()
        .title(gettext("Name"))
        .text(&webapp_cell.borrow().app_name)
        .build();
    group_website.add(&name_row);

    // ── Card 2: Appearance ───────────────────────────────────
    let group_appearance = adw::PreferencesGroup::new();
    group_appearance.set_title(&gettext("Appearance"));
    group_appearance.set_description(Some(&gettext("Icon and application category")));

    let icon_row = adw::ActionRow::builder()
        .title(gettext("Icon"))
        .subtitle(gettext("Choose an icon for the webapp"))
        .build();
    let icon_preview = gtk::Image::new();
    icon_preview.set_pixel_size(32);
    crate::webapp_row::load_icon(&icon_preview, &webapp_cell.borrow().app_icon);
    icon_row.add_prefix(&icon_preview);
    let icon_btn = gtk::Button::with_label(&gettext("Select"));
    icon_btn.set_valign(gtk::Align::Center);
    icon_row.add_suffix(&icon_btn);
    group_appearance.add(&icon_row);

    // favicon picker (initially hidden)
    let favicon_flow = gtk::FlowBox::new();
    favicon_flow.set_max_children_per_line(6);
    favicon_flow.set_min_children_per_line(3);
    favicon_flow.set_homogeneous(true);
    favicon_flow.set_selection_mode(gtk::SelectionMode::Single);
    favicon_flow.set_visible(false);

    let cat_model = gtk::StringList::new(CATEGORIES);
    let cat_dropdown = gtk::DropDown::new(Some(cat_model), gtk::Expression::NONE);
    let current_cat = webapp_cell.borrow().main_category().to_string();
    if let Some(pos) = CATEGORIES.iter().position(|c| *c == current_cat) {
        cat_dropdown.set_selected(pos as u32);
    }
    let cat_row = adw::ActionRow::builder()
        .title(gettext("Category"))
        .subtitle(gettext("Application menu category"))
        .build();
    cat_dropdown.set_valign(gtk::Align::Center);
    cat_row.add_suffix(&cat_dropdown);
    group_appearance.add(&cat_row);

    // ── Card 3: Behavior ─────────────────────────────────────
    let group_behavior = adw::PreferencesGroup::new();
    group_behavior.set_title(&gettext("Behavior"));
    group_behavior.set_description(Some(&gettext("How the webapp opens and runs")));

    let mode_switch = gtk::Switch::new();
    mode_switch.set_valign(gtk::Align::Center);
    mode_switch.set_active(webapp_cell.borrow().app_mode == AppMode::App);
    let mode_row = adw::ActionRow::builder()
        .title(gettext("App Mode"))
        .subtitle(gettext(
            "Opens as a native window without browser interface",
        ))
        .build();
    mode_row.add_suffix(&mode_switch);
    mode_row.set_activatable_widget(Some(&mode_switch));
    group_behavior.add(&mode_row);

    let browser_row = adw::ActionRow::builder().title(gettext("Browser")).build();
    {
        let br = browsers.borrow();
        let bid = &webapp_cell.borrow().browser;
        let name = br
            .get_by_id(bid)
            .map(|b| b.display_name().to_string())
            .unwrap_or_else(|| bid.clone());
        browser_row.set_subtitle(&name);
    }
    let browser_btn = gtk::Button::with_label(&gettext("Select"));
    browser_btn.set_valign(gtk::Align::Center);
    browser_row.add_suffix(&browser_btn);
    browser_row.set_visible(webapp_cell.borrow().app_mode != AppMode::App);
    group_behavior.add(&browser_row);

    let profile_switch = gtk::Switch::new();
    profile_switch.set_valign(gtk::Align::Center);
    let has_custom_profile = webapp_cell.borrow().app_profile != "Default"
        && webapp_cell.borrow().app_profile != "Browser";
    profile_switch.set_active(has_custom_profile);
    let profile_row = adw::ExpanderRow::builder()
        .title(gettext("Separate Profile"))
        .subtitle(gettext("Allows independent cookies and sessions"))
        .show_enable_switch(true)
        .enable_expansion(has_custom_profile)
        .build();
    let profile_entry = adw::EntryRow::builder()
        .title(gettext("Profile Name"))
        .text(&webapp_cell.borrow().app_profile)
        .build();
    profile_row.add_row(&profile_entry);
    profile_row.set_visible(webapp_cell.borrow().app_mode != AppMode::App);
    group_behavior.add(&profile_row);

    form.append(&group_website);
    form.append(&group_appearance);
    form.append(&favicon_flow);
    form.append(&group_behavior);

    // -- buttons --
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_top(8);

    let cancel_btn = gtk::Button::with_label(&gettext("Cancel"));
    let save_label = gettext("Save");
    let save_btn = gtk::Button::with_label(&save_label);
    save_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);
    form.append(&btn_box);

    clamp.set_child(Some(&form));
    scroll.set_child(Some(&clamp));
    overlay.set_child(Some(&scroll));
    outer.append(&overlay);
    win.set_content(Some(&outer));

    // -- wire up signals --

    // Template button → populate URL, name, icon, category after selection
    if let Some(ref tb) = tmpl_btn {
        let wc = webapp_cell.clone();
        let w = win.clone();
        let ur = url_row.clone();
        let nr = name_row.clone();
        let ip = icon_preview.clone();
        let cd = cat_dropdown.clone();
        tb.connect_clicked(move |_| {
            let wc2 = wc.clone();
            let ur2 = ur.clone();
            let nr2 = nr.clone();
            let ip2 = ip.clone();
            let cd2 = cd.clone();
            template_gallery::show(&w, move |template_id| {
                log::info!("Template callback received: {}", &template_id);
                let reg = build_default_registry();
                if let Some(tpl) = reg.get(&template_id) {
                    log::info!("Template found: {} url={}", &tpl.name, &tpl.url);
                    wc2.borrow_mut().apply_template(tpl);
                    // clone data before dropping borrow — set_text triggers connect_changed
                    let (url, name, icon, cat) = {
                        let data = wc2.borrow();
                        (
                            data.app_url.clone(),
                            data.app_name.clone(),
                            data.app_icon.clone(),
                            data.main_category().to_string(),
                        )
                    };
                    ur2.set_text(&url);
                    nr2.set_text(&name);
                    crate::webapp_row::load_icon(&ip2, &icon);
                    if let Some(pos) = CATEGORIES.iter().position(|c| *c == cat) {
                        cd2.set_selected(pos as u32);
                    }
                }
            });
        });
    }

    // URL changed → update model + auto-detect with debounce
    let debounce_handle: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    {
        let wc = webapp_cell.clone();
        let db_handle = debounce_handle.clone();
        let detect_btn_ref = detect_btn.clone();
        url_row.connect_changed(move |row| {
            wc.borrow_mut().app_url = row.text().to_string();
            // cancel previous debounce
            if let Some(id) = db_handle.borrow_mut().take() {
                id.remove();
            }
            // schedule auto-detect after 800ms idle
            let btn = detect_btn_ref.clone();
            let handle = db_handle.clone();
            let text = row.text().to_string();
            let source = glib::timeout_add_local_once(
                std::time::Duration::from_millis(800),
                move || {
                    handle.borrow_mut().take();
                    // only trigger if URL looks valid (has a dot)
                    if text.contains('.') && text.len() > 3 {
                        btn.emit_clicked();
                    }
                },
            );
            *db_handle.borrow_mut() = Some(source);
        });
    }

    // Name changed
    {
        let wc = webapp_cell.clone();
        name_row.connect_changed(move |row| {
            wc.borrow_mut().app_name = row.text().to_string();
        });
    }

    // Category changed
    {
        let wc = webapp_cell.clone();
        cat_dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if idx < CATEGORIES.len() {
                wc.borrow_mut().set_main_category(CATEGORIES[idx]);
            }
        });
    }

    // Mode switch
    {
        let wc = webapp_cell.clone();
        let br = browser_row.clone();
        let pr = profile_row.clone();
        let brs = browsers.clone();
        let brow = browser_row.clone();
        mode_switch.connect_state_set(move |_, active| {
            let mut app = wc.borrow_mut();
            if active {
                app.app_mode = AppMode::App;
                // keep browser field unchanged → restored on switch back
            } else {
                app.app_mode = AppMode::Browser;
                // if browser was __viewer__ (legacy data), pick default
                if app.browser == "__viewer__" || app.browser.is_empty() {
                    if let Some(def) = brs.borrow().default_browser() {
                        app.browser = def.browser_id.clone();
                    }
                }
                // update browser row subtitle
                let name = brs
                    .borrow()
                    .get_by_id(&app.browser)
                    .map(|b| b.display_name().to_string())
                    .unwrap_or_else(|| app.browser.clone());
                brow.set_subtitle(&name);
            }
            drop(app);
            br.set_visible(!active);
            pr.set_visible(!active);
            glib::Propagation::Proceed
        });
    }

    // Browser select
    {
        let wc = webapp_cell.clone();
        let br = browsers.clone();
        let brow = browser_row.clone();
        let w = win.clone();
        browser_btn.connect_clicked(move |_| {
            let current = wc.borrow().browser.clone();
            let wcx = wc.clone();
            let browx = brow.clone();
            let brx = br.clone();
            let browsers_snapshot = brx.borrow().clone();
            browser_dialog::show(&w, &browsers_snapshot, &current, move |id| {
                wcx.borrow_mut().browser = id.clone();
                let name = brx
                    .borrow()
                    .get_by_id(&id)
                    .map(|b| b.display_name().to_string())
                    .unwrap_or(id);
                browx.set_subtitle(&name);
            });
        });
    }

    // Profile expansion
    {
        let wc = webapp_cell.clone();
        let pe = profile_entry.clone();
        profile_row.connect_enable_expansion_notify(move |row| {
            if row.enables_expansion() {
                let name = wc.borrow().derive_profile_name();
                wc.borrow_mut().app_profile = name.clone();
                pe.set_text(&name);
            } else {
                wc.borrow_mut().app_profile = "Default".into();
            }
        });
    }

    // Profile name entry
    {
        let wc = webapp_cell.clone();
        profile_entry.connect_changed(move |row| {
            let text = row.text().to_string();
            if !text.is_empty() {
                wc.borrow_mut().app_profile = text;
            }
        });
    }

    // Detect (fetch favicon + title)
    {
        let wc = webapp_cell.clone();
        let nr = name_row.clone();
        let ff = favicon_flow.clone();
        let ip = icon_preview.clone();
        let sb = spinner_box.clone();
        detect_btn.connect_clicked(move |_| {
            let url = wc.borrow().app_url.clone();
            if url.is_empty() {
                return;
            }
            sb.set_visible(true);

            let (tx, rx) = std::sync::mpsc::channel::<favicon::SiteInfo>();

            std::thread::spawn(move || {
                let info = match favicon::fetch_site_info(&url) {
                    Ok(info) => info,
                    Err(e) => {
                        log::error!("Fetch site info: {e}");
                        favicon::SiteInfo {
                            title: String::new(),
                            icon_paths: Vec::new(),
                        }
                    }
                };
                let _ = tx.send(info);
            });

            // poll result from main thread
            let wcr = wc.clone();
            let nrr = nr.clone();
            let ffr = ff.clone();
            let ipr = ip.clone();
            let sbr = sb.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
                match rx.try_recv() {
                    Ok(info) => {
                        sbr.set_visible(false);
                        if !info.title.is_empty() {
                            nrr.set_text(&info.title);
                            wcr.borrow_mut().app_name = info.title.clone();
                        }
                        if !info.icon_paths.is_empty() {
                            while let Some(c) = ffr.first_child() {
                                ffr.remove(&c);
                            }
                            for path in &info.icon_paths {
                                let img = gtk::Image::new();
                                img.set_pixel_size(48);
                                img.set_from_file(Some(path));
                                ffr.append(&img);
                            }
                            ffr.set_visible(true);

                            if let Some(first) = info.icon_paths.first() {
                                let path_str = first.to_string_lossy().to_string();
                                ipr.set_from_file(Some(first));
                                wcr.borrow_mut().app_icon = path_str.clone();
                                wcr.borrow_mut().app_icon_url = path_str;
                            }
                        }
                        glib::ControlFlow::Break
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(_) => {
                        sbr.set_visible(false);
                        glib::ControlFlow::Break
                    }
                }
            });

            // favicon picker selection
            let wcc = wc.clone();
            let ipc = ip.clone();
            ff.connect_child_activated(move |_, child| {
                if let Some(img) = child.child().and_then(|c| c.downcast::<gtk::Image>().ok()) {
                    if let Some(file) = img.file() {
                        let path = file.to_string();
                        ipc.set_from_file(Some(&*path));
                        wcc.borrow_mut().app_icon = path.clone();
                        wcc.borrow_mut().app_icon_url = path;
                    }
                }
            });
        });
    }

    // Icon file chooser
    {
        let wc = webapp_cell.clone();
        let ip = icon_preview.clone();
        let w = win.clone();
        icon_btn.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title(&gettext("Select Icon"));
            let filter = gtk::FileFilter::new();
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/svg+xml");
            filter.add_mime_type("image/x-icon");
            filter.set_name(Some(&gettext("Images")));
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let wcx = wc.clone();
            let ipx = ip.clone();
            dialog.open(
                Some(&w),
                gio::Cancellable::NONE,
                move |result: Result<gio::File, glib::Error>| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let ps = path.to_string_lossy().to_string();
                            ipx.set_from_file(Some(&path));
                            wcx.borrow_mut().app_icon = ps.clone();
                            wcx.borrow_mut().app_icon_url = ps;
                        }
                    }
                },
            );
        });
    }

    // Cancel
    {
        let w = win.clone();
        cancel_btn.connect_clicked(move |_| w.close());
    }

    // Save
    {
        let wc = webapp_cell.clone();
        let w = win.clone();
        save_btn.connect_clicked(move |_| {
            let mut app = wc.borrow().clone();

            // validate
            if app.app_name.trim().is_empty() || app.app_url.trim().is_empty() {
                return;
            }

            // normalize URL: prepend https:// if missing scheme
            let url_str = app.app_url.trim().to_string();
            if !url_str.starts_with("http://")
                && !url_str.starts_with("https://")
                && !url_str.starts_with("file://")
            {
                app.app_url = format!("https://{url_str}");
            } else {
                app.app_url = url_str;
            }

            // validate URL format
            if url::Url::parse(&app.app_url).is_err() {
                return;
            }

            let result = if is_new {
                service::create_webapp(&app)
            } else {
                service::update_webapp(&app)
            };

            match &result {
                Ok(()) => log::info!(
                    "Saved webapp '{}' mode={:?}",
                    app.app_name,
                    app.app_mode
                ),
                Err(e) => log::error!("Save webapp failed: {e}"),
            }

            w.close();
            on_done(DialogResult {
                saved: result.is_ok(),
                webapp: app,
            });
        });
    }

    // ESC
    let esc = gtk::EventControllerKey::new();
    {
        let w = win.clone();
        esc.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                w.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
    }
    win.add_controller(esc);

    win.present();

    // focus URL if new, name if edit
    if is_new {
        url_row.grab_focus();
    } else {
        name_row.grab_focus();
    }
}
