use crate::utils::spinbot::*;
use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(MessageData, Debug, Clone, Serialize, Deserialize)]
pub enum UiBattleSignal {
    Mounted,
    Init(UiBattleState),
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct UiBattleState(pub Vec<UiBattlePlayer>);

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct UiBattlePlayer {
    pub name: String,
    pub health: DataBinding<Scalar>,
    pub power: DataBinding<SpinBotPower>,
}

fn use_battle(context: &mut WidgetContext) {
    if !context.state.has::<UiBattleState>() {
        context.life_cycle.mount(|context| {
            context.signals.write(UiBattleSignal::Mounted);
        });

        context.life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(UiBattleSignal::Init(state)) = msg.as_any().downcast_ref() {
                    let _ = context.state.write_with(state.to_owned());
                }
            }
        });
    }
}

#[pre_hooks(use_battle)]
pub fn battle(mut context: WidgetContext) -> WidgetNode {
    let mut state = context.state.read::<UiBattleState>();

    make_widget!(content_box)
        .listed_slot(
            make_widget!(horizontal_box).with_props(ContentBoxItemLayout {
                anchors: Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: 0.0,
                },
                margin: Rect {
                    left: 10.0,
                    right: 10.0,
                    top: 10.0,
                    bottom: -110.0,
                },
                ..Default::default()
            }),
        )
        .listed_slot(
            make_widget!(horizontal_box).with_props(ContentBoxItemLayout {
                anchors: Rect {
                    top: 1.0,
                    bottom: 1.0,
                    left: 0.0,
                    right: 1.0,
                },
                margin: Rect {
                    left: 10.0,
                    right: 10.0,
                    top: -110.0,
                    bottom: 10.0,
                },
                ..Default::default()
            }),
        )
        .into()
}
