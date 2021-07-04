use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

const SIZE: Scalar = 128.0;

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct MainMenuButtonProps {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
}

#[pre_hooks(use_button_notified_state)]
pub fn main_menu_button(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        ..
    } = context;

    let button_props = props
        .clone()
        .with(NavItemActive)
        .with(ButtonNotifyProps(id.to_owned().into()));
    let main_menu_button_props = props.read_cloned_or_default::<MainMenuButtonProps>();
    let ButtonProps { selected, .. } = state.read_cloned_or_default();

    let background_props = Props::new(ImageBoxProps {
        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
            horizontal_alignment: 0.5,
            vertical_alignment: 0.5,
            outside: false,
        }),
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: if selected {
                "ui/ui-icon-bg-black.svg".to_owned()
            } else {
                "ui/ui-icon-bg-white.svg".to_owned()
            },
            ..Default::default()
        }),
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        margin: Rect {
            bottom: 26.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let icon_props = Props::new(ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: format!("ui/ui-icon-{}.svg", &main_menu_button_props.id),
            ..Default::default()
        }),
        width: ImageBoxSizeValue::Exact(SIZE),
        height: ImageBoxSizeValue::Exact(SIZE),
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        margin: Rect {
            bottom: 26.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let text_props = Props::new(TextBoxProps {
        text: main_menu_button_props.label,
        height: TextBoxSizeValue::Exact(16.0),
        horizontal_align: TextBoxHorizontalAlign::Center,
        font: TextBoxFont {
            name: "fonts/thraex.json".to_owned(),
            size: 16.0,
        },
        color: if selected {
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        },
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 1.0,
            top: 1.0,
            bottom: 1.0,
        },
        margin: Rect {
            top: -16.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let arrow = if selected {
        let arrow_props = Props::new(ImageBoxProps {
            width: ImageBoxSizeValue::Exact(SIZE * 0.5),
            height: ImageBoxSizeValue::Exact(SIZE * 0.5),
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: "ui/ui-icon-arrow-right.svg".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .with(ContentBoxItemLayout {
            anchors: Rect {
                left: 0.5,
                right: 0.5,
                top: 0.0,
                bottom: 0.0,
            },
            offset: Vec2 {
                x: SIZE * -0.5,
                y: SIZE * 0.5,
            },
            align: Vec2 { x: 0.5, y: 0.5 },
            ..Default::default()
        });

        widget! {
            (#{"arrow"} image_box: {arrow_props})
        }
    } else {
        widget! {()}
    };

    widget! {
        (#{key} button: {button_props} {
            content = (#{"content"} content_box [
                (#{"background"} image_box: {background_props})
                (#{"image"} image_box: {icon_props})
                (#{"label"} text_box: {text_props})
                {arrow}
            ])
        })
    }
}
