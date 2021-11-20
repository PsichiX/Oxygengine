use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::Entity,
    Scalar,
};
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Debug, Default, Clone)]
pub struct AudioState {
    pub current_time: Option<Scalar>,
}

pub trait Audio: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn create_source(
        &mut self,
        entity: Entity,
        data: &[u8],
        streaming: bool,
        looped: bool,
        playback_rate: Scalar,
        volume: Scalar,
        play: bool,
        notify_ready: Arc<AtomicBool>,
    );
    fn destroy_source(&mut self, entity: Entity);
    fn has_source(&mut self, entity: Entity) -> bool;
    fn update_source(
        &mut self,
        entity: Entity,
        looped: bool,
        playback_rate: Scalar,
        volume: Scalar,
        play: Option<bool>,
    );
    fn get_source_state(&self, entity: Entity) -> Option<AudioState>;
    fn get_asset_id(&self, path: &str) -> Option<AssetId>;
    fn update_cache(&mut self, _assets: &AssetsDatabase) {}
}
