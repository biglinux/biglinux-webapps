use serial_test::serial;
use std::{ffi::OsString, fs, os::unix::fs::PermissionsExt, path::Path};
use webapps_manager::service::detect_browsers;

struct Installation {
    root: tempfile::TempDir,
    previous: Vec<(&'static str, Option<OsString>)>,
}

impl Installation {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let prefix = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../biglinux-webapps/usr");
        let settings = [
            ("PATH", root.path().join("bin")),
            ("HOME", root.path().join("home")),
            ("XDG_DATA_HOME", root.path().join("data")),
            ("XDG_CONFIG_HOME", root.path().join("config")),
            ("XDG_CACHE_HOME", root.path().join("cache")),
            ("FLATPAK_USER_DIR", root.path().join("user")),
            ("FLATPAK_SYSTEM_DIR", root.path().join("system")),
            ("BIGLINUX_WEBAPPS_PREFIX", prefix),
        ];
        let previous = settings
            .iter()
            .map(|(name, _)| (*name, std::env::var_os(name)))
            .collect();
        for (name, value) in settings {
            std::env::set_var(name, value);
        }
        fs::create_dir_all(root.path().join("bin")).unwrap();
        let fixture = Self { root, previous };
        fixture.command("xdg-settings", "exit 1");
        fixture.command("xdg-mime", "exit 1");
        fixture
    }

    fn command(&self, name: &str, body: &str) {
        let path = self.root.path().join("bin").join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn deploy(&self, installation: &str, app_id: &str) {
        fs::create_dir_all(self.root.path().join(installation).join("app").join(app_id)).unwrap();
    }
}

impl Drop for Installation {
    fn drop(&mut self) {
        for (name, value) in &self.previous {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}

fn flatpak_ids() -> Vec<String> {
    detect_browsers()
        .browsers
        .into_iter()
        .filter(|browser| browser.browser_id.starts_with("flatpak-"))
        .map(|browser| browser.browser_id)
        .collect()
}

#[test]
#[serial]
fn command_output_detects_supported_apps_once() {
    let fixture = Installation::new();
    fixture.command("flatpak", "printf '%s\\n' org.mozilla.firefox com.brave.Browser org.mozilla.firefox org.example.Unrelated com.google.ChromeDev");
    fixture.deploy("user", "org.mozilla.firefox");
    let ids = flatpak_ids();
    assert_eq!(ids.len(), 3);
    for id in [
        "flatpak-firefox",
        "flatpak-brave-browser",
        "flatpak-chrome-unstable",
    ] {
        assert!(ids.iter().any(|found| found == id), "{id}");
    }
}

#[test]
#[serial]
fn command_failure_recovers_user_and_system_installations() {
    let fixture = Installation::new();
    fixture.command("flatpak", "printf '%s\\n' com.microsoft.Edge; exit 1");
    fixture.deploy("user", "org.mozilla.firefox");
    fixture.deploy("system", "com.brave.Browser");
    let ids = flatpak_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().any(|id| id == "flatpak-firefox"));
    assert!(ids.iter().any(|id| id == "flatpak-brave-browser"));
}

#[test]
#[serial]
fn missing_command_recovers_exported_launcher() {
    let fixture = Installation::new();
    let export = fixture
        .root
        .path()
        .join("system/exports/bin/com.google.ChromeDev");
    fs::create_dir_all(export.parent().unwrap()).unwrap();
    fs::write(export, "#!/bin/sh\n").unwrap();
    assert_eq!(flatpak_ids(), ["flatpak-chrome-unstable"]);
}

#[test]
#[serial]
fn empty_command_output_recovers_xdg_user_installation() {
    let fixture = Installation::new();
    fixture.command("flatpak", "exit 0");
    std::env::remove_var("FLATPAK_USER_DIR");
    fixture.deploy("data/flatpak", "io.gitlab.librewolf-community");
    assert_eq!(flatpak_ids(), ["flatpak-librewolf"]);
}

#[test]
#[serial]
fn explicit_installation_path_excludes_other_user_roots() {
    let fixture = Installation::new();
    fixture.command("flatpak", "exit 0");
    fixture.deploy("home/.local/share/flatpak", "org.mozilla.firefox");
    fixture.deploy("data/flatpak", "com.brave.Browser");
    assert!(flatpak_ids().is_empty());
}

#[test]
#[serial]
fn failed_default_query_uses_mime_fallback() {
    let fixture = Installation::new();
    fixture.command(
        "flatpak",
        "printf '%s\\n' com.brave.Browser org.mozilla.firefox",
    );
    fixture.command(
        "xdg-settings",
        "printf '%s\\n' com.brave.Browser.desktop; exit 1",
    );
    fixture.command("xdg-mime", "printf '%s\\n' org.mozilla.firefox.desktop");
    let browsers = detect_browsers();
    assert_eq!(
        browsers.default_browser().unwrap().browser_id,
        "flatpak-firefox"
    );
}

#[test]
#[serial]
#[ignore = "Requires an isolated GTK display"]
fn gtk_lists_flatpak_installations_and_preserves_saved_selection() {
    use gtk4::prelude::*;
    use libadwaita::prelude::*;
    use std::{cell::RefCell, rc::Rc};
    let fixture = Installation::new();
    fixture.command("flatpak", "exit 1");
    fixture.deploy("user", "com.brave.Browser");
    fixture.deploy("system", "org.mozilla.firefox");
    let mut browsers = detect_browsers();
    browsers
        .browsers
        .retain(|browser| browser.browser_id.starts_with("flatpak-"));
    assert_eq!(browsers.browsers.len(), 2);
    libadwaita::init().unwrap();
    let window = libadwaita::Window::new();
    window.present();
    let selected = Rc::new(RefCell::new(String::new()));
    let completed = selected.clone();
    webapps_manager::browser_dialog::show(
        &window,
        &browsers,
        "flatpak-brave",
        false,
        true,
        move |selection| *completed.borrow_mut() = selection.browser_id,
    );
    let dialog = window.visible_dialog().unwrap();
    let mut pending = vec![dialog.clone().upcast::<gtk4::Widget>()];
    let mut active = 0;
    let mut titles = Vec::new();
    let mut ok = None;
    while let Some(widget) = pending.pop() {
        if let Some(check) = widget.downcast_ref::<gtk4::CheckButton>() {
            active += usize::from(check.is_active());
        }
        if let Some(row) = widget.downcast_ref::<libadwaita::ActionRow>() {
            titles.push(row.title().to_string());
        }
        if let Some(button) = widget.downcast_ref::<gtk4::Button>() {
            if button.label().as_deref() == Some("OK") {
                ok = Some(button.clone());
            }
        }
        let mut child = widget.first_child();
        while let Some(widget) = child {
            child = widget.next_sibling();
            pending.push(widget);
        }
    }
    assert!(titles.iter().any(|title| title == "Brave"));
    assert!(titles.iter().any(|title| title == "Firefox"));
    assert_eq!(active, 1, "The saved Flatpak alias must remain selected");
    ok.unwrap().emit_clicked();
    assert_eq!(*selected.borrow(), "flatpak-brave");
    window.close();
}
