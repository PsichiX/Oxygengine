use crate::{
    components::{avatar_combat::*, blink::*, health::*, level_up::*},
    resources::GlobalEvent,
    utils::*,
};
use oxygengine::prelude::*;

pub type AvatarCombatSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a Board,
    &'a BoardSystemCache,
    &'a mut Events<GlobalEvent>,
    Comp<&'a BoardAvatar>,
    Comp<&'a mut AvatarCombat>,
    Comp<&'a mut Health>,
    Comp<&'a mut Blink>,
    Comp<&'a LevelUp>,
);

pub fn avatar_combat_system(universe: &mut Universe) {
    let (world, lifecycle, board, cache, mut events, ..) =
        universe.query_resources::<AvatarCombatSystemResources>();

    let dt = lifecycle.delta_time_seconds();

    for (my_entity, (avatar, combat)) in world.query::<(&BoardAvatar, &mut AvatarCombat)>().iter() {
        let my_token = match avatar.token() {
            Some(token) => token,
            None => continue,
        };
        let (entity, attack) = match combat.process(dt) {
            Some(data) => data,
            None => continue,
        };
        let my_location = match board.token_location(my_token) {
            Some(location) => location,
            None => continue,
        };
        let other_token = match cache.token_by_entity(entity) {
            Some(token) => token,
            None => continue,
        };
        if my_token == other_token {
            continue;
        }
        let other_location = match board.token_location(other_token) {
            Some(location) => location,
            None => continue,
        };
        let (dx, dy) = board.location_relative(my_location, other_location);
        if !is_touching_side(dx, dy) {
            continue;
        }
        let assume_dead = if let Ok(mut health) = world.get_mut::<Health>(entity) {
            health.0 = health.0.saturating_sub(attack);
            health.0 == 0
        } else {
            false
        };
        if let Ok(mut blink) = world.get_mut::<Blink>(entity) {
            blink.0 = 0.5;
        }
        if assume_dead {
            if let Ok(level_up) = world.get::<LevelUp>(entity) {
                events.send(GlobalEvent::LevelUp(my_entity, level_up.0));
            }
        }
    }
}
