use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::config;
use crate::models::WebApp;

use super::builder::generate_desktop_entry;

pub fn desktop_file_id(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|host| host.replace('.', "")))
        .unwrap_or_else(|| "webapp".into())
}

pub fn desktop_file_path(webapp: &WebApp) -> PathBuf {
    let filename = webapp
        .desktop_file_name()
        .map(|file_name| file_name.as_str().to_string())
        .unwrap_or_else(|| {
            format!(
                "biglinux-webapp-{}.desktop",
                desktop_file_id(&webapp.app_url)
            )
        });

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
