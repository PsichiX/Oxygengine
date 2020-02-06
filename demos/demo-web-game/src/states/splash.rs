use crate::states::game::GameState;
use oxygengine::prelude::*;

#[derive(Default)]
pub struct SplashState;

impl State for SplashState {
    fn on_enter(&mut self, world: &mut World) {
        let token = world.read_resource::<AppLifeCycle>().current_state_token();

        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect))
            .with(CompositeTransform::scale(720.0.into()))
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(
                Image::new("splash.png").align(0.5.into()).into(),
            ))
            .with(CompositeTransform::default())
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(
                Text::new("Verdana", "Click to play!")
                    .color(Color::white())
                    .align(TextAlign::Center)
                    .baseline(TextBaseLine::Bottom)
                    .size(64.0)
                    .into(),
            ))
            .with(CompositeTransform::translation([0.5, -32.0].into()))
            .with(CompositeCameraAlignment([0.5, 1.0].into()))
            .with(NonPersistent(token))
            .build();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let input = world.read_resource::<InputController>();
        // NOTE: web browsers require user input to be triggered before playing any audio.
        if input.trigger_or_default("mouse-left") == TriggerState::Pressed {
            return StateChange::Swap(Box::new(GameState::default()));
        }
        StateChange::None
    }
}
