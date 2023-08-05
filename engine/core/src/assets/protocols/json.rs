use crate::assets::protocol::{AssetLoadResult, AssetProtocol};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::str::from_utf8;

pub struct JsonAsset(Value);

impl JsonAsset {
    pub fn get(&self) -> &Value {
        &self.0
    }

    pub fn deserialize<T>(&self) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_value(self.0.clone()) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub struct JsonAssetProtocol;

impl AssetProtocol for JsonAssetProtocol {
    fn name(&self) -> &str {
        "json"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match serde_json::from_str(data) {
            Ok(value) => AssetLoadResult::Data(Box::new(JsonAsset(value))),
            Err(error) => AssetLoadResult::Error(format!("Error loading JSON asset: {:?}", error)),
        }
    }
}
