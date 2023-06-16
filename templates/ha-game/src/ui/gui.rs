use crate::{
    resources::game_state_info::GameStateInfo,
    ui::components::{dialogue::*, items::*},
};
use oxygengine::{core::ecs::ResRead, user_interface::raui::core::prelude::*};

pub fn gui(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        process_context,
        ..
    } = context;

    let info = match process_context.owned_ref::<ResRead<GameStateInfo>>() {
        Some(info) => info,
        None => return Default::default(),
    };

    make_widget!(content_box)
        .key(key)
        .listed_slot(
            make_widget!(heart_items)
                .key("health")
                .with_props(ItemsProps {
                    count: info.player.health,
                    capacity: info.player.health_capacity,
                    danger_threshold: 1,
                    reversed: false,
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.0,
                        right: 0.4,
                        top: 0.0,
                        bottom: 1.0,
                    },
                    margin: Rect {
                        left: 2.0,
                        right: 2.0,
                        top: 2.0,
                        bottom: 2.0,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(sword_items)
                .key("weapons")
                .with_props(ItemsProps {
                    count: info.player.weapons,
                    capacity: info.player.weapons_capacity,
                    danger_threshold: 1,
                    reversed: true,
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.6,
                        right: 1.0,
                        top: 0.0,
                        bottom: 1.0,
                    },
                    margin: Rect {
                        left: 2.0,
                        right: 2.0,
                        top: 2.0,
                        bottom: 2.0,
                    },
                    align: Vec2 { x: 1.0, y: 0.0 },
                    ..Default::default()
                }),
        )
        .listed_slot(
            info.area
                .as_ref()
                .map(|name| {
                    make_widget!(dialogue)
                        .key("dialogue")
                        .with_props(name.to_owned())
                        .with_props(ContentBoxItemLayout {
                            anchors: Rect {
                                left: 0.0,
                                right: 1.0,
                                top: 1.0,
                                bottom: 1.0,
                            },
                            margin: Rect {
                                left: 32.0,
                                right: 32.0,
                                top: -32.0,
                                bottom: 6.0,
                            },
                            ..Default::default()
                        })
                        .into()
                })
                .unwrap_or(WidgetNode::None),
        )
        .into()
}
