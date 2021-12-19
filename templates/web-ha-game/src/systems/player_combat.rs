use crate::{
    components::{avatar_combat::*, player::*, weapon::*},
    resources::effects::*,
    utils::*,
};
use oxygengine::prelude::*;

const POINTS_PER_ATTACK: usize = 1;
const ATTACK_EFFECT_DURATION: Scalar = 0.5;

pub type PlayerCombatSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    &'a Board,
    &'a HaBoardSettings,
    &'a CameraCache,
    &'a BoardSystemCache,
    &'a mut Effects,
    Comp<&'a BoardAvatar>,
    Comp<&'a mut AvatarCombat>,
    Comp<&'a mut Weapon>,
    Comp<&'a Player>,
);

pub fn player_combat_system(universe: &mut Universe) {
    let (world, input, board, settings, camera_cache, board_cache, mut effects, ..) =
        universe.query_resources::<PlayerCombatSystemResources>();

    let pointer_board_location =
        input_pointer_to_board_location(&input, &camera_cache, &board, &settings);

    if pointer_board_location.is_none() {
        return;
    }

    for (_, (avatar, combat, weapon)) in world
        .query::<(&BoardAvatar, &mut AvatarCombat, &mut Weapon)>()
        .with::<Player>()
        .iter()
    {
        let (points, token) = match (weapon.0.checked_sub(POINTS_PER_ATTACK), avatar.token()) {
            (Some(points), Some(token)) => (points, token),
            _ => continue,
        };
        let (from, to) = match (board.token_location(token), pointer_board_location) {
            (Some(from), Some(to)) => (from, to),
            _ => continue,
        };
        let (dx, dy) = board.location_relative(from, to);
        if !is_touching_side(dx, dy) {
            continue;
        }
        let other_token = match board.occupancy(to) {
            Ok(Some(token)) => token,
            _ => continue,
        };
        if let Some(entity) = board_cache.entity_by_token(other_token) {
            combat.try_attack(entity, POINTS_PER_ATTACK);
            weapon.0 = points;
            effects.attack(to, ATTACK_EFFECT_DURATION);
        }
    }
}
