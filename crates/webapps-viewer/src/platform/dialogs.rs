use libadwaita as adw;
use libadwaita::prelude::*;

pub fn confirm_dialog_with_cancel_id(
    heading: &str,
    body: &str,
    cancel_response: &str,
    cancel_label: &str,
    confirm_response: &str,
    confirm_label: &str,
    destructive: bool,
) -> adw::AlertDialog {
    let dialog = adw::AlertDialog::builder()
        .heading(heading)
        .body(body)
        .close_response(cancel_response)
        .default_response(cancel_response)
        .build();
    dialog.add_response(cancel_response, cancel_label);
    dialog.add_response(confirm_response, confirm_label);
    if destructive {
        dialog.set_response_appearance(confirm_response, adw::ResponseAppearance::Destructive);
    }
    dialog
}
