use crate::components::{speed::Speed, KeyboardMovementTag};
use oxygengine::prelude::*;

// system that moves tagged entities.
pub struct KeyboardMovementSystem;

impl<'s> System<'s> for KeyboardMovementSystem {
    type SystemData = (
        // we will read input.
        Read<'s, InputController>,
        // we will read delta time from app lifecycle.
        ReadExpect<'s, AppLifeCycle>,
        // we will read speed components.
        ReadStorage<'s, Speed>,
        // we will filter by tag.
        ReadStorage<'s, KeyboardMovementTag>,
        // we will write to transforms.
        WriteStorage<'s, CompositeTransform>,
    );

    fn run(
        &mut self,
        (input, lifecycle, speed, keyboard_movement, mut transforms): Self::SystemData,
    ) {
        let dt = lifecycle.delta_time_seconds() as Scalar;
        let hor = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
        let ver = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
        let offset = Vec2::new(hor, ver);

        for (_, speed, transform) in (&keyboard_movement, &speed, &mut transforms).join() {
            transform.set_translation(transform.get_translation() + offset * speed.0 * dt);
        }
    }
}
