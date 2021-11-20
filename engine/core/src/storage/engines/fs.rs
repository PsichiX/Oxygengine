#![cfg(not(feature = "web"))]

use crate::storage::{StorageEngine, StorageError, StorageResult};
use std::{
    fs::{read, write},
    path::{Path, PathBuf},
};

#[derive(Default, Clone)]
pub struct FsStorageEngine {
    pub root: PathBuf,
}

impl FsStorageEngine {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            root: path.as_ref().to_path_buf(),
        }
    }
}

impl StorageEngine for FsStorageEngine {
    fn load(&mut self, path: &str) -> StorageResult<Vec<u8>> {
        let path = self.root.join(path);
        match read(&path) {
            Ok(data) => Ok(data),
            Err(error) => Err(StorageError::CouldNotLoadData(error.to_string())),
        }
    }

    fn store(&mut self, path: &str, data: &[u8]) -> StorageResult<()> {
        let path = self.root.join(path);
        match write(&path, data) {
            Ok(_) => Ok(()),
            Err(error) => Err(StorageError::CouldNotStoreData(error.to_string())),
        }
    }
}
