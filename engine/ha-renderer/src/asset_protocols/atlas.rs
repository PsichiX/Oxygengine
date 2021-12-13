use crate::{asset_protocols::image::ImageAsset, math::*};
use core::{
    assets::{
        asset::{Asset, AssetId},
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
    },
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Keys, HashMap, HashSet},
    str::from_utf8,
};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct TileSetPage {
    pub cols: usize,
    pub rows: usize,
    pub layers: usize,
    pub cell_size: Vec2,
    #[serde(default)]
    pub padding: Vec2,
    #[serde(default)]
    pub spacing: Vec2,
    #[serde(default)]
    pub tile_margin: Vec2,
}

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AtlasRegion {
    pub rect: Rect,
    pub layer: usize,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum AtlasAssetSource {
    /// { page image asset: { region name: region data } }
    Raw(HashMap<String, HashMap<String, AtlasRegion>>),
    TileSet(HashMap<String, TileSetPage>),
}

impl AtlasAssetSource {
    pub fn page_names(&self) -> AtlasAssetSourcePageNameIter<'_> {
        match self {
            Self::Raw(pages) => AtlasAssetSourcePageNameIter::Raw(pages.keys()),
            Self::TileSet(pages) => AtlasAssetSourcePageNameIter::TileSet(pages.keys()),
        }
    }
}

pub enum AtlasAssetSourcePageNameIter<'a> {
    Raw(Keys<'a, String, HashMap<String, AtlasRegion>>),
    TileSet(Keys<'a, String, TileSetPage>),
}

impl<'a> Iterator for AtlasAssetSourcePageNameIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Raw(iter) => iter.next().map(|k| k.as_str()),
            Self::TileSet(iter) => iter.next().map(|k| k.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtlasAsset {
    /// { page image asset: { image name: region data } }
    pub page_mappings: HashMap<String, HashMap<String, AtlasRegion>>,
    /// { page image asset: (page image size, asset id) }
    pub pages_image_assets: HashMap<String, (Vec2, AssetId)>,
}

pub struct AtlasAssetProtocol;

impl AssetProtocol for AtlasAssetProtocol {
    fn name(&self) -> &str {
        "atlas"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let source = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<AtlasAssetSource>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<AtlasAssetSource>(data).unwrap()
        } else {
            bincode::deserialize::<AtlasAssetSource>(&data).unwrap()
        };
        let pages = source
            .page_names()
            .map(|k| (k.to_owned(), format!("image://{}", k)))
            .collect();
        let source = match source {
            AtlasAssetSource::Raw(source) => source,
            AtlasAssetSource::TileSet(source) => source
                .into_iter()
                .map(|(k, page)| {
                    let cols = page.cols.max(1);
                    let rows = page.rows.max(1);
                    let layers = page.layers.max(1);
                    let count = cols * rows * layers;
                    let mappings = (0..count)
                        .map(|i| {
                            let layer = i / (cols * rows);
                            let row = (i / cols) % rows;
                            let col = i % cols;
                            let m = page.tile_margin;
                            let x = page.padding.x
                                + col as Scalar * page.cell_size.x
                                + (cols - 1) as Scalar * page.spacing.x
                                + m.x;
                            let y = page.padding.y
                                + row as Scalar * page.cell_size.y
                                + (rows - 1) as Scalar * page.spacing.y
                                + m.y;
                            let rect = Rect::new(
                                x,
                                y,
                                page.cell_size.x - m.x - m.x,
                                page.cell_size.y - m.y - m.y,
                            );
                            (format!("{}x{}", col, row), AtlasRegion { rect, layer })
                        })
                        .collect::<HashMap<_, _>>();
                    (k, mappings)
                })
                .collect::<HashMap<_, _>>(),
        };
        AssetLoadResult::Yield(Some(Box::new(source)), pages)
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }

    fn on_resume(&mut self, meta: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let source = *meta
            .unwrap()
            .downcast::<HashMap<String, HashMap<String, AtlasRegion>>>()
            .unwrap();
        let mut ids = HashSet::<&str>::default();
        for mappings in source.values() {
            for key in mappings.keys() {
                if ids.contains(key.as_str()) {
                    return AssetLoadResult::Error(format!(
                        "Atlas has duplicate image IDs: {}",
                        key
                    ));
                }
                ids.insert(key);
            }
        }
        let pages_image_assets = source
            .keys()
            .filter_map(|k| {
                if let Some((_, asset)) = list.iter().find(|(n, _)| k == n) {
                    if let Some(image) = asset.get::<ImageAsset>() {
                        let size = vec2(image.width as _, image.height as _);
                        return Some((k.to_owned(), (size, asset.id())));
                    }
                }
                None
            })
            .collect();

        AssetLoadResult::Data(Box::new(AtlasAsset {
            page_mappings: source,
            pages_image_assets,
        }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        Some(
            asset
                .get::<AtlasAsset>()
                .unwrap()
                .pages_image_assets
                .iter()
                .map(|(_, id)| AssetVariant::Id(id.1))
                .collect(),
        )
    }
}
