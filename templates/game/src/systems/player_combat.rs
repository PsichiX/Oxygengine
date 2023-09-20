use crate::{
    components::{avatar_combat::*, player::*, weapon::*},
    resources::effects::*,
};
use oxygengine::prelude::*;

const POINTS_PER_ATTACK: usize = 1;
const ATTACK_EFFECT_DURATION: Scalar = 0.5;

pub type PlayerCombatSystemResources<'a> = (
    WorldRef,
    &'a InputStack,
    &'a Board,
    &'a BoardSystemCache,
    &'a mut Effects,
    Comp<&'a BoardAvatar>,
    Comp<&'a mut AvatarCombat>,
    Comp<&'a mut Weapon>,
    Comp<&'a Player>,
    Comp<&'a InputStackInstance>,
);

pub fn player_combat_system(universe: &mut Universe) {
    let (world, input_stack, board, board_cache, mut effects, ..) =
        universe.query_resources::<PlayerCombatSystemResources>();

    for (_, (avatar, input, combat, weapon)) in world
        .query::<(
            &BoardAvatar,
            &InputStackInstance,
            &mut AvatarCombat,
            &mut Weapon,
        )>()
        .with::<&Player>()
        .iter()
    {
        let input = match input_stack.listener_by_instance(input) {
            Some(input) => input,
            None => continue,
        };
        if !input.trigger_state_or_default("action").is_on() {
            continue;
        }

        let token = match avatar.token() {
            Some(token) => token,
            _ => continue,
        };

        let location = match board.token_location(token) {
            Some(location) => location,
            _ => continue,
        };

        for other_location in board.locations_around_cardinal(location, 1) {
            if let Ok(Some(other_token)) = board.occupancy(other_location) {
                if let Some(entity) = board_cache.entity_by_token(other_token) {
                    if let Some(new_count) = weapon.0.checked_sub(POINTS_PER_ATTACK) {
                        if combat.try_attack(entity, POINTS_PER_ATTACK) {
                            weapon.0 = new_count;
                            effects.attack(other_location, ATTACK_EFFECT_DURATION);
                        }
                    }
                }
            }
        }
    }
}
