use crate::components::{avatar_combat::*, weapon::*, *};
use oxygengine::prelude::*;

const POINTS_PER_ATTACK: usize = 1;

pub type PlayerCombatSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    Comp<&'a mut AvatarCombat>,
    Comp<&'a mut Weapon>,
    Comp<&'a Player>,
);

pub fn player_combat_system(universe: &mut Universe) {
    let (world, input, ..) = universe.query_resources::<PlayerCombatSystemResources>();

    if !input.trigger_or_default("attack").is_pressed() {
        return;
    }

    for (_, (combat, weapon)) in world
        .query::<(&mut AvatarCombat, &mut Weapon)>()
        .with::<Player>()
        .iter()
    {
        if let Some(points) = weapon.0.checked_sub(POINTS_PER_ATTACK) {
            combat.try_attack(POINTS_PER_ATTACK);
            weapon.0 = points;
        }
    }
}
