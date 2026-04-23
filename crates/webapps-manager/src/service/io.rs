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
        if !name.starts_with("icons/") {
            continue;
        }
        let fname = name.strip_prefix("icons/").unwrap_or(&name);
        // strict filename: must be non-empty, no path separators, no ..
        if fname.is_empty() || fname.contains('/') || fname.contains('\\') || fname.contains("..") {
            continue;
        }

        // Bail before creating the file when the declared size already exceeds
        // the cap. Avoids the `create + truncate + remove` round-trip on
        // decompression-bomb archives.
        if entry.size() > MAX_EXTRACTED_FILE_BYTES {
            log::warn!(
                "Skipped oversized zip entry: {fname} (declared {} bytes)",
                entry.size()
            );
            continue;
        }

        let dest = icons_dir.join(fname);
        // Verify dest stays within icons_dir. Failure to canonicalize → DENY.
        // The previous behaviour silently allowed extraction when canonicalize
        // failed (e.g. transient FS error), defeating the path-escape defence.
        let parent = dest.parent().ok_or_else(|| {
            anyhow::anyhow!("Refusing import: zip entry {fname} has no parent directory")
        })?;
        let parent_canonical = parent
            .canonicalize()
            .with_context(|| format!("Refusing import: cannot canonicalize parent of {fname}"))?;
        if parent_canonical != icons_canonical {
            log::warn!(
                "Refusing import of {fname}: would escape icons dir (target parent: {})",
                parent_canonical.display()
            );
            continue;
        }

        let mut out = fs::File::create(&dest)?;
        // Defence-in-depth: even if entry.size() lied, cap the actual copy.
        let copied = std::io::copy(&mut entry.by_ref().take(MAX_EXTRACTED_FILE_BYTES), &mut out)?;
        if copied >= MAX_EXTRACTED_FILE_BYTES {
            log::warn!(
                "Truncated oversized zip entry post-decompression: {fname} (>{MAX_EXTRACTED_FILE_BYTES} bytes)"
            );
            let _ = fs::remove_file(&dest);
        }
    }

    // import webapps, skip duplicates
    let existing = load_webapps();
    let mut seen = existing
        .webapps
        .iter()
        .map(|app| (app.app_name.clone(), app.app_url.clone()))
        .collect::<std::collections::HashSet<_>>();
    let mut imported = 0usize;
    let mut duplicates = 0usize;

    for app in imported_apps {
        let dedupe_key = (app.app_name.clone(), app.app_url.clone());
        if seen.contains(&dedupe_key) {
            duplicates += 1;
            continue;
        }

        // generate new app_file
        let mut new_app = app;
        new_app.app_file = generate_app_file(&new_app.browser, &new_app.app_url);
        if let Err(e) = create_webapp(&new_app) {
            log::error!("Import webapp {}: {e}", new_app.app_name);
        } else {
            seen.insert(dedupe_key);
            imported += 1;
        }
    }

    Ok((imported, duplicates))
}
