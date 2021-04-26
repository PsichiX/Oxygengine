use crate::{states::game::GameState, ui::screens::menu::new_game::NewGameSignal};
use oxygengine::prelude::*;

#[derive(Debug)]
pub struct MenuState;

impl State for MenuState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<PrefabManager>()
            .instantiate_world("menu-scene", world)
            .unwrap();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let Some(ui) = world
            .write_resource::<UserInterfaceRes>()
            .application_mut("menu")
        {
            for (_, msg) in ui.consume_signals() {
                if let Some(NewGameSignal(animal)) = msg.as_any().downcast_ref() {
                    return StateChange::Swap(Box::new(GameState::new(
                        "yaml://levels/0.yaml".to_owned(),
                        *animal,
                    )));
                }
            }
        }

        StateChange::None
    }
}
