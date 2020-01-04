use crate::components::{speed::Speed, KeyboardMovementTag};
use oxygengine::prelude::*;

pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        // create entity with camera to view scene.
        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect))
            .with(CompositeTransform::scale(2432.0.into()))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(().into()))
            .with(CompositeTransform::translation((-1216.0).into()))
            .with(CompositeMapChunk::new("map.map".into(), "ground".into()))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(().into()))
            .with(CompositeTransform::translation((-1216.0).into()))
            .with(CompositeMapChunk::new("map.map".into(), "roads".into()))
            .build();
    }
}
