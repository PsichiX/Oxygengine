use core::storage::{StorageEngine, StorageError, StorageResult};
use std::fmt::Write;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

#[derive(Default, Copy, Clone)]
pub struct WebStorageEngine;

impl StorageEngine for WebStorageEngine {
    fn load(&mut self, path: &str) -> StorageResult<Vec<u8>> {
        if let Ok(Some(storage)) = window().local_storage() {
            if let Ok(Some(data)) = storage.get_item(path) {
                Self::str_to_bytes(&data)
            } else {
                Err(StorageError::CouldNotLoadData(format!(
                    "Could not load data from: {}",
                    path
                )))
            }
        } else {
            Err(StorageError::CouldNotLoadData(format!(
                "Could not load data from: {}",
                path
            )))
        }
    }

    fn store(&mut self, path: &str, data: &[u8]) -> StorageResult<()> {
        if let Ok(Some(storage)) = window().local_storage() {
            if storage
                .set_item(path, &Self::bytes_to_string(data)?)
                .is_ok()
            {
                Ok(())
            } else {
                Err(StorageError::CouldNotStoreData(format!(
                    "Could not store data to: {}",
                    path
                )))
            }
        } else {
            Err(StorageError::CouldNotStoreData(format!(
                "Could not store data to: {}",
                path
            )))
        }
    }
}

impl WebStorageEngine {
    fn str_to_bytes(data: &str) -> StorageResult<Vec<u8>> {
        let mut result = Vec::with_capacity(data.len() / 2);
        for i in 0..data.len() / 2 {
            let chunk = &data[(i * 2)..(i * 2 + 2)];
            if let Ok(byte) = u8::from_str_radix(chunk, 16) {
                result.push(byte);
            } else {
                return Err(StorageError::CouldNotStoreData(format!(
                    "Could not convert to byte: {}",
                    chunk
                )));
            }
        }
        Ok(result)
    }

    fn bytes_to_string(data: &[u8]) -> StorageResult<String> {
        let mut result = String::with_capacity(data.len() * 2);
        for byte in data {
            if write!(&mut result, "{:02X}", byte).is_err() {
                return Err(StorageError::CouldNotStoreData(format!(
                    "Could not convert byte: {}",
                    byte
                )));
            }
        }
        Ok(result)
    }
}
