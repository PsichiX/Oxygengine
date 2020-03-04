use crate::math::Rect;
use core::{
    assets::{
        asset::{Asset, AssetID},
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
    },
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::{any::Any, str::from_utf8};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetInfo {
    pub image: String,
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
    #[serde(default)]
    #[serde(alias = "tileWidth")]
    pub tile_width: Scalar,
    #[serde(default)]
    #[serde(alias = "tileHeight")]
    pub tile_height: Scalar,
    #[serde(default)]
    #[serde(alias = "paddingCol")]
    pub padding_col: Scalar,
    #[serde(default)]
    #[serde(alias = "paddingRow")]
    pub padding_row: Scalar,
    #[serde(default)]
    #[serde(alias = "marginCol")]
    pub margin_col: Scalar,
    #[serde(default)]
    #[serde(alias = "marginRow")]
    pub margin_row: Scalar,
}

impl TilesetInfo {
    pub fn image_name(&self) -> String {
        let parts = self.image.split("://").collect::<Vec<_>>();
        if parts.len() > 1 {
            parts[1].to_owned()
        } else {
            self.image.clone()
        }
    }

    pub fn tiles(&self) -> usize {
        self.cols * self.rows
    }

    pub fn frame(&self, col: usize, row: usize) -> Option<Rect> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        Some(Rect {
            x: self.margin_col + self.tile_width * col as Scalar + self.padding_col * col as Scalar,
            y: self.margin_row
                + self.tile_height * row as Scalar
                + self.padding_row * row as Scalar,
            w: self.tile_width,
            h: self.tile_height,
        })
    }
}

pub struct TilesetAsset {
    info: TilesetInfo,
    image_asset: AssetID,
}

impl TilesetAsset {
    pub fn info(&self) -> &TilesetInfo {
        &self.info
    }

    pub fn image_asset(&self) -> AssetID {
        self.image_asset
    }
}

pub struct TilesetAssetProtocol;

impl AssetProtocol for TilesetAssetProtocol {
    fn name(&self) -> &str {
        "tiles"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        let info: TilesetInfo = serde_json::from_str(data).unwrap();
        let image = info.image.clone();
        AssetLoadResult::Yield(Some(Box::new(info)), vec![("image".to_owned(), image)])
    }

    fn on_resume(&mut self, payload: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let info = *(payload.unwrap() as Box<dyn Any + Send>)
            .downcast::<TilesetInfo>()
            .unwrap();
        let image_asset = list
            .get(0)
            .expect("Could not obtain tileset image asset")
            .1
            .id();
        AssetLoadResult::Data(Box::new(TilesetAsset { info, image_asset }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        if let Some(asset) = asset.get::<TilesetAsset>() {
            Some(vec![AssetVariant::Id(asset.image_asset)])
        } else {
            None
        }
    }
}
