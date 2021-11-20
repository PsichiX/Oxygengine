use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct BarProps {
    pub fill: Scalar,
    pub color: Color,
}

pub fn bar(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;
    let BarProps { fill, color } = props.read_cloned_or_default();
    let fill = fill.max(0.0).min(1.0);

    make_widget!(content_box)
        .key(key)
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
        .listed_slot(
            make_widget!(image_box)
                .with_props(ImageBoxProps {
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: "atlases/ui.yaml@dot".to_owned(),
                        tint: color,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.0,
                        right: fill,
                        top: 0.0,
                        bottom: 1.0,
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
        .into()
}
