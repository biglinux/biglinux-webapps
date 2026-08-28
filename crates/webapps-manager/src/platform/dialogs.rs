use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;

pub fn confirm_dialog(
    heading: &str,
    body: &str,
    cancel_label: &str,
    confirm_response: &str,
    confirm_label: &str,
    destructive: bool,
) -> adw::AlertDialog {
    let dialog = adw::AlertDialog::builder()
        .heading(heading)
        .body(body)
        .close_response("cancel")
        .default_response("cancel")
        .build();
    dialog.add_response("cancel", cancel_label);
    dialog.add_response(confirm_response, confirm_label);
    if destructive {
        dialog.set_response_appearance(confirm_response, adw::ResponseAppearance::Destructive);
    }
    dialog
}

pub fn content_dialog(heading: &str, close_label: &str) -> (adw::AlertDialog, gtk::Box) {
    let dialog = adw::AlertDialog::builder()
        .heading(heading)
        .close_response("close")
        .default_response("close")
        .build();
    dialog.add_response("close", close_label);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(6);
    content.set_margin_bottom(6);
    dialog.set_extra_child(Some(&content));
    (dialog, content)
}
