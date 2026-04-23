use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use std::cell::RefCell;
use std::rc::Rc;
use webapps_core::templates::{default_registry, TemplateRegistry, WebAppTemplate};

/// One-shot owning callback container — fires its inner closure at most once.
type OnceCallbackCell = Rc<RefCell<Option<Box<dyn FnOnce(String)>>>>;

/// Show template gallery. Fires the callback at most once on the first
/// selection — guards against the user double-activating a row before the
/// modal closes, which would otherwise apply the template twice and clobber
/// any edits made between the two invocations.
pub fn show(parent: &impl IsA<gtk::Widget>, on_selected: impl FnOnce(String) + 'static) {
    let registry = default_registry();
    let on_selected_cell: OnceCallbackCell = Rc::new(RefCell::new(Some(Box::new(on_selected))));
    let callback: Rc<dyn Fn(String)> = {
        let cell = on_selected_cell.clone();
        Rc::new(move |template_id| {
            if let Some(cb) = cell.borrow_mut().take() {
                cb(template_id);
            }
        })
    };

    let dialog = adw::Dialog::builder()
        .title(gettext("Choose a Template"))
        .build();
    crate::geometry::bind_adw_dialog(&dialog, "template-gallery.json", 600, 500);

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some(&gettext("Search templates...")));
    search_entry.set_hexpand(true);
    header.set_title_widget(Some(&search_entry));
    toolbar.add_top_bar(&header);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    scroll.set_child(Some(&main_box));
    toolbar.set_content(Some(&scroll));
    dialog.set_child(Some(&toolbar));

    populate_all(&main_box, registry, &callback, &dialog);

    {
        let mb = main_box.clone();
        let cb = callback.clone();
        let d = dialog.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            clear_box(&mb);
            if query.is_empty() {
                populate_all(&mb, registry, &cb, &d);
            } else {
                populate_search(&mb, registry, &query, &cb, &d);
            }
        });
    }

    dialog.present(Some(parent));
}

fn populate_all(
    container: &gtk::Box,
    registry: &TemplateRegistry,
    callback: &Rc<dyn Fn(String)>,
    dialog: &adw::Dialog,
) {
    let mut categories = registry.categories();
    categories.sort();
    for cat in &categories {
        let templates = registry.get_by_category(cat);
        if templates.is_empty() {
            continue;
        }
        add_category_section(container, cat, &templates, callback, dialog);
    }
}

fn populate_search(
    container: &gtk::Box,
    registry: &TemplateRegistry,
    query: &str,
    callback: &Rc<dyn Fn(String)>,
    dialog: &adw::Dialog,
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
        dialog,
    );
}

fn add_category_section(
    container: &gtk::Box,
    category: &str,
    templates: &[&WebAppTemplate],
    callback: &Rc<dyn Fn(String)>,
    dialog: &adw::Dialog,
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
        icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        crate::webapp_row::load_icon(&icon, &tpl.icon);
        row.add_prefix(&icon);

        // DRM badge → indicate Browser mode required. Tooltip isn't announced
        // by AT-SPI, so expose the information via an accessible label.
        if tpl.requires_drm {
            let drm_label = gettext("Requires external browser (DRM)");
            let drm_icon = gtk::Image::from_icon_name("web-browser-symbolic");
            drm_icon.set_pixel_size(16);
            drm_icon.set_tooltip_text(Some(&drm_label));
            drm_icon.update_property(&[gtk::accessible::Property::Label(&drm_label)]);
            drm_icon.add_css_class("dim-label");
            row.add_suffix(&drm_icon);
        }

        let cb = callback.clone();
        let tid = tpl.template_id.clone();
        let d = dialog.clone();
        row.connect_activated(move |_| {
            cb(tid.clone());
            d.close();
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
