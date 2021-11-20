use crate::math::*;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

pub type TileMapCoord = (usize, usize);

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TileMapAsset {
    #[serde(default)]
    pub x: isize,
    #[serde(default)]
    pub y: isize,
    pub cols: usize,
    pub rows: usize,
    pub cell_size: Vec2,
    #[serde(default)]
    pub values: Vec<usize>,
}

impl TileMapAsset {
    pub fn coord(&self, index: usize) -> TileMapCoord {
        (index % self.cols, index / self.cols)
    }

    pub fn index(&self, coord: TileMapCoord) -> usize {
        coord.0 + coord.1 * self.cols
    }

    pub fn value_at(&self, coord: TileMapCoord) -> Option<usize> {
        if coord.0 >= self.cols || coord.1 >= self.rows {
            return None;
        }
        self.values.get(self.index(coord)).copied()
    }
}

pub struct TileMapAssetProtocol;

impl AssetProtocol for TileMapAssetProtocol {
    fn name(&self) -> &str {
        "tilemap"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let data = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<TileMapAsset>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<TileMapAsset>(data).unwrap()
        } else {
            bincode::deserialize::<TileMapAsset>(&data).unwrap()
        };
        AssetLoadResult::Data(Box::new(data))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
