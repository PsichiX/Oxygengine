use crate::{
    fetch::{FetchEngine, FetchProcessReader},
    id::ID,
};
use specs::{System, Write};
use std::{
    collections::HashMap,
    mem::replace,
    sync::{Arc, Mutex},
};

pub type AssetID = ID<()>;

pub trait Asset: Send + Sync {
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
}

pub trait AssetProtocol: Send + Sync {
    fn name(&self) -> &str;
    fn load(&mut self, fetch_engine: &mut FetchEngine, path: &str);
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
}

pub struct BytesAssetProtocol {}

pub struct AssetsDatabase {
    fetch_engines: Vec<Box<FetchEngine>>,
    protocols: HashMap<String, Box<AssetProtocol>>,
    assets: HashMap<AssetID, (String, Box<Asset>)>,
    table: HashMap<String, AssetID>,
    loading: HashMap<String, Box<FetchProcessReader>>,
}

impl AssetsDatabase {
    pub fn new(fetch_engine: Box<FetchEngine>) -> Self {
        Self {
            fetch_engines: vec![fetch_engine],
            protocols: Default::default(),
            assets: Default::default(),
            table: Default::default(),
            loading: Default::default(),
        }
    }

    pub fn push_fetch_engine(&mut self, fetch_engine: Box<FetchEngine>) {
        self.fetch_engines.push(fetch_engine);
    }

    pub fn pop_fetch_engine(&mut self) -> Option<Box<FetchEngine>> {
        if !self.fetch_engines.is_empty() {
            self.fetch_engines.pop()
        } else {
            None
        }
    }

    pub fn fetch_engine(&self) -> &Box<FetchEngine> {
        self.fetch_engines.last().unwrap()
    }

    pub fn fetch_engine_mut(&mut self) -> &mut Box<FetchEngine> {
        self.fetch_engines.last_mut().unwrap()
    }

    pub fn register(&mut self, mut protocol: Box<AssetProtocol>) {
        protocol.on_register();
        let name = protocol.name().to_owned();
        self.protocols.insert(name, protocol);
    }

    pub fn unregister(&mut self, protocol_name: &str) -> Option<Box<AssetProtocol>> {
        if let Some(mut protocol) = self.protocols.remove(protocol_name) {
            protocol.on_unregister();
            Some(protocol)
        } else {
            None
        }
    }

    pub fn load(&mut self, path: &str) -> bool {
        let parts = path.split("://").take(2).collect::<Vec<_>>();
        if parts.len() == 2 {
            let prot = parts[0];
            if let Some(protocol) = self.protocols.get_mut(prot) {
                let path = parts[1];
                // protocol.
                return true;
            }
        }
        false
    }

    pub fn insert(&mut self, path: &str, mut asset: Box<Asset>) -> AssetID {
        let id = AssetID::new();
        asset.on_register();
        self.assets.insert(id, (path.to_owned(), asset));
        self.table.insert(path.to_owned(), id);
        id
    }

    pub fn remove_by_id(&mut self, id: AssetID) -> Option<Box<Asset>> {
        if let Some((path, mut asset)) = self.assets.remove(&id) {
            self.table.remove(&path);
            asset.on_unregister();
            Some(asset)
        } else {
            None
        }
    }

    pub fn remove_by_path(&mut self, path: &str) -> Option<Box<Asset>> {
        if let Some(id) = self.table.remove(path) {
            if let Some((_, mut asset)) = self.assets.remove(&id) {
                asset.on_unregister();
                return Some(asset);
            }
        }
        None
    }

    pub fn id_by_path(&self, path: &str) -> Option<AssetID> {
        self.table.get(path).map(|id| *id)
    }

    pub fn path_by_id<'a>(&'a self, id: AssetID) -> Option<&'a str> {
        self.assets.get(&id).map(|(path, _)| path.as_str())
    }

    pub fn asset_by_id(&self, id: AssetID) -> Option<&Box<Asset>> {
        self.assets.get(&id).map(|(_, asset)| asset)
    }

    pub fn asset_by_path(&self, path: &str) -> Option<&Box<Asset>> {
        if let Some(id) = self.table.get(path) {
            if let Some((_, asset)) = self.assets.get(id) {
                return Some(asset);
            }
        }
        None
    }
}

pub struct AssetsSystem;

impl<'s> System<'s> for AssetsSystem {
    type SystemData = Option<Write<'s, AssetsDatabase>>;

    fn run(&mut self, _: Self::SystemData) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::*;

    #[test]
    fn test_general() {
        let mut db = AssetsDatabase::new(FsFetchEngine::new(&"."));
        db.load("bytes://Cargo.toml");
    }
}
