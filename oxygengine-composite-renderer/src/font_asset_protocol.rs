use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};

pub struct FontAsset {
    bytes: Vec<u8>,
}

impl FontAsset {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub struct FontAssetProtocol;

impl AssetProtocol for FontAssetProtocol {
    fn name(&self) -> &str {
        "font"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        AssetLoadResult::Data(Box::new(FontAsset { bytes: data }))
    }
}
