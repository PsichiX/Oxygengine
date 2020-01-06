use crate::components::{speed::Speed, KeyboardMovementTag};
use oxygengine::prelude::*;

pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        // create entity with camera to view scene.
        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect))
            .with(CompositeTransform::scale(400.0.into()))
            .with(AudioSource::from(
                AudioSourceConfig::new("ambient.ogg".into())
                    .streaming(true)
                    .play(true),
            ))
            .build();

        // create ground entity.
        world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::green().a(128),
                    rect: Rect::with_size([1024.0, 64.0].into()).align([0.5, 0.5].into()),
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .with(RigidBody2d::new(
                RigidBodyDesc::new().translation(Vector::y() * 150.0),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(512.0, 32.0)),
            ))))
            .with(Collider2dBody::Me)
            .with(Physics2dSyncCompositeTransform)
            .build();

        // create first obstacle entity.
        world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::red().a(128),
                    rect: Rect::with_size([20.0, 20.0].into()).align([0.5, 0.5].into()),
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .with(RigidBody2d::new(
                RigidBodyDesc::new().translation(Vector::new(20.0, 150.0 - 32.0 - 10.0)),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(10.0, 10.0)),
            ))))
            .with(Collider2dBody::Me)
            .with(Physics2dSyncCompositeTransform)
            .build();

        // create second obstacle entity.
        world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::red().a(128),
                    rect: Rect::with_size([20.0, 20.0].into()).align([0.5, 0.5].into()),
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .with(RigidBody2d::new(
                RigidBodyDesc::new().translation(Vector::new(-100.0, 150.0 - 32.0 - 10.0)),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(10.0, 10.0)),
            ))))
            .with(Collider2dBody::Me)
            .with(Physics2dSyncCompositeTransform)
            .build();

        // create player entity.
        world
            .create_entity()
            .with(CompositeRenderable(
                Image::new("logo.png").align([0.5, 0.5].into()).into(),
            ))
            .with(CompositeTransform::scale([0.5, 0.5].into()))
            .with(KeyboardMovementTag)
            .with(Speed(50.0))
            .with(RigidBody2d::new(
                RigidBodyDesc::new()
                    .translation(Vector::y() * -100.0)
                    .linear_damping(0.05)
                    .angular_damping(0.5),
            ))
            .with(Collider2d::new(
                ColliderDesc::new(ShapeHandle::new(Ball::new(64.0))).density(1.0),
            ))
            .with(Collider2dBody::Me)
            .with(Physics2dSyncCompositeTransform)
            .build();

        // create hint text.
        world
            .create_entity()
            .with(CompositeRenderable(
                Text {
                    color: Color::white(),
                    font: "Verdana".into(),
                    align: TextAlign::Center,
                    text: "Use WSAD to move".into(),
                    position: 0.0.into(),
                    size: 24.0,
                }
                .into(),
            ))
            .with(CompositeTransform::translation([0.0, -100.0].into()))
            .with(CompositeRenderDepth(1.0))
            .build();
    }
}
