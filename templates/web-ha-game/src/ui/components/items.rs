use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ItemsProps {
    pub capacity: usize,
    pub count: usize,
}

pub fn heart_items(context: WidgetContext) -> WidgetNode {
    items(context, "atlases/ui.yaml@heart")
}

pub fn sword_items(context: WidgetContext) -> WidgetNode {
    items(context, "atlases/ui.yaml@sword")
}

fn items(context: WidgetContext, image: &str) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;
    let ItemsProps {
        mut capacity,
        mut count,
    } = props.read_cloned_or_default();
    capacity = capacity.max(1);
    count = count.min(capacity);

    make_widget!(size_box)
        .with_props(SizeBoxProps {
            width: SizeBoxSizeValue::Content,
            height: SizeBoxSizeValue::Content,
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(content_box)
                .listed_slot(
                    make_widget!(image_box)
                        .with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Image(ImageBoxImage {
                                id: "atlases/ui.yaml@dot".to_owned(),
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
                .listed_slot(make_widget!(image_box).with_props(ImageBoxProps {
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: "atlases/ui.yaml@bar-rect".to_owned(),
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
                }))
                .listed_slot(
                    make_widget!(horizontal_box)
                        .key(key)
                        .listed_slots((0..capacity).map(|index| {
                            let tint = if index < count {
                                Color::default()
                            } else {
                                Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }
                            };
                            make_widget!(image_box).with_props(ImageBoxProps {
                                width: ImageBoxSizeValue::Exact(8.0),
                                height: ImageBoxSizeValue::Exact(8.0),
                                material: ImageBoxMaterial::Image(ImageBoxImage {
                                    id: image.to_owned(),
                                    tint,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            })
                        }))
                        .with_props(ContentBoxItemLayout {
                            margin: Rect {
                                left: 2.0,
                                right: 1.0,
                                top: 3.0,
                                bottom: 2.0,
                            },
                            ..Default::default()
                        }),
                ),
        )
        .into()
}
