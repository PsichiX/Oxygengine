use crate::assets::protocol::{AssetLoadResult, AssetProtocol};
use serde::de::DeserializeOwned;
use std::str::from_utf8;
use toml::Value;

pub struct TomlAsset(Value);

impl TomlAsset {
    pub fn get(&self) -> &Value {
        &self.0
    }

    pub fn deserialize<T>(&self) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        match self.0.clone().try_into() {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub struct TomlAssetProtocol;

impl AssetProtocol for TomlAssetProtocol {
    fn name(&self) -> &str {
        "toml"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match toml::from_str(data) {
            Ok(value) => AssetLoadResult::Data(Box::new(TomlAsset(value))),
            Err(error) => AssetLoadResult::Error(format!("Error loading TOML asset: {:?}", error)),
        }
    }
}
