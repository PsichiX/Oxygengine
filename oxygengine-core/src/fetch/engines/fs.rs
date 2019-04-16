#![cfg(not(feature = "web"))]

use crate::fetch::{FetchCancelReason, FetchEngine, FetchProcess, FetchProcessReader, FetchStatus};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FsFetchEngine {
    root_path: PathBuf,
}

impl FsFetchEngine {
    pub fn new<S: AsRef<Path>>(root_path: &S) -> Self {
        Self {
            root_path: root_path.as_ref().into(),
        }
    }
}

impl FetchEngine for FsFetchEngine {
    fn fetch(&mut self, path: &str) -> Result<Box<FetchProcessReader>, FetchStatus> {
        #[cfg(feature = "parallel")]
        {
            let path = self.root_path.join(path);
            let process = FetchProcess::new_start();
            let mut p = process.clone();
            rayon::spawn(move || {
                if let Ok(bytes) = std::fs::read(path) {
                    p.done(bytes);
                } else {
                    p.cancel(FetchCancelReason::Error);
                }
            });
            Ok(Box::new(process))
        }
        #[cfg(not(feature = "parallel"))]
        {
            if let Ok(bytes) = std::fs::read(self.root_path.join(path)) {
                Ok(Box::new(FetchProcess::new_done(bytes)))
            } else {
                Err(FetchStatus::Canceled(FetchCancelReason::Error))
            }
        }
    }
}
