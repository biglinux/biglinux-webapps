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
    let dialog = build_dialog();
    dialog.present(Some(parent));
}

fn build_dialog() -> adw::Dialog {
    let dialog = adw::Dialog::builder()
        .title(gettext("Welcome to WebApps Manager"))
        .build();
    crate::geometry::bind_adw_dialog(&dialog, "welcome-dialog.json", 640, 480);

    let toolbar = adw::ToolbarView::new();
    let headerbar = adw::HeaderBar::builder().show_title(false).build();
    headerbar.add_css_class("flat");
    toolbar.add_top_bar(&headerbar);

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let icon = gtk::Image::from_icon_name("big-webapps");
    icon.set_pixel_size(64);
    icon.set_halign(gtk::Align::Center);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    content.append(&icon);

    let title = gtk::Label::builder()
        .label(format!(
            "<span size='x-large' weight='bold'>{}</span>",
            gettext("Welcome to WebApps Manager")
        ))
        .use_markup(true)
        .halign(gtk::Align::Center)
        .build();
    content.append(&title);

    let explanation = gtk::Label::builder()
        .label(format!(
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

    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    sep.set_margin_top(12);
    content.append(&sep);

    // "don't show again" — AdwSwitchRow wires label↔switch for a11y automatically
    let show_switch_row = adw::SwitchRow::builder()
        .title(gettext("Don't show this again"))
        .active(false)
        .margin_top(12)
        .build();

    let switch_list = gtk::ListBox::new();
    switch_list.set_selection_mode(gtk::SelectionMode::None);
    switch_list.add_css_class("boxed-list");
    switch_list.append(&show_switch_row);
    content.append(&switch_list);

    let btn = gtk::Button::builder()
        .label(gettext("Let's Start"))
        .halign(gtk::Align::Center)
        .margin_top(24)
        .build();
    btn.add_css_class("suggested-action");
    btn.connect_clicked({
        let dialog_weak = dialog.downgrade();
        let sw = show_switch_row.clone();
        move |_| {
            // switch ON = "don't show" → mark shown
            if sw.is_active() {
                service::mark_welcome_shown();
            }
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        }
    });
    dialog.set_default_widget(Some(&btn));
    content.append(&btn);

    toolbar.set_content(Some(&content));
    dialog.set_child(Some(&toolbar));

    dialog
}
