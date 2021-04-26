use crate::states::{loading::LoadingState, menu::MenuState};
use oxygengine::{prelude::*, user_interface::raui::core::implement_message_data};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct IntroStopSignal;
implement_message_data!(IntroStopSignal);

#[derive(Debug, Default)]
pub struct IntroState;

impl State for IntroState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<PrefabManager>()
            .instantiate_world("intro-scene", world)
            .unwrap();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let mut go_to_next_scene = false;

        if let Some(ui) = world
            .write_resource::<UserInterfaceRes>()
            .application_mut("intro")
        {
            for (_, msg) in ui.consume_signals() {
                if let Some(IntroStopSignal) = msg.as_any().downcast_ref() {
                    go_to_next_scene = true;
                    break;
                }
            }
        }

        let input = world.read_resource::<InputController>();
        if input.trigger_or_default("accept").is_pressed()
            || input.trigger_or_default("cancel").is_pressed()
            || input.trigger_or_default("pointer-action").is_pressed()
            || input.trigger_or_default("pointer-context").is_pressed()
        {
            go_to_next_scene = true;
        }

        if go_to_next_scene {
            StateChange::Swap(Box::new(LoadingState::new(
                "main.pack",
                Box::new(MenuState),
                true,
            )))
        } else {
            StateChange::None
        }
    }
}
