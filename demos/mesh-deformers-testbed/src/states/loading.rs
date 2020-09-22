use crate::states::game::GameState;
use oxygengine::prelude::*;

#[derive(Default)]
pub struct LoadingState {
    preloader: Option<AssetPackPreloader>,
}

impl State for LoadingState {
    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &mut world.write_resource::<AssetsDatabase>();
        if let Some(preloader) = &mut self.preloader {
            if preloader.process(assets).unwrap() {
                return StateChange::Swap(Box::new(GameState));
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
