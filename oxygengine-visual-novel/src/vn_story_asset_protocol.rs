use crate::story::Story;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    prefab::Prefab,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VnStoryAsset(Story);

impl VnStoryAsset {
    pub fn get(&self) -> &Story {
        &self.0
    }
}

pub struct VnStoryAssetProtocol;

impl AssetProtocol for VnStoryAssetProtocol {
    fn name(&self) -> &str {
        "vn-story"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match Story::from_prefab_str(data) {
            Ok(result) => AssetLoadResult::Data(Box::new(VnStoryAsset(result))),
            Err(error) => AssetLoadResult::Error(format!(
                "Error loading visual novel story asset: {:?}",
                error
            )),
        }
    }
}
