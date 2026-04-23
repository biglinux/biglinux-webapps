use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::gio;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::WebApp;

use super::super::tasks;
use super::super::ui::DialogWidgets;

pub(crate) fn setup_detection_handler(widgets: &DialogWidgets, webapp_cell: Rc<RefCell<WebApp>>) {
    let name_row = widgets.name_row.clone();
    let favicon_flow = widgets.favicon_flow.clone();
    let icon_preview = widgets.icon_preview.clone();
    let spinner_box = widgets.spinner_box.clone();
    widgets.detect_button.connect_clicked(move |_| {
        let url = webapp_cell.borrow().app_url.clone();
        if url.is_empty() {
            return;
        }

        spinner_box.set_visible(true);
        let name_row = name_row.clone();
        let favicon_flow = favicon_flow.clone();
        let icon_preview = icon_preview.clone();
        let spinner_box = spinner_box.clone();
        let webapp_cell = webapp_cell.clone();
        tasks::detect_site_info(url, move |info| {
            spinner_box.set_visible(false);
            if !info.title.is_empty() {
                name_row.set_text(&info.title);
                webapp_cell.borrow_mut().app_name = info.title.clone();
            }
            if !info.icon_paths.is_empty() {
                while let Some(child) = favicon_flow.first_child() {
                    favicon_flow.remove(&child);
                }
                for (index, path) in info.icon_paths.iter().enumerate() {
                    let image = gtk::Image::new();
                    image.set_pixel_size(48);
                    image.set_from_file(Some(path));
                    // Each icon is selectable; give it a distinct accessible name
                    // so screen-reader users can tell candidates apart.
                    let label =
                        gettext("Icon candidate {n}").replace("{n}", &(index + 1).to_string());
                    image.update_property(&[gtk::accessible::Property::Label(&label)]);
                    favicon_flow.append(&image);
                }
                favicon_flow.set_visible(true);

                if let Some(first_icon) = info.icon_paths.first() {
                    let path = first_icon.to_string_lossy().to_string();
                    icon_preview.set_from_file(Some(first_icon));
                    webapp_cell.borrow_mut().app_icon = path.clone();
                    webapp_cell.borrow_mut().app_icon_url = path;
                }
            }
        });
    });
}

pub(crate) fn setup_favicon_picker(widgets: &DialogWidgets, webapp_cell: Rc<RefCell<WebApp>>) {
    let icon_preview = widgets.icon_preview.clone();
    widgets
        .favicon_flow
        .connect_child_activated(move |_, child| {
            if let Some(image) = child.child().and_then(|c| c.downcast::<gtk::Image>().ok()) {
                if let Some(file) = image.file() {
                    let path = file.to_string();
                    icon_preview.set_from_file(Some(&*path));
                    webapp_cell.borrow_mut().app_icon = path.clone();
                    webapp_cell.borrow_mut().app_icon_url = path;
                }
            }
        });
}

pub(crate) fn setup_icon_picker(widgets: &DialogWidgets, webapp_cell: Rc<RefCell<WebApp>>) {
    let icon_preview = widgets.icon_preview.clone();
    let webapp_dialog = widgets.dialog.clone();
    widgets.icon_button.connect_clicked(move |_| {
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

        let icon_preview = icon_preview.clone();
        let webapp_cell = webapp_cell.clone();
        // FileDialog needs a top-level GtkWindow; AdwDialog is not one, so
        // walk up to the ApplicationWindow via the widget tree root.
        let toplevel = webapp_dialog.root().and_downcast::<gtk::Window>();
        dialog.open(
            toplevel.as_ref(),
            gio::Cancellable::NONE,
            move |result: Result<gio::File, glib::Error>| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let path_string = path.to_string_lossy().to_string();
                        icon_preview.set_from_file(Some(&path));
                        webapp_cell.borrow_mut().app_icon = path_string.clone();
                        webapp_cell.borrow_mut().app_icon_url = path_string;
                    }
                }
            },
        );
    });
}
