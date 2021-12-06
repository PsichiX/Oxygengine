use crate::{components::board_avatar::*, resources::board::*};
use oxygengine_core::{
    app::AppLifeCycle,
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct BoardSystemCache {
    avatars: HashMap<Entity, BoardToken>,
    avatars_table: HashMap<BoardToken, Entity>,
}

impl BoardSystemCache {
    pub fn token_by_entity(&self, entity: Entity) -> Option<BoardToken> {
        self.avatars.get(&entity).cloned()
    }

    pub fn entity_by_token(&self, token: BoardToken) -> Option<Entity> {
        self.avatars_table.get(&token).cloned()
    }
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
            cache.avatars_table.remove(&token);
        }
    }

    for (entity, avatar) in world.query::<&mut BoardAvatar>().iter() {
        if avatar.token.is_none() {
            if let Ok(token) = board.acquire_token(avatar.location()) {
                avatar.token = Some(token);
                cache.avatars.insert(entity, token);
                cache.avatars_table.insert(token, entity);
            }
        }
        if let Some(token) = avatar.token {
            if let Some((action, time, completed)) = &mut avatar.active_action {
                if let Some(location) = board.token_location(token) {
                    *time += dt;
                    let duration = action.duration();
                    let (dx, dy) = board.location_relative(avatar.location, location);
                    if dx != 0 || dy != 0 {
                        avatar.direction = match (dx.signum(), dy.signum()) {
                            (-1, -1) => Some(BoardDirection::NorthWest),
                            (0, -1) => Some(BoardDirection::North),
                            (1, -1) => Some(BoardDirection::NorthEast),
                            (-1, 0) => Some(BoardDirection::West),
                            (1, 0) => Some(BoardDirection::East),
                            (-1, 1) => Some(BoardDirection::SouthWest),
                            (0, 1) => Some(BoardDirection::South),
                            (1, 1) => Some(BoardDirection::SouthEast),
                            _ => avatar.direction,
                        };
                    }
                    if *time >= duration {
                        avatar.location = location;
                        *time = duration;
                        *completed = true;
                    }
                }
                if !*completed {
                    continue;
                }
            }
            avatar.active_action = None;
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
