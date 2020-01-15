use crate::scriptable::ScriptableValue;
use core::assets::protocol::{AssetLoadResult, AssetProtocol};
use std::str::from_utf8;

pub struct JsonAsset(ScriptableValue);

impl JsonAsset {
    pub fn get(&self) -> &ScriptableValue {
        &self.0
    }
}

pub struct JsonAssetProtocol;

impl AssetProtocol for JsonAssetProtocol {
    fn name(&self) -> &str {
        "json"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        if let Ok(value) = serde_json::from_str(data) {
            AssetLoadResult::Data(Box::new(JsonAsset(value)))
        } else {
            AssetLoadResult::None
        }
    }
}
