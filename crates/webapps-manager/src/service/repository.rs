use anyhow::{Context, Result};
use fs4::FileExt as Fs4FileExt;
use std::{fs, path::PathBuf};
use webapps_core::{config, models::WebAppCollection, storage::write_atomic};

pub(crate) fn webapps_json_path() -> PathBuf {
    config::data_dir().join("webapps.json")
}

pub(super) struct WebappsLock {
    file: fs::File,
}

impl WebappsLock {
    pub(super) fn acquire() -> Result<Self> {
        fs::create_dir_all(config::data_dir())?;
        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(config::data_dir().join("webapps.json.lock"))?;
        <fs::File as Fs4FileExt>::lock(&file).context("Lock webapps registry")?;
        Ok(Self { file })
    }
}

impl Drop for WebappsLock {
    fn drop(&mut self) {
        let _ = <fs::File as Fs4FileExt>::unlock(&self.file);
    }
}

pub fn load_webapps() -> WebAppCollection {
    try_load_webapps().unwrap_or_else(|err| {
        log::error!("Read webapps registry: {err:#}");
        WebAppCollection::default()
    })
}

pub fn try_load_webapps() -> Result<WebAppCollection> {
    let path = webapps_json_path();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(WebAppCollection::default())
        }
        Err(err) => return Err(err).context("Read webapps.json"),
    };
    let webapps =
        serde_json::from_str(&contents).context("Parse webapps.json; original file preserved")?;
    Ok(WebAppCollection { webapps })
}

pub(super) fn write_collection(collection: &WebAppCollection) -> Result<()> {
    write_atomic(
        &webapps_json_path(),
        &serde_json::to_vec_pretty(&collection.webapps)?,
    )
}

pub fn save_webapps(collection: &WebAppCollection) -> Result<()> {
    let _lock = WebappsLock::acquire()?;
    try_load_webapps()?;
    write_collection(collection)
}
