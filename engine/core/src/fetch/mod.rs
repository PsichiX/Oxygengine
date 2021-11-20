pub mod engines;

use crate::{id::ID, Scalar};
use std::{
    mem::replace,
    sync::{Arc, RwLock},
};

pub type FetchProcessId = ID<FetchProcess>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FetchCancelReason {
    User,
    Error,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FetchStatus {
    Empty,
    InProgress(Scalar),
    Done,
    Canceled(FetchCancelReason),
    Read,
}

impl Default for FetchStatus {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Clone)]
pub struct FetchProcess {
    id: FetchProcessId,
    inner: Arc<RwLock<(FetchStatus, Option<Vec<u8>>)>>,
}

impl Default for FetchProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl FetchProcess {
    #[inline]
    pub fn new() -> Self {
        Self {
            id: FetchProcessId::new(),
            inner: Arc::new(RwLock::new((FetchStatus::Empty, None))),
        }
    }

    #[inline]
    pub fn new_start() -> Self {
        Self {
            id: FetchProcessId::new(),
            inner: Arc::new(RwLock::new((FetchStatus::InProgress(0.0), None))),
        }
    }

    #[inline]
    pub fn new_done(data: Vec<u8>) -> Self {
        Self {
            id: FetchProcessId::new(),
            inner: Arc::new(RwLock::new((FetchStatus::Done, Some(data)))),
        }
    }

    #[inline]
    pub fn new_cancel(reason: FetchCancelReason) -> Self {
        Self {
            id: FetchProcessId::new(),
            inner: Arc::new(RwLock::new((FetchStatus::Canceled(reason), None))),
        }
    }

    #[inline]
    pub fn id(&self) -> FetchProcessId {
        self.id
    }

    pub fn status(&self) -> FetchStatus {
        self.inner.read().map(|meta| meta.0).unwrap_or_default()
    }

    pub fn start(&mut self) {
        if let Ok(mut meta) = self.inner.write() {
            *meta = (FetchStatus::InProgress(0.0), None);
        }
    }

    pub fn progress(&mut self, value: Scalar) {
        if let Ok(mut meta) = self.inner.write() {
            *meta = (FetchStatus::InProgress(value), None);
        }
    }

    pub fn done(&mut self, data: Vec<u8>) {
        if let Ok(mut meta) = self.inner.write() {
            *meta = (FetchStatus::Done, Some(data));
        }
    }

    pub fn cancel(&mut self, reason: FetchCancelReason) {
        if let Ok(mut meta) = self.inner.write() {
            *meta = (FetchStatus::Canceled(reason), None);
        }
    }

    pub fn readers_count(&self) -> usize {
        Arc::strong_count(&self.inner) + Arc::weak_count(&self.inner) - 1
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        if let Ok(mut meta) = self.inner.write() {
            if meta.0 == FetchStatus::Done {
                let old: (FetchStatus, Option<Vec<u8>>) =
                    replace(&mut meta, (FetchStatus::Read, None));
                return old.1;
            }
        }
        None
    }

    pub fn byte_size(&self) -> Option<usize> {
        if let Ok(meta) = self.inner.read() {
            if meta.0 == FetchStatus::Done {
                if let Some(bytes) = meta.1.as_ref() {
                    return Some(bytes.len());
                }
            }
        }
        None
    }
}

pub trait FetchEngine: Send + Sync {
    fn fetch(&mut self, path: &str) -> Result<Box<FetchProcess>, FetchStatus>;

    fn cancel(&mut self, mut reader: FetchProcess) {
        reader.cancel(FetchCancelReason::User)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(not(feature = "web"))]
    fn test_fetch() {
        let mut engine = engines::fs::FsFetchEngine::new(&".");
        let reader = engine.fetch("Cargo.toml").unwrap();
        let reader2 = reader.clone();
        #[cfg(feature = "parallel")]
        {
            assert_eq!(reader.status(), FetchStatus::InProgress(0.0));
            assert_eq!(reader2.status(), FetchStatus::InProgress(0.0));
        }
        loop {
            match reader.status() {
                FetchStatus::InProgress(_) => continue,
                _ => break,
            }
        }
        assert_eq!(reader.status(), FetchStatus::Done);
        assert_eq!(reader2.status(), FetchStatus::Done);
        assert!(!reader.read().unwrap().is_empty());
        assert_eq!(reader.status(), FetchStatus::Read);
        assert_eq!(reader2.status(), FetchStatus::Read);
    }
}
