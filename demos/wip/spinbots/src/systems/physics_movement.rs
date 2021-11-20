use crate::{
    components::spinbot::SpinBot,
    utils::physics::{Arena, GRAVITY},
};
use oxygengine::prelude::*;

pub type PhysicsMovementSystemResources<'a> = (
    WorldRef,
    &'a Arena,
    &'a mut Physics2dWorld,
    Comp<&'a RigidBody2d>,
);

pub fn physics_movement_system(universe: &mut Universe) {
    let (world, arena, mut physics, ..) =
        universe.query_resources::<PhysicsMovementSystemResources>();

    for (_, body) in world.query::<&RigidBody2d>().iter() {
        if let Some(handle) = body.handle() {
            if let Some(body) = physics.body_mut(handle) {
                let pos = body.position().translation.vector;
                let g = arena
                    .shape
                    .normal_2d(Vec2::new(pos.x, pos.y))
                    .unwrap_or_default()
                    * GRAVITY;
                let force = Force::linear(Vector::new(g.x, g.y));
                body.apply_force(0, &force, ForceType::AccelerationChange, false);
            }
        }
    }
}
