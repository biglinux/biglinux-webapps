use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use crate::geometry;

const DIALOG_GEOMETRY: &str = "webapp-dialog.json";
const DIALOG_DEFAULT_WIDTH: i32 = 720;
const DIALOG_DEFAULT_HEIGHT: i32 = 640;

pub(super) struct DialogShell {
    pub dialog: adw::Dialog,
    pub outer: gtk::Box,
    pub form: gtk::Box,
    pub spinner_box: gtk::Box,
    pub cancel_button: gtk::Button,
    pub save_button: gtk::Button,
}

pub(super) fn build_dialog_shell(is_new: bool) -> DialogShell {
    let dialog_title = if is_new {
        gettext("New WebApp")
    } else {
        gettext("Edit WebApp")
    };
    let dialog = adw::Dialog::builder().title(&dialog_title).build();
    geometry::bind_adw_dialog(
        &dialog,
        DIALOG_GEOMETRY,
        DIALOG_DEFAULT_WIDTH,
        DIALOG_DEFAULT_HEIGHT,
    );

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // HeaderBar: Cancel on the left, Save on the right — matches Adwaita dialog
    // patterns used in Contacts, Calendar and Files.
    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);

    let cancel_button = gtk::Button::with_label(&gettext("Cancel"));
    let save_button = gtk::Button::with_label(&gettext("Save"));
    save_button.add_css_class("suggested-action");
    header.pack_start(&cancel_button);
    header.pack_end(&save_button);

    outer.append(&header);

    let overlay = gtk::Overlay::new();
    let spinner_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    spinner_box.set_halign(gtk::Align::Center);
    spinner_box.set_valign(gtk::Align::Center);
    let spinner = adw::Spinner::new();
    spinner.set_width_request(32);
    spinner.set_height_request(32);
    spinner_box.append(&spinner);
    spinner_box.append(&gtk::Label::new(Some(&gettext("Loading…"))));
    spinner_box.set_visible(false);
    overlay.add_overlay(&spinner_box);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(640);
    clamp.set_tightening_threshold(500);

    let form = gtk::Box::new(gtk::Orientation::Vertical, 18);
    form.set_margin_top(24);
    form.set_margin_bottom(24);
    form.set_margin_start(24);
    form.set_margin_end(24);

    clamp.set_child(Some(&form));
    scroll.set_child(Some(&clamp));
    overlay.set_child(Some(&scroll));
    outer.append(&overlay);

    DialogShell {
        dialog,
        outer,
        form,
        spinner_box,
        cancel_button,
        save_button,
    }
}
