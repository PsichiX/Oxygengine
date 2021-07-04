use crate::{
    component::{CompositeRenderable, CompositeTilemap},
    composite_renderer::{Command, Image, Renderable},
    tileset_asset_protocol::{TilesetAsset, TilesetInfo},
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeTilemapSystemCache {
    images_cache: HashMap<String, String>,
    atlas_table: HashMap<AssetId, String>,
    infos_cache: HashMap<String, TilesetInfo>,
}
pub type CompositeTilemapSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a mut CompositeTilemapSystemCache,
    Comp<&'a mut CompositeRenderable>,
    Comp<&'a mut CompositeTilemap>,
);

#[allow(clippy::many_single_char_names)]
pub fn composite_tilemap_system(universe: &mut Universe) {
    let (world, assets, mut cache, ..) =
        universe.query_resources::<CompositeTilemapSystemResources>();

    for id in assets.lately_loaded_protocol("tiles") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded tileset asset");
        let path = asset.path().to_owned();
        let asset = asset
            .get::<TilesetAsset>()
            .expect("trying to use non-tileset asset");
        let image = asset.info().image_name();
        let info = asset.info().clone();
        cache.images_cache.insert(path.clone(), image);
        cache.atlas_table.insert(id, path.clone());
        cache.infos_cache.insert(path, info);
    }
    for id in assets.lately_unloaded_protocol("tiles") {
        if let Some(path) = cache.atlas_table.remove(id) {
            cache.images_cache.remove(&path);
            cache.infos_cache.remove(&path);
        }
    }

    for (_, (renderable, tilemap)) in world
        .query::<(&mut CompositeRenderable, &mut CompositeTilemap)>()
        .iter()
    {
        if tilemap.dirty {
            if let Some(tileset) = tilemap.tileset() {
                if let Some(name) = cache.images_cache.get(tileset) {
                    let r = if let Some(info) = cache.infos_cache.get(tileset) {
                        let grid = tilemap.grid();
                        let mut commands = Vec::with_capacity(grid.len() * 4);
                        for row in 0..grid.rows() {
                            for col in 0..grid.cols() {
                                if let Some(cell) = grid.get(col, row) {
                                    if !cell.visible {
                                        continue;
                                    }
                                    if let Some(frame) = info.frame(cell.col, cell.row) {
                                        let (a, b, c, d, e, f) =
                                            cell.matrix(col, row, frame.w, frame.h).into();
                                        commands.push(Command::Store);
                                        commands.push(Command::Transform(a, b, c, d, e, f));
                                        commands.push(Command::Draw(
                                            Image {
                                                image: name.to_owned().into(),
                                                source: Some(frame),
                                                destination: None,
                                                alignment: 0.0.into(),
                                            }
                                            .into(),
                                        ));
                                        commands.push(Command::Restore);
                                    }
                                }
                            }
                        }
                        Renderable::Commands(commands)
                    } else {
                        Renderable::None
                    };
                    renderable.0 = r;
                    tilemap.dirty = false;
                }
            }
        }
    }
}
