#![cfg(not(feature = "web"))]

use crate::storage::{StorageEngine, StorageError, StorageResult};
use std::{
    env::var,
    fs::{read, write},
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct FsStorageEngine {
    pub root: PathBuf,
}

impl Default for FsStorageEngine {
    fn default() -> Self {
        Self {
            root: match var("OXY_STORAGE_ENGINE_PATH") {
                Ok(value) => value.into(),
                Err(_) => Default::default(),
            },
        }
    }
}

impl FsStorageEngine {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            root: match var("OXY_STORAGE_ENGINE_PATH") {
                Ok(value) => value.into(),
                Err(_) => path.as_ref().into(),
            },
        }
    }
}

impl StorageEngine for FsStorageEngine {
    fn load(&mut self, path: &str) -> StorageResult<Vec<u8>> {
        let path = self.root.join(path);
        match read(path) {
            Ok(data) => Ok(data),
            Err(error) => Err(StorageError::CouldNotLoadData(error.to_string())),
        }
    }

    fn store(&mut self, path: &str, data: &[u8]) -> StorageResult<()> {
        let path = self.root.join(path);
        match write(path, data) {
            Ok(_) => Ok(()),
            Err(error) => Err(StorageError::CouldNotStoreData(error.to_string())),
        }
    }
}
