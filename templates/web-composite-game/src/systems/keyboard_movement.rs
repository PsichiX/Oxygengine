use crate::components::{speed::Speed, KeyboardMovementTag};
use oxygengine::prelude::*;

pub type KeyboardMovementSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    &'a mut Physics2dWorld,
    &'a AppLifeCycle,
    Comp<&'a Speed>,
    Comp<&'a KeyboardMovementTag>,
    Comp<&'a RigidBody2d>,
);

pub fn keyboard_movement_system(universe: &mut Universe) {
    let (world, input, mut physics, lifecycle, ..) =
        universe.query_resources::<KeyboardMovementSystemResources>();

    // calculate force to apply.
    let dt = lifecycle.delta_time_seconds();
    let hor = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
    let ver = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
    let force = Vector::new(hor as Scalar, ver as Scalar) * dt * 7.0;

    // iterate over all bodies with speed and keyboard movement components.
    for (_, (speed, body)) in world
        .query::<(&Speed, &RigidBody2d)>()
        .with::<KeyboardMovementTag>()
        .iter()
    {
        // get physical body by handle.
        if let Some(handle) = body.handle() {
            if let Some(body) = physics.body_mut(handle) {
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
