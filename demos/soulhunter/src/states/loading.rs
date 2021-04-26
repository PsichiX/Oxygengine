use oxygengine::prelude::*;

pub struct LoadingState {
    pack_path: String,
    state_to_swap: Option<Box<dyn State>>,
    preloader: Option<AssetPackPreloader>,
    show_progress: bool,
}

impl LoadingState {
    pub fn new(pack_path: &str, state_to_swap: Box<dyn State>, show_progress: bool) -> Self {
        Self {
            pack_path: pack_path.to_owned(),
            state_to_swap: Some(state_to_swap),
            preloader: None,
            show_progress,
        }
    }
}

impl State for LoadingState {
    fn on_enter(&mut self, world: &mut World) {
        if self.show_progress {
            world
                .write_resource::<PrefabManager>()
                .instantiate_world("loading-scene", world)
                .unwrap();
        }
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &mut world.write_resource::<AssetsDatabase>();
        if let Some(preloader) = &mut self.preloader {
            if preloader.process(assets).unwrap() {
                if let Some(state) = std::mem::take(&mut self.state_to_swap) {
                    return StateChange::Swap(state);
                }
            }
        } else {
            if assets.fetch_engines_stack_size() > 1 {
                assets.pop_fetch_engine();
            }
            self.preloader = Some(
                AssetPackPreloader::new(&self.pack_path, assets, vec!["*set://assets.txt"])
                    .expect("could not create asset pack preloader"),
            );
        }
        StateChange::None
    }
}
