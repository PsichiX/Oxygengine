#![allow(clippy::type_complexity)]

use crate::{
    components::{death::Death, health::Health},
    resources::spawner::{DespawnEffect, Spawner},
};
use oxygengine::prelude::*;

pub struct HealthSystem;

impl<'s> System<'s> for HealthSystem {
    type SystemData = (
        Entities<'s>,
        Write<'s, Spawner>,
        ReadStorage<'s, Health>,
        ReadStorage<'s, Death>,
    );

    fn run(&mut self, (entities, mut spawner, healths, deaths): Self::SystemData) {
        for (entity, health) in (&entities, &healths).join() {
            if health.0 == 0 {
                let effect = if let Some(death) = deaths.get(entity) {
                    death.0
                } else {
                    DespawnEffect::None
                };
                spawner.despawn(entity, effect);
            }
        }
    }
}
