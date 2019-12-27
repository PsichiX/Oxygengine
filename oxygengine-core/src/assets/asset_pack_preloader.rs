use crate::{
    assets::{database::AssetsDatabase, protocols::pack::PackAsset},
    error::*,
};

#[derive(Debug)]
pub struct AssetPackPreloader {
    paths: Vec<String>,
    ready_pack: bool,
    ready_from_pack: bool,
}

impl AssetPackPreloader {
    pub fn new(
        path: &str,
        assets: &mut AssetsDatabase,
        assets_from_pack: Vec<&str>,
    ) -> Result<Self> {
        let path = format!("pack://{}", path);
        if let Err(error) = assets.load(&path) {
            Err(Error::Message(format!(
                "Could not load asset pack: {}\n{:?}",
                path, error
            )))
        } else {
            let paths = std::iter::once(path)
                .chain(assets_from_pack.into_iter().map(|p| p.to_owned()))
                .collect::<Vec<String>>();
            Ok(Self {
                paths,
                ready_pack: false,
                ready_from_pack: false,
            })
        }
    }

    pub fn process(&mut self, assets: &mut AssetsDatabase) -> Result<bool> {
        if !self.ready_pack {
            if assets.are_ready(self.paths.iter().take(1)) {
                let path = &self.paths[0];
                if let Some(asset) = assets.asset_by_path(path) {
                    if let Some(pack) = asset.get::<PackAsset>() {
                        let engine = pack.make_fetch_engine();
                        assets.push_fetch_engine(Box::new(engine));
                        self.ready_pack = true;
                        if self.paths.len() > 1 {
                            for path in self.paths.iter().skip(1) {
                                if let Err(error) = assets.load(path) {
                                    return Err(Error::Message(format!(
                                        "Cannot load asset from pack: {}\n{:?}",
                                        path, error
                                    )));
                                }
                            }
                        } else {
                            self.ready_from_pack = true;
                        }
                    } else {
                        return Err(Error::Message(format!("Asset is not a pack: {}", path)));
                    }
                } else {
                    return Err(Error::Message(format!(
                        "Asset pack is not loaded: {}",
                        path
                    )));
                }
            }
            if !self.ready_pack {
                return Ok(false);
            }
        }
        if !self.ready_from_pack {
            if assets.are_ready(self.paths.iter().skip(1)) {
                self.ready_from_pack = true;
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn is_ready(&self) -> bool {
        self.ready_pack && self.ready_from_pack
    }
}
