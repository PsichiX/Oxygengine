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
    collections::{HashMap, HashSet},
    str::from_utf8,
};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct FontAssetSource {
    pub line_height: usize,
    pub line_base: usize,
    pub sdf_resolution: usize,
    pub pages: Vec<FontAssetSourcePage>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct FontAssetSourcePage {
    pub image: String,
    pub characters: HashMap<char, FontAssetSourceCharacter>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct FontAssetSourceCharacter {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub xoffset: isize,
    pub yoffset: isize,
    pub xadvance: isize,
}

#[derive(Debug, Clone)]
pub struct FontAsset {
    pub line_height: usize,
    pub line_base: usize,
    pub sdf_resolution: usize,
    pub characters: HashMap<char, FontAssetCharacter>,
    /// [ (page image size, asset id) ]
    pub pages_image_assets: Vec<(Vec2, AssetId)>,
}

#[derive(Debug, Clone)]
pub struct FontAssetCharacter {
    pub page: usize,
    pub image_location: Vec2,
    pub image_size: Vec2,
    pub size: Vec2,
    pub offset: Vec2,
    pub line_advance: Scalar,
}

pub struct FontAssetProtocol;

impl AssetProtocol for FontAssetProtocol {
    fn name(&self) -> &str {
        "font"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let source = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<FontAssetSource>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<FontAssetSource>(data).unwrap()
        } else {
            bincode::deserialize::<FontAssetSource>(&data).unwrap()
        };
        let pages = source
            .pages
            .iter()
            .map(|page| (page.image.to_owned(), format!("image://{}", page.image)))
            .collect();
        AssetLoadResult::Yield(Some(Box::new(source)), pages)
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }

    fn on_resume(&mut self, meta: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let source = *meta.unwrap().downcast::<FontAssetSource>().unwrap();
        let mut ids = HashSet::<char>::default();
        for page in &source.pages {
            for c in page.characters.keys() {
                if ids.contains(c) {
                    return AssetLoadResult::Error(format!("Font has duplicate characters: {}", c));
                }
                ids.insert(*c);
            }
        }
        let pages_image_assets = source
            .pages
            .iter()
            .filter_map(|page| {
                if let Some((_, asset)) = list.iter().find(|(n, _)| page.image == *n) {
                    if let Some(image) = asset.get::<ImageAsset>() {
                        let size = vec2(image.width as _, image.height as _);
                        return Some((size, asset.id()));
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        let sdf_resolution = source.sdf_resolution;
        let characters = source
            .pages
            .into_iter()
            .enumerate()
            .flat_map(|(i, page)| {
                page.characters.into_iter().map(move |(id, character)| {
                    (
                        id,
                        FontAssetCharacter {
                            page: i,
                            image_location: Vec2::new(character.x as f32, character.y as f32),
                            image_size: Vec2::new(character.width as f32, character.height as f32),
                            size: Vec2::new(
                                character.width as f32
                                    - sdf_resolution as f32
                                    - sdf_resolution as f32,
                                character.height as f32
                                    - sdf_resolution as f32
                                    - sdf_resolution as f32,
                            ),
                            offset: Vec2::new(
                                character.xoffset as f32 - sdf_resolution as f32,
                                character.yoffset as f32 - sdf_resolution as f32,
                            ),
                            line_advance: character.xadvance as f32,
                        },
                    )
                })
            })
            .collect::<HashMap<_, _>>();

        AssetLoadResult::Data(Box::new(FontAsset {
            line_height: source.line_height,
            line_base: source.line_base,
            sdf_resolution: source.sdf_resolution,
            characters,
            pages_image_assets,
        }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        Some(
            asset
                .get::<FontAsset>()
                .unwrap()
                .pages_image_assets
                .iter()
                .map(|(_, id)| AssetVariant::Id(*id))
                .collect(),
        )
    }
}
