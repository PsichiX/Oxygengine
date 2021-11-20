use crate::ui::components::items::*;
use oxygengine::user_interface::raui::core::prelude::*;

pub fn gui(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, .. } = context;

    make_widget!(content_box)
        .key(key)
        .listed_slot(
            make_widget!(heart_items)
                .key("health")
                .with_props(ItemsProps {
                    capacity: 3,
                    count: 2,
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
                .key("strength")
                .with_props(ItemsProps {
                    capacity: 3,
                    count: 1,
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
