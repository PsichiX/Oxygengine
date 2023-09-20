use crate::{
    components::{health::*, player::*, weapon::*},
    resources::effects::*,
};
use oxygengine::prelude::*;

pub type PlayerCollectSecretsSystemResources<'a> = (
    WorldRef,
    &'a Board,
    &'a mut Effects,
    Comp<&'a BoardAvatar>,
    Comp<&'a mut Health>,
    Comp<&'a mut Weapon>,
    Comp<&'a Player>,
);

pub fn player_collect_secrets_system(universe: &mut Universe) {
    let (world, board, mut effects, ..) =
        universe.query_resources::<PlayerCollectSecretsSystemResources>();

    for (_, (avatar, player, health, weapon)) in world
        .query::<(&BoardAvatar, &Player, &mut Health, &mut Weapon)>()
        .iter()
    {
        if let Some(token) = avatar.token() {
            if let Some(location) = board.token_location(token) {
                for location in board.locations_around_cardinal(location, 1) {
                    if let Some(secrets) = effects.remove_secret(location) {
                        health.0 = (health.0 + secrets.health).min(player.health_capacity());
                        weapon.0 = (weapon.0 + secrets.weapons).min(player.weapons_capacity());
                    }
                }
            }
        }
    }
}
