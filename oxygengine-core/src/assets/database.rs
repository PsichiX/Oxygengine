use crate::{
    assets::{
        asset::{Asset, AssetId},
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
    },
    fetch::{FetchEngine, FetchProcessReader, FetchStatus},
};
use std::{any::TypeId, borrow::BorrowMut, collections::HashMap};

#[derive(Debug, Clone, PartialEq)]
pub enum LoadStatus {
    InvalidPath(String),
    UnknownProtocol(String),
    FetchError(FetchStatus),
}

pub trait AssetsDatabaseErrorReporter: Send + Sync {
    fn on_report(&mut self, protocol: &str, path: &str, message: &str);
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LoggerAssetsDatabaseErrorReporter;

impl AssetsDatabaseErrorReporter for LoggerAssetsDatabaseErrorReporter {
    fn on_report(&mut self, protocol: &str, path: &str, message: &str) {
        error!(
            "Assets database loading `{}://{}` error: {}",
            protocol, path, message
        );
    }
}

pub struct AssetsDatabase {
    pub max_bytes_per_frame: Option<usize>,
    fetch_engines: Vec<Box<dyn FetchEngine>>,
    protocols: HashMap<String, Box<dyn AssetProtocol>>,
    assets: HashMap<AssetId, (String, Asset)>,
    table: HashMap<String, AssetId>,
    loading: HashMap<String, (String, Box<dyn FetchProcessReader>)>,
    #[allow(clippy::type_complexity)]
    yielded: HashMap<String, (String, Meta, Vec<(String, String)>)>,
    lately_loaded: Vec<(String, AssetId)>,
    lately_unloaded: Vec<(String, AssetId)>,
    error_reporters: HashMap<TypeId, Box<dyn AssetsDatabaseErrorReporter>>,
}

impl AssetsDatabase {
    pub fn new<FE>(fetch_engine: FE) -> Self
    where
        FE: FetchEngine + 'static,
    {
        Self {
            max_bytes_per_frame: None,
            fetch_engines: vec![Box::new(fetch_engine)],
            protocols: Default::default(),
            assets: Default::default(),
            table: Default::default(),
            loading: Default::default(),
            yielded: Default::default(),
            lately_loaded: vec![],
            lately_unloaded: vec![],
            error_reporters: Default::default(),
        }
    }

    pub fn register_error_reporter<T>(&mut self, reporter: T)
    where
        T: AssetsDatabaseErrorReporter + 'static,
    {
        self.error_reporters
            .insert(TypeId::of::<T>(), Box::new(reporter));
    }

    pub fn unregister_error_reporter<T>(&mut self)
    where
        T: AssetsDatabaseErrorReporter + 'static,
    {
        self.error_reporters.remove(&TypeId::of::<T>());
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

    pub fn loaded_ids(&self) -> Vec<AssetId> {
        self.assets.iter().map(|(id, _)| *id).collect()
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

    pub fn lately_loaded(&self) -> impl Iterator<Item = &AssetId> {
        self.lately_loaded.iter().map(|(_, id)| id)
    }

    pub fn lately_loaded_paths(&self) -> impl Iterator<Item = &str> {
        self.lately_loaded.iter().map(|(path, _)| path.as_str())
    }

    pub fn lately_loaded_protocol<'a>(
        &'a self,
        protocol: &'a str,
    ) -> impl Iterator<Item = &'a AssetId> {
        self.lately_loaded
            .iter()
            .filter_map(move |(prot, id)| if protocol == prot { Some(id) } else { None })
    }

    pub fn lately_unloaded(&self) -> impl Iterator<Item = &AssetId> {
        self.lately_unloaded.iter().map(|(_, id)| id)
    }

    pub fn lately_unloaded_paths(&self) -> impl Iterator<Item = &str> {
        self.lately_unloaded.iter().map(|(path, _)| path.as_str())
    }

    pub fn lately_unloaded_protocol<'a>(
        &'a self,
        protocol: &'a str,
    ) -> impl Iterator<Item = &'a AssetId> {
        self.lately_unloaded
            .iter()
            .filter_map(move |(prot, id)| if protocol == prot { Some(id) } else { None })
    }

    pub fn is_ready(&self) -> bool {
        self.loading.is_empty() && self.yielded.is_empty()
    }

    pub fn are_ready<I, S>(&self, iter: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        !iter
            .into_iter()
            .any(|path| !self.table.contains_key(path.as_ref()))
    }

    pub fn push_fetch_engine(&mut self, fetch_engine: Box<dyn FetchEngine>) {
        self.fetch_engines.push(fetch_engine);
    }

    pub fn pop_fetch_engine(&mut self) -> Option<Box<dyn FetchEngine>> {
        if !self.fetch_engines.is_empty() {
            self.fetch_engines.pop()
        } else {
            None
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn fetch_engine(&self) -> &Box<dyn FetchEngine> {
        self.fetch_engines.last().unwrap()
    }

    #[allow(clippy::borrowed_box)]
    pub fn fetch_engine_mut(&mut self) -> &mut Box<dyn FetchEngine> {
        self.fetch_engines.last_mut().unwrap()
    }

    pub fn with_fetch_engine<F, R>(&mut self, mut action: F) -> R
    where
        F: FnMut(&mut dyn FetchEngine) -> R,
    {
        let fetch_engine: &mut dyn FetchEngine = self.fetch_engine_mut().borrow_mut();
        action(fetch_engine)
    }

    pub fn register<FE>(&mut self, mut protocol: FE)
    where
        FE: AssetProtocol + 'static,
    {
        protocol.on_register();
        let name = protocol.name().to_owned();
        self.protocols.insert(name, Box::new(protocol));
    }

    pub fn unregister(&mut self, protocol_name: &str) -> Option<Box<dyn AssetProtocol>> {
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
                let reader = self.fetch_engine_mut().fetch(subpath);
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

    pub fn insert(&mut self, path: &str, asset: Asset) -> AssetId {
        let id = asset.id();
        self.lately_loaded.push((asset.protocol().to_owned(), id));
        self.assets.insert(id, (path.to_owned(), asset));
        self.table.insert(path.to_owned(), id);
        id
    }

    pub fn remove_by_id(&mut self, id: AssetId) -> Option<Asset> {
        if let Some((path, asset)) = self.assets.remove(&id) {
            self.table.remove(&path);
            self.lately_unloaded.push((asset.protocol().to_owned(), id));
            if let Some(protocol) = self.protocols.get_mut(asset.protocol()) {
                if let Some(list) = protocol.on_unload(&asset) {
                    self.remove_by_variants(&list);
                }
            }
            Some(asset)
        } else {
            None
        }
    }

    pub fn remove_by_path(&mut self, path: &str) -> Option<Asset> {
        if let Some(id) = self.table.remove(path) {
            if let Some((_, asset)) = self.assets.remove(&id) {
                self.lately_unloaded.push((asset.protocol().to_owned(), id));
                if let Some(protocol) = self.protocols.get_mut(asset.protocol()) {
                    if let Some(list) = protocol.on_unload(&asset) {
                        self.remove_by_variants(&list);
                    }
                }
                return Some(asset);
            }
        }
        None
    }

    pub fn remove_by_variants(&mut self, variants: &[AssetVariant]) {
        for v in variants {
            match v {
                AssetVariant::Id(id) => self.remove_by_id(*id),
                AssetVariant::Path(path) => self.remove_by_path(path),
            };
        }
    }

    pub fn id_by_path(&self, path: &str) -> Option<AssetId> {
        self.table.get(path).cloned()
    }

    pub fn path_by_id(&self, id: AssetId) -> Option<&str> {
        self.assets.get(&id).map(|(path, _)| path.as_str())
    }

    pub fn asset_by_id(&self, id: AssetId) -> Option<&Asset> {
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
        self.lately_loaded.clear();
        self.lately_unloaded.clear();
        let to_dispatch = {
            let mut bytes_read = 0;
            self.loading
                .iter()
                .filter_map(|(path, (prot, reader))| {
                    if bytes_read < self.max_bytes_per_frame.unwrap_or(std::usize::MAX) {
                        if let Some(data) = reader.read() {
                            bytes_read += data.len();
                            return Some((path.to_owned(), prot.to_owned(), data));
                        }
                    }
                    None
                })
                .collect::<Vec<_>>()
        };
        for (path, prot, data) in to_dispatch {
            if let Some(protocol) = self.protocols.get_mut(&prot) {
                match protocol.on_load_with_path(&path, data) {
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
                    AssetLoadResult::Error(message) => {
                        for reporter in self.error_reporters.values_mut() {
                            reporter.on_report(&prot, &path, &message);
                        }
                    }
                }
            }
        }
        self.loading.retain(|_, (_, reader)| {
            matches!(
                reader.status(),
                FetchStatus::InProgress(_) | FetchStatus::Done
            )
        });
        let yielded = std::mem::take(&mut self.yielded);
        for (path, (prot, meta, list)) in yielded {
            if list.iter().all(|(_, path)| self.table.contains_key(path)) {
                let ptr = self as *const Self;
                if let Some(protocol) = self.protocols.get_mut(&prot) {
                    let list = list
                        .iter()
                        .map(|(key, path)| unsafe {
                            let asset = &(*ptr).table[path];
                            let asset = &(*ptr).assets[asset].1;
                            (key.as_str(), asset)
                        })
                        .collect::<Vec<_>>();
                    match protocol.on_resume(meta, &list) {
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
                        AssetLoadResult::Error(message) => {
                            for reporter in self.error_reporters.values_mut() {
                                reporter.on_report(&prot, &path, &message);
                            }
                        }
                    }
                }
            } else {
                self.yielded.insert(path, (prot, meta, list));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::protocols::prelude::*;
    use crate::fetch::*;

    #[test]
    fn test_general() {
        let mut fetch_engine = engines::map::MapFetchEngine::default();
        fetch_engine.map.insert(
            "assets.txt".to_owned(),
            br#"
                txt://a.txt
                txt://b.txt
            "#
            .to_vec(),
        );
        fetch_engine.map.insert("a.txt".to_owned(), b"A".to_vec());
        fetch_engine.map.insert("b.txt".to_owned(), b"B".to_vec());

        let mut database = AssetsDatabase::new(fetch_engine);
        database.register(TextAssetProtocol);
        database.register(SetAssetProtocol);
        assert_eq!(database.load("set://assets.txt"), Ok(()));
        assert_eq!(database.loaded_count(), 0);
        assert_eq!(database.loading_count(), 1);
        assert_eq!(database.yielded_count(), 0);
        assert_eq!(database.yielded_deps_count(), 0);

        for _ in 0..2 {
            database.process();
        }
        assert_eq!(database.loaded_count(), 3);
        assert_eq!(database.loading_count(), 0);
        assert_eq!(database.yielded_count(), 0);
        assert_eq!(database.yielded_deps_count(), 0);

        assert!(database.asset_by_path("set://assets.txt").is_some());
        assert_eq!(
            &database
                .asset_by_path("set://assets.txt")
                .unwrap()
                .to_full_path(),
            "set://assets.txt"
        );
        assert!(database
            .asset_by_path("set://assets.txt")
            .unwrap()
            .is::<SetAsset>());
        assert_eq!(
            database
                .asset_by_path("set://assets.txt")
                .unwrap()
                .get::<SetAsset>()
                .unwrap()
                .paths()
                .to_vec(),
            vec!["txt://a.txt".to_owned(), "txt://b.txt".to_owned()],
        );

        assert!(database.asset_by_path("txt://a.txt").is_some());
        assert!(database
            .asset_by_path("txt://a.txt")
            .unwrap()
            .is::<TextAsset>());
        assert_eq!(
            database
                .asset_by_path("txt://a.txt")
                .unwrap()
                .get::<TextAsset>()
                .unwrap()
                .get(),
            "A"
        );

        assert!(database.asset_by_path("txt://b.txt").is_some());
        assert_eq!(
            database
                .asset_by_path("txt://b.txt")
                .unwrap()
                .get::<TextAsset>()
                .unwrap()
                .get(),
            "B"
        );

        assert!(database.remove_by_path("set://assets.txt").is_some());
        assert_eq!(database.loaded_count(), 0);
        assert_eq!(database.loading_count(), 0);
        assert_eq!(database.yielded_count(), 0);
        assert_eq!(database.yielded_deps_count(), 0);
    }
}
