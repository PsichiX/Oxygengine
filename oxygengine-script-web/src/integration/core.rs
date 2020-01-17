use crate::{
    interface::{ResourceAccess, ResourceModify},
    json_asset_protocol::JsonAsset,
    scriptable::{ScriptableMap, ScriptableValue},
};
use core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct AppLifeCycleScripted {
    pub running: bool,
    pub delta_time_seconds: f64,
    pub current_state_token: StateToken,
}

impl From<&AppLifeCycle> for AppLifeCycleScripted {
    fn from(value: &AppLifeCycle) -> Self {
        Self {
            running: value.running,
            delta_time_seconds: value.delta_time_seconds(),
            current_state_token: value.current_state_token(),
        }
    }
}

impl ResourceModify<AppLifeCycleScripted> for AppLifeCycle {
    fn modify_resource(&mut self, source: AppLifeCycleScripted) {
        self.running = source.running;
    }
}

impl ResourceAccess for AppLifeCycle {
    fn access_resource(&mut self, _value: ScriptableValue) -> ScriptableValue {
        ScriptableValue::Null
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AssetsDatabaseLoaderScripted {
    pub ready: bool,
    pub loaded_count: usize,
    pub loaded_paths: Vec<String>,
    pub loading_count: usize,
    pub loading_paths: Vec<String>,
    pub lately_loaded_paths: Vec<String>,
    pub lately_unloaded_paths: Vec<String>,
}

impl From<&AssetsDatabase> for AssetsDatabaseLoaderScripted {
    fn from(value: &AssetsDatabase) -> Self {
        Self {
            ready: value.is_ready(),
            loaded_count: value.loaded_count(),
            loaded_paths: value.loaded_paths(),
            loading_count: value.loading_count(),
            loading_paths: value.loading_paths(),
            lately_loaded_paths: value
                .lately_loaded_paths()
                .map(|p| p.to_owned())
                .collect::<Vec<_>>(),
            lately_unloaded_paths: value
                .lately_unloaded_paths()
                .map(|p| p.to_owned())
                .collect::<Vec<_>>(),
        }
    }
}

impl ResourceModify<AssetsDatabaseLoaderScripted> for AssetsDatabase {
    fn modify_resource(&mut self, _: AssetsDatabaseLoaderScripted) {}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AssetsDatabaseScripted {
    #[serde(default)]
    pub load: Option<Vec<String>>,
    #[serde(default)]
    pub unload: Option<Vec<String>>,
}

impl From<&AssetsDatabase> for AssetsDatabaseScripted {
    fn from(_: &AssetsDatabase) -> Self {
        Self {
            load: None,
            unload: None,
        }
    }
}

impl ResourceModify<AssetsDatabaseScripted> for AssetsDatabase {
    fn modify_resource(&mut self, _source: AssetsDatabaseScripted) {}
}

impl ResourceAccess for AssetsDatabase {
    fn access_resource(&mut self, value: ScriptableValue) -> ScriptableValue {
        match value {
            ScriptableValue::Object(object) => {
                if let Some(ScriptableValue::Array(load)) = object.get("load") {
                    for path in load {
                        if let ScriptableValue::String(path) = path {
                            drop(self.load(&path));
                        }
                    }
                    return ScriptableValue::Bool(true);
                } else if let Some(ScriptableValue::Array(load)) = object.get("unload") {
                    for path in load {
                        if let ScriptableValue::String(path) = path {
                            self.remove_by_path(&path);
                        }
                    }
                    return ScriptableValue::Bool(true);
                } else if let Some(ScriptableValue::String(path)) = object.get("serve-pack") {
                    if let Some(asset) = self.asset_by_path(path) {
                        if let Some(pack) = asset.get::<PackAsset>() {
                            let engine = pack.make_fetch_engine();
                            self.push_fetch_engine(Box::new(engine));
                            return ScriptableValue::Bool(true);
                        }
                    }
                } else if let Some(ScriptableValue::Array(paths)) = object.get("loaded") {
                    let map = paths
                        .into_iter()
                        .filter_map(|path| {
                            if let ScriptableValue::String(path) = path {
                                let path = path.to_string();
                                let result =
                                    ScriptableValue::Bool(self.id_by_path(&path).is_some());
                                Some((path, result))
                            } else {
                                None
                            }
                        })
                        .collect::<ScriptableMap<_, _>>();
                    return ScriptableValue::Object(map);
                }
            }
            ScriptableValue::Array(paths) => {
                let map = paths
                    .into_iter()
                    .filter_map(|path| {
                        let path = path.to_string();
                        if let Some(asset) = self.asset_by_path(&path) {
                            if let Some(protocol) = path.split("://").next() {
                                let data = match protocol {
                                    "text" => ScriptableValue::String(
                                        asset.get::<TextAsset>().unwrap().get().to_owned(),
                                    ),
                                    "json" => asset.get::<JsonAsset>().unwrap().get().clone(),
                                    "binary" => ScriptableValue::Array(
                                        asset
                                            .get::<BinaryAsset>()
                                            .unwrap()
                                            .get()
                                            .iter()
                                            .map(|byte| ScriptableValue::Number((*byte).into()))
                                            .collect::<Vec<_>>(),
                                    ),
                                    _ => return None,
                                };
                                return Some((path, data));
                            }
                        }
                        None
                    })
                    .collect::<ScriptableMap<_, _>>();
                return ScriptableValue::Object(map);
            }
            _ => {}
        }
        ScriptableValue::Null
    }
}
