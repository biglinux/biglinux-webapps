use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use webapps_core::models::WebApp;

use super::{create_webapp, try_load_webapps};

/// Max size per extracted file from import zip → prevent decompression bombs
const MAX_EXTRACTED_FILE_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

pub fn export_webapps(zip_path: &Path) -> Result<String> {
    let mut col = try_load_webapps()?;
    if col.webapps.is_empty() {
        return Ok("no_webapps".into());
    }

    let stage = archive_staging()?;
    let icons_stage = stage.path().join("icons");
    fs::create_dir_all(&icons_stage)?;
    for (index, app) in col.webapps.iter_mut().enumerate() {
        let icon_path = Path::new(&app.app_icon);
        if !icon_path.is_absolute() {
            continue;
        }
        let extension = icon_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        let relative = format!("icons/{index}.{extension}");
        fs::copy(icon_path, stage.path().join(&relative)).context("Export saved icon")?;
        app.app_icon = relative.clone();
        app.app_icon_url = relative;
    }
    let manifest = serde_json::to_string_pretty(&col.webapps)?;
    fs::write(stage.path().join("webapps.json"), manifest)?;
    let archive = stage.path().join("export.zip");
    let output = archive_command()
        .args(["a", "-tzip", "-bd", "-y", "--"])
        .arg(&archive)
        .arg("webapps.json")
        .arg("icons")
        .current_dir(stage.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .context("spawn 7z export")?;
    ensure_7z_success("export", zip_path, &output)?;
    webapps_core::storage::write_atomic(zip_path, &fs::read(&archive)?)?;
    Ok("ok".into())
}

pub fn import_webapps(zip_path: &Path) -> Result<(usize, usize)> {
    let manifest = read_archive_entry_to_string(zip_path, "webapps.json", MAX_EXTRACTED_FILE_BYTES)
        .context("Read webapps.json from import archive")?;
    let imported_apps: Vec<WebApp> = serde_json::from_str(&manifest)?;

    anyhow::ensure!(
        imported_apps.len() <= 1000,
        "Archive contains too many webapps"
    );
    let stage = archive_staging()?;
    let entries = list_archive_entries(zip_path)?;
    anyhow::ensure!(entries.len() <= 2000, "Archive contains too many entries");
    let total: u64 = entries
        .iter()
        .map(|entry| entry.size)
        .try_fold(0u64, u64::checked_add)
        .ok_or_else(|| anyhow::anyhow!("Archive size overflow"))?;
    anyhow::ensure!(
        total <= 200 * 1024 * 1024,
        "Archive exceeds total size limit"
    );
    let mut extracted = std::collections::HashMap::new();
    for entry in entries {
        let Some(name) = entry.path.strip_prefix("icons/") else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        anyhow::ensure!(
            !name.contains(['/', '\\', '*', '?', '[', ']']) && !name.contains(".."),
            "Unsafe icon archive path"
        );
        anyhow::ensure!(
            entry.size <= MAX_EXTRACTED_FILE_BYTES,
            "Oversized archive icon"
        );
        anyhow::ensure!(!extracted.contains_key(name), "Duplicate archive icon");
        let dest = stage.path().join(name);
        let mut output = fs::File::create(&dest)?;
        let count = copy_archive_entry_capped(
            zip_path,
            &entry.path,
            &mut output,
            MAX_EXTRACTED_FILE_BYTES,
        )?;
        anyhow::ensure!(
            count <= MAX_EXTRACTED_FILE_BYTES,
            "Oversized extracted icon"
        );
        extracted.insert(name.to_owned(), dest);
    }
    let existing = try_load_webapps()?;
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
        new_app.app_file.clear();
        let icon = Path::new(&new_app.app_icon);
        if icon.is_absolute() || new_app.app_icon.starts_with("icons/") {
            let name = icon
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let restored = extracted
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("Missing archived icon: {name}"))?;
            new_app.app_icon = restored.to_string_lossy().into_owned();
            new_app.app_icon_url = new_app.app_icon.clone();
        }
        if let Err(e) = create_webapp(&new_app) {
            return Err(e).with_context(|| {
                format!(
                    "Import {} after {imported} successful entries",
                    new_app.app_name
                )
            });
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
    let output = archive_command()
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
    let mut child = archive_command()
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

fn archive_staging() -> Result<tempfile::TempDir> {
    let cache = webapps_core::config::cache_dir();
    fs::create_dir_all(&cache)?;
    Ok(tempfile::tempdir_in(cache)?)
}

fn archive_command() -> Command {
    let program = if std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|path| path.join("7z").is_file()))
    {
        "7z"
    } else {
        "7zz"
    };
    Command::new(program)
}
