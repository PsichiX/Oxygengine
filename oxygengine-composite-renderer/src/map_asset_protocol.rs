use crate::{
    composite_renderer::{Command, Image},
    sprite_sheet_asset_protocol::SpriteSheetAsset,
};
use core::assets::{
    asset::{Asset, AssetID},
    database::AssetsDatabase,
    protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerObject {
    pub name: String,
    pub object_type: String,
    pub visible: bool,
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerData {
    Tiles(Vec<usize>),
    Objects(Vec<LayerObject>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub data: LayerData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub cols: usize,
    pub rows: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub sprite_sheets: Vec<String>,
    pub tiles_mapping: HashMap<usize, (String, String)>,
    pub layers: Vec<Layer>,
}

impl Map {
    pub fn size(&self) -> (usize, usize) {
        (self.cols * self.tile_width, self.rows * self.tile_height)
    }

    pub fn build_render_commands_from_layer_by_name<'a>(
        &self,
        name: &str,
        chunk_offset: (usize, usize),
        chunk_size: Option<(usize, usize)>,
        assets: &AssetsDatabase,
    ) -> Option<Vec<Command<'a>>> {
        let index = self.layers.iter().position(|layer| layer.name == name)?;
        self.build_render_commands_from_layer(index, chunk_offset, chunk_size, assets)
    }

    pub fn build_render_commands_from_layer<'a>(
        &self,
        index: usize,
        chunk_offset: (usize, usize),
        chunk_size: Option<(usize, usize)>,
        assets: &AssetsDatabase,
    ) -> Option<Vec<Command<'a>>> {
        if self.tiles_mapping.is_empty() {
            return None;
        }
        let layer = self.layers.get(index)?;
        if let LayerData::Tiles(data) = &layer.data {
            let atlases = self
                .sprite_sheets
                .iter()
                .map(|s| {
                    let info = assets.asset_by_path(&format!("atlas://{}", s))?.get::<SpriteSheetAsset>()?.info();
                    Some((s.clone(), info))
                })
                .collect::<Option<HashMap<_, _>>>()?;
            let mut commands = Vec::with_capacity(2 + self.cols * self.rows);
            commands.push(Command::Store);
            let w = self.tile_width as f32;
            let h = self.tile_height as f32;
            let chunk_size = chunk_size.unwrap_or_else(|| (self.cols, self.rows));
            let cols_start = chunk_offset.0.min(self.cols);
            let cols_end = (chunk_offset.0 + chunk_size.0).min(self.cols);
            let rows_start = chunk_offset.1.min(self.rows);
            let rows_end = (chunk_offset.1 + chunk_size.1).min(self.rows);
            for col in cols_start..cols_end {
                for row in rows_start..rows_end {
                    let i = self.cols * row + col;
                    let id = data.get(i).unwrap_or_else(|| &0);
                    if let Some((sprite_sheet, name)) = &self.tiles_mapping.get(id) {
                        let info = atlases.get(sprite_sheet)?;
                        let x = w * col as f32;
                        let y = h * row as f32;
                        let frame = info.frames.get(name)?.frame;
                        commands.push(Command::Draw(
                            Image::new_owned(info.meta.image_name())
                                .source(Some(frame))
                                .destination(Some([x, y, w, h].into()))
                                .into(),
                        ));
                    }
                }
            }
            commands.push(Command::Restore);
            Some(commands)
        } else {
            None
        }
    }
}

pub struct MapAsset {
    map: Map,
    sprite_sheet_assets: Vec<AssetID>,
}

impl MapAsset {
    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn sprite_sheet_assets(&self) -> &[AssetID] {
        &self.sprite_sheet_assets
    }
}

pub struct MapAssetProtocol;

impl AssetProtocol for MapAssetProtocol {
    fn name(&self) -> &str {
        "map"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let map: Map = bincode::deserialize(&data).unwrap();
        let list = map
            .sprite_sheets
            .iter()
            .map(|s| (s.clone(), format!("atlas://{}", s)))
            .collect::<Vec<_>>();
        AssetLoadResult::Yield(Some(Box::new(map)), list)
    }

    fn on_resume(&mut self, payload: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let map = *(payload.unwrap() as Box<dyn Any + Send>)
            .downcast::<Map>()
            .unwrap();
        let sprite_sheet_assets = list
            .into_iter()
            .map(|(_, asset)| asset.id())
            .collect::<Vec<_>>();
        AssetLoadResult::Data(Box::new(MapAsset {
            map,
            sprite_sheet_assets,
        }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        if let Some(asset) = asset.get::<MapAsset>() {
            Some(
                asset
                    .sprite_sheet_assets
                    .iter()
                    .map(|a| AssetVariant::Id(*a))
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    }
}
