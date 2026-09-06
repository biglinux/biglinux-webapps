use std::cell::RefCell;
use std::rc::Rc;

use crate::platform::file_dialogs::FilePicker;
use adw::prelude::*;
use gettextrs::gettext;
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
    let caches = Rc::new(RefCell::new(Vec::new()));
    let closed_caches = caches.clone();
    widgets
        .dialog
        .connect_closed(move |_| closed_caches.borrow_mut().clear());
    let generation = Rc::new(std::cell::Cell::new(0u64));
    let closed = generation.clone();
    widgets
        .dialog
        .connect_closed(move |_| closed.set(closed.get().wrapping_add(1)));
    let save_on_change = widgets.save_button.clone();
    let save_button = widgets.save_button.clone();
    let changed = generation.clone();
    let spinner_on_change = spinner_box.clone();
    widgets.url_row.connect_changed(move |_| {
        changed.set(changed.get().wrapping_add(1));
        spinner_on_change.set_visible(false);
        save_on_change.set_sensitive(true);
    });
    widgets.detect_button.connect_clicked(move |_| {
        let url = webapp_cell.borrow().app_url.clone();
        if url.is_empty() {
            return;
        }

        let save_button = save_button.clone();
        save_button.set_sensitive(false);
        let caches = caches.clone();
        let request = generation.get().wrapping_add(1);
        generation.set(request);
        let generation = generation.clone();
        let original_name = webapp_cell.borrow().app_name.clone();
        let original_icon = webapp_cell.borrow().app_icon.clone();
        while let Some(child) = favicon_flow.first_child() {
            favicon_flow.remove(&child);
        }
        favicon_flow.set_visible(false);
        spinner_box.set_visible(true);
        let name_row = name_row.clone();
        let favicon_flow = favicon_flow.clone();
        let icon_preview = icon_preview.clone();
        let spinner_box = spinner_box.clone();
        let webapp_cell = webapp_cell.clone();
        tasks::detect_site_info(url, move |info| {
            if generation.get() != request {
                return;
            }
            if let Some(cache) = info.cache {
                caches.borrow_mut().push(cache);
            }
            spinner_box.set_visible(false);
            save_button.set_sensitive(true);
            if original_name.is_empty()
                && webapp_cell.borrow().app_name.is_empty()
                && !info.title.is_empty()
            {
                name_row.set_text(&info.title);
                webapp_cell.borrow_mut().app_name = info.title.clone();
            }
            if !info.icons.is_empty() {
                while let Some(child) = favicon_flow.first_child() {
                    favicon_flow.remove(&child);
                }
                for (index, candidate) in info.icons.iter().enumerate() {
                    let path = &candidate.path;
                    let image = gtk::Image::new();
                    image.set_pixel_size(48);
                    image.set_from_file(Some(path));
                    // Each icon is selectable; give it a distinct accessible name
                    // so screen-reader users can tell candidates apart.
                    let label =
                        gettext("Icon candidate {n}").replace("{n}", &(index + 1).to_string());
                    let quality = if candidate.scalable {
                        "SVG".to_owned()
                    } else {
                        format!("{} × {} px", candidate.width, candidate.height)
                    };
                    let label = if index == 0 {
                        gettext("Recommended icon")
                    } else {
                        label
                    };
                    image
                        .set_tooltip_text(Some(&format!("{label} — {quality}\n{}", candidate.url)));
                    image.update_property(&[gtk::accessible::Property::Label(&label)]);
                    favicon_flow.append(&image);
                }
                favicon_flow.set_visible(true);

                if let Some(first_icon) = info
                    .icons
                    .first()
                    .filter(|_| webapp_cell.borrow().app_icon == original_icon)
                {
                    let path = first_icon.path.to_string_lossy().to_string();
                    icon_preview.set_from_file(Some(&first_icon.path));
                    webapp_cell.borrow_mut().app_icon = path.clone();
                    webapp_cell.borrow_mut().app_icon_url = path;
                }
            } else {
                let label = gtk::Label::new(Some(&gettext(
                    "No icon with sufficient quality found (minimum 64 × 64 px).",
                )));
                label.set_wrap(true);
                favicon_flow.append(&label);
                favicon_flow.set_visible(true);
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
        let icon_preview = icon_preview.clone();
        let webapp_cell = webapp_cell.clone();
        // FileDialog needs a top-level GtkWindow; AdwDialog is not one, so
        // walk up to the ApplicationWindow via the widget tree root.
        let toplevel = webapp_dialog.root().and_downcast::<gtk::Window>();
        FilePicker::new(gettext("Select Icon"))
            .mime_filter(
                &gettext("Images"),
                &["image/png", "image/svg+xml", "image/x-icon"],
            )
            .open_optional(toplevel.as_ref(), move |path| {
                let path_string = path.to_string_lossy().to_string();
                icon_preview.set_from_file(Some(&path));
                webapp_cell.borrow_mut().app_icon = path_string.clone();
                webapp_cell.borrow_mut().app_icon_url = path_string;
            });
    });
}
