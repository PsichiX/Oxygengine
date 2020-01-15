#![allow(clippy::type_complexity)]

use crate::components::{speed::Speed, KeyboardMovementTag};
use oxygengine::prelude::*;

// system that moves tagged entities.
pub struct KeyboardMovementSystem;

impl<'s> System<'s> for KeyboardMovementSystem {
    type SystemData = (
        // we will read input.
        Read<'s, InputController>,
        // we will read physics world.
        Write<'s, Physics2dWorld>,
        // we will read delta time from app lifecycle.
        ReadExpect<'s, AppLifeCycle>,
        // we will read speed components.
        ReadStorage<'s, Speed>,
        // we will filter by tag.
        ReadStorage<'s, KeyboardMovementTag>,
        // we will write to rigid body.
        ReadStorage<'s, RigidBody2d>,
    );

    fn run(
        &mut self,
        (input, mut world, lifecycle, speed, keyboard_movement, bodies): Self::SystemData,
    ) {
        // calculate force to apply.
        let dt = lifecycle.delta_time_seconds();
        let hor = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
        let ver = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
        let force = Vector::new(hor as f64, ver as f64) * dt;

        // iterate over all bodies with speed and keyboard movement components.
        for (_, speed, body) in (&keyboard_movement, &speed, &bodies).join() {
            // get physical body by handle.
            if let Some(handle) = body.handle() {
                if let Some(body) = world.body_mut(handle) {
                    // apply force as velocity change multiplayed by delta time.
                    body.apply_force(
                        0,
                        &Force::linear(force * speed.0),
                        ForceType::VelocityChange,
                        true,
                    );
                }
            }
        }
    }
}
