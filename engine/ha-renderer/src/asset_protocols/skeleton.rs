use crate::mesh::skeleton::SkeletonHierarchy;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonAsset(SkeletonHierarchy);

impl SkeletonAsset {
    pub fn new(hierarchy: SkeletonHierarchy) -> Self {
        Self(hierarchy)
    }

    pub fn get(&self) -> &SkeletonHierarchy {
        &self.0
    }
}

pub struct SkeletonAssetProtocol;

impl AssetProtocol for SkeletonAssetProtocol {
    fn name(&self) -> &str {
        "skeleton"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let data = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<SkeletonAsset>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<SkeletonAsset>(data).unwrap()
        } else {
            bincode::deserialize::<SkeletonAsset>(&data).unwrap()
        };
        AssetLoadResult::Data(Box::new(data))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
