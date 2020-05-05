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
pub struct FontFace {
    pub font: String,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default = "FontFace::default_weight")]
    pub weight: Scalar,
    #[serde(default = "FontFace::default_stretch")]
    pub stretch: Scalar,
    #[serde(default)]
    pub variant: Option<String>,
}

impl FontFace {
    fn default_weight() -> Scalar {
        400.0
    }

    fn default_stretch() -> Scalar {
        100.0
    }
}

pub struct FontFaceAsset {
    face: FontFace,
    font_asset: AssetID,
}

impl FontFaceAsset {
    pub fn face(&self) -> &FontFace {
        &self.face
    }

    pub fn font_asset(&self) -> AssetID {
        self.font_asset
    }
}

pub struct FontFaceAssetProtocol;

impl AssetProtocol for FontFaceAssetProtocol {
    fn name(&self) -> &str {
        "fontface"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        let face: FontFace = serde_json::from_str(data).unwrap();
        let font = face.font.clone();
        AssetLoadResult::Yield(Some(Box::new(face)), vec![("font".to_owned(), font)])
    }

    fn on_resume(&mut self, payload: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let face = *(payload.unwrap() as Box<dyn Any + Send>)
            .downcast::<FontFace>()
            .unwrap();
        let font_asset = list
            .get(0)
            .expect("Could not obtain font face font asset")
            .1
            .id();
        AssetLoadResult::Data(Box::new(FontFaceAsset { face, font_asset }))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        if let Some(asset) = asset.get::<FontFaceAsset>() {
            Some(vec![AssetVariant::Id(asset.font_asset)])
        } else {
            None
        }
    }
}
