use std::{ffi::OsString, fs, os::unix::fs::symlink, path::PathBuf};

use serial_test::serial;

use super::*;

struct SearchPath {
    directory: tempfile::TempDir,
    previous: Option<OsString>,
}

impl SearchPath {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let previous = std::env::var_os("PATH");
        std::env::set_var("PATH", directory.path());
        Self {
            directory,
            previous,
        }
    }

    fn program(&self, name: &str, mode: u32) -> PathBuf {
        let path = self.directory.path().join(name);
        fs::write(&path, "#!/bin/sh\nexit 1\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }
}

impl Drop for SearchPath {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var("PATH", value),
            None => std::env::remove_var("PATH"),
        }
    }
}

fn definition(paths: Vec<String>) -> BrowserDef {
    let mut definition = crate::browsers::find_def("brave").unwrap().clone();
    definition.native_paths = paths;
    definition.desktop_aliases.clear();
    definition
}

#[test]
#[serial]
fn path_shims_do_not_prove_a_registered_browser_is_installed() {
    let search = SearchPath::new();
    let wrapper = search.program("browser-tweaks-chromium-base", 0o755);
    symlink(wrapper, search.directory.path().join("brave-beta")).unwrap();
    let browser = definition(vec!["/missing/bin/brave-beta".into()]);
    assert!(native_browser_path(&browser).is_none());
}

#[test]
#[serial]
fn internal_binary_basename_does_not_select_another_channel() {
    let search = SearchPath::new();
    search.program("google-chrome", 0o755);
    let browser = definition(vec!["/missing/chrome-beta/google-chrome".into()]);
    assert!(native_browser_path(&browser).is_none());
}

#[test]
#[serial]
fn installed_alternative_wins_over_a_path_shim() {
    let search = SearchPath::new();
    search.program("brave", 0o755);
    let installed = search.program("actual-brave", 0o755);
    let browser = definition(vec![
        "/missing/bin/brave".into(),
        installed.display().to_string(),
    ]);
    assert_eq!(native_browser_path(&browser).as_deref(), installed.to_str());
}

#[test]
#[serial]
fn explicit_command_names_support_custom_installations() {
    let search = SearchPath::new();
    let installed = search.program("custom-browser", 0o755);
    assert_eq!(
        native_browser_path(&definition(vec!["custom-browser".into()])).as_deref(),
        installed.to_str()
    );
}

#[test]
#[serial]
fn directories_and_non_executable_files_are_not_browsers() {
    let search = SearchPath::new();
    let file = search.program("browser", 0o644);
    let browser = definition(vec![
        search.directory.path().display().to_string(),
        file.display().to_string(),
    ]);
    assert!(native_browser_path(&browser).is_none());
}
