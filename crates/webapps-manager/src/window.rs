use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::gio;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use webapps_core::config;
use webapps_core::models::{BrowserCollection, WebApp, WebAppCollection};

use crate::{browser_dialog, service, webapp_dialog, webapp_row};

struct AppState {
    webapps: WebAppCollection,
    browsers: BrowserCollection,
    filter_text: String,
}

pub fn build(app: &adw::Application) {
    // migrate legacy .desktop files on first run
    let migrated = service::migrate_legacy_desktops();
    if migrated > 0 {
        log::info!("Migrated {migrated} legacy webapps from .desktop files");
    }

    let state = Rc::new(RefCell::new(AppState {
        webapps: service::load_webapps(),
        browsers: service::detect_browsers(),
        filter_text: String::new(),
    }));
    let browsers = Rc::new(RefCell::new(state.borrow().browsers.clone()));

    let win = adw::ApplicationWindow::builder()
        .application(app)
        .title(gettext("WebApps Manager"))
        .default_width(800)
        .default_height(650)
        .build();

    // toast overlay
    let toast_overlay = adw::ToastOverlay::new();

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // -- headerbar --
    let header = adw::HeaderBar::new();

    // search toggle
    let search_btn = gtk::ToggleButton::new();
    search_btn.set_icon_name("system-search-symbolic");
    search_btn.set_tooltip_text(Some(&gettext("Search WebApps")));
    search_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Search WebApps"))]);
    header.pack_start(&search_btn);

    // add button
    let add_btn = gtk::Button::from_icon_name("list-add-symbolic");
    add_btn.set_tooltip_text(Some(&gettext("Add WebApp")));
    add_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Add WebApp"))]);
    header.pack_start(&add_btn);

    // menu
    let menu = gio::Menu::new();
    menu.append(Some(&gettext("Import WebApps")), Some("win.import"));
    menu.append(Some(&gettext("Export WebApps")), Some("win.export"));
    menu.append(
        Some(&gettext("Browse Applications Folder")),
        Some("win.browse-apps"),
    );
    menu.append(
        Some(&gettext("Browse Profiles Folder")),
        Some("win.browse-profiles"),
    );

    let danger = gio::Menu::new();
    danger.append(Some(&gettext("Remove All WebApps")), Some("win.remove-all"));
    menu.append_section(None, &danger);

    let about_section = gio::Menu::new();
    about_section.append(Some(&gettext("About")), Some("win.about"));
    menu.append_section(None, &about_section);

    let menu_btn = gtk::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    menu_btn.set_menu_model(Some(&menu));
    menu_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Main Menu"))]);
    header.pack_end(&menu_btn);

    main_box.append(&header);

    // -- search bar --
    let search_bar = gtk::SearchBar::new();
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_hexpand(true);
    search_bar.set_child(Some(&search_entry));
    search_bar.connect_entry(&search_entry);
    search_btn
        .bind_property("active", &search_bar, "search-mode-enabled")
        .bidirectional()
        .build();
    main_box.append(&search_bar);

    // -- content area --
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(900);
    clamp.set_tightening_threshold(700);

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content_box.set_margin_start(16);
    content_box.set_margin_end(16);
    content_box.set_margin_top(8);
    content_box.set_margin_bottom(16);
    clamp.set_child(Some(&content_box));
    scroll.set_child(Some(&clamp));

    main_box.append(&scroll);

    // a11y live region → announce search result count to screen readers
    let status_label = gtk::Label::new(None);
    status_label.set_visible(false);
    status_label.set_accessible_role(gtk::AccessibleRole::Status);
    main_box.append(&status_label);

    toast_overlay.set_child(Some(&main_box));
    win.set_content(Some(&toast_overlay));

    // -- populate --
    let content_ref = Rc::new(content_box);
    let toast_ref = Rc::new(toast_overlay);
    let win_rc = Rc::new(win);
    let status_ref = Rc::new(status_label);

    populate_list(
        &content_ref,
        &state,
        &browsers,
        &win_rc,
        &toast_ref,
        &status_ref,
    );

    // -- search handler --
    {
        let st = state.clone();
        let cr = content_ref.clone();
        let br = browsers.clone();
        let wr = win_rc.clone();
        let tr = toast_ref.clone();
        let sr = status_ref.clone();
        search_entry.connect_search_changed(move |entry| {
            st.borrow_mut().filter_text = entry.text().to_string();
            populate_list(&cr, &st, &br, &wr, &tr, &sr);
        });
    }

    // -- add button --
    {
        let st = state.clone();
        let br = browsers.clone();
        let cr = content_ref.clone();
        let wr = win_rc.clone();
        let tr = toast_ref.clone();
        let sr = status_ref.clone();
        add_btn.connect_clicked(move |_| {
            let mut new_app = WebApp::default();
            new_app.app_file = service::generate_app_file(&new_app.browser, &new_app.app_url);
            if let Some(def) = br.borrow().default_browser() {
                new_app.browser = def.browser_id.clone();
            }
            let stx = st.clone();
            let brx = br.clone();
            let crx = cr.clone();
            let wrx = wr.clone();
            let trx = tr.clone();
            let srx = sr.clone();
            webapp_dialog::show(&*wr, new_app, br.clone(), true, move |result| {
                if result.saved {
                    refresh_state(&stx);
                    populate_list(&crx, &stx, &brx, &wrx, &trx, &srx);
                    show_toast(&trx, &gettext("WebApp created successfully"));
                }
            });
        });
    }

    // -- GActions --

    // About
    let about_action = gio::SimpleAction::new("about", None);
    {
        let w = win_rc.clone();
        about_action.connect_activate(move |_, _| {
            let about = adw::AboutDialog::builder()
                .application_name(gettext("WebApps Manager"))
                .application_icon("big-webapps")
                .developer_name("BigLinux")
                .version(config::APP_VERSION)
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/biglinux/biglinux-webapps")
                .build();
            about.present(Some(&*w));
        });
    }
    win_rc.add_action(&about_action);

    // Import
    let import_action = gio::SimpleAction::new("import", None);
    {
        let w = win_rc.clone();
        let st = state.clone();
        let cr = content_ref.clone();
        let br = browsers.clone();
        let tr = toast_ref.clone();
        let sr = status_ref.clone();
        import_action.connect_activate(move |_, _| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title(&gettext("Import WebApps"));
            let filter = gtk::FileFilter::new();
            filter.add_pattern("*.zip");
            filter.set_name(Some(&gettext("ZIP files")));
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let stx = st.clone();
            let crx = cr.clone();
            let brx = br.clone();
            let wrx = w.clone();
            let trx = tr.clone();
            let srx = sr.clone();
            dialog.open(
                Some(&*w),
                gio::Cancellable::NONE,
                move |result: Result<gio::File, glib::Error>| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            match service::import_webapps(&path) {
                                Ok((imported, dups)) => {
                                    refresh_state(&stx);
                                    populate_list(&crx, &stx, &brx, &wrx, &trx, &srx);
                                    let msg =
                                        gettext("Imported {imported}, skipped {dups} duplicates")
                                            .replace("{imported}", &imported.to_string())
                                            .replace("{dups}", &dups.to_string());
                                    show_toast(&trx, &msg);
                                }
                                Err(e) => {
                                    show_toast(&trx, &format!("{}: {e}", gettext("Import failed")));
                                }
                            }
                        }
                    }
                },
            );
        });
    }
    win_rc.add_action(&import_action);

    // Export
    let export_action = gio::SimpleAction::new("export", None);
    {
        let w = win_rc.clone();
        let tr = toast_ref.clone();
        export_action.connect_activate(move |_, _| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title(&gettext("Export WebApps"));
            dialog.set_initial_name(Some("webapps-export.zip"));

            let trx = tr.clone();
            dialog.save(Some(&*w), gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        match service::export_webapps(&path) {
                            Ok(status) => {
                                let msg = if status == "no_webapps" {
                                    gettext("No WebApps")
                                } else {
                                    gettext("WebApps exported successfully")
                                };
                                show_toast(&trx, &msg);
                            }
                            Err(e) => {
                                show_toast(&trx, &format!("{}: {e}", gettext("Export failed")))
                            }
                        }
                    }
                }
            });
        });
    }
    win_rc.add_action(&export_action);

    // Browse apps folder
    let browse_apps_action = gio::SimpleAction::new("browse-apps", None);
    browse_apps_action.connect_activate(|_, _| {
        let path = config::applications_dir();
        let _ = open::that(path);
    });
    win_rc.add_action(&browse_apps_action);

    // Browse profiles
    let browse_profiles_action = gio::SimpleAction::new("browse-profiles", None);
    browse_profiles_action.connect_activate(|_, _| {
        let path = config::profiles_dir();
        let _ = std::fs::create_dir_all(&path);
        let _ = open::that(path);
    });
    win_rc.add_action(&browse_profiles_action);

    // Remove all
    let remove_all_action = gio::SimpleAction::new("remove-all", None);
    {
        let st = state.clone();
        let cr = content_ref.clone();
        let br = browsers.clone();
        let w = win_rc.clone();
        let tr = toast_ref.clone();
        let sr = status_ref.clone();
        remove_all_action.connect_activate(move |_, _| {
            let dialog = adw::AlertDialog::builder()
                .heading(gettext("Remove All WebApps"))
                .body(gettext("This will delete all webapps and their desktop entries. This cannot be undone."))
                .build();
            dialog.add_response("cancel", &gettext("Cancel"));
            dialog.add_response("delete", &gettext("Remove All"));
            dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);

            let stx = st.clone();
            let crx = cr.clone();
            let brx = br.clone();
            let wrx = w.clone();
            let trx = tr.clone();
            let srx = sr.clone();
            dialog.connect_response(None, move |_, response| {
                if response == "delete" {
                    if let Err(e) = service::delete_all_webapps() {
                        show_toast(&trx, &format!("{}: {e}", gettext("Failed to remove all WebApps")));
                    } else {
                        refresh_state(&stx);
                        populate_list(&crx, &stx, &brx, &wrx, &trx, &srx);
                        show_toast(&trx, &gettext("All WebApps have been removed"));
                    }
                }
            });
            dialog.present(Some(&*w));
        });
    }
    win_rc.add_action(&remove_all_action);

    // -- keyboard shortcuts --
    let shortcuts = gtk::ShortcutController::new();
    shortcuts.set_scope(gtk::ShortcutScope::Managed);

    // Ctrl+N → new
    {
        let ab = add_btn;
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<ctrl>n"),
            Some(gtk::CallbackAction::new(move |_, _| {
                ab.emit_clicked();
                glib::Propagation::Stop
            })),
        ));
    }

    // Ctrl+F → search
    {
        let sb = search_btn;
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<ctrl>f"),
            Some(gtk::CallbackAction::new(move |_, _| {
                sb.set_active(!sb.is_active());
                glib::Propagation::Stop
            })),
        ));
    }

    // Ctrl+Q → quit
    {
        let a = app.clone();
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<ctrl>q"),
            Some(gtk::CallbackAction::new(move |_, _| {
                a.quit();
                glib::Propagation::Stop
            })),
        ));
    }

    win_rc.add_controller(shortcuts);

    win_rc.present();

    // -- welcome dialog (first run) --
    crate::welcome_dialog::show_if_needed(&win_rc);
}

fn refresh_state(state: &Rc<RefCell<AppState>>) {
    let mut s = state.borrow_mut();
    s.webapps = service::load_webapps();
}

fn populate_list(
    content: &Rc<gtk::Box>,
    state: &Rc<RefCell<AppState>>,
    browsers: &Rc<RefCell<BrowserCollection>>,
    win: &Rc<adw::ApplicationWindow>,
    toast: &Rc<adw::ToastOverlay>,
    status: &Rc<gtk::Label>,
) {
    // clear
    while let Some(child) = content.first_child() {
        content.remove(&child);
    }

    let s = state.borrow();
    let filter = if s.filter_text.is_empty() {
        None
    } else {
        Some(s.filter_text.as_str())
    };
    let categorized = s.webapps.categorized(filter);

    // a11y: announce result count to screen readers via live region
    let total: usize = categorized.values().map(|v| v.len()).sum();
    if filter.is_some() {
        status.set_label(&format!("{total} webapps"));
    }

    if categorized.is_empty() {
        // empty state
        let status_page = adw::StatusPage::builder()
            .icon_name("big-webapps")
            .title(gettext("No WebApps"))
            .description(gettext("Press + to create your first webapp"))
            .vexpand(true)
            .build();
        status_page.add_css_class("empty-state-icon");
        content.append(&status_page);
        return;
    }

    let mut cats: Vec<&String> = categorized.keys().collect();
    cats.sort();

    for cat in cats {
        let apps = &categorized[cat];
        if apps.is_empty() {
            continue;
        }

        // category header
        let header = gtk::Label::new(Some(cat));
        header.set_halign(gtk::Align::Start);
        header.add_css_class("title-4");
        header.add_css_class("category-header");
        header.set_accessible_role(gtk::AccessibleRole::Heading);
        content.append(&header);

        // listbox
        let listbox = gtk::ListBox::new();
        listbox.add_css_class("boxed-list");
        listbox.set_selection_mode(gtk::SelectionMode::None);

        let mut sorted_apps: Vec<&&WebApp> = apps.iter().collect();
        sorted_apps.sort_by(|a, b| a.app_name.to_lowercase().cmp(&b.app_name.to_lowercase()));

        for app in sorted_apps {
            let st = state.clone();
            let br = browsers.clone();
            let _cr_ref = Rc::new(content.as_ref().clone());
            let st2 = state.clone();
            let br2 = browsers.clone();
            let wr = win.clone();
            let tr = toast.clone();
            let wr2 = win.clone();
            let tr2 = toast.clone();
            let wr3 = win.clone();
            let tr3 = toast.clone();

            let callbacks = Rc::new(webapp_row::RowCallbacks {
                on_edit: {
                    let st = st.clone();
                    let br = br.clone();
                    let cr = Rc::new(content.as_ref().clone());
                    let wr = wr.clone();
                    let tr = tr.clone();
                    let sr = status.clone();
                    Box::new(move |app: &WebApp| {
                        let stx = st.clone();
                        let brx = br.clone();
                        let crx = cr.clone();
                        let wrx = wr.clone();
                        let trx = tr.clone();
                        let srx = sr.clone();
                        let cr2 = Rc::new(crx.as_ref().clone());
                        webapp_dialog::show(&*wr, app.clone(), br.clone(), false, move |result| {
                            if result.saved {
                                refresh_state(&stx);
                                populate_list(&cr2, &stx, &brx, &wrx, &trx, &srx);
                                show_toast(&trx, &gettext("WebApp updated successfully"));
                            }
                        });
                    })
                },
                on_browser: {
                    let st = st2.clone();
                    let br = br2.clone();
                    let cr = Rc::new(content.as_ref().clone());
                    let wr = wr2.clone();
                    let tr = tr2.clone();
                    let sr = status.clone();
                    Box::new(move |app: &WebApp| {
                        let app_cell = Rc::new(RefCell::new(app.clone()));
                        let stx = st.clone();
                        let brx = br.clone();
                        let crx = cr.clone();
                        let wrx = wr.clone();
                        let trx = tr.clone();
                        let srx = sr.clone();
                        let cr2 = Rc::new(crx.as_ref().clone());
                        browser_dialog::show(&*wr, &br.borrow(), &app.browser, move |new_id| {
                            app_cell.borrow_mut().browser = new_id;
                            let updated = app_cell.borrow().clone();
                            if let Err(e) = service::update_webapp(&updated) {
                                show_toast(&trx, &format!("Failed: {e}"));
                            } else {
                                refresh_state(&stx);
                                populate_list(&cr2, &stx, &brx, &wrx, &trx, &srx);
                                show_toast(&trx, &gettext("Browser changed"));
                            }
                        });
                    })
                },
                on_delete: {
                    let st = state.clone();
                    let br = browsers.clone();
                    let cr = Rc::new(content.as_ref().clone());
                    let wr = wr3.clone();
                    let tr = tr3.clone();
                    let sr = status.clone();
                    Box::new(move |app: &WebApp| {
                        let dialog = adw::AlertDialog::builder()
                            .heading(gettext("Delete WebApp"))
                            .body(format!("{}\n{}", app.app_name, app.app_url))
                            .build();
                        dialog.add_response("cancel", &gettext("Cancel"));
                        dialog.add_response("delete", &gettext("Delete"));
                        dialog.set_response_appearance(
                            "delete",
                            adw::ResponseAppearance::Destructive,
                        );

                        let shared = !service::profile_shared(app);
                        let has_profile =
                            app.app_profile != "Default" && app.app_profile != "Browser";

                        // add checkbox for profile deletion if applicable
                        let del_profile = Rc::new(RefCell::new(false));
                        if has_profile && shared {
                            let check = gtk::CheckButton::with_label(&gettext(
                                "Also delete configuration folder",
                            ));
                            let dp = del_profile.clone();
                            check.connect_toggled(move |c| {
                                *dp.borrow_mut() = c.is_active();
                            });
                            dialog.set_extra_child(Some(&check));
                        }

                        let app_c = app.clone();
                        let stx = st.clone();
                        let brx = br.clone();
                        let crx = cr.clone();
                        let wrx = wr.clone();
                        let trx = tr.clone();
                        let srx = sr.clone();
                        let cr2 = Rc::new(crx.as_ref().clone());
                        dialog.connect_response(None, move |_, response| {
                            if response == "delete" {
                                let dp = *del_profile.borrow();
                                if let Err(e) = service::delete_webapp(&app_c, dp) {
                                    show_toast(&trx, &format!("Failed: {e}"));
                                } else {
                                    refresh_state(&stx);
                                    populate_list(&cr2, &stx, &brx, &wrx, &trx, &srx);
                                    show_toast(&trx, &gettext("WebApp deleted successfully"));
                                }
                            }
                        });
                        dialog.present(Some(&*wr));
                    })
                },
            });

            let row = webapp_row::build_row(app, &callbacks);
            listbox.append(&row);
        }

        content.append(&listbox);
    }
}

fn show_toast(overlay: &Rc<adw::ToastOverlay>, message: &str) {
    let toast = adw::Toast::new(message);
    toast.set_timeout(3);
    overlay.add_toast(toast);
}
