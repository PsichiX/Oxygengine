use crate::assets::{
    asset::{Asset, AssetId},
    protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
};
use std::str::from_utf8;

pub struct SetAsset {
    paths: Vec<String>,
    ids: Vec<AssetId>,
}

impl SetAsset {
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    pub fn ids(&self) -> &[AssetId] {
        &self.ids
    }
}

pub struct SetAssetProtocol;

impl AssetProtocol for SetAssetProtocol {
    fn name(&self) -> &str {
        "set"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap().to_owned();
        let list = data
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let path = line.trim();
                if path.is_empty() || path.starts_with('#') {
                    None
                } else {
                    Some((i.to_string(), path.to_owned()))
                }
            })
            .collect::<Vec<_>>();
        AssetLoadResult::Yield(None, list)
    }

    fn on_resume(&mut self, _: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let paths = list
            .iter()
            .map(|(_, asset)| asset.to_full_path())
            .collect::<Vec<_>>();
        let ids = list.iter().map(|(_, asset)| asset.id()).collect::<Vec<_>>();
        AssetLoadResult::Data(Box::new(SetAsset { paths, ids }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        asset
            .get::<SetAsset>()
            .map(|asset| asset.ids().iter().map(|id| AssetVariant::Id(*id)).collect())
    }
}
