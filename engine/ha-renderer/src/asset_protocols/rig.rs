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
use std::{collections::HashMap, str::from_utf8};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RigAssetError {
    Skeleton(SkeletonError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigAssetControl {
    pub control: ScriptFunctionReference,
    /// {field name: field string value}
    pub bindings: HashMap<String, String>,
}

impl RigAssetControl {
    pub fn new(control: ScriptFunctionReference) -> Self {
        Self {
            control,
            bindings: Default::default(),
        }
    }

    pub fn binding(mut self, field_name: impl ToString, field_value: impl ToString) -> Self {
        self.bindings
            .insert(field_name.to_string(), field_value.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigAsset {
    skeleton: SkeletonHierarchy,
    #[serde(default)]
    deformer: Deformer,
    #[serde(default)]
    controls: Vec<RigAssetControl>,
}

impl RigAsset {
    pub fn new(
        skeleton: SkeletonHierarchy,
        deformer: Deformer,
        controls: Vec<RigAssetControl>,
    ) -> Self {
        Self {
            skeleton,
            deformer,
            controls,
        }
    }

    pub fn skeleton(&self) -> &SkeletonHierarchy {
        &self.skeleton
    }

    pub fn deformer(&self) -> &Deformer {
        &self.deformer
    }

    pub fn controls(&self) -> &[RigAssetControl] {
        &self.controls
    }

    pub fn build_skeleton(&self) -> Result<Skeleton, RigAssetError> {
        self.skeleton
            .to_owned()
            .try_into()
            .map_err(RigAssetError::Skeleton)
    }

    pub fn build_controls(&self, registry: &Registry) -> Vec<RigControl> {
        self.controls
            .iter()
            .filter_map(|control| {
                let function = registry.find_function(control.control.query())?;
                let struct_type = function.signature().struct_handle.as_ref()?.clone();
                Some(RigControl {
                    struct_type,
                    function,
                    bindings: control.bindings.to_owned(),
                })
            })
            .collect()
    }

    pub fn build_rig(&self, registry: &Registry) -> Result<Rig, RigAssetError> {
        Ok(Rig {
            skeleton: self.build_skeleton()?,
            deformer: self.deformer.to_owned(),
            controls: self.build_controls(registry),
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
