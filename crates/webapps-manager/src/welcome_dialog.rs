use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;

use crate::service;

/// Show welcome dialog if first run. Returns immediately.
pub fn show_if_needed(parent: &adw::ApplicationWindow) {
    if !service::should_show_welcome() {
        return;
    }
    let dialog = build_dialog(parent);
    dialog.present();
}

fn build_dialog(parent: &adw::ApplicationWindow) -> adw::Window {
    let dialog = adw::Window::builder()
        .title(&gettext("Welcome to WebApps Manager"))
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .width_request(640)
        .height_request(400)
        .build();

    // ESC → close
    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.connect_key_pressed({
        let dialog_weak = dialog.downgrade();
        move |_, key, _, _| {
            if key == gdk4::Key::Escape {
                if let Some(d) = dialog_weak.upgrade() {
                    d.destroy();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    });
    dialog.add_controller(key_ctrl);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // flat headerbar
    let headerbar = adw::HeaderBar::builder()
        .show_title(false)
        .build();
    headerbar.add_css_class("flat");
    main_box.append(&headerbar);

    // content
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // icon + title
    let icon = gtk::Image::from_icon_name("big-webapps");
    icon.set_pixel_size(64);
    icon.set_halign(gtk::Align::Center);
    content.append(&icon);

    let title = gtk::Label::builder()
        .label(&format!("<span size='x-large' weight='bold'>{}</span>", gettext("Welcome to WebApps Manager")))
        .use_markup(true)
        .halign(gtk::Align::Center)
        .build();
    content.append(&title);

    // explanation
    let explanation = gtk::Label::builder()
        .label(&format!(
            "<b>{}</b>\n\n{}\n\n<b>{}</b>\n\n\
            \u{2022} <b>{}</b>: {}\n\
            \u{2022} <b>{}</b>: {}\n\
            \u{2022} <b>{}</b>: {}",
            gettext("What are WebApps?"),
            gettext("WebApps are web applications that run in a dedicated browser window, providing a more app-like experience for your favorite websites."),
            gettext("Benefits of using WebApps:"),
            gettext("Focus"), gettext("Work without the distractions of other browser tabs"),
            gettext("Desktop Integration"), gettext("Quick access from your application menu"),
            gettext("Isolated Profiles"), gettext("Each webapp can have its own cookies and settings"),
        ))
        .use_markup(true)
        .wrap(true)
        .max_width_chars(60)
        .halign(gtk::Align::Start)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    content.append(&explanation);

    // separator
    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    sep.set_margin_top(12);
    content.append(&sep);

    // "don't show again" switch
    let switch_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .build();

    let switch_label = gtk::Label::builder()
        .label(&gettext("Don't show this again"))
        .xalign(0.0)
        .hexpand(true)
        .build();
    let show_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(false)
        .build();

    switch_box.append(&switch_label);
    switch_box.append(&show_switch);
    content.append(&switch_box);

    // "Let's Start" button
    let btn = gtk::Button::builder()
        .label(&gettext("Let's Start"))
        .halign(gtk::Align::Center)
        .margin_top(24)
        .build();
    btn.add_css_class("suggested-action");
    btn.connect_clicked({
        let dialog_weak = dialog.downgrade();
        let sw = show_switch.clone();
        move |_| {
            // switch ON = "don't show" → mark shown
            if sw.is_active() {
                service::mark_welcome_shown();
            }
            if let Some(d) = dialog_weak.upgrade() {
                d.destroy();
            }
        }
    });
    content.append(&btn);

    main_box.append(&content);
    dialog.set_content(Some(&main_box));

    dialog
}
