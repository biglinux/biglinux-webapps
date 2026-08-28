use std::path::PathBuf;

use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Debug, Clone)]
pub struct FilePicker {
    title: String,
    initial_name: Option<String>,
    filters: Vec<FileFilterSpec>,
}

#[derive(Debug, Clone)]
struct FileFilterSpec {
    label: String,
    patterns: Vec<String>,
    mime_types: Vec<String>,
}

impl FilePicker {
    pub fn new(title: String) -> Self {
        Self {
            title,
            initial_name: None,
            filters: Vec::new(),
        }
    }

    pub fn save_file(title: String) -> Self {
        Self::new(title)
    }

    pub fn initial_name(mut self, name: &str) -> Self {
        self.initial_name = Some(name.to_string());
        self
    }

    pub fn pattern_filter(mut self, label: &str, patterns: &[&str]) -> Self {
        self.filters.push(FileFilterSpec {
            label: label.to_string(),
            patterns: patterns.iter().map(|pattern| pattern.to_string()).collect(),
            mime_types: Vec::new(),
        });
        self
    }

    pub fn mime_filter(mut self, label: &str, mime_types: &[&str]) -> Self {
        self.filters.push(FileFilterSpec {
            label: label.to_string(),
            patterns: Vec::new(),
            mime_types: mime_types
                .iter()
                .map(|mime_type| mime_type.to_string())
                .collect(),
        });
        self
    }

    pub fn open<F>(self, parent: &impl IsA<gtk::Window>, callback: F)
    where
        F: FnOnce(PathBuf) + 'static,
    {
        let dialog = self.dialog();
        dialog.open(Some(parent), gio::Cancellable::NONE, move |result| {
            let Ok(file) = result else {
                return;
            };
            if let Some(path) = file.path() {
                callback(path);
            }
        });
    }

    pub fn open_optional<F>(self, parent: Option<&gtk::Window>, callback: F)
    where
        F: FnOnce(PathBuf) + 'static,
    {
        let dialog = self.dialog();
        dialog.open(parent, gio::Cancellable::NONE, move |result| {
            let Ok(file) = result else {
                return;
            };
            if let Some(path) = file.path() {
                callback(path);
            }
        });
    }

    pub fn save<F>(self, parent: &impl IsA<gtk::Window>, callback: F)
    where
        F: FnOnce(PathBuf) + 'static,
    {
        let dialog = self.dialog();
        dialog.save(Some(parent), gio::Cancellable::NONE, move |result| {
            let Ok(file) = result else {
                return;
            };
            if let Some(path) = file.path() {
                callback(path);
            }
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
        if !self.filters.is_empty() {
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            for spec in &self.filters {
                let filter = gtk::FileFilter::new();
                filter.set_name(Some(&spec.label));
                for pattern in &spec.patterns {
                    filter.add_pattern(pattern);
                }
                for mime_type in &spec.mime_types {
                    filter.add_mime_type(mime_type);
                }
                filters.append(&filter);
            }
            dialog.set_filters(Some(&filters));
        }
        dialog
    }
}
