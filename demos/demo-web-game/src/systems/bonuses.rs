#![allow(clippy::type_complexity)]

use crate::{
    components::{
        ammo::Ammo,
        bonus::{Bonus, BonusType},
        health::Health,
    },
    resources::spawner::{DespawnEffect, Spawner},
};
use oxygengine::prelude::*;

pub struct BonusesSystem;

impl<'s> System<'s> for BonusesSystem {
    type SystemData = (
        Read<'s, Physics2dWorld>,
        Write<'s, Spawner>,
        WriteStorage<'s, Health>,
        WriteStorage<'s, Ammo>,
        ReadStorage<'s, Bonus>,
    );

    fn run(&mut self, (world, mut spawner, mut healths, mut ammo, bonuses): Self::SystemData) {
        for (a, b) in world.active_contacts() {
            if let Some(bonus) = bonuses.get(*a) {
                spawner.despawn(*a, DespawnEffect::None);
                match bonus.0 {
                    BonusType::Health(v) => {
                        if let Some(health) = healths.get_mut(*b) {
                            health.0 += v;
                        }
                    }
                    BonusType::Ammo(v) => {
                        if let Some(ammo) = ammo.get_mut(*b) {
                            ammo.0 += v;
                        }
                    }
                }
            }
        }
    }
}
