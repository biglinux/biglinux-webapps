use std::path::PathBuf;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Debug, Clone)]
pub struct FilePicker {
    title: String,
    initial_name: Option<String>,
}

impl FilePicker {
    pub fn save_file(title: String) -> Self {
        Self {
            title,
            initial_name: None,
        }
    }

    pub fn initial_name(mut self, name: &str) -> Self {
        self.initial_name = Some(name.to_string());
        self
    }

    pub fn save_result<F>(self, parent: &impl IsA<gtk::Window>, callback: F)
    where
        F: FnOnce(Result<Option<PathBuf>, glib::Error>) + 'static,
    {
        let dialog = self.dialog();
        dialog.save(Some(parent), gio::Cancellable::NONE, move |result| {
            callback(result.map(|file| file.path()));
        });
    }

    fn dialog(&self) -> gtk::FileDialog {
        let dialog = gtk::FileDialog::builder()
            .title(&self.title)
            .modal(true)
            .build();
        if let Some(name) = &self.initial_name {
            dialog.set_initial_name(Some(name));
        }
        dialog
    }
}
