#![allow(clippy::type_complexity)]

use crate::{
    components::{
        bullet::Bullet, death::Death, health::Health, lifetime::Lifetime, player::Player,
    },
    resources::spawner::{DespawnEffect, Spawner},
};
use oxygengine::prelude::*;

pub struct DestructionSystem;

impl<'s> System<'s> for DestructionSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Physics2dWorld>,
        ReadExpect<'s, AppLifeCycle>,
        Write<'s, Spawner>,
        ReadStorage<'s, Bullet>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Health>,
        ReadStorage<'s, Death>,
        WriteStorage<'s, Lifetime>,
    );

    fn run(
        &mut self,
        (
            entities,
            world,
            lifecycle,
            mut spawner,
            bullets,
            players,
            mut healths,
            deaths,
            mut lifetimes,
        ): Self::SystemData,
    ) {
        for (a, b) in world.active_contacts() {
            if let Some(bullet) = bullets.get(*a) {
                if let Some(health) = healths.get_mut(*a) {
                    if health.0 > 0 {
                        health.0 -= 1;
                    }
                } else {
                    let effect = if let Some(death) = deaths.get(*a) {
                        death.0
                    } else {
                        DespawnEffect::None
                    };
                    spawner.despawn(*a, effect);
                }
                let effect = if let Some(death) = deaths.get(*b) {
                    death.0
                } else {
                    DespawnEffect::None
                };
                if let Some(player) = players.get(*b) {
                    if bullet.0 != player.0 {
                        if let Some(health) = healths.get_mut(*b) {
                            if health.0 > 0 {
                                health.0 -= 1;
                            }
                        } else {
                            spawner.despawn(*b, effect);
                        }
                    }
                } else {
                    spawner.despawn(*b, effect);
                }
            }
        }

        let dt = lifecycle.delta_time_seconds();
        for (entity, lifetime) in (&entities, &mut lifetimes).join() {
            lifetime.0 -= dt;
            if lifetime.0 <= 0.0 {
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
