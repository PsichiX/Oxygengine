use crate::material::{
    common::{BakedMaterialShaders, MaterialSignature, MaterialValue},
    graph::{function::MaterialFunction, MaterialGraph},
    MaterialDrawOptions,
};
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct BakedMaterialAsset {
    pub signature: MaterialSignature,
    pub baked: BakedMaterialShaders,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum MaterialAsset {
    None,
    Graph {
        #[serde(default)]
        default_values: HashMap<String, MaterialValue>,
        #[serde(default)]
        draw_options: MaterialDrawOptions,
        content: MaterialGraph,
    },
    Domain(MaterialGraph),
    Baked {
        #[serde(default)]
        default_values: HashMap<String, MaterialValue>,
        #[serde(default)]
        draw_options: MaterialDrawOptions,
        content: Vec<BakedMaterialAsset>,
    },
    Function(MaterialFunction),
}

impl Default for MaterialAsset {
    fn default() -> Self {
        Self::None
    }
}

pub struct MaterialAssetProtocol;

impl AssetProtocol for MaterialAssetProtocol {
    fn name(&self) -> &str {
        "material"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let material = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<MaterialAsset>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<MaterialAsset>(data).unwrap()
        } else {
            bincode::deserialize::<MaterialAsset>(&data).unwrap()
        };
        AssetLoadResult::Data(Box::new(material))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
