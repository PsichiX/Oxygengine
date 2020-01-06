#![allow(clippy::type_complexity)]

use crate::{
    components::{player::Player, speed::Speed},
    resources::turn::TurnManager,
};
use oxygengine::prelude::*;

pub struct PlayerControlSystem;

impl<'s> System<'s> for PlayerControlSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, InputController>,
        Write<'s, Physics2dWorld>,
        ReadExpect<'s, AppLifeCycle>,
        Read<'s, TurnManager>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Speed>,
        ReadStorage<'s, RigidBody2d>,
    );

    fn run(
        &mut self,
        (entities, input, mut world, lifecycle, turns, players, speed, bodies): Self::SystemData,
    ) {
        let dt = lifecycle.delta_time_seconds();
        let hor =
            -input.axis_or_default("move-left") as f64 + input.axis_or_default("move-right") as f64;
        let ver =
            -input.axis_or_default("move-up") as f64 + input.axis_or_default("move-down") as f64;

        if let Some(active) = turns.selected() {
            for (entity, _, speed, body) in (&entities, &players, &speed, &bodies).join() {
                if entity == active {
                    if let Some(handle) = body.handle() {
                        if let Some(body) = world.body_mut(handle) {
                            body.apply_local_force(
                                0,
                                &Force::from_slice(&[0.0, speed.0 * -ver * dt, hor * speed.1]),
                                ForceType::VelocityChange,
                                true,
                            );
                        }
                    }
                }
            }
        }
    }
}
