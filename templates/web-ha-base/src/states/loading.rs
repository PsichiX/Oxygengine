use crate::states::game::GameState;
use oxygengine::prelude::*;

#[derive(Default)]
pub struct LoadingState {
    preloader: Option<AssetPackPreloader>,
}

impl State for LoadingState {
    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut assets = universe.expect_resource_mut::<AssetsDatabase>();
        if let Some(preloader) = &mut self.preloader {
            if preloader.process(&mut assets).unwrap() {
                return StateChange::Swap(Box::new(GameState::default()));
            }
        } else {
            self.preloader = Some(
                AssetPackPreloader::new("assets.pack", &mut assets, vec!["set://assets.txt"])
                    .expect("could not create asset pack preloader"),
            );
        }
        StateChange::None
    }
}
