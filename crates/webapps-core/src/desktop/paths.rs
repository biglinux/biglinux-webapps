use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::config;
use crate::models::WebApp;
use crate::subprocess::SubprocessSpec;

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
    format!(
        "biglinux-webapp-{}-{:016x}.desktop",
        desktop_file_id(url).chars().take(80).collect::<String>(),
        identity_hash(url)
    )
}

fn identity_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

pub fn viewer_app_id(webapp: &WebApp) -> String {
    webapp
        .desktop_file_name()
        .and_then(|file_name| {
            file_name
                .as_str()
                .strip_prefix("biglinux-webapp-")
                .and_then(|name| name.strip_suffix(".desktop"))
                .map(sanitize_id_part)
        })
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| desktop_file_id(&webapp.app_url))
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
    crate::storage::write_atomic(&path, content.as_bytes())?;
    log::info!("Installed desktop entry: {}", path.display());
    Ok(())
}

pub fn remove_desktop_entry(webapp: &WebApp) -> Result<()> {
    let path = desktop_file_path(webapp);
    if path.exists() {
        fs::remove_file(&path)?;
        log::info!("Removed desktop entry: {}", path.display());
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

pub fn refresh_desktop_database() {
    let apps_dir = config::applications_dir();
    // run() blocks and reaps the child to prevent zombie processes
    match SubprocessSpec::builder()
        .program("update-desktop-database")
        .arg(&apps_dir)
        .build()
        .run()
    {
        Ok(out) if out.status.success() => {}
        Ok(out) => log::warn!("update-desktop-database exited with {:?}", out.status),
        Err(err) => log::warn!("update-desktop-database not found or failed: {err}"),
    }
}
