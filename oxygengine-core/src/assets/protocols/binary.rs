use crate::assets::protocol::{AssetLoadResult, AssetProtocol};

pub struct BinaryAsset(Vec<u8>);

impl BinaryAsset {
    pub fn get(&self) -> &[u8] {
        &self.0
    }
}

pub struct BinaryAssetProtocol;

impl AssetProtocol for BinaryAssetProtocol {
    fn name(&self) -> &str {
        "bin"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        AssetLoadResult::Data(Box::new(BinaryAsset(data)))
    }
}
