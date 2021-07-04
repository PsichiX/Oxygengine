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
                // // NOTE: web browsers require user input to be triggered before playing any audio.
                // let input = universe.expect_resource::<InputController>();
                // if input.trigger_or_default("mouse-left") == TriggerState::Pressed {
                //     return StateChange::Swap(Box::new(GameState::default()));
                // }
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
