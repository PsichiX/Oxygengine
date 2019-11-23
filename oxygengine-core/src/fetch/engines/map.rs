use crate::fetch::{FetchCancelReason, FetchEngine, FetchProcess, FetchProcessReader, FetchStatus};
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct MapFetchEngine {
    pub map: HashMap<String, Vec<u8>>,
}

impl MapFetchEngine {
    pub fn new(map: HashMap<String, Vec<u8>>) -> Self {
        Self { map }
    }
}

impl FetchEngine for MapFetchEngine {
    fn fetch(&mut self, path: &str) -> Result<Box<dyn FetchProcessReader>, FetchStatus> {
        if let Some(bytes) = self.map.get(path) {
            Ok(Box::new(FetchProcess::new_done(bytes.to_vec())))
        } else {
            Err(FetchStatus::Canceled(FetchCancelReason::Error))
        }
    }
}
