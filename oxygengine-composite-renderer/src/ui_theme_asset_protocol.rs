use crate::resource::UiThemed;
use core::{Ignite, assets::protocol::{AssetLoadResult, AssetProtocol}};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct UiThemeAsset(HashMap<String, UiThemed>);

impl UiThemeAsset {
    pub fn get(&self) -> &HashMap<String, UiThemed> {
        &self.0
    }
}

pub struct UiThemeAssetProtocol;

impl AssetProtocol for UiThemeAssetProtocol {
    fn name(&self) -> &str {
        "ui-theme"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match serde_json::from_str::<UiThemeAsset>(data) {
            Ok(result) => AssetLoadResult::Data(Box::new(result)),
            Err(error) => {
                AssetLoadResult::Error(format!("Error loading ui theme asset: {:?}", error))
            }
        }
    }
}
