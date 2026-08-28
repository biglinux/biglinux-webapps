use gettextrs::gettext;

use crate::platform::desktop;
use crate::platform::file_dialogs::FilePicker;
use crate::{service, ui_async};

use super::super::context::WindowContext;
use super::super::list;

pub(super) fn install_import_action(context: &WindowContext) {
    let context_ref = context.clone();
    desktop::install_action(&*context.window, "import", move || {
        let context_inner = context_ref.clone();
        let parent = context_ref.window.clone();
        FilePicker::new(gettext("Import WebApps"))
            .pattern_filter(&gettext("ZIP files"), &["*.zip"])
            .open(&*parent, move |path| {
                let context = context_inner.clone();
                ui_async::run_with_result(
                    move || service::import_webapps(&path),
                    move |result| match result {
                        Ok((imported, duplicates)) => {
                            list::refresh_and_render(&context);
                            context.show_toast(&format_import_result_message(imported, duplicates));
                        }
                        Err(err) => {
                            context.show_toast(&format!("{}: {err}", gettext("Import failed")));
                        }
                    },
                );
            });
    });
}

pub(super) fn install_export_action(context: &WindowContext) {
    let context_ref = context.clone();
    desktop::install_action(&*context.window, "export", move || {
        let context_inner = context_ref.clone();
        let parent = context_ref.window.clone();
        FilePicker::save_file(gettext("Export WebApps"))
            .initial_name("webapps-export.zip")
            .save(&*parent, move |path| {
                let context = context_inner.clone();
                ui_async::run_with_result(
                    move || service::export_webapps(&path),
                    move |result| match result {
                        Ok(status) => {
                            context.show_toast(&export_status_message(&status));
                        }
                        Err(err) => {
                            context.show_toast(&format!("{}: {err}", gettext("Export failed")));
                        }
                    },
                );
            });
    });
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
