use crate::storage::{StorageEngine, StorageError, StorageResult};
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct MapStorageEngine {
    pub map: HashMap<String, Vec<u8>>,
}

impl MapStorageEngine {
    pub fn new(map: HashMap<String, Vec<u8>>) -> Self {
        Self { map }
    }
}

impl StorageEngine for MapStorageEngine {
    fn load(&mut self, path: &str) -> StorageResult<Vec<u8>> {
        if let Some(bytes) = self.map.get(path) {
            Ok(bytes.to_vec())
        } else {
            Err(StorageError::CouldNotLoadData(path.to_owned()))
        }
    }

    fn store(&mut self, path: &str, data: &[u8]) -> StorageResult<()> {
        self.map.insert(path.to_owned(), data.to_vec());
        Ok(())
    }
}
