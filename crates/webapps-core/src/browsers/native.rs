use std::{os::unix::fs::PermissionsExt, path::Path};

use super::BrowserDef;
use crate::config;

pub fn native_browser_path(definition: &BrowserDef) -> Option<String> {
    definition
        .native_paths
        .iter()
        .find_map(|candidate| registered_path(candidate))
        .or_else(|| nix_browser_path(definition))
}

fn registered_path(candidate: &str) -> Option<String> {
    let path = Path::new(candidate);
    if path.is_absolute() {
        return executable(path).then(|| candidate.to_owned());
    }
    if path.components().count() == 1 {
        return command_path(candidate);
    }
    None
}

fn executable(path: &Path) -> bool {
    if config::is_flatpak() {
        return config::host_command("test")
            .arg("-f")
            .arg(path)
            .arg("-a")
            .arg("-x")
            .arg(path)
            .status()
            .is_ok_and(|status| status.success());
    }
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

fn command_path(name: &str) -> Option<String> {
    if config::is_flatpak() {
        let output = config::host_command("which").arg(name).output().ok()?;
        let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        return (output.status.success() && executable(Path::new(&path))).then_some(path);
    }
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|directory| directory.join(name))
        .find(|path| executable(path))
        .map(|path| path.to_string_lossy().into_owned())
}

fn nix_browser_path(definition: &BrowserDef) -> Option<String> {
    let names = definition
        .native_paths
        .iter()
        .filter(|path| Path::new(path).parent() == Some(Path::new("/usr/bin")))
        .filter_map(|path| Path::new(path).file_name()?.to_str())
        .chain(definition.desktop_aliases.iter().map(String::as_str));
    for name in names {
        let Some(path) = command_path(name) else {
            continue;
        };
        // Nix profiles contain real installations; distro PATH shims may exist without a browser.
        if canonical_path(&path).is_some_and(|path| path.starts_with("/nix/store/")) {
            return Some(path);
        }
    }
    None
}

fn canonical_path(path: &str) -> Option<String> {
    if config::is_flatpak() {
        let output = config::host_command("readlink")
            .args(["-f", path])
            .output()
            .ok()?;
        return output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    std::fs::canonicalize(path)
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests;
