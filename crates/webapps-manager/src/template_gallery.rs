use gtk4 as gtk;
use libadwaita as adw;

use crate::platform::tooltip;
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
    search_entry.update_property(&[gtk::accessible::Property::Label(&gettext(
        "Search templates",
    ))]);
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

    let flow = gtk::FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_homogeneous(true);
    flow.set_row_spacing(6);
    flow.set_column_spacing(6);
    flow.set_min_children_per_line(2);
    flow.set_max_children_per_line(4);
    flow.update_property(&[gtk::accessible::Property::Label(category)]);

    for tpl in templates {
        flow.append(&build_template_tile(tpl, callback, dialog));
    }

    container.append(&flow);
}

/// One template rendered as an icon tile.
///
/// The activatable widget is a real `gtk::Button` (not a bare activatable
/// `FlowBoxChild`) so it is keyboard-reachable and AT-SPI exposes a click
/// action — a `FlowBoxChild` alone would report `actions=[]`.
fn build_template_tile(
    tpl: &WebAppTemplate,
    callback: &Rc<dyn Fn(String)>,
    dialog: &adw::Dialog,
) -> gtk::FlowBoxChild {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_halign(gtk::Align::Center);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(6);
    content.set_margin_end(6);
    content.set_width_request(108);

    let icon = gtk::Image::new();
    icon.set_pixel_size(48);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    crate::webapp_row::load_icon(&icon, &tpl.icon);
    content.append(&icon);

    let name = gtk::Label::new(Some(&tpl.name));
    name.set_wrap(true);
    name.set_justify(gtk::Justification::Center);
    name.set_max_width_chars(12);
    name.add_css_class("caption");
    content.append(&name);

    // DRM templates can only run in an external browser. Tooltips aren't
    // announced by AT-SPI, so the constraint goes into the accessible name too.
    let mut accessible_name = tpl.name.clone();
    if tpl.requires_drm {
        let drm_hint = gettext("Requires external browser (DRM)");
        let drm_icon = gtk::Image::from_icon_name("web-browser-symbolic");
        drm_icon.set_pixel_size(12);
        drm_icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        drm_icon.add_css_class("dim-label");
        tooltip::set(&drm_icon, &drm_hint);
        content.append(&drm_icon);
        accessible_name = format!("{} — {drm_hint}", tpl.name);
    }

    let button = gtk::Button::builder().child(&content).build();
    button.add_css_class("flat");
    tooltip::set(&button, &tpl.url);
    button.update_property(&[gtk::accessible::Property::Label(&accessible_name)]);

    let cb = callback.clone();
    let tid = tpl.template_id.clone();
    let d = dialog.clone();
    button.connect_clicked(move |_| {
        cb(tid.clone());
        d.close();
    });

    let child = gtk::FlowBoxChild::new();
    child.set_child(Some(&button));
    // Focus lives on the inner button so keyboard traversal hits the action.
    child.set_focusable(false);
    child
}

fn clear_box(bx: &gtk::Box) {
    while let Some(child) = bx.first_child() {
        bx.remove(&child);
    }
}
