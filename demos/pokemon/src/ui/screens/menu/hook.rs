use crate::ui::screens::menu::state::*;
use oxygengine::user_interface::raui::core::{implement_message_data, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSignal {
    None,
    Register,
    Unregister,
    Show,
}
implement_message_data!(MenuSignal);

impl Default for MenuSignal {
    fn default() -> Self {
        Self::None
    }
}

widget_hook! {
    pub use_menu(life_cycle) {
        life_cycle.mount(|context| {
            drop(context.state.write(MenuState::default()));
            context.signals.write(MenuSignal::Register);
        });

        life_cycle.unmount(|context| {
            context.signals.write(MenuSignal::Unregister);
        });

        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(MenuSignal::Show) = msg.as_any().downcast_ref() {
                    if !context.animator.has("") {
                        let mut state = context.state.read_cloned_or_default::<MenuState>();
                        state.opened = !state.opened;
                        drop(context.animator.change("", Some(Animation::Value(AnimatedValue {
                            name: "phase".to_owned(),
                            duration: 0.25,
                        }))));
                        drop(context.state.write(state));
                    }
                }
            }
        });
    }
}
