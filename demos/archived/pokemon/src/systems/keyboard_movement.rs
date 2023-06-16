#![allow(clippy::type_complexity)]

use crate::components::{
    keyboard_movement::{Direction, KeyboardMovement},
    speed::Speed,
};
use oxygengine::prelude::*;

pub type KeyboardMovementSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    &'a AppLifeCycle,
    Comp<&'a Speed>,
    Comp<&'a mut KeyboardMovement>,
    Comp<&'a mut CompositeTransform>,
    Comp<&'a mut CompositeSpriteAnimation>,
);

// system that moves tagged entities.
pub fn keyboard_movement_system(universe: &mut Universe) {
    let (world, inputs, lifecycle, ..) =
        universe.query_resources::<KeyboardMovementSystemResources>();

    let dt = lifecycle.delta_time_seconds();
    let diff = Vec2::from(
        inputs
            .mirror_multi_axis_or_default([("move-left", "move-right"), ("move-up", "move-down")]),
    );
    let is_moving = diff.sqr_magnitude() > 0.01;
    let diff = diff * dt;

    for (_, (speed, keyboard_movement, transform, animation)) in world
        .query::<(
            &Speed,
            &mut KeyboardMovement,
            &mut CompositeTransform,
            &mut CompositeSpriteAnimation,
        )>()
        .iter()
    {
        transform.set_translation(transform.get_translation() + diff * speed.0);

        let direction = if !is_moving {
            keyboard_movement.direction
        } else if diff.x.abs() > 0.5 {
            if diff.x < 0.5 {
                Direction::West
            } else {
                Direction::East
            }
        } else if diff.y < 0.5 {
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
