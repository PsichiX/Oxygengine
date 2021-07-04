use crate::{states::game::GameState, ui::screens::menu::new_game::NewGameSignal};
use oxygengine::prelude::*;

#[derive(Debug)]
pub struct MenuState;

impl State for MenuState {
    fn on_enter(&mut self, universe: &mut Universe) {
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("menu-scene", universe)
            .unwrap();
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let ui = universe.expect_resource::<UserInterface>();
        for (_, (_, msg)) in ui.all_signals_received() {
            if let Some(NewGameSignal(animal)) = msg.as_any().downcast_ref() {
                return StateChange::Swap(Box::new(GameState::new(
                    "yaml://levels/0.yaml".to_owned(),
                    *animal,
                )));
            }
        }

        StateChange::None
    }
}
