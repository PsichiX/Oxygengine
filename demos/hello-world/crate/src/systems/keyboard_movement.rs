use crate::components::{KeyboardMovementTag, Speed};
use oxygengine::prelude::*;

pub struct KeyboardMovementSystem;

impl<'s> System<'s> for KeyboardMovementSystem {
    type SystemData = (
        Read<'s, InputController>,
        ReadExpect<'s, AppLifeCycle>,
        ReadStorage<'s, Speed>,
        ReadStorage<'s, KeyboardMovementTag>,
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
