use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gdk as gdk4;
use gtk4::glib;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use webapps_core::models::{AppMode, WebApp};

use crate::service;

/// Load icon into GtkImage — resolves via theme or file path with crisp SVG
pub fn load_icon(image: &gtk::Image, icon_ref: &str) {
    let resolved = service::resolve_icon_path(icon_ref);
    let p = std::path::Path::new(&resolved);
    if p.is_absolute() && p.exists() {
        if resolved.ends_with(".svg") {
            let target = image.pixel_size().max(32) * 4;
            match gdk_pixbuf::Pixbuf::from_file_at_size(p, target, target) {
                Ok(pixbuf) => {
                    let tex = gdk4::Texture::for_pixbuf(&pixbuf);
                    image.set_paintable(Some(&tex));
                }
                Err(_) => image.set_from_file(Some(p)),
            }
        } else {
            image.set_from_file(Some(p));
        }
    } else {
        image.set_icon_name(Some(&resolved));
    }
}

/// Callbacks from webapp row actions
pub struct RowCallbacks {
    pub on_edit: Box<dyn Fn(&WebApp)>,
    pub on_browser: Box<dyn Fn(&WebApp)>,
    pub on_delete: Box<dyn Fn(&WebApp)>,
}

/// Build an `adw::ActionRow` with an icon prefix and the three linked action
/// buttons (change browser, edit, remove) as a suffix.
pub fn build_row(webapp: &WebApp, callbacks: &Rc<RowCallbacks>) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(glib_markup_escape(&webapp.app_name))
        .subtitle(glib_markup_escape(&webapp.app_url))
        .activatable(false)
        .build();

    let icon = gtk::Image::new();
    icon.set_pixel_size(40);
    icon.add_css_class("webapp-icon");
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    load_icon(&icon, &webapp.app_icon);
    row.add_prefix(&icon);

    row.add_suffix(&build_actions_group(webapp, callbacks));
    row
}

fn build_actions_group(webapp: &WebApp, callbacks: &Rc<RowCallbacks>) -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.set_valign(gtk::Align::Center);
    group.add_css_class("linked");
    group.add_css_class("webapp-actions");

    group.append(&build_browser_button(webapp, callbacks));
    group.append(&build_edit_button(webapp, callbacks));
    group.append(&build_delete_button(webapp, callbacks));
    group
}

fn build_browser_button(webapp: &WebApp, callbacks: &Rc<RowCallbacks>) -> gtk::Button {
    let icon_name = if webapp.app_mode == AppMode::App {
        "big-webapps".to_string()
    } else {
        webapps_core::models::Browser {
            browser_id: webapp.browser.clone(),
            is_default: false,
        }
        .icon_name()
    };

    let button = segmented_button_with_loaded_icon(&icon_name, &gettext("Change browser"));
    {
        let cb = callbacks.clone();
        let app = webapp.clone();
        button.connect_clicked(move |_| (cb.on_browser)(&app));
    }
    button
}

fn build_edit_button(webapp: &WebApp, callbacks: &Rc<RowCallbacks>) -> gtk::Button {
    let button = segmented_button("document-edit-symbolic", &gettext("Edit"));
    {
        let cb = callbacks.clone();
        let app = webapp.clone();
        button.connect_clicked(move |_| (cb.on_edit)(&app));
    }
    button
}

fn build_delete_button(webapp: &WebApp, callbacks: &Rc<RowCallbacks>) -> gtk::Button {
    let button = segmented_button("user-trash-symbolic", &gettext("Remove"));
    button.add_css_class("destructive");
    {
        let cb = callbacks.clone();
        let app = webapp.clone();
        button.connect_clicked(move |_| (cb.on_delete)(&app));
    }
    button
}

fn segmented_button(icon_name: &str, label: &str) -> gtk::Button {
    let image = gtk::Image::from_icon_name(icon_name);
    image.set_pixel_size(22);
    finalize_segmented_button(image, label)
}

/// Variant that resolves the icon through `load_icon` so named browser icons
/// like `brave-browser` or the bundled `big-webapps` paintable render correctly.
fn segmented_button_with_loaded_icon(icon_name: &str, label: &str) -> gtk::Button {
    let image = gtk::Image::new();
    image.set_pixel_size(24);
    load_icon(&image, icon_name);
    finalize_segmented_button(image, label)
}

fn finalize_segmented_button(image: gtk::Image, label: &str) -> gtk::Button {
    image.set_accessible_role(gtk::AccessibleRole::Presentation);
    let button = gtk::Button::builder().child(&image).build();
    button.set_valign(gtk::Align::Center);
    button.set_tooltip_text(Some(label));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

fn glib_markup_escape(value: &str) -> glib::GString {
    glib::markup_escape_text(value)
}
