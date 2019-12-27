use crate::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    fetch::engines::map::MapFetchEngine,
};
use std::collections::HashMap;

pub struct PackAsset(HashMap<String, Vec<u8>>);

impl PackAsset {
    pub fn get_asset_data(&self, path: &str) -> Option<&[u8]> {
        self.0.get(path).map(|d| d.as_ref())
    }

    pub fn make_fetch_engine(&self) -> MapFetchEngine {
        MapFetchEngine::new(self.0.clone())
    }
}

pub struct PackAssetProtocol;

impl AssetProtocol for PackAssetProtocol {
    fn name(&self) -> &str {
        "pack"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        match bincode::deserialize(&data) {
            Ok(data) => AssetLoadResult::Data(Box::new(PackAsset(data))),
            Err(_) => AssetLoadResult::None,
        }
    }
}
