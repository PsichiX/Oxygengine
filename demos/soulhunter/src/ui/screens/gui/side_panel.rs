use crate::utils::rgba_to_raui_color;
use oxygengine::user_interface::raui::core::{implement_props_data, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Side {
    Left,
    Right,
}

impl Default for Side {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SidePanelProps {
    pub label: String,
    pub label_height: Scalar,
    pub side: Side,
}
implement_props_data!(SidePanelProps);

pub fn side_panel(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let SidePanelProps {
        label,
        label_height,
        side,
    } = props.read_cloned_or_default();

    let background_props = ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "images/ui-white-black.svg".to_owned(),
            source_rect: Some(Rect {
                left: match side {
                    Side::Left => 34.0,
                    Side::Right => 0.0,
                },
                right: match side {
                    Side::Left => 256.0,
                    Side::Right => 256.0 - 34.0,
                },
                top: 34.0,
                bottom: 34.0,
            }),
            scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                source: Rect {
                    left: 34.0,
                    right: 34.0,
                    top: 0.0,
                    bottom: 0.0,
                },
                destination: Rect {
                    left: 16.0,
                    right: 16.0,
                    top: 0.0,
                    bottom: 0.0,
                },
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let label_size_props = SizeBoxProps {
        width: SizeBoxSizeValue::Fill,
        height: SizeBoxSizeValue::Exact(label_height),
        ..Default::default()
    };

    let label_props = TextBoxProps {
        text: label,
        alignment: match side {
            Side::Left => TextBoxAlignment::Right,
            Side::Right => TextBoxAlignment::Left,
        },
        font: TextBoxFont {
            name: "fonts/aquatico.json".to_owned(),
            size: label_height,
        },
        color: rgba_to_raui_color(96, 96, 96, 255),
        ..Default::default()
    };

    let list_props = Props::new(ContentBoxItemLayout {
        margin: Rect {
            left: 32.0,
            right: 32.0,
            top: 16.0,
            bottom: 16.0,
        },
        ..Default::default()
    })
    .with(VerticalBoxProps {
        separation: 16.0,
        ..Default::default()
    });

    widget! {
        (#{key} content_box: {props.clone()} [
            (#{"background"} image_box: {background_props})
            (#{"list"} vertical_box: {list_props} [
                (#{"label-size"} size_box: {label_size_props} {
                    content = (#{"label"} text_box: {label_props})
                })
                {content}
            ])
        ])
    }
}
