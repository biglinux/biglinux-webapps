use adw::prelude::*;
use big_app_kit::desktop;
use big_app_kit::dialogs;
use gettextrs::gettext;
use libadwaita as adw;

use crate::{service, ui_async};

use super::super::context::WindowContext;
use super::super::list;

pub(super) fn install(context: &WindowContext) {
    let context_ref = context.clone();
    desktop::install_action(&*context.window, "remove-all", move || {
        present_first_confirmation(&context_ref);
    });
}

/// First confirmation gate — requires an explicit "Continue" before the second
/// dialog is presented. Two taps guard against accidentally destroying the
/// whole collection.
fn present_first_confirmation(context: &WindowContext) {
    let dialog = dialogs::confirm_dialog(
        &gettext("Remove All WebApps"),
        &gettext("Are you sure you want to remove all your WebApps? This action cannot be undone."),
        &gettext("Cancel"),
        "continue",
        &gettext("Continue"),
        true,
    );

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
    let dialog = dialogs::confirm_dialog(
        &gettext("Final Confirmation"),
        &gettext("Are you ABSOLUTELY sure you want to remove ALL your WebApps?"),
        &gettext("No, Cancel"),
        "confirm",
        &gettext("Yes, Remove All"),
        true,
    );

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
