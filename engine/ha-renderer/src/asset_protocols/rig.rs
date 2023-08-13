use crate::mesh::rig::{
    control::RigControl,
    deformer::Deformer,
    skeleton::{Skeleton, SkeletonError, SkeletonHierarchy},
    Rig,
};
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    scripting::{intuicio::core::registry::Registry, ScriptFunctionReference},
};
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
    #[serde(default)]
    control: Option<ScriptFunctionReference>,
}

impl RigAsset {
    pub fn new(
        skeleton: SkeletonHierarchy,
        deformer: Deformer,
        control: Option<ScriptFunctionReference>,
    ) -> Self {
        Self {
            skeleton,
            deformer,
            control,
        }
    }

    pub fn skeleton(&self) -> &SkeletonHierarchy {
        &self.skeleton
    }

    pub fn deformer(&self) -> &Deformer {
        &self.deformer
    }

    pub fn control(&self) -> Option<&ScriptFunctionReference> {
        self.control.as_ref()
    }

    pub fn build_skeleton(&self) -> Result<Skeleton, RigAssetError> {
        self.skeleton
            .to_owned()
            .try_into()
            .map_err(RigAssetError::Skeleton)
    }

    pub fn build_control(&self, registry: &Registry) -> Option<RigControl> {
        self.control
            .as_ref()
            .and_then(|control| registry.find_function(control.query()))
            .map(|function| RigControl { function })
    }

    pub fn build_rig(&self, registry: &Registry) -> Result<Rig, RigAssetError> {
        Ok(Rig {
            skeleton: self.build_skeleton()?,
            deformer: self.deformer.to_owned(),
            control: self.build_control(registry),
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
