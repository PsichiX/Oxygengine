use crate::assets::{
    asset::{Asset, AssetId},
    protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaAsset {
    #[serde(default)]
    target: Vec<String>,
    #[serde(skip)]
    ids: HashMap<String, AssetId>,
}

impl MetaAsset {
    pub fn with_target(mut self, target: impl ToString) -> Self {
        self.target.push(target.to_string());
        self
    }

    pub fn target(&self) -> impl Iterator<Item = &str> {
        self.target.iter().map(|path| path.as_str())
    }

    pub fn asset_ids(&self) -> impl Iterator<Item = (&str, AssetId)> {
        self.ids.iter().map(|(key, id)| (key.as_str(), *id))
    }

    pub fn asset_id(&self, key: &str) -> Option<AssetId> {
        self.ids.get(key).copied()
    }
}

pub struct MetaAssetProtocol;

impl AssetProtocol for MetaAssetProtocol {
    fn name(&self) -> &str {
        "meta"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let data = if path.ends_with(".asset") {
            let data = from_utf8(&data).unwrap();
            match serde_json::from_str::<MetaAsset>(data) {
                Ok(value) => value,
                Err(error) => {
                    return AssetLoadResult::Error(format!(
                        "Error loading Meta JSON asset: {:?}",
                        error
                    ));
                }
            }
        } else {
            match bincode::deserialize::<MetaAsset>(&data) {
                Ok(value) => value,
                Err(error) => {
                    return AssetLoadResult::Error(format!(
                        "Error loading Meta binary asset: {:?}",
                        error
                    ));
                }
            }
        };
        let list = data
            .target
            .iter()
            .map(|path| (path.to_owned(), path.to_owned()))
            .collect();
        AssetLoadResult::Yield(None, list)
    }

    fn on_resume(&mut self, _: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let target = list
            .iter()
            .map(|(_, asset)| asset.to_full_path())
            .collect::<Vec<_>>();
        let ids = list
            .iter()
            .map(|(key, asset)| (key.to_string(), asset.id()))
            .collect::<HashMap<_, _>>();
        AssetLoadResult::Data(Box::new(MetaAsset { target, ids }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        asset
            .get::<MetaAsset>()
            .map(|asset| asset.ids.values().map(|id| AssetVariant::Id(*id)).collect())
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
