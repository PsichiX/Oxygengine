pub mod engines;

#[derive(Debug, Clone)]
pub enum StorageError {
    /// path to resource.
    CouldNotLoadData(String),
    /// path to resource.
    CouldNotStoreData(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

pub trait StorageEngine: Send + Sync {
    fn load(&mut self, path: &str) -> StorageResult<Vec<u8>>;
    fn store(&mut self, path: &str, data: &[u8]) -> StorageResult<()>;
}
