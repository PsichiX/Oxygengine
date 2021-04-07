#![allow(clippy::type_complexity)]

use crate::components::{
    keyboard_movement::{Direction, KeyboardMovement},
    speed::Speed,
};
use oxygengine::prelude::*;

// system that moves tagged entities.
pub struct KeyboardMovementSystem;

impl<'s> System<'s> for KeyboardMovementSystem {
    type SystemData = (
        Read<'s, InputController>,
        ReadExpect<'s, AppLifeCycle>,
        Read<'s, UserInterfaceRes>,
        ReadStorage<'s, Speed>,
        WriteStorage<'s, KeyboardMovement>,
        WriteStorage<'s, CompositeTransform>,
        WriteStorage<'s, CompositeSpriteAnimation>,
    );

    fn run(
        &mut self,
        (
            input,
            lifecycle,
            user_interface,
            speed,
            mut keyboard_movement,
            mut transform,
            mut animation,
        ): Self::SystemData,
    ) {
        if user_interface.last_frame_captured() {
            return;
        }

        let dt = lifecycle.delta_time_seconds();
        let hor = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
        let ver = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
        let diff = Vec2::new(hor, ver) * dt;
        let is_moving = hor.abs() + ver.abs() > 0.01;

        let iter = (
            &mut keyboard_movement,
            &speed,
            &mut transform,
            &mut animation,
        )
            .join();
        for (keyboard_movement, speed, transform, animation) in iter {
            transform.set_translation(transform.get_translation() + diff * speed.0);

            let direction = if !is_moving {
                keyboard_movement.direction
            } else if hor.abs() > 0.5 {
                if hor < 0.5 {
                    Direction::West
                } else {
                    Direction::East
                }
            } else if ver < 0.5 {
                Direction::North
            } else {
                Direction::South
            };
            let was_moving = keyboard_movement.is_moving;
            let old_direction = keyboard_movement.direction;

            if direction != old_direction || is_moving != was_moving {
                if is_moving {
                    let name = match direction {
                        Direction::East => "walk-east",
                        Direction::West => "walk-west",
                        Direction::North => "walk-north",
                        Direction::South => "walk-south",
                    };
                    animation.play(name, 4.0, true);
                } else {
                    let name = match direction {
                        Direction::East => "idle-east",
                        Direction::West => "idle-west",
                        Direction::North => "idle-north",
                        Direction::South => "idle-south",
                    };
                    animation.play(name, 4.0, true);
                }
                keyboard_movement.is_moving = is_moving;
                keyboard_movement.direction = direction;
            }
        }
    }
}
