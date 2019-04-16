use crate::assets::asset::Asset;
use std::any::Any;

pub enum AssetLoadResult {
    None,
    Data(Box<dyn Any + Send + Sync>),
    /// (meta, [(key, path to load)])
    Yield(Box<dyn Any + Send + Sync>, Vec<(String, String)>),
}

pub trait AssetProtocol: Send + Sync {
    fn name(&self) -> &str;

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult;

    fn on_resume(
        &mut self,
        _meta: Box<dyn Any + Send + Sync>,
        _list: &[(&str, &Asset)],
    ) -> Option<Box<dyn Any + Send + Sync>> {
        None
    }

    fn on_register(&mut self) {}

    fn on_unregister(&mut self) {}
}
