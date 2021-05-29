pub mod collectible;
pub mod dialog_box;
pub mod side_panel;

use self::{collectible::*, dialog_box::*, side_panel::*};
use crate::utils::rgba_to_raui_color;
use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct GuiProps {
    pub steps: usize,
    pub collected_stars: usize,
    pub collected_shields: usize,
}

pub type GuiRemoteProps = DataBinding<GuiProps>;

fn use_gui(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(data) = msg.as_any().downcast_ref::<GuiRemoteProps>() {
                drop(context.state.write_with(data.clone()));
            }
        }
    });
}

#[pre_hooks(use_gui)]
pub fn gui(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, state, .. } = context;

    let gui_props = state
        .read_cloned_or_default::<GuiRemoteProps>()
        .read_cloned_or_default();

    let steps_size_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        },
        align: Vec2 { x: 0.0, y: 0.0 },
        offset: Vec2 { x: 0.0, y: 40.0 },
        ..Default::default()
    })
    .with(SizeBoxProps {
        width: SizeBoxSizeValue::Exact(300.0),
        ..Default::default()
    });

    let steps_props = SidePanelProps {
        label: "Steps".to_owned(),
        label_height: 32.0,
        side: Side::Left,
    };

    let steps_value_props = TextBoxProps {
        text: gui_props.steps.to_string(),
        height: TextBoxSizeValue::Exact(48.0),
        horizontal_align: TextBoxHorizontalAlign::Right,
        font: TextBoxFont {
            name: "fonts/aquatico.json".to_owned(),
            size: 48.0,
        },
        color: rgba_to_raui_color(0, 0, 0, 255),
        ..Default::default()
    };

    let collected_size_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 1.0,
            right: 1.0,
            top: 0.0,
            bottom: 0.0,
        },
        align: Vec2 { x: 1.0, y: 0.0 },
        offset: Vec2 { x: 0.0, y: 40.0 },
        ..Default::default()
    })
    .with(SizeBoxProps {
        width: SizeBoxSizeValue::Exact(300.0),
        ..Default::default()
    });

    let collected_props = SidePanelProps {
        label: "Collected".to_owned(),
        label_height: 32.0,
        side: Side::Right,
    };

    let dialog_size_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 1.0,
            top: 1.0,
            bottom: 1.0,
        },
        align: Vec2 { x: 0.0, y: 1.0 },
        ..Default::default()
    })
    .with(SizeBoxProps {
        width: SizeBoxSizeValue::Fill,
        height: SizeBoxSizeValue::Exact(180.0),
        ..Default::default()
    });

    widget! {
        (#{key} content_box [
            (#{"steps-size"} size_box: {steps_size_props} {
                content = (#{"steps"} side_panel: {steps_props} {
                    content = (#{"value"} text_box: {steps_value_props})
                })
            })
            (#{"collected-size"} size_box: {collected_size_props} {
                content = (#{"collected"} side_panel: {collected_props} {
                    content = (#{"list"} vertical_box [
                        (#{"stars"} collectible: {CollectibleProps {
                            image: "images/item-star.svg".to_owned(),
                            value: gui_props.collected_stars,
                        }})
                        (#{"shields"} collectible: {CollectibleProps {
                            image: "images/item-shield.svg".to_owned(),
                            value: gui_props.collected_shields,
                        }})
                    ])
                })
            })
            (#{"dialog-size"} size_box: {dialog_size_props} {
                content = (#{"dialog"} dialog_box: {DialogBoxProps {
                    name: Some("Narrator".to_owned()),
                    text: "Hello,\nWorld!".to_owned(),
                }})
            })
        ])
    }
}
