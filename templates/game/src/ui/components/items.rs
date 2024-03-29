use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ItemsProps {
    pub count: usize,
    pub capacity: usize,
    pub danger_threshold: usize,
    pub reversed: bool,
}

pub fn heart_items(context: WidgetContext) -> WidgetNode {
    items(context, "atlases/ui/atlas.json@heart")
}

pub fn sword_items(context: WidgetContext) -> WidgetNode {
    items(context, "atlases/ui/atlas.json@sword")
}

fn items(context: WidgetContext, image: &str) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;
    let ItemsProps {
        count,
        capacity,
        reversed,
        danger_threshold,
    } = props.read_cloned_or_default();

    make_widget!(size_box)
        .key(key)
        .with_props(SizeBoxProps {
            width: SizeBoxSizeValue::Fill,
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(content_box)
                .key("overlay")
                .listed_slot(
                    make_widget!(image_box)
                        .key("background")
                        .with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Image(ImageBoxImage {
                                id: "atlases/ui/atlas.json@dot".to_owned(),
                                tint: Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.5,
                                },
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .with_props(ContentBoxItemLayout {
                            margin: Rect {
                                left: -1.0,
                                right: -1.0,
                                top: -1.0,
                                bottom: -1.0,
                            },
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(image_box)
                        .key("frame")
                        .with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Image(ImageBoxImage {
                                id: "atlases/ui/atlas.json@bar-rect".to_owned(),
                                scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                                    source: Rect {
                                        left: 1.0,
                                        right: 1.0,
                                        top: 2.0,
                                        bottom: 2.0,
                                    },
                                    destination: Rect {
                                        left: 1.0,
                                        right: 1.0,
                                        top: 2.0,
                                        bottom: 2.0,
                                    },
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(horizontal_box)
                        .key("list")
                        .with_props(HorizontalBoxProps {
                            separation: 4.0,
                            reversed,
                            ..Default::default()
                        })
                        .with_props(ContentBoxItemLayout {
                            margin: Rect {
                                left: 2.0,
                                right: 2.0,
                                top: 3.0,
                                bottom: 3.0,
                            },
                            ..Default::default()
                        })
                        .listed_slot(
                            make_widget!(image_box)
                                .key("icon")
                                .with_props(ImageBoxProps {
                                    width: ImageBoxSizeValue::Exact(9.0),
                                    height: ImageBoxSizeValue::Exact(9.0),
                                    material: ImageBoxMaterial::Image(ImageBoxImage {
                                        id: image.to_owned(),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                })
                                .with_props(FlexBoxItemLayout {
                                    fill: 0.0,
                                    grow: 0.0,
                                    shrink: 0.0,
                                    ..Default::default()
                                }),
                        )
                        .listed_slot(make_widget!(text_box).key("value").with_props(
                            TextBoxProps {
                                text: format!("{}/{}", count, capacity),
                                font: TextBoxFont {
                                    name: "fonts/pixel/font.json".to_owned(),
                                    size: 8.0,
                                },
                                horizontal_align: if reversed {
                                    TextBoxHorizontalAlign::Right
                                } else {
                                    TextBoxHorizontalAlign::Left
                                },
                                vertical_align: TextBoxVerticalAlign::Bottom,
                                color: if count > danger_threshold {
                                    Color::default()
                                } else {
                                    Color {
                                        r: 1.0,
                                        g: 0.1,
                                        b: 0.1,
                                        a: 1.0,
                                    }
                                },
                                ..Default::default()
                            },
                        )),
                ),
        )
        .into()
}
