use crate::states::main::MainState;
use oxygengine::{core::assets::database::AssetsDatabase, prelude::*};

pub struct LoadingState;

impl State for LoadingState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<AssetsDatabase>()
            .load("set://assets.txt")
            .expect("cannot load `assets.txt`");
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &world.read_resource::<AssetsDatabase>();
        if assets.is_ready() {
            StateChange::Swap(Box::new(MainState::default()))
        } else {
            StateChange::None
        }
    }
}
