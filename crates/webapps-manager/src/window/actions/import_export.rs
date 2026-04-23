#[allow(unused_imports)]
use adw::prelude::*;
use gettextrs::gettext;
use gtk::gio;
#[allow(unused_imports)]
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use crate::{service, ui_async};

use super::super::context::WindowContext;
use super::super::list;

pub(super) fn install_import_action(context: &WindowContext) {
    let action = gio::SimpleAction::new("import", None);
    let context_ref = context.clone();
    action.connect_activate(move |_, _| {
        let dialog = gtk::FileDialog::new();
        dialog.set_title(&gettext("Import WebApps"));

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.zip");
        filter.set_name(Some(&gettext("ZIP files")));
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        let context = context_ref.clone();
        dialog.open(
            Some(&*context_ref.window),
            gio::Cancellable::NONE,
            move |result: Result<gio::File, glib::Error>| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let context = context.clone();
                        ui_async::run_with_result(
                            move || service::import_webapps(&path),
                            move |result| match result {
                                Ok((imported, duplicates)) => {
                                    list::refresh_and_render(&context);
                                    context.show_toast(&format_import_result_message(
                                        imported, duplicates,
                                    ));
                                }
                                Err(err) => {
                                    context.show_toast(&format!(
                                        "{}: {err}",
                                        gettext("Import failed")
                                    ));
                                }
                            },
                        );
                    }
                }
            },
        );
    });
    context.window.add_action(&action);
}

pub(super) fn install_export_action(context: &WindowContext) {
    let action = gio::SimpleAction::new("export", None);
    let context_ref = context.clone();
    action.connect_activate(move |_, _| {
        let dialog = gtk::FileDialog::new();
        dialog.set_title(&gettext("Export WebApps"));
        dialog.set_initial_name(Some("webapps-export.zip"));

        let context = context_ref.clone();
        dialog.save(
            Some(&*context_ref.window),
            gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let context = context.clone();
                        ui_async::run_with_result(
                            move || service::export_webapps(&path),
                            move |result| match result {
                                Ok(status) => {
                                    context.show_toast(&export_status_message(&status));
                                }
                                Err(err) => {
                                    context.show_toast(&format!(
                                        "{}: {err}",
                                        gettext("Export failed")
                                    ));
                                }
                            },
                        );
                    }
                }
            },
        );
    });
    context.window.add_action(&action);
}

fn format_import_result_message(imported: usize, duplicates: usize) -> String {
    gettext("Imported {imported}, skipped {dups} duplicates")
        .replace("{imported}", &imported.to_string())
        .replace("{dups}", &duplicates.to_string())
}

fn export_status_message(status: &str) -> String {
    if status == "no_webapps" {
        gettext("No WebApps")
    } else {
        gettext("WebApps exported successfully")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_status_message_handles_empty_collection() {
        assert_eq!(export_status_message("no_webapps"), "No WebApps");
    }

    #[test]
    fn export_status_message_handles_success() {
        assert_eq!(export_status_message("ok"), "WebApps exported successfully");
    }

    #[test]
    fn format_import_result_message_interpolates_counts() {
        let message = format_import_result_message(3, 2);
        assert!(message.contains('3'));
        assert!(message.contains('2'));
    }
}
