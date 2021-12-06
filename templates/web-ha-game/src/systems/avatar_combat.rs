use crate::components::{avatar_combat::*, health::*};
use oxygengine::prelude::*;

pub type AvatarCombatSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a Board,
    &'a BoardSystemCache,
    Comp<&'a BoardAvatar>,
    Comp<&'a mut AvatarCombat>,
    Comp<&'a mut Health>,
);

pub fn avatar_combat_system(universe: &mut Universe) {
    let (world, lifecycle, board, cache, ..) =
        universe.query_resources::<AvatarCombatSystemResources>();

    let dt = lifecycle.delta_time_seconds();

    for (_, (avatar, combat)) in world.query::<(&BoardAvatar, &mut AvatarCombat)>().iter() {
        if let Some(my_token) = avatar.token() {
            let attack = match combat.process(dt) {
                Some(points) => points,
                None => continue,
            };
            let location = match board.token_location(my_token) {
                Some(location) => location,
                None => continue,
            };
            for (other_token, _) in board.tokens_in_range(location, 1) {
                if my_token == other_token {
                    continue;
                }
                let entity = match cache.entity_by_token(other_token) {
                    Some(entity) => entity,
                    None => continue,
                };
                if let Ok(mut health) = world.get_mut::<Health>(entity) {
                    health.0 = health.0.saturating_sub(attack);
                }
            }
        }
    }
}
