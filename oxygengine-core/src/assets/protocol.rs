use crate::assets::asset::{Asset, AssetID};
use std::any::Any;

pub type Meta = Option<Box<dyn Any + Send + Sync>>;

#[derive(Debug, Clone)]
pub enum AssetVariant {
    Id(AssetID),
    Path(String),
}

impl From<AssetID> for AssetVariant {
    fn from(id: AssetID) -> Self {
        AssetVariant::Id(id)
    }
}

impl From<&str> for AssetVariant {
    fn from(path: &str) -> Self {
        AssetVariant::Path(path.to_owned())
    }
}

impl From<String> for AssetVariant {
    fn from(path: String) -> Self {
        AssetVariant::Path(path)
    }
}

impl From<&String> for AssetVariant {
    fn from(path: &String) -> Self {
        AssetVariant::Path(path.clone())
    }
}

pub enum AssetLoadResult {
    Error(String),
    Data(Box<dyn Any + Send + Sync>),
    /// (meta, [(key, path to load)])
    Yield(Meta, Vec<(String, String)>),
}

pub trait AssetProtocol: Send + Sync {
    fn name(&self) -> &str;

    fn on_load_with_path(&mut self, _path: &str, data: Vec<u8>) -> AssetLoadResult {
        self.on_load(data)
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult;

    fn on_resume(&mut self, _meta: Meta, _list: &[(&str, &Asset)]) -> AssetLoadResult {
        AssetLoadResult::Error(format!(
            "Protocol {} does not implement `on_resume` method!",
            std::any::type_name::<Self>()
        ))
    }

    fn on_unload(&mut self, _asset: &Asset) -> Option<Vec<AssetVariant>> {
        None
    }

    fn on_register(&mut self) {}

    fn on_unregister(&mut self) {}
}
