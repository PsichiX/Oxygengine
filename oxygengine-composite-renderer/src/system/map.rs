use crate::{
    component::{CompositeMapChunk, CompositeRenderable, CompositeSurfaceCache},
    composite_renderer::Renderable,
    map_asset_protocol::{Map, MapAsset},
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeMapSystemCache {
    maps_cache: HashMap<String, Map>,
    maps_table: HashMap<AssetId, String>,
}

pub type CompositeMapSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a mut CompositeMapSystemCache,
    Comp<&'a mut CompositeMapChunk>,
    Comp<&'a mut CompositeRenderable>,
    Comp<&'a mut CompositeSurfaceCache>,
);

pub fn composite_map_system(universe: &mut Universe) {
    let (world, assets, mut cache, ..) = universe.query_resources::<CompositeMapSystemResources>();

    for id in assets.lately_loaded_protocol("map") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded map asset");
        let path = asset.path().to_owned();
        let asset = asset
            .get::<MapAsset>()
            .expect("trying to use non-map asset");
        let map = asset.map().clone();
        cache.maps_cache.insert(path.clone(), map);
        cache.maps_table.insert(id, path);
    }
    for id in assets.lately_unloaded_protocol("map") {
        if let Some(path) = cache.maps_table.remove(id) {
            cache.maps_cache.remove(&path);
        }
    }

    for (_, (chunk, renderable, surface)) in world
        .query::<(
            &mut CompositeMapChunk,
            &mut CompositeRenderable,
            Option<&mut CompositeSurfaceCache>,
        )>()
        .iter()
    {
        if chunk.dirty {
            if let Some(map) = cache.maps_cache.get(chunk.map_name()) {
                let r = if let Some(commands) = map.build_render_commands_from_layer_by_name(
                    chunk.layer_name(),
                    chunk.offset(),
                    chunk.size(),
                    &assets,
                ) {
                    Renderable::Commands(commands)
                } else {
                    Renderable::None
                };
                renderable.0 = r;
                if let Some(surface) = surface {
                    surface.rebuild();
                }
                chunk.dirty = false;
            }
        }
    }
}
