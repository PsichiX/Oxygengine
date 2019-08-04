pub mod engines;

pub mod prelude {
    pub use super::{engines::prelude::*, engines::*};
}

use crate::id::ID;
use std::{
    mem::replace,
    sync::{Arc, Mutex},
};

pub type FetchProcessID = ID<FetchProcess>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FetchCancelReason {
    User,
    Error,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FetchStatus {
    Empty,
    InProgress(f32),
    Done,
    Canceled(FetchCancelReason),
}

pub trait FetchProcessReader: Send + Sync {
    fn status(&self) -> FetchStatus;
    fn read(&self) -> Option<Vec<u8>>;
    fn box_clone(&self) -> Box<FetchProcessReader>;
}

impl Clone for Box<FetchProcessReader> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct FetchProcess {
    id: FetchProcessID,
    inner: Arc<Mutex<(FetchStatus, Option<Vec<u8>>)>>,
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
            id: FetchProcessID::new(),
            inner: Arc::new(Mutex::new((FetchStatus::Empty, None))),
        }
    }

    #[inline]
    pub fn new_start() -> Self {
        Self {
            id: FetchProcessID::new(),
            inner: Arc::new(Mutex::new((FetchStatus::InProgress(0.0), None))),
        }
    }

    #[inline]
    pub fn new_done(data: Vec<u8>) -> Self {
        Self {
            id: FetchProcessID::new(),
            inner: Arc::new(Mutex::new((FetchStatus::Done, Some(data)))),
        }
    }

    #[inline]
    pub fn new_cancel(reason: FetchCancelReason) -> Self {
        Self {
            id: FetchProcessID::new(),
            inner: Arc::new(Mutex::new((FetchStatus::Canceled(reason), None))),
        }
    }

    #[inline]
    pub fn id(&self) -> FetchProcessID {
        self.id
    }

    pub fn start(&mut self) {
        if let Ok(mut meta) = self.inner.lock() {
            *meta = (FetchStatus::InProgress(0.0), None);
        }
    }

    pub fn progress(&mut self, value: f32) {
        if let Ok(mut meta) = self.inner.lock() {
            *meta = (FetchStatus::InProgress(value), None);
        }
    }

    pub fn done(&mut self, data: Vec<u8>) {
        if let Ok(mut meta) = self.inner.lock() {
            *meta = (FetchStatus::Done, Some(data));
        }
    }

    pub fn cancel(&mut self, reason: FetchCancelReason) {
        if let Ok(mut meta) = self.inner.lock() {
            *meta = (FetchStatus::Canceled(reason), None);
        }
    }

    pub fn readers_count(&self) -> usize {
        Arc::strong_count(&self.inner) + Arc::weak_count(&self.inner) - 1
    }
}

impl FetchProcessReader for FetchProcess {
    fn status(&self) -> FetchStatus {
        if let Ok(meta) = self.inner.lock() {
            meta.0
        } else {
            FetchStatus::Empty
        }
    }

    fn read(&self) -> Option<Vec<u8>> {
        if let Ok(mut meta) = self.inner.lock() {
            if meta.0 == FetchStatus::Done {
                let old: (FetchStatus, Option<Vec<u8>>) =
                    replace(&mut meta, (FetchStatus::Empty, None));
                return old.1;
            }
        }
        None
    }

    fn box_clone(&self) -> Box<FetchProcessReader> {
        Box::new((*self).clone())
    }
}

pub trait FetchEngine: Send + Sync {
    fn fetch(&mut self, path: &str) -> Result<Box<FetchProcessReader>, FetchStatus>;

    fn cancel(&mut self, reader: Box<FetchProcessReader>) {
        #[allow(clippy::cast_ptr_alignment)]
        let ptr = Box::into_raw(reader) as *mut FetchProcess;
        unsafe {
            (*ptr).cancel(FetchCancelReason::User);
            Box::from_raw(ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "web"))]
    fn test_general() {
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
        assert_eq!(reader.status(), FetchStatus::Empty);
        assert_eq!(reader2.status(), FetchStatus::Empty);
        drop(reader);
        drop(reader2);
    }
}
