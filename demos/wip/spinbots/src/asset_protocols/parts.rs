use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Default, Serialize, Deserialize)]
pub struct Parts {
    pub drivers: Vec<String>,
    pub disks: Vec<String>,
    pub layers: Vec<String>,
}

pub struct PartsAsset(Vec<AssetId>);

pub struct PartsAssetProtocol;

impl AssetProtocol for PartsAssetProtocol {
    fn name(&self) -> &str {
        "parts"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        let parts = serde_yaml::from_str::<Parts>(data).unwrap();
        let mut list =
            Vec::with_capacity(2 * (parts.drivers.len() + parts.disks.len() + parts.layers.len()));
        for name in &parts.drivers {
            list.push((
                format!("{}-config", name),
                format!("part://drivers/{}/config.yaml", name),
            ));
            list.push((
                format!("{}-image", name),
                format!("svg://drivers/{}/image.svg", name),
            ));
        }
        for name in &parts.disks {
            list.push((
                format!("{}-config", name),
                format!("part://disks/{}/config.yaml", name),
            ));
            list.push((
                format!("{}-image", name),
                format!("svg://disks/{}/image.svg", name),
            ));
        }
        for name in &parts.layers {
            list.push((
                format!("{}-config", name),
                format!("part://layers/{}/config.yaml", name),
            ));
            list.push((
                format!("{}-image", name),
                format!("svg://layers/{}/image.svg", name),
            ));
        }
        AssetLoadResult::Yield(None, list)
    }

    fn on_resume(&mut self, payload: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let assets = list.iter().map(|(_, asset)| asset.id()).collect();
        AssetLoadResult::Data(Box::new(PartsAsset(assets)))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        asset
            .get::<PartsAsset>()
            .map(|asset| asset.0.iter().map(|id| AssetVariant::Id(*id)).collect())
    }
}
