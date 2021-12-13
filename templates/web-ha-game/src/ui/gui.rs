use crate::{resources::game_state_info::GameStateInfo, ui::components::items::*};
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
                    capacity: info.player.health_capacity,
                    count: info.player.health,
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.0,
                        right: 0.5,
                        top: 0.0,
                        bottom: 1.0,
                    },
                    margin: Rect {
                        left: 6.0,
                        right: 6.0,
                        top: 6.0,
                        bottom: 6.0,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(sword_items)
                .key("weapons")
                .with_props(ItemsProps {
                    capacity: info.player.weapons_capacity,
                    count: info.player.weapons,
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.5,
                        right: 1.0,
                        top: 0.0,
                        bottom: 1.0,
                    },
                    margin: Rect {
                        left: 6.0,
                        right: 6.0,
                        top: 6.0,
                        bottom: 6.0,
                    },
                    align: Vec2 { x: 1.0, y: 0.0 },
                    ..Default::default()
                }),
        )
        // .listed_slot(
        //     make_widget!(dialogue)
        //         .key("dialogue")
        //         .with_props("Hello World!\nWelcome to Oxygengine ;)".to_owned())
        //         .with_props(ContentBoxItemLayout {
        //             anchors: Rect {
        //                 left: 0.0,
        //                 right: 1.0,
        //                 top: 1.0,
        //                 bottom: 1.0,
        //             },
        //             margin: Rect {
        //                 left: 12.0,
        //                 right: 12.0,
        //                 top: -42.0,
        //                 bottom: 6.0,
        //             },
        //             ..Default::default()
        //         }),
        // )
        .into()
}
