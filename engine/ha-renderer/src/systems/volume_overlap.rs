use crate::components::{transform::*, volume::*, volume_overlap::*};
use core::{
    app::AppLifeCycle,
    ecs::{
        components::{Events, Tag},
        Comp, Entity, Universe, WorldRef,
    },
};
use std::collections::HashSet;

pub enum HaVolumeOverlapEvent {
    Begin(Entity),
    End(Entity),
}

#[derive(Debug, Default)]
pub struct HaVolumeOverlapSystemCache {
    active_overlaps: HashSet<(Entity, Entity)>,
}

pub type HaVolumeOverlapSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a mut HaVolumeOverlapSystemCache,
    Comp<&'a Tag>,
    Comp<&'a HaTransform>,
    Comp<&'a HaVolume>,
    Comp<&'a mut HaVolumeOverlap>,
    Comp<&'a mut Events<HaVolumeOverlapEvent>>,
);

pub fn ha_volume_overlap_system(universe: &mut Universe) {
    let (world, lifecycle, mut cache, ..) =
        universe.query_resources::<HaVolumeOverlapSystemResources>();

    let dt = lifecycle.delta_time_seconds();

    for (entity_a, (transform_a, volume_a, mut overlap, events)) in world
        .query::<(
            &HaTransform,
            Option<&HaVolume>,
            &mut HaVolumeOverlap,
            &mut Events<HaVolumeOverlapEvent>,
        )>()
        .iter()
    {
        overlap.time += dt;
        if overlap.time < overlap.delay {
            continue;
        }
        overlap.time = 0.0;

        for (entity_b, (tag, transform_b, volume_b)) in
            world.query::<(&Tag, &HaTransform, &HaVolume)>().iter()
        {
            if entity_a != entity_b && overlap.filters.validate_tag(tag.0.as_ref()) {
                let overlaps = if let Some(volume_a) = volume_a {
                    HaVolume::world_space_overlaps(
                        volume_a,
                        &transform_a.world_matrix(),
                        volume_b,
                        &transform_b.world_matrix(),
                    )
                    .is_some()
                } else {
                    volume_b
                        .world_space_contains(
                            &transform_b.world_matrix(),
                            transform_a.get_translation(),
                        )
                        .is_some()
                };
                let active = cache.active_overlaps.contains(&(entity_a, entity_b));
                match (overlaps, active) {
                    (true, false) => {
                        events.send(HaVolumeOverlapEvent::Begin(entity_b));
                        cache.active_overlaps.insert((entity_a, entity_b));
                    }
                    (false, true) => {
                        events.send(HaVolumeOverlapEvent::End(entity_b));
                        cache.active_overlaps.remove(&(entity_a, entity_b));
                    }
                    _ => {}
                }
            }
        }
    }
}
