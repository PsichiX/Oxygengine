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
            .build();

        // create player entity.
        let player = world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::red(),
                    rect: [-50.0, -50.0, 100.0, 100.0].into(),
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .with(KeyboardMovementTag)
            .with(Speed(100.0))
            .build();

        // create eye attached to player.
        world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::yellow(),
                    rect: [-10.0, -10.0, 20.0, 20.0].into(),
                }
                .into(),
            ))
            .with(CompositeTransform::translation((-20.0).into()))
            .with(Parent(player))
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
            .with(CompositeTransform::translation([0.0, 100.0].into()))
            .with(CompositeRenderDepth(-1.0))
            .build();
    }
}
