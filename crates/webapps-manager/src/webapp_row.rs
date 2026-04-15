use gtk4 as gtk;
use gtk4::gdk as gdk4;
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
            // rasterize SVG at 4x requested pixel_size → crisp on HiDPI
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

/// Build a row widget for a webapp in the list
pub fn build_row(webapp: &WebApp, callbacks: &std::rc::Rc<RowCallbacks>) -> gtk::ListBoxRow {
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.add_css_class("webapp-row");

    // webapp icon
    let icon = gtk::Image::new();
    icon.set_pixel_size(48);
    icon.add_css_class("webapp-icon");
    load_icon(&icon, &webapp.app_icon);
    hbox.append(&icon);

    // info column
    let info = gtk::Box::new(gtk::Orientation::Vertical, 2);
    info.set_hexpand(true);
    info.set_valign(gtk::Align::Center);

    let name_label = gtk::Label::new(Some(&webapp.app_name));
    name_label.set_halign(gtk::Align::Start);
    name_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    name_label.add_css_class("heading");
    info.append(&name_label);

    let url_label = gtk::Label::new(Some(&webapp.app_url));
    url_label.set_halign(gtk::Align::Start);
    url_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    url_label.add_css_class("dim-label");
    url_label.add_css_class("caption");
    info.append(&url_label);

    hbox.append(&info);

    // action buttons
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    actions.set_valign(gtk::Align::Center);

    // browser indicator — resolve through same icon pipeline
    let browser_btn = gtk::Button::new();
    let browser_img = gtk::Image::new();
    browser_img.set_pixel_size(24);
    if webapp.app_mode == AppMode::App {
        browser_img.set_icon_name(Some("application-x-executable-symbolic"));
        browser_btn.set_tooltip_text(Some(&gettext("App mode")));
    } else {
        let browser_icon = webapps_core::models::Browser {
            browser_id: webapp.browser.clone(),
            is_default: false,
        }.icon_name();
        load_icon(&browser_img, &browser_icon);
        browser_btn.set_tooltip_text(Some(&gettext("Change browser")));
    }
    browser_btn.set_child(Some(&browser_img));
    browser_btn.add_css_class("flat");
    browser_btn.add_css_class("action-btn");
    {
        let cb = callbacks.clone();
        let app = webapp.clone();
        browser_btn.connect_clicked(move |_| (cb.on_browser)(&app));
    }
    actions.append(&browser_btn);

    // edit button
    let edit_btn = gtk::Button::from_icon_name("document-edit-symbolic");
    edit_btn.set_tooltip_text(Some(&gettext("Edit")));
    edit_btn.add_css_class("flat");
    edit_btn.add_css_class("action-btn");
    {
        let cb = callbacks.clone();
        let app = webapp.clone();
        edit_btn.connect_clicked(move |_| (cb.on_edit)(&app));
    }
    actions.append(&edit_btn);

    // delete button
    let del_btn = gtk::Button::from_icon_name("user-trash-symbolic");
    del_btn.set_tooltip_text(Some(&gettext("Delete")));
    del_btn.add_css_class("flat");
    del_btn.add_css_class("action-btn");
    del_btn.add_css_class("error");
    {
        let cb = callbacks.clone();
        let app = webapp.clone();
        del_btn.connect_clicked(move |_| (cb.on_delete)(&app));
    }
    actions.append(&del_btn);

    hbox.append(&actions);

    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&hbox));
    row.set_activatable(true);
    row
}
