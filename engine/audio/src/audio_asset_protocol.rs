use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};

pub struct AudioAsset(Vec<u8>);

impl AudioAsset {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

pub struct AudioAssetProtocol;

impl AssetProtocol for AudioAssetProtocol {
    fn name(&self) -> &str {
        "audio"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        AssetLoadResult::Data(Box::new(AudioAsset(data)))
    }
}
