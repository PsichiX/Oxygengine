use crate::{
    components::{health::*, player::*, weapon::*},
    resources::effects::*,
    TILE_VALUE_BUILDING,
};
use oxygengine::prelude::*;

pub type AreaSystemResources<'a> = (
    WorldRef,
    &'a Board,
    &'a HaBoardSettings,
    &'a mut Effects,
    Comp<&'a Events<HaVolumeOverlapEvent>>,
    Comp<&'a Player>,
    Comp<&'a HaVolumeOverlap>,
    Comp<&'a HaTransform>,
    Comp<&'a HaVolume>,
);

pub fn area_system(universe: &mut Universe) {
    let (world, board, settings, mut effects, ..) =
        universe.query_resources::<AreaSystemResources>();

    let (health_count, weapons_count) =
        match world.query::<(&Player, &Health, &Weapon)>().iter().next() {
            Some((_, (player, health, weapon))) => (
                player.health_capacity().saturating_sub(health.0),
                player.weapons_capacity().saturating_sub(weapon.0),
            ),
            None => return,
        };

    for (_, events) in world
        .query::<&Events<HaVolumeOverlapEvent>>()
        .with::<&Player>()
        .with::<&HaVolumeOverlap>()
        .iter()
    {
        for message in events.read() {
            match message {
                HaVolumeOverlapEvent::Begin(entity) => {
                    effects.clear_secrets();
                    let mut query = match world.query_one::<(&HaTransform, &HaVolume)>(*entity) {
                        Ok(query) => query,
                        _ => continue,
                    };
                    let (transform, volume) = match query.get() {
                        Some(components) => components,
                        None => continue,
                    };
                    let half_extents = match volume {
                        HaVolume::Box(half_extents) => *half_extents,
                        _ => continue,
                    };
                    let from = transform.world_matrix().mul_point(-half_extents);
                    let from = world_position_to_board_location(from, &board, &settings);
                    let to = transform.world_matrix().mul_point(half_extents);
                    let to = world_position_to_board_location(to, &board, &settings);
                    let mut location_values = board
                        .tile_values(from..to)
                        .filter(|(_, value)| *value == TILE_VALUE_BUILDING)
                        .map(|(location, _)| (location, SecretEffect::default()))
                        .collect::<Vec<_>>();
                    if !location_values.is_empty() {
                        for _ in 0..health_count {
                            let index = rand::random::<usize>() % location_values.len();
                            location_values[index].1.health += 1;
                        }
                        for _ in 0..weapons_count {
                            let index = rand::random::<usize>() % location_values.len();
                            location_values[index].1.weapons += 1;
                        }
                        for (location, data) in location_values {
                            if data.is_valid() {
                                effects.add_secret(location, data);
                            }
                        }
                    }
                }
                HaVolumeOverlapEvent::End(_) => {
                    effects.clear_secrets();
                }
            }
        }
    }
}
