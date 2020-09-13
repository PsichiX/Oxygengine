use crate::math::Rect;
use core::{
    assets::{
        asset::{Asset, AssetID},
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
    },
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SpriteSheetInfo {
    pub meta: SpriteSheetInfoMeta,
    #[serde(default)]
    pub frames: HashMap<String, SpriteSheetInfoFrame>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SpriteSheetInfoMeta {
    pub image: String,
    #[serde(default)]
    pub size: SpriteSheetInfoMetaSize,
    #[serde(default)]
    pub scale: String,
}

impl SpriteSheetInfoMeta {
    pub fn image_name(&self) -> String {
        let parts = self.image.split("://").collect::<Vec<_>>();
        if parts.len() > 1 {
            parts[1].to_owned()
        } else {
            self.image.clone()
        }
    }
}

#[derive(Ignite, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SpriteSheetInfoMetaSize {
    #[serde(default)]
    pub w: Scalar,
    #[serde(default)]
    pub h: Scalar,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpriteSheetInfoFrame {
    #[serde(default)]
    pub frame: Rect,
    #[serde(default)]
    pub rotated: bool,
    #[serde(default)]
    pub trimmed: bool,
    #[serde(alias = "spriteSourceSize")]
    #[serde(default)]
    pub sprite_source_size: Rect,
    #[serde(alias = "sourceSize")]
    #[serde(default)]
    pub source_size: SpriteSheetInfoMetaSize,
}

pub struct SpriteSheetAsset {
    info: SpriteSheetInfo,
    image_asset: AssetID,
}

impl SpriteSheetAsset {
    pub fn info(&self) -> &SpriteSheetInfo {
        &self.info
    }

    pub fn image_asset(&self) -> AssetID {
        self.image_asset
    }
}

pub struct SpriteSheetAssetProtocol;

impl AssetProtocol for SpriteSheetAssetProtocol {
    fn name(&self) -> &str {
        "atlas"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        let info: SpriteSheetInfo = serde_json::from_str(data).unwrap();
        let image = info.meta.image.clone();
        AssetLoadResult::Yield(Some(Box::new(info)), vec![("image".to_owned(), image)])
    }

    fn on_resume(&mut self, payload: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let info = *(payload.unwrap() as Box<dyn Any + Send>)
            .downcast::<SpriteSheetInfo>()
            .unwrap();
        let image_asset = list
            .get(0)
            .expect("Could not obtain sprite sheet image asset")
            .1
            .id();
        AssetLoadResult::Data(Box::new(SpriteSheetAsset { info, image_asset }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        if let Some(asset) = asset.get::<SpriteSheetAsset>() {
            Some(vec![AssetVariant::Id(asset.image_asset)])
        } else {
            None
        }
    }
}
