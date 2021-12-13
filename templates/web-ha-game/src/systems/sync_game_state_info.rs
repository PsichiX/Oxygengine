use crate::{
    components::{health::*, weapon::*, *},
    resources::game_state_info::*,
};
use oxygengine::prelude::*;

pub type SyncGameStateInfoSystemResources<'a> = (
    WorldRef,
    &'a mut UserInterface,
    &'a mut GameStateInfo,
    Comp<&'a Player>,
    Comp<&'a Health>,
    Comp<&'a Weapon>,
);

pub fn sync_game_state_info_system(universe: &mut Universe) {
    let (world, mut ui, mut info, ..) =
        universe.query_resources::<SyncGameStateInfoSystemResources>();

    let app = match ui.application_mut("") {
        Some(app) => app,
        None => return,
    };
    let info: &mut GameStateInfo = &mut *info;
    let mut dirty = false;

    if let Some((_, (player, health, weapon))) =
        world.query::<(&Player, &Health, &Weapon)>().iter().next()
    {
        let health_capacity = player.health_capacity();
        let weapons_capacity = player.weapons_capacity();
        let player_info = GameStatePlayerInfo {
            health: health.0.min(health_capacity),
            health_capacity,
            weapons: weapon.0.min(weapons_capacity),
            weapons_capacity,
            level: player.level,
        };
        if info.player != player_info {
            dirty = true;
            info.player = player_info;
        }
    }
    if dirty {
        app.mark_dirty();
    }
}
