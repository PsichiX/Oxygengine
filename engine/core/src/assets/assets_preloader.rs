use crate::{assets::database::AssetsDatabase, error::*, fetch::FetchEngine};

enum Phase {
    None,
    Start(Box<dyn FetchEngine>),
    Loading,
    Ready,
}

pub struct AssetsPreloader {
    paths: Vec<String>,
    phase: Phase,
}

impl AssetsPreloader {
    pub fn new<E>(engine: E, paths: Vec<&str>) -> Self
    where
        E: FetchEngine + 'static,
    {
        let paths = paths.into_iter().map(|p| p.to_owned()).collect::<Vec<_>>();
        Self {
            paths,
            phase: Phase::Start(Box::new(engine)),
        }
    }

    pub fn process(&mut self, assets: &mut AssetsDatabase) -> Result<bool> {
        match std::mem::replace(&mut self.phase, Phase::None) {
            Phase::Start(engine) => {
                assets.push_fetch_engine(engine);
                if self.paths.is_empty() {
                    self.phase = Phase::Ready;
                } else {
                    self.phase = Phase::Loading;
                    for path in &self.paths {
                        if let Err(error) = assets.load(path) {
                            return Err(Error::Message(format!(
                                "Cannot load asset: {}\n{:?}",
                                path, error
                            )));
                        }
                    }
                }
            }
            Phase::Loading => {
                if assets.is_ready() && assets.are_ready(self.paths.iter()) {
                    assets.pop_fetch_engine();
                    self.phase = Phase::Ready;
                } else {
                    self.phase = Phase::Loading;
                }
            }
            Phase::Ready => {
                self.phase = Phase::Ready;
                return Ok(true);
            }
            Phase::None => {}
        }
        Ok(false)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.phase, Phase::Ready)
    }
}
