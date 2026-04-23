use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use crate::{service, ui_async};

use super::super::context::WindowContext;
use super::super::list;

pub(super) fn install(context: &WindowContext) {
    let action = gtk::gio::SimpleAction::new("remove-all", None);
    let context_ref = context.clone();
    action.connect_activate(move |_, _| present_first_confirmation(&context_ref));
    context.window.add_action(&action);
}

/// First confirmation gate — requires an explicit "Continue" before the second
/// dialog is presented. Two taps guard against accidentally destroying the
/// whole collection.
fn present_first_confirmation(context: &WindowContext) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettext("Remove All WebApps"))
        .body(gettext(
            "Are you sure you want to remove all your WebApps? This action cannot be undone.",
        ))
        .build();
    dialog.add_response("cancel", &gettext("Cancel"));
    dialog.add_response("continue", &gettext("Continue"));
    dialog.set_response_appearance("continue", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let context_cb = context.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "continue" {
            present_final_confirmation(&context_cb);
        }
    });
    dialog.present(Some(&*context.window));
}

/// Second gate: last-chance confirmation. Both responses default to Cancel so a
/// stray Enter press never deletes everything.
fn present_final_confirmation(context: &WindowContext) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettext("Final Confirmation"))
        .body(gettext(
            "Are you ABSOLUTELY sure you want to remove ALL your WebApps?",
        ))
        .build();
    dialog.add_response("cancel", &gettext("No, Cancel"));
    dialog.add_response("confirm", &gettext("Yes, Remove All"));
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let context_cb = context.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "confirm" {
            remove_all_async(&context_cb);
        }
    });
    dialog.present(Some(&*context.window));
}

fn remove_all_async(context: &WindowContext) {
    // delete_all_webapps spawns `update-desktop-database` per entry; push that
    // to a worker so the main loop keeps up.
    let context_done = context.clone();
    ui_async::run_with_result(service::delete_all_webapps, move |result| match result {
        Ok(()) => {
            list::refresh_and_render(&context_done);
            context_done.show_toast(&gettext("All WebApps have been removed"));
        }
        Err(err) => {
            context_done.show_toast(&format!(
                "{}: {err}",
                gettext("Failed to remove all WebApps")
            ));
        }
    });
}
