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
                    count: info.player.health,
                    danger_threshold: 1,
                    reversed: false,
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.0,
                        right: 0.25,
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
                    danger_threshold: 1,
                    reversed: true,
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.75,
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
