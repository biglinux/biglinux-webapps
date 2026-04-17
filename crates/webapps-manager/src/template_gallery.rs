use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use std::rc::Rc;
use webapps_core::templates::{build_default_registry, TemplateRegistry, WebAppTemplate};

/// Show template gallery. Fires callback immediately on selection.
pub fn show(parent: &impl IsA<gtk::Window>, on_selected: impl Fn(String) + 'static) {
    let registry = build_default_registry();
    let callback: Rc<dyn Fn(String)> = Rc::new(on_selected);

    let win = adw::Window::builder()
        .title(gettext("Choose a Template"))
        .default_width(600)
        .default_height(500)
        .modal(true)
        .transient_for(parent)
        .build();

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // headerbar with search
    let header = adw::HeaderBar::new();
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some(&gettext("Search templates...")));
    search_entry.set_hexpand(true);
    header.set_title_widget(Some(&search_entry));
    content.append(&header);

    // scrollable content
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    scroll.set_child(Some(&main_box));
    content.append(&scroll);

    // initial populate
    populate_all(&main_box, &registry, &callback, &win);

    // search handler
    {
        let mb = main_box.clone();
        let reg = registry;
        let cb = callback.clone();
        let w = win.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            clear_box(&mb);
            if query.is_empty() {
                populate_all(&mb, &reg, &cb, &w);
            } else {
                populate_search(&mb, &reg, &query, &cb, &w);
            }
        });
    }

    // ESC to close
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

    win.set_content(Some(&content));
    win.present();
}

fn populate_all(
    container: &gtk::Box,
    registry: &TemplateRegistry,
    callback: &Rc<dyn Fn(String)>,
    win: &adw::Window,
) {
    let mut categories = registry.categories();
    categories.sort();
    for cat in &categories {
        let templates = registry.get_by_category(cat);
        if templates.is_empty() {
            continue;
        }
        add_category_section(container, cat, &templates, callback, win);
    }
}

fn populate_search(
    container: &gtk::Box,
    registry: &TemplateRegistry,
    query: &str,
    callback: &Rc<dyn Fn(String)>,
    win: &adw::Window,
) {
    let results = registry.search(query);
    if results.is_empty() {
        let label = gtk::Label::new(Some(&gettext("No templates found")));
        label.add_css_class("dim-label");
        label.set_margin_top(24);
        container.append(&label);
        return;
    }
    add_category_section(
        container,
        &gettext("Search Results"),
        &results,
        callback,
        win,
    );
}

fn add_category_section(
    container: &gtk::Box,
    category: &str,
    templates: &[&WebAppTemplate],
    callback: &Rc<dyn Fn(String)>,
    win: &adw::Window,
) {
    let header = gtk::Label::new(Some(category));
    header.set_halign(gtk::Align::Start);
    header.add_css_class("title-4");
    header.set_margin_top(8);
    header.set_accessible_role(gtk::AccessibleRole::Heading);
    container.append(&header);

    let listbox = gtk::ListBox::new();
    listbox.add_css_class("boxed-list");
    listbox.set_selection_mode(gtk::SelectionMode::None);

    for tpl in templates {
        let row = adw::ActionRow::builder()
            .title(&tpl.name)
            .subtitle(&tpl.url)
            .activatable(true)
            .build();
        let icon = gtk::Image::new();
        icon.set_pixel_size(32);
        crate::webapp_row::load_icon(&icon, &tpl.icon);
        row.add_prefix(&icon);

        // DRM badge → indicate Browser mode required
        if tpl.requires_drm {
            let drm_icon = gtk::Image::from_icon_name("web-browser-symbolic");
            drm_icon.set_pixel_size(16);
            drm_icon.set_tooltip_text(Some(&gettext("Requires Browser mode (DRM)")));
            drm_icon.add_css_class("dim-label");
            row.add_suffix(&drm_icon);
        }

        // fire callback immediately, then close gallery
        let cb = callback.clone();
        let tid = tpl.template_id.clone();
        let w = win.clone();
        row.connect_activated(move |_| {
            cb(tid.clone());
            w.close();
        });

        listbox.append(&row);
    }

    container.append(&listbox);
}

fn clear_box(bx: &gtk::Box) {
    while let Some(child) = bx.first_child() {
        bx.remove(&child);
    }
}
