use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs;
use std::path::{Path, PathBuf};

use webapps_core::config;
use webapps_core::models::WebAppCollection;

pub(crate) fn webapps_json_path() -> PathBuf {
    config::data_dir().join("webapps.json")
}

fn lock_path() -> PathBuf {
    config::data_dir().join("webapps.json.lock")
}

/// RAII guard around a `fs2::FileExt` exclusive lock on `webapps.json.lock`.
///
/// Held across the load → mutate → save sequence so two `big-webapps-gui`
/// instances cannot race and overwrite each other's edits.
struct WebappsLock {
    file: fs::File,
}

impl WebappsLock {
    fn acquire() -> Result<Self> {
        let dir = config::data_dir();
        fs::create_dir_all(&dir).context("Create data dir for webapps lock")?;
        let path = lock_path();
        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("Open lock file {}", path.display()))?;
        // Blocks if another process holds the lock — that is exactly the desired
        // serialisation semantics, and webapps.json edits complete in milliseconds.
        FileExt::lock_exclusive(&file)
            .with_context(|| format!("Acquire exclusive lock on {}", path.display()))?;
        Ok(Self { file })
    }
}

impl Drop for WebappsLock {
    fn drop(&mut self) {
        // Lock is released automatically on close, but be explicit so it is
        // visible to readers and survives a future File handle leak refactor.
        let _ = FileExt::unlock(&self.file);
    }
}

pub fn load_webapps() -> WebAppCollection {
    let path = webapps_json_path();
    if !path.exists() {
        return WebAppCollection::default();
    }
    read_collection(&path)
}

fn read_collection(path: &Path) -> WebAppCollection {
    match fs::read_to_string(path) {
        Ok(data) => match serde_json::from_str::<Vec<serde_json::Value>>(&data) {
            Ok(vals) => WebAppCollection::load_from_json(&vals),
            Err(err) => {
                log::error!("Parse webapps.json: {err}");
                WebAppCollection::default()
            }
        },
        Err(err) => {
            log::error!("Read webapps.json: {err}");
            WebAppCollection::default()
        }
    }
}

pub fn save_webapps(collection: &WebAppCollection) -> Result<()> {
    let dir = config::data_dir();
    fs::create_dir_all(&dir)?;

    // Hold the lock across rename — concurrent saves must not interleave.
    let _lock = WebappsLock::acquire()?;

    let json = serde_json::to_string_pretty(&collection.webapps)?;
    let path = webapps_json_path();
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Atomically load + transform + save with a single lock acquisition.
///
/// Use this for any read-modify-write sequence on `webapps.json`. It guarantees
/// that no other process modifies the file between the read and the write — the
/// previous code performed two separate lock-less operations which could race.
pub fn mutate_webapps<F>(mutator: F) -> Result<()>
where
    F: FnOnce(&mut WebAppCollection) -> Result<()>,
{
    let dir = config::data_dir();
    fs::create_dir_all(&dir)?;
    let _lock = WebappsLock::acquire()?;

    let path = webapps_json_path();
    let mut collection = if path.exists() {
        read_collection(&path)
    } else {
        WebAppCollection::default()
    };
    mutator(&mut collection)?;

    let json = serde_json::to_string_pretty(&collection.webapps)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}
