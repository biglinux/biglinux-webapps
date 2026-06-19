use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

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

    let stage = tempfile::tempdir().context("Create export staging directory")?;
    let manifest = serde_json::to_string_pretty(&col.webapps)?;
    fs::write(stage.path().join("webapps.json"), manifest)?;
    let icons_stage = stage.path().join("icons");
    fs::create_dir_all(&icons_stage)?;

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
                fs::copy(icon_path, icons_stage.join(fname))?;
            }
        }
    }

    let output = Command::new("7z")
        .args(["a", "-tzip", "-bd", "-y", "--"])
        .arg(zip_path)
        .arg("webapps.json")
        .arg("icons")
        .current_dir(stage.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .context("spawn 7z export")?;
    ensure_7z_success("export", zip_path, &output)?;
    Ok("ok".into())
}

pub fn import_webapps(zip_path: &Path) -> Result<(usize, usize)> {
    let manifest = read_archive_entry_to_string(zip_path, "webapps.json", MAX_EXTRACTED_FILE_BYTES)
        .context("Read webapps.json from import archive")?;
    let imported_apps: Vec<WebApp> = serde_json::from_str(&manifest)?;

    let icons_dir = config::data_dir().join("icons");
    fs::create_dir_all(&icons_dir)?;
    let icons_canonical = icons_dir.canonicalize()?;
    for entry in list_archive_entries(zip_path)? {
        let name = entry.path;
        if !name.starts_with("icons/") {
            continue;
        }
        let fname = name.strip_prefix("icons/").unwrap_or(&name);
        // strict filename: must be non-empty, no path separators, no ..
        if fname.is_empty() || fname.contains('/') || fname.contains('\\') || fname.contains("..") {
            continue;
        }

        if entry.size > MAX_EXTRACTED_FILE_BYTES {
            log::warn!(
                "Skipped oversized zip entry: {fname} (declared {} bytes)",
                entry.size
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
        let copied =
            copy_archive_entry_capped(zip_path, &name, &mut out, MAX_EXTRACTED_FILE_BYTES)?;
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

#[derive(Debug)]
struct ArchiveEntry {
    path: String,
    size: u64,
}

fn list_archive_entries(zip_path: &Path) -> Result<Vec<ArchiveEntry>> {
    let output = Command::new("7z")
        .args(["l", "-slt", "-bd", "--"])
        .arg(zip_path)
        .stdin(Stdio::null())
        .output()
        .context("spawn 7z list")?;
    ensure_7z_success("list", zip_path, &output)?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    let mut path: Option<String> = None;
    let mut size = 0_u64;
    let mut in_entries = false;
    for line in text.lines() {
        if line == "----------" {
            if !in_entries {
                in_entries = true;
                path = None;
                size = 0;
            } else if let Some(path) = path.take() {
                entries.push(ArchiveEntry { path, size });
                size = 0;
            }
            continue;
        }
        if !in_entries {
            continue;
        }
        if let Some(value) = line.strip_prefix("Path = ") {
            if let Some(previous) = path.replace(value.to_string()) {
                entries.push(ArchiveEntry {
                    path: previous,
                    size,
                });
                size = 0;
            }
        } else if let Some(value) = line.strip_prefix("Size = ") {
            size = value.parse().unwrap_or(0);
        }
    }
    if let Some(path) = path {
        entries.push(ArchiveEntry { path, size });
    }
    Ok(entries)
}

fn read_archive_entry_to_string(zip_path: &Path, entry: &str, max_bytes: u64) -> Result<String> {
    let mut bytes = Vec::new();
    let copied = copy_archive_entry_capped(zip_path, entry, &mut bytes, max_bytes)?;
    if copied > max_bytes {
        anyhow::bail!("archive entry {entry} exceeds {max_bytes} bytes");
    }
    Ok(String::from_utf8(bytes)?)
}

fn copy_archive_entry_capped(
    zip_path: &Path,
    entry: &str,
    out: &mut dyn Write,
    max_bytes: u64,
) -> Result<u64> {
    let mut child = Command::new("7z")
        .args(["x", "-so", "-bd", "--"])
        .arg(zip_path)
        .arg(entry)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn 7z extract")?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture 7z stdout"))?;
    let mut copied = 0_u64;
    let mut buf = [0_u8; 8192];
    loop {
        let read = stdout.read(&mut buf)?;
        if read == 0 {
            break;
        }
        copied = copied.saturating_add(read as u64);
        if copied > max_bytes {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(copied);
        }
        out.write_all(&buf[..read])?;
    }
    drop(stdout);
    let output = child.wait_with_output()?;
    ensure_7z_success("extract", zip_path, &output)?;
    Ok(copied)
}

fn ensure_7z_success(action: &str, path: &Path, output: &std::process::Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    let mut details = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if details.is_empty() {
        details = String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    anyhow::bail!("7z {action} failed for {}: {details}", path.display())
}
