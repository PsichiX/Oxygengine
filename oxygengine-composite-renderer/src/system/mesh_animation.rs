use crate::{
    component::{CompositeMesh, CompositeMeshAnimation},
    mesh_animation_asset_protocol::{MeshAnimation, MeshAnimationAsset},
};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
    Scalar,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeMeshAnimationSystemCache {
    animations_cache: HashMap<String, MeshAnimation>,
    animations_table: HashMap<AssetId, String>,
}

pub type CompositeMeshAnimationSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a mut CompositeMeshAnimationSystemCache,
    Comp<&'a mut CompositeMesh>,
    Comp<&'a mut CompositeMeshAnimation>,
);

pub fn composite_mesh_animation_system(universe: &mut Universe) {
    let (world, lifecycle, assets, mut cache, ..) =
        universe.query_resources::<CompositeMeshAnimationSystemResources>();

    for id in assets.lately_loaded_protocol("mesh-anim") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded mesh animation asset");
        let path = asset.path().to_owned();
        let asset = asset
            .get::<MeshAnimationAsset>()
            .expect("trying to use non-mesh-animation asset");
        let animation = asset.animation().clone();
        cache.animations_cache.insert(path.clone(), animation);
        cache.animations_table.insert(id, path);
    }
    for id in assets.lately_unloaded_protocol("mesh-anim") {
        if let Some(path) = cache.animations_table.remove(id) {
            cache.animations_cache.remove(&path);
        }
    }

    let dt = lifecycle.delta_time_seconds() as Scalar;
    for (_, (mesh, animation)) in world
        .query::<(&mut CompositeMesh, &mut CompositeMeshAnimation)>()
        .iter()
    {
        if animation.dirty {
            if let Some(asset) = cache.animations_cache.get(animation.animation()) {
                animation.process(dt, asset, mesh);
            }
        }
    }
}
