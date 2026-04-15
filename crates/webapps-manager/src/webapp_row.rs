use gtk4 as gtk;
use gtk4::gdk as gdk4;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use webapps_core::models::{AppMode, WebApp};

use crate::service;

/// Callbacks from webapp row actions
pub struct RowCallbacks {
    pub on_edit: Box<dyn Fn(&WebApp)>,
    pub on_browser: Box<dyn Fn(&WebApp)>,
    pub on_delete: Box<dyn Fn(&WebApp)>,
}

/// Build a row widget for a webapp in the list
pub fn build_row(webapp: &WebApp, callbacks: &std::rc::Rc<RowCallbacks>) -> gtk::ListBoxRow {
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);

    // icon — prefer icon name for theme lookup (crisp SVG at any size)
    let icon = gtk::Image::new();
    icon.set_pixel_size(48);
    let icon_path = service::resolve_icon_path(&webapp.app_icon);
    let p = std::path::Path::new(&icon_path);
    if p.is_absolute() && p.exists() {
        if icon_path.ends_with(".svg") {
            // rasterize SVG at 2x target → crisp on HiDPI
            match gdk_pixbuf::Pixbuf::from_file_at_size(p, 192, 192) {
                Ok(pixbuf) => {
                    let tex = gdk4::Texture::for_pixbuf(&pixbuf);
                    icon.set_paintable(Some(&tex));
                }
                Err(_) => icon.set_from_file(Some(p)),
            }
        } else {
            icon.set_from_file(Some(p));
        }
    } else {
        icon.set_icon_name(Some(&icon_path));
    }
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

    // action buttons — linked group
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    actions.add_css_class("linked");
    actions.set_valign(gtk::Align::Center);

    // browser indicator — show app icon for App mode, browser icon otherwise
    let browser_icon_name = if webapp.app_mode == AppMode::App {
        "application-x-executable-symbolic".to_string()
    } else {
        webapps_core::models::Browser {
            browser_id: webapp.browser.clone(),
            is_default: false,
        }.icon_name()
    };
    let browser_btn = gtk::Button::from_icon_name(&browser_icon_name);
    let browser_tooltip = if webapp.app_mode == AppMode::App {
        gettext("App mode")
    } else {
        gettext("Change browser")
    };
    browser_btn.set_tooltip_text(Some(&browser_tooltip));
    browser_btn.add_css_class("flat");
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
