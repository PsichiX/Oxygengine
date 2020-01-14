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
                let input = &world.read_resource::<InputController>();
                // NOTE: web browsers require user input to be triggered before playing any audio.
                if input.trigger_or_default("mouse-left") == TriggerState::Pressed {
                    return StateChange::Swap(Box::new(WebScriptBootState::new("main")));
                }
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
