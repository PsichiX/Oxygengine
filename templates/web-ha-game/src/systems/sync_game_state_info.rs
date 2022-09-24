use crate::{
    components::{health::*, player::*, weapon::*},
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
    Comp<&'a Events<HaVolumeOverlapEvent>>,
);

pub fn sync_game_state_info_system(universe: &mut Universe) {
    let (world, mut ui, mut info, ..) =
        universe.query_resources::<SyncGameStateInfoSystemResources>();

    let app = match ui.application_mut("") {
        Some(app) => app,
        None => return,
    };
    let mut dirty = false;

    if let Some((_, (player, health, weapon, events))) = world
        .query::<(&Player, &Health, &Weapon, &Events<HaVolumeOverlapEvent>)>()
        .iter()
        .next()
    {
        let player_info = GameStatePlayerInfo {
            health: health.0,
            health_capacity: player.health_capacity(),
            weapons: weapon.0,
            weapons_capacity: player.weapons_capacity(),
        };
        if info.player != player_info {
            info.player = player_info;
            dirty = true;
        }
        for message in events.read() {
            match message {
                HaVolumeOverlapEvent::Begin(entity) => {
                    if let Ok(name) = world.get::<&Name>(*entity) {
                        info.area = Some(name.0.as_ref().to_owned());
                        dirty = true;
                    }
                }
                HaVolumeOverlapEvent::End(_) => {
                    info.area = None;
                    dirty = true;
                }
            }
        }
    }

    if dirty {
        app.mark_dirty();
    }
}
