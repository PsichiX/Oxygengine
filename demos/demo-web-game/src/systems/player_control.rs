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

        if let Some(active) = turns.selected_playing() {
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

                            // let isometry = body.position();
                            // let p = isometry.translation;
                            // let r = isometry.rotation.angle();
                            //
                            // let entity = world
                            //     .create_entity()
                            //     .with(CompositeRenderable(().into()))
                            //     .with(CompositeRenderDepth(25.0))
                            //     .with(
                            //         CompositeSprite::new("sprites.0.json".into(), tank_type.into())
                            //             .align(0.5.into()),
                            //     )
                            //     .with(CompositeTransform::default())
                            //     .with(RigidBody2d::new(
                            //         RigidBodyDesc::new()
                            //             .translation(Vector::new(x as f64, y as f64))
                            //             .gravity_enabled(false)
                            //             .rotation(r)
                            //             .linear_damping(0.5)
                            //             .angular_damping(2.0),
                            //     ))
                            //     .with(Collider2d::new(
                            //         ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector::new(40.0, 40.0))))
                            //             .density(1.0),
                            //     ))
                            //     .with(Collider2dBody::Me)
                            //     .with(Physics2dSyncCompositeTransform)
                            //     .with(NonPersistent(token))
                            //     .with(Speed(50.0, 0.1))
                            //     .with(Player(t))
                            //     .build();
                        }
                    }
                }
            }
        }
    }
}
