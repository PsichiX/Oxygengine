use oxygengine::{
    composite_renderer::{component::*, composite_renderer::*, math::*},
    prelude::*,
};
use std::f32::consts::PI;

pub struct HierarchyState;

impl State for HierarchyState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect))
            .with(CompositeTransform::scale(800.0.into()))
            .build();

        let root = world
            .create_entity()
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::red(),
                rect: [-50.0, -50.0, 100.0, 100.0].into(),
            })))
            .with(CompositeTransform::translation([0.0, 0.0].into()))
            .build();

        let first = world
            .create_entity()
            .with(Parent(root))
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::green().a(192),
                rect: [-50.0, -50.0, 100.0, 100.0].into(),
            })))
            .with(CompositeTransform::translation([50.0, 50.0].into()))
            .build();

        let second = world
            .create_entity()
            .with(Parent(first))
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::blue().a(192),
                rect: [-50.0, -50.0, 100.0, 100.0].into(),
            })))
            .with(
                CompositeTransform::translation([50.0, 50.0].into())
                    .with_rotation(PI * 0.25)
                    .with_scale(1.25.into()),
            )
            .build();

        world
            .create_entity()
            .with(Parent(second))
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::yellow().a(192),
                rect: [-50.0, -50.0, 100.0, 100.0].into(),
            })))
            .with(CompositeTransform::translation([50.0, 50.0].into()).with_rotation(PI * 0.25))
            .build();

        world
            .create_entity()
            .with(Parent(root))
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::blue(),
                rect: [-50.0, -50.0, 100.0, 100.0].into(),
            })))
            .with(CompositeTransform::translation([-150.0, 0.0].into()))
            .build();

        world
            .create_entity()
            .with(Parent(root))
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::magenta(),
                rect: [-50.0, -50.0, 100.0, 100.0].into(),
            })))
            .with(CompositeTransform::translation([0.0, -150.0].into()))
            .build();
    }
}
