use crate::{
    board_location_to_world_position,
    components::{board_avatar_sync::*, board_chunk_sync::*},
    resources::*,
};
use oxygengine_core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
};
use oxygengine_ha_renderer::{
    asset_protocols::tilemap::TileMapAsset, components::transform::HaTransform,
};
use oxygengine_overworld::{
    components::board_avatar::BoardAvatar,
    resources::board::{Board, BoardLocation},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaBoardSyncSystemCache {
    tilemaps: HashMap<AssetId, BoardLocation>,
}

pub type HaBoardSyncSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a mut Board,
    &'a HaBoardSettings,
    &'a mut HaBoardSyncSystemCache,
    Comp<&'a mut HaTransform>,
    Comp<&'a BoardAvatar>,
    Comp<&'a HaBoardAvatarSync>,
    Comp<&'a HaBoardChunkSync>,
);

pub fn ha_board_sync_system(universe: &mut Universe) {
    let (world, assets, mut board, settings, mut cache, ..) =
        universe.query_resources::<HaBoardSyncSystemResources>();

    let mut rebuild_board_navigation = false;
    for id in assets.lately_loaded_protocol("tilemap") {
        if let Some(asset) = assets.asset_by_id(*id) {
            if let Some(asset) = asset.get::<TileMapAsset>() {
                let location = (
                    asset.x / asset.cols as isize / asset.cell_size.x as isize,
                    asset.y / asset.rows as isize / asset.cell_size.y as isize,
                )
                    .into();
                if board.ensure_chunk(location).is_ok() {
                    cache.tilemaps.insert(*id, location);
                    let traverse_rules = board.traverse_rules.clone();
                    let chunk = board.chunk_mut(location).unwrap();
                    for (from, to) in asset
                        .values
                        .iter()
                        .copied()
                        .zip(chunk.write_values().iter_mut())
                    {
                        if settings.is_tile_value_valid(from) {
                            *to = Some(from);
                        } else {
                            *to = None;
                        }
                    }
                    if chunk.rebuild_navigation(&traverse_rules).is_ok() {
                        rebuild_board_navigation = true;
                    }
                }
            }
        }
    }
    for id in assets.lately_unloaded_protocol("tilemap") {
        if let Some(location) = cache.tilemaps.remove(id) {
            cache.tilemaps.remove(id);
            if board.destroy_chunk(location).is_ok() {
                rebuild_board_navigation = true;
            }
        }
    }

    if rebuild_board_navigation {
        let _ = board.rebuild_navigation();
    }

    for (_, (transform, avatar, sync)) in world
        .query::<(&mut HaTransform, &BoardAvatar, &HaBoardAvatarSync)>()
        .iter()
    {
        let from = board_location_to_world_position(avatar.location(), &board, &settings);
        let mut position = avatar
            .active_action()
            .and_then(|(action, time, _)| Some((avatar.token()?, action, time)))
            .and_then(|(token, action, time)| {
                let to = board_location_to_world_position(
                    board.token_location(token)?,
                    &board,
                    &settings,
                );
                let diff = to - from;
                Some(from + diff * action.progress(time))
            })
            .unwrap_or(from)
            + sync.offset;
        if sync.snap_to_pixel {
            position.x = position.x.round();
            position.y = position.y.round();
        }
        transform.set_translation(position);
    }

    for (_, (transform, sync)) in world
        .query::<(&mut HaTransform, &HaBoardChunkSync)>()
        .iter()
    {
        let position =
            board_location_to_world_position((sync.0, (0, 0).into()).into(), &board, &settings);
        transform.set_translation(position);
    }
}
