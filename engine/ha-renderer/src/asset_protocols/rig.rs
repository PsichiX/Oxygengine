use crate::mesh::rig::{
    deformer::Deformer,
    skeleton::{SkeletonError, SkeletonHierarchy},
    Rig,
};
use core::assets::protocol::{AssetLoadResult, AssetProtocol};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RigAssetError {
    Skeleton(SkeletonError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigAsset {
    skeleton: SkeletonHierarchy,
    #[serde(default)]
    deformer: Deformer,
}

impl RigAsset {
    pub fn new(skeleton: SkeletonHierarchy, deformer: Deformer) -> Self {
        Self { skeleton, deformer }
    }

    pub fn skeleton(&self) -> &SkeletonHierarchy {
        &self.skeleton
    }

    pub fn deformer(&self) -> &Deformer {
        &self.deformer
    }

    pub fn rig(&self) -> Result<Rig, RigAssetError> {
        Ok(Rig {
            skeleton: self
                .skeleton
                .to_owned()
                .try_into()
                .map_err(|error| RigAssetError::Skeleton(error))?,
            deformer: self.deformer.to_owned(),
        })
    }
}

pub struct RigAssetProtocol;

impl AssetProtocol for RigAssetProtocol {
    fn name(&self) -> &str {
        "rig"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let data = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<RigAsset>(data).unwrap()
        } else {
            bincode::deserialize::<RigAsset>(&data).unwrap()
        };
        AssetLoadResult::Data(Box::new(data))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
