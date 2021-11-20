use crate::{components::board_avatar::*, resources::board::*};
use oxygengine_core::{
    app::AppLifeCycle,
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct BoardSystemCache {
    avatars: HashMap<Entity, BoardToken>,
}

pub type BoardSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a EntityChanges,
    &'a mut Board,
    &'a mut BoardSystemCache,
    Comp<&'a mut BoardAvatar>,
);

pub fn board_system(universe: &mut Universe) {
    let (world, lifecycle, changes, mut board, mut cache, ..) =
        universe.query_resources::<BoardSystemResources>();

    let dt = lifecycle.delta_time_seconds();

    for entity in changes.despawned() {
        if let Some(token) = cache.avatars.remove(&entity) {
            board.release_token(token);
        }
    }

    for (entity, avatar) in world.query::<&mut BoardAvatar>().iter() {
        if avatar.token.is_none() {
            if let Ok(token) = board.acquire_token(avatar.location()) {
                avatar.token = Some(token);
                cache.avatars.insert(entity, token);
            }
        }
        if let Some(token) = avatar.token {
            if let Some((action, time, completed)) = &mut avatar.active_action {
                if *completed {
                    avatar.active_action = None;
                } else {
                    *time += dt;
                    let duration = action.duration();
                    if *time >= duration {
                        if let Some(location) = board.token_location(token) {
                            avatar.location = location;
                        }
                        *time = duration;
                        *completed = true;
                    }
                    continue;
                }
            }
            if let Some(action) = avatar.actions_queue.pop_front() {
                let success = match &action {
                    BoardAvatarAction::Move { x, y, .. } => board.move_token(token, *x, *y).is_ok(),
                    BoardAvatarAction::MoveStep { direction, .. } => {
                        board.move_step_token(token, *direction).is_ok()
                    }
                    BoardAvatarAction::Teleport { location, .. } => {
                        board.teleport_token(token, *location).is_ok()
                    }
                };
                if success {
                    avatar.active_action = Some((action, 0.0, false));
                } else {
                    avatar.actions_queue.clear();
                    avatar.active_action = None;
                }
            }
        }
    }
}
