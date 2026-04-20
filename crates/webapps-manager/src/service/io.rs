use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use webapps_core::config;
use webapps_core::models::WebApp;

use super::{create_webapp, generate_app_file, load_webapps};

/// Max size per extracted file from import zip → prevent decompression bombs
const MAX_EXTRACTED_FILE_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

pub fn export_webapps(zip_path: &Path) -> Result<String> {
    let col = load_webapps();
    if col.webapps.is_empty() {
        return Ok("no_webapps".into());
    }

    let file = fs::File::create(zip_path).context("Create zip file")?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // write manifest
    let manifest = serde_json::to_string_pretty(&col.webapps)?;
    zip.start_file("webapps.json", options)?;
    zip.write_all(manifest.as_bytes())?;

    // copy icons
    for app in &col.webapps {
        if app.app_icon_url.is_empty() {
            continue;
        }
        let icon_path = Path::new(&app.app_icon_url);
        if icon_path.is_file() {
            let fname = icon_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if !fname.is_empty() {
                zip.start_file(format!("icons/{fname}"), options)?;
                let mut f = fs::File::open(icon_path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            }
        }
    }

    zip.finish()?;
    Ok("ok".into())
}

pub fn import_webapps(zip_path: &Path) -> Result<(usize, usize)> {
    let file = fs::File::open(zip_path).context("Open zip file")?;
    let mut archive = zip::ZipArchive::new(file)?;

    // read manifest
    let manifest = {
        let mut entry = archive.by_name("webapps.json")?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        buf
    };
    let imported_apps: Vec<WebApp> = serde_json::from_str(&manifest)?;

    // extract icons
    let icons_dir = config::data_dir().join("icons");
    fs::create_dir_all(&icons_dir)?;
    let icons_canonical = icons_dir.canonicalize()?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.starts_with("icons/") {
            let fname = name.strip_prefix("icons/").unwrap_or(&name);
            // strict filename: must be non-empty, no path separators, no ..
            if fname.is_empty()
                || fname.contains('/')
                || fname.contains('\\')
                || fname.contains("..")
            {
                continue;
            }
            let dest = icons_dir.join(fname);
            // verify dest stays within icons_dir
            if let Ok(canonical) = dest.parent().map(|p| p.canonicalize()).transpose() {
                if canonical.as_deref() != Some(icons_canonical.as_path()) {
                    continue;
                }
            }
            let mut out = fs::File::create(&dest)?;
            // cap extracted size → prevent decompression bombs
            let copied =
                std::io::copy(&mut entry.by_ref().take(MAX_EXTRACTED_FILE_BYTES), &mut out)?;
            if copied >= MAX_EXTRACTED_FILE_BYTES {
                log::warn!(
                    "Skipped oversized zip entry: {fname} (>{MAX_EXTRACTED_FILE_BYTES} bytes)"
                );
                let _ = fs::remove_file(&dest);
            }
        }
    }

    // import webapps, skip duplicates
    let existing = load_webapps();
    let mut imported = 0usize;
    let mut duplicates = 0usize;

    for app in imported_apps {
        let is_dup = existing
            .webapps
            .iter()
            .any(|e| e.app_name == app.app_name && e.app_url == app.app_url);
        if is_dup {
            duplicates += 1;
            continue;
        }
        // generate new app_file
        let mut new_app = app;
        new_app.app_file = generate_app_file(&new_app.browser, &new_app.app_url);
        if let Err(e) = create_webapp(&new_app) {
            log::error!("Import webapp {}: {e}", new_app.app_name);
        } else {
            imported += 1;
        }
    }

    Ok((imported, duplicates))
}
