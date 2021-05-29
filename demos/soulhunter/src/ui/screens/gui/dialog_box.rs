use crate::{ui::screens::gui::side_panel::*, utils::rgba_to_raui_color};
use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct DialogBoxProps {
    pub name: Option<String>,
    pub text: String,
}

pub fn dialog_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let DialogBoxProps { name, text } = props.read_cloned_or_default();
    if text.is_empty() {
        return widget! {()};
    }

    let content_props = ContentBoxItemLayout {
        margin: Rect {
            left: 48.0,
            right: 48.0,
            top: 64.0,
            bottom: 0.0,
        },
        ..Default::default()
    };

    let background_props = ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "images/ui-white-black.svg".to_owned(),
            source_rect: Some(Rect {
                left: 0.0,
                right: 256.0,
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

    let text_props = Props::new(ContentBoxItemLayout {
        margin: Rect {
            left: 32.0,
            right: 32.0,
            top: 32.0,
            bottom: 32.0,
        },
        ..Default::default()
    })
    .with(TextBoxProps {
        text,
        font: TextBoxFont {
            name: "fonts/aquatico.json".to_owned(),
            size: 32.0,
        },
        color: rgba_to_raui_color(0, 0, 0, 255),
        ..Default::default()
    });

    let name_box = if let Some(name) = name {
        let name_size_props = Props::new(ContentBoxItemLayout {
            anchors: Rect {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            ..Default::default()
        })
        .with(SizeBoxProps {
            width: SizeBoxSizeValue::Exact(300.0),
            height: SizeBoxSizeValue::Content,
            ..Default::default()
        });

        widget! {
            (#{"name-size"} size_box: {name_size_props} {
                content = (#{"name"} side_panel: {SidePanelProps {
                    label: name,
                    label_height: 24.0,
                    side: Side::Left,
                }})
            })
        }
    } else {
        widget! {()}
    };

    widget! {
        (#{key} content_box [
            (#{"content"} content_box: {content_props} [
                (#{"background"} image_box: {background_props})
                (#{"text"} text_box: {text_props})
            ])
            {name_box}
        ])
    }
}
