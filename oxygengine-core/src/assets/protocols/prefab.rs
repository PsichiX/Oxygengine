use crate::{
    assets::{
        asset::Asset,
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
    },
    prefab::{Prefab, PrefabScene},
};
use std::{any::Any, str::from_utf8};

pub struct PrefabAsset(PrefabScene);

impl PrefabAsset {
    pub fn get(&self) -> &PrefabScene {
        &self.0
    }
}

pub struct PrefabAssetProtocol;

impl AssetProtocol for PrefabAssetProtocol {
    fn name(&self) -> &str {
        "prefab"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        match PrefabScene::from_prefab_str(from_utf8(&data).unwrap()) {
            Ok(data) => {
                if data.dependencies.is_empty() {
                    AssetLoadResult::Data(Box::new(PrefabAsset(data)))
                } else {
                    let list = data
                        .dependencies
                        .iter()
                        .enumerate()
                        .map(|(i, path)| (i.to_string(), path.to_owned()))
                        .collect::<Vec<_>>();
                    AssetLoadResult::Yield(Some(Box::new(data)), list)
                }
            }
            Err(error) => {
                AssetLoadResult::Error(format!("Error loading prefab asset: {:?}", error))
            }
        }
    }

    fn on_resume(&mut self, payload: Meta, _: &[(&str, &Asset)]) -> AssetLoadResult {
        let prefab = *(payload.unwrap() as Box<dyn Any + Send>)
            .downcast::<PrefabScene>()
            .unwrap();
        AssetLoadResult::Data(Box::new(PrefabAsset(prefab)))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        if let Some(asset) = asset.get::<PrefabAsset>() {
            Some(
                asset
                    .get()
                    .dependencies
                    .iter()
                    .map(|path| AssetVariant::Path(path.to_owned()))
                    .collect(),
            )
        } else {
            None
        }
    }
}
