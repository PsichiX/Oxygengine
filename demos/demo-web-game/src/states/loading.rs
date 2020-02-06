use crate::states::splash::SplashState;
use oxygengine::prelude::*;

#[derive(Default)]
pub struct LoadingState {
    preloader: Option<AssetPackPreloader>,
}

impl State for LoadingState {
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
                Text::new("Verdana", "Loading")
                    .color(Color::white())
                    .align(TextAlign::Center)
                    .size(64.0)
                    .into(),
            ))
            .with(CompositeTransform::default())
            .with(NonPersistent(token))
            .build();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &mut world.write_resource::<AssetsDatabase>();
        if let Some(preloader) = &mut self.preloader {
            if preloader.process(assets).unwrap() {
                return StateChange::Swap(Box::new(SplashState));
            }
        } else {
            self.preloader = Some(
                AssetPackPreloader::new("assets.pack", assets, vec!["set://assets.txt"])
                    .expect("could not create asset pack preloader"),
            );
        }
        StateChange::None
    }
}
