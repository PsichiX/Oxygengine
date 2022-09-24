use crate::components::{player::*, speed::*};
use oxygengine::prelude::*;

pub type PlayerMovementSystemResources<'a> = (
    WorldRef,
    &'a InputStack,
    &'a AppLifeCycle,
    Comp<&'a mut HaTransform>,
    Comp<&'a mut HaSpriteAnimationInstance>,
    Comp<&'a Player>,
    Comp<&'a Speed>,
    Comp<&'a InputStackInstance>,
);

pub fn player_movement_system(universe: &mut Universe) {
    let (world, input_stack, lifecycle, ..) =
        universe.query_resources::<PlayerMovementSystemResources>();

    let delta_time = lifecycle.delta_time_seconds();

    for (_, (transform, animation, speed, input)) in world
        .query::<(
            &mut HaTransform,
            &mut HaSpriteAnimationInstance,
            &Speed,
            &InputStackInstance,
        )>()
        .with::<&Player>()
        .iter()
    {
        let input = match input_stack.listener_by_instance(input) {
            Some(input) => input,
            None => continue,
        };

        let mut direction = Vec3::from(input.axes_state_or_default("move"));
        if direction.magnitude() > 1.0 {
            direction.normalize();
        }

        let velocity = direction * speed.0;
        let is_walking = velocity.magnitude_squared() > 1.0e-4;
        let position = transform.get_translation();
        transform.set_translation(position + velocity * delta_time);

        if is_walking {
            let mut scale = transform.get_scale();
            let dot = velocity.dot(Vec3::unit_x());
            if dot > 0.0 {
                scale.x = scale.x.abs();
            } else if dot < -0.0 {
                scale.x = -scale.x.abs();
            }
            transform.set_scale(scale);
        }

        animation.set_value("walk", SpriteAnimationValue::Bool(is_walking));
        animation.speed = velocity.magnitude();
    }
}
