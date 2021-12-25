use crate::{
    components::{health::*, player::*, weapon::*},
    resources::effects::*,
    utils::*,
};
use oxygengine::prelude::*;

pub type PlayerCollectSecretsSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    &'a Board,
    &'a HaBoardSettings,
    &'a CameraCache,
    &'a mut Effects,
    Comp<&'a BoardAvatar>,
    Comp<&'a mut Health>,
    Comp<&'a mut Weapon>,
    Comp<&'a Player>,
);

pub fn player_collect_secrets_system(universe: &mut Universe) {
    let (world, input, board, settings, camera_cache, mut effects, ..) =
        universe.query_resources::<PlayerCollectSecretsSystemResources>();

    let pointer_board_location =
        input_pointer_to_board_location(&input, &camera_cache, &board, &settings);

    if pointer_board_location.is_none() {
        return;
    }

    for (_, (avatar, player, health, weapon)) in world
        .query::<(&BoardAvatar, &Player, &mut Health, &mut Weapon)>()
        .iter()
    {
        let token = match avatar.token() {
            Some(token) => token,
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
        if let Some(secrets) = effects.remove_secret(to) {
            health.0 = (health.0 + secrets.health).min(player.health_capacity());
            weapon.0 = (weapon.0 + secrets.weapons).min(player.weapons_capacity());
        }
    }
}
