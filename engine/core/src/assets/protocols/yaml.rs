use crate::assets::protocol::{AssetLoadResult, AssetProtocol};
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::str::from_utf8;

pub struct YamlAsset(Value);

impl YamlAsset {
    pub fn get(&self) -> &Value {
        &self.0
    }

    pub fn deserialize<T>(&self) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        match serde_yaml::from_value(self.0.clone()) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub struct YamlAssetProtocol;

impl AssetProtocol for YamlAssetProtocol {
    fn name(&self) -> &str {
        "yaml"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match serde_yaml::from_str(data) {
            Ok(value) => AssetLoadResult::Data(Box::new(YamlAsset(value))),
            Err(error) => AssetLoadResult::Error(format!("Error loading YAML asset: {:?}", error)),
        }
    }
}
