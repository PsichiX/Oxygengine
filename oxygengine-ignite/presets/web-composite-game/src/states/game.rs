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

        // create player entity.
        world
            .create_entity()
            .with(CompositeRenderable(
                Image::new("logo.png").align([0.5, 0.5].into()).into(),
            ))
            .with(CompositeTransform::scale([0.5, 0.5].into()))
            .with(KeyboardMovementTag)
            .with(Speed(100.0))
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
            .with(CompositeRenderDepth(1.0))
            .build();
    }
}
