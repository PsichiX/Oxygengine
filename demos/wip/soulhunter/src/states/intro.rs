use crate::states::{loading::LoadingState, menu::MenuState};
use oxygengine::{prelude::*, user_interface::MessageData};
use serde::{Deserialize, Serialize};

#[derive(MessageData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct IntroStopSignal;

#[derive(Debug, Default)]
pub struct IntroState;

impl State for IntroState {
    fn on_enter(&mut self, universe: &mut Universe) {
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("intro-scene", universe)
            .unwrap();
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut go_to_next_scene = false;

        let ui = universe.expect_resource::<UserInterface>();
        for (_, (_, msg)) in ui.all_signals_received() {
            if let Some(IntroStopSignal) = msg.as_any().downcast_ref() {
                go_to_next_scene = true;
                break;
            }
        }

        let input = universe.expect_resource::<InputController>();
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
