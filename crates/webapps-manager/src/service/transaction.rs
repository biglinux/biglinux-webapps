use super::repository::{try_load_webapps, write_collection, WebappsLock};
use anyhow::{Context, Result};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use webapps_core::{models::WebAppCollection, storage::write_atomic};

pub(super) struct RegistryTransaction {
    pub collection: WebAppCollection,
    originals: BTreeMap<PathBuf, Option<Vec<u8>>>,
    committed: bool,
    renames: Vec<(PathBuf, PathBuf)>,
    _lock: WebappsLock,
}

impl RegistryTransaction {
    pub fn begin() -> Result<Self> {
        let lock = WebappsLock::acquire()?;
        Ok(Self {
            collection: try_load_webapps()?,
            originals: BTreeMap::new(),
            committed: false,
            renames: Vec::new(),
            _lock: lock,
        })
    }

    pub fn capture(&mut self, path: &Path) -> Result<()> {
        if self.originals.contains_key(path) {
            return Ok(());
        }
        let original = match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => return Err(err).with_context(|| format!("Snapshot {}", path.display())),
        };
        self.originals.insert(path.to_path_buf(), original);
        Ok(())
    }

    pub fn write(&mut self, path: &Path, bytes: &[u8]) -> Result<()> {
        self.capture(path)?;
        write_atomic(path, bytes)
    }

    pub fn remove(&mut self, path: &Path) -> Result<()> {
        self.capture(path)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn rename_unclaimed(&mut self, original: &Path, destination: &Path) -> Result<()> {
        if !original.exists() || destination.exists() {
            return Ok(());
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(original, destination)?;
        self.renames
            .push((original.to_path_buf(), destination.to_path_buf()));
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        write_collection(&self.collection)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for RegistryTransaction {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for (original, destination) in self.renames.iter().rev() {
            if let Err(error) = fs::rename(destination, original) {
                log::error!("Restore {}: {error}", original.display());
            }
        }
        for (path, original) in &self.originals {
            let result = match original {
                Some(bytes) => write_atomic(path, bytes),
                None => match fs::remove_file(path) {
                    Ok(()) => Ok(()),
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(err) => Err(err.into()),
                },
            };
            if let Err(err) = result {
                log::error!("Restore {}: {err:#}", path.display());
            }
        }
    }
}
