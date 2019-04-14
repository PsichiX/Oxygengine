use crate::id::ID;
use std::collections::HashMap;

pub type AssetID = ID<()>;

pub trait Asset {
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
}

pub trait AssetProtocol {
    fn name(&self) -> &str;
    fn load(&mut self, path: &str);
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
}

pub struct AssetsDatabase {
    assets: HashMap<AssetID, (String, Box<Asset>)>,
    table: HashMap<String, AssetID>,
    protocols: HashMap<String, Box<AssetProtocol>>,
}

impl AssetsDatabase {
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
