use crate::{resource::VnStoryManager, vn_story_asset_protocol::VnStoryAsset};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::Universe,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct VnStorySystemCache {
    story_table: HashMap<AssetId, String>,
}

pub type VnStorySystemResources<'a> = (
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a mut VnStoryManager,
    &'a mut VnStorySystemCache,
);

pub fn vn_story_system(universe: &mut Universe) {
    let (lifecycle, assets, mut manager, mut cache) =
        universe.query_resources::<VnStorySystemResources>();

    for id in assets.lately_loaded_protocol("vn-story") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded visual novel story asset");
        let path = asset.path().to_owned();
        let asset = asset
            .get::<VnStoryAsset>()
            .expect("trying to use non visual novel story asset");
        let story = asset.get().clone();
        manager.register_story(&path, story);
        cache.story_table.insert(id, path);
    }
    for id in assets.lately_unloaded_protocol("vn-story") {
        if let Some(path) = cache.story_table.remove(id) {
            manager.unregister_story(&path);
        }
    }

    manager.process(lifecycle.delta_time_seconds());
}
