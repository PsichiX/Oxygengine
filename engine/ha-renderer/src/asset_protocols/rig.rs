use crate::mesh::rig::{
    control::RigControl,
    deformer::Deformer,
    skeleton::{Skeleton, SkeletonError, SkeletonHierarchy},
    Rig,
};
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    scripting::{
        intuicio::core::registry::Registry, ScriptFunctionReference, ScriptStructReference,
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RigAssetError {
    Skeleton(SkeletonError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigAssetControl {
    pub struct_type: ScriptStructReference,
    pub init_function: Option<ScriptFunctionReference>,
    pub solve_function: Option<ScriptFunctionReference>,
    /// {field name: field string value}
    pub bindings: HashMap<String, String>,
}

impl RigAssetControl {
    pub fn new(struct_type: ScriptStructReference) -> Self {
        Self {
            struct_type,
            init_function: None,
            solve_function: None,
            bindings: Default::default(),
        }
    }

    pub fn init_function(mut self, function: ScriptFunctionReference) -> Self {
        self.init_function = Some(function);
        self
    }

    pub fn solve_function(mut self, function: ScriptFunctionReference) -> Self {
        self.solve_function = Some(function);
        self
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
                let struct_type = registry.find_struct(control.struct_type.query())?;
                let init_function = control
                    .init_function
                    .as_ref()
                    .and_then(|function| registry.find_function(function.query()))
                    .or_else(|| registry.find_function(control.struct_type.method("init").query()));
                let solve_function = control
                    .solve_function
                    .as_ref()
                    .and_then(|function| registry.find_function(function.query()))
                    .or_else(|| {
                        registry.find_function(control.struct_type.method("solve").query())
                    })?;
                Some(RigControl {
                    struct_type,
                    init_function,
                    solve_function,
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
