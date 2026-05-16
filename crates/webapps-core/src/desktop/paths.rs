use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::config;
use crate::models::WebApp;

use super::builder::generate_desktop_entry;

pub fn desktop_file_id(url: &str) -> String {
    let Ok(parsed) = url::Url::parse(url) else {
        return "webapp".into();
    };

    let Some(host) = parsed.host_str() else {
        return "webapp".into();
    };

    let mut id = sanitize_id_part(&host.replace('.', ""));
    let path_id = sanitize_id_part(parsed.path().trim_matches('/'));
    if !path_id.is_empty() {
        id.push('_');
        id.push_str(&path_id);
    }

    if id.is_empty() {
        "webapp".into()
    } else {
        id
    }
}

pub fn legacy_host_desktop_file_id(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|host| sanitize_id_part(&host.replace('.', "")))
        })
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| "webapp".into())
}

pub fn viewer_desktop_filename(url: &str) -> String {
    format!("biglinux-webapp-{}.desktop", desktop_file_id(url))
}

fn sanitize_id_part(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut previous_was_separator = false;

    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            result.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator && !result.is_empty() {
            result.push('_');
            previous_was_separator = true;
        }
    }

    result.trim_matches('_').to_string()
}

pub fn desktop_file_path(webapp: &WebApp) -> PathBuf {
    let filename = webapp
        .desktop_file_name()
        .map(|file_name| file_name.as_str().to_string())
        .unwrap_or_else(|| viewer_desktop_filename(&webapp.app_url));

    config::applications_dir().join(filename)
}

pub fn install_desktop_entry(webapp: &WebApp) -> Result<()> {
    let path = desktop_file_path(webapp);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = generate_desktop_entry(webapp);
    fs::write(&path, content)?;
    log::info!("Installed desktop entry: {}", path.display());
    refresh_desktop_database();
    Ok(())
}

pub fn remove_desktop_entry(webapp: &WebApp) -> Result<()> {
    let path = desktop_file_path(webapp);
    if path.exists() {
        fs::remove_file(&path)?;
        log::info!("Removed desktop entry: {}", path.display());
        refresh_desktop_database();
    }
    Ok(())
}

pub fn remove_desktop_file(filename: &str) -> Result<()> {
    let path = config::applications_dir().join(filename);
    if path.exists() {
        fs::remove_file(&path)?;
        log::info!("Removed old desktop entry: {}", path.display());
    }
    Ok(())
}

fn refresh_desktop_database() {
    let apps_dir = config::applications_dir();
    // status() blocks and reaps the child to prevent zombie processes
    match std::process::Command::new("update-desktop-database")
        .arg(&apps_dir)
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => log::warn!("update-desktop-database exited with {status}"),
        Err(err) => log::warn!("update-desktop-database not found or failed: {err}"),
    }

    if std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase()
        .contains("gnome")
    {
        let commands: &[&[&str]] = &[
            &["reset", "/org/gnome/shell/app-picker-layout"],
            &[
                "write",
                "/org/gnome/desktop/app-folders/folders/WebApps/categories",
                "['Webapps']",
            ],
        ];
        for args in commands {
            match std::process::Command::new("dconf").args(*args).status() {
                Ok(status) if status.success() => {}
                Ok(status) => log::warn!("dconf {} exited with {status}", args[0]),
                Err(err) => log::warn!("dconf {} failed: {err}", args[0]),
            }
        }
    }
}
