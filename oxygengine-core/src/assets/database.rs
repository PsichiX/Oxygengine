use crate::{
    assets::{
        asset::{Asset, AssetID},
        protocol::{AssetLoadResult, AssetProtocol},
    },
    fetch::{FetchEngine, FetchProcessReader, FetchStatus},
};
use std::{any::Any, borrow::BorrowMut, collections::HashMap, mem::replace};

#[derive(Debug, Clone, PartialEq)]
pub enum LoadStatus {
    InvalidPath(String),
    UnknownProtocol(String),
    FetchError(FetchStatus),
}

pub struct AssetsDatabase {
    fetch_engines: Vec<Box<FetchEngine>>,
    protocols: HashMap<String, Box<AssetProtocol>>,
    assets: HashMap<AssetID, (String, Asset)>,
    table: HashMap<String, AssetID>,
    loading: HashMap<String, (String, Box<FetchProcessReader>)>,
    yielded: HashMap<String, (String, Box<dyn Any + Send + Sync>, Vec<(String, String)>)>,
}

impl AssetsDatabase {
    pub fn new(fetch_engine: Box<FetchEngine>) -> Self {
        Self {
            fetch_engines: vec![fetch_engine],
            protocols: Default::default(),
            assets: Default::default(),
            table: Default::default(),
            loading: Default::default(),
            yielded: Default::default(),
        }
    }

    pub fn loaded_count(&self) -> usize {
        self.assets.len()
    }

    pub fn loaded_paths(&self) -> Vec<String> {
        self.assets
            .iter()
            .map(|(_, (_, a))| a.to_full_path())
            .collect()
    }

    pub fn loading_count(&self) -> usize {
        self.loading.len()
    }

    pub fn loading_paths(&self) -> Vec<String> {
        let mut result = self
            .loading
            .iter()
            .map(|(path, (prot, _))| format!("{}://{}", prot, path))
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    pub fn yielded_count(&self) -> usize {
        self.yielded.len()
    }

    pub fn yielded_paths(&self) -> Vec<String> {
        let mut result = self
            .yielded
            .iter()
            .map(|(path, (prot, _, _))| format!("{}://{}", prot, path))
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    pub fn yielded_deps_count(&self) -> usize {
        self.yielded
            .iter()
            .map(|(_, (_, _, list))| list.len())
            .sum()
    }

    pub fn yielded_deps_paths(&self) -> Vec<String> {
        let mut result = self
            .yielded
            .iter()
            .flat_map(|(_, (_, _, list))| {
                list.iter().map(|(p, _)| p.to_owned()).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        result.sort();
        result.dedup();
        result
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

    pub fn with_fetch_engine<F, R>(&mut self, mut action: F) -> R
    where
        F: FnMut(&mut FetchEngine) -> R,
    {
        let fetch_engine: &mut FetchEngine = self.fetch_engine_mut().borrow_mut();
        action(fetch_engine)
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

    pub fn load(&mut self, path: &str) -> Result<(), LoadStatus> {
        if self.table.contains_key(path) {
            return Ok(());
        }
        let parts = path.split("://").take(2).collect::<Vec<_>>();
        if parts.len() == 2 {
            let prot = parts[0];
            let subpath = parts[1];
            if self.protocols.contains_key(prot) {
                let reader = self.fetch_engine_mut().fetch(path);
                match reader {
                    Ok(reader) => {
                        self.loading
                            .insert(subpath.to_owned(), (prot.to_owned(), reader));
                        Ok(())
                    }
                    Err(status) => Err(LoadStatus::FetchError(status)),
                }
            } else {
                Err(LoadStatus::UnknownProtocol(prot.to_owned()))
            }
        } else {
            Err(LoadStatus::InvalidPath(path.to_owned()))
        }
    }

    pub fn insert(&mut self, path: &str, asset: Asset) -> AssetID {
        let id = AssetID::new();
        self.assets.insert(id, (path.to_owned(), asset));
        self.table.insert(path.to_owned(), id);
        id
    }

    pub fn remove_by_id(&mut self, id: AssetID) -> Option<Asset> {
        if let Some((path, asset)) = self.assets.remove(&id) {
            self.table.remove(&path);
            Some(asset)
        } else {
            None
        }
    }

    pub fn remove_by_path(&mut self, path: &str) -> Option<Asset> {
        if let Some(id) = self.table.remove(path) {
            if let Some((_, asset)) = self.assets.remove(&id) {
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

    pub fn asset_by_id(&self, id: AssetID) -> Option<&Asset> {
        self.assets.get(&id).map(|(_, asset)| asset)
    }

    pub fn asset_by_path(&self, path: &str) -> Option<&Asset> {
        if let Some(id) = self.table.get(path) {
            if let Some((_, asset)) = self.assets.get(id) {
                return Some(asset);
            }
        }
        None
    }

    pub fn process(&mut self) {
        loop {
            let to_dispatch = {
                self.loading
                    .iter()
                    .filter_map(|(path, (prot, reader))| {
                        if let Some(data) = reader.read() {
                            Some((path.to_owned(), prot.to_owned(), data))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };
            for (path, prot, data) in to_dispatch {
                if let Some(protocol) = self.protocols.get_mut(&prot) {
                    match protocol.on_load(data) {
                        AssetLoadResult::Data(data) => {
                            let asset = Asset::new(&prot, &path, data);
                            self.insert(&asset.to_full_path(), asset);
                        }
                        AssetLoadResult::Yield(meta, list) => {
                            let list = list
                                .into_iter()
                                .filter(|(_, path)| self.load(&path).is_ok())
                                .collect();
                            self.yielded.insert(path, (prot, meta, list));
                        }
                        _ => {}
                    }
                }
            }
            if !self
                .loading
                .iter()
                .any(|(_, (_, reader))| reader.status() == FetchStatus::Done)
            {
                break;
            }
        }
        self.loading.retain(|_, (_, reader)| {
            if let FetchStatus::InProgress(_) = reader.status() {
                true
            } else {
                false
            }
        });
        self.yielded = replace(&mut self.yielded, Default::default())
            .into_iter()
            .filter_map(|(path, (prot, meta, list))| {
                if list.iter().all(|(_, path)| self.table.contains_key(path)) {
                    let ptr = self as *const Self;
                    if let Some(protocol) = self.protocols.get_mut(&prot) {
                        let list = list
                            .iter()
                            .map(|(key, path)| unsafe {
                                let asset = &(&*ptr).table[path];
                                let asset = &(&*ptr).assets[asset].1;
                                (key.as_str(), asset)
                            })
                            .collect::<Vec<_>>();
                        if let Some(data) = protocol.on_resume(meta, &list) {
                            let asset = Asset::new(&prot, &path, data);
                            self.insert(&asset.to_full_path(), asset);
                        }
                    }
                    None
                } else {
                    Some((path, (prot, meta, list)))
                }
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::*;
    use std::str::from_utf8;

    struct TextAssetProtocol;

    impl AssetProtocol for TextAssetProtocol {
        fn name(&self) -> &str {
            "text"
        }

        fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
            let data = from_utf8(&data).unwrap().to_owned();
            AssetLoadResult::Data(Box::new(data))
        }
    }

    struct SetAssetProtocol;

    impl AssetProtocol for SetAssetProtocol {
        fn name(&self) -> &str {
            "set"
        }

        fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
            let data = from_utf8(&data).unwrap().to_owned();
            let list = data
                .split('\n')
                .enumerate()
                .map(|(i, line)| (i.to_string(), line.to_owned()))
                .collect();
            AssetLoadResult::Yield(Box::new(()), list)
        }

        fn on_resume(
            &mut self,
            _: Box<dyn Any + Send + Sync>,
            list: &[(&str, &Asset)],
        ) -> Option<Box<dyn Any + Send + Sync>> {
            let data = list
                .iter()
                .map(|(_, asset)| asset.to_full_path())
                .collect::<Vec<_>>();
            Some(Box::new(data))
        }
    }

    #[test]
    fn test_general() {
        let mut fetch_engine = MapFetchEngine::default();
        fetch_engine
            .map
            .insert("text://a.txt".to_owned(), b"A".to_vec());
        fetch_engine
            .map
            .insert("text://b.txt".to_owned(), b"B".to_vec());
        fetch_engine.map.insert(
            "set://list.txt".to_owned(),
            b"text://a.txt\ntext://b.txt".to_vec(),
        );

        let mut database = AssetsDatabase::new(Box::new(fetch_engine));
        database.register(Box::new(TextAssetProtocol));
        database.register(Box::new(SetAssetProtocol));
        assert_eq!(database.load("set://list.txt"), Ok(()));
        assert_eq!(database.loaded_count(), 0);
        assert_eq!(database.loading_count(), 1);
        assert_eq!(database.yielded_count(), 0);
        assert_eq!(database.yielded_deps_count(), 0);

        database.process();
        assert_eq!(database.loaded_count(), 3);
        assert_eq!(database.loading_count(), 0);
        assert_eq!(database.yielded_count(), 0);
        assert_eq!(database.yielded_deps_count(), 0);

        assert!(database.asset_by_path("set://list.txt").is_some());
        assert_eq!(
            &database
                .asset_by_path("set://list.txt")
                .unwrap()
                .to_full_path(),
            "set://list.txt"
        );
        assert!(database
            .asset_by_path("set://list.txt")
            .unwrap()
            .is::<Vec<String>>());
        assert_eq!(
            database
                .asset_by_path("set://list.txt")
                .unwrap()
                .get::<Vec<String>>()
                .unwrap(),
            &vec!["text://a.txt".to_owned(), "text://b.txt".to_owned()],
        );

        assert!(database.asset_by_path("text://a.txt").is_some());
        assert!(database
            .asset_by_path("text://a.txt")
            .unwrap()
            .is::<String>());
        assert_eq!(
            database
                .asset_by_path("text://a.txt")
                .unwrap()
                .get::<String>()
                .unwrap()
                .as_str(),
            "A"
        );

        assert!(database.asset_by_path("text://b.txt").is_some());
        assert_eq!(
            database
                .asset_by_path("text://b.txt")
                .unwrap()
                .get::<String>()
                .unwrap()
                .as_str(),
            "B"
        );
    }
}
