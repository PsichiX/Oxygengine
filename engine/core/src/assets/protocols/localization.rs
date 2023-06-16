use crate::assets::protocol::{AssetLoadResult, AssetProtocol};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LocalizationAsset {
    pub language: String,
    pub dictionary: HashMap<String, String>,
}

pub struct LocalizationAssetProtocol;

impl AssetProtocol for LocalizationAssetProtocol {
    fn name(&self) -> &str {
        "locals"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match serde_json::from_str::<LocalizationAsset>(data) {
            Ok(result) => AssetLoadResult::Data(Box::new(result)),
            Err(error) => {
                AssetLoadResult::Error(format!("Error loading localization asset: {:?}", error))
            }
        }
    }
}
