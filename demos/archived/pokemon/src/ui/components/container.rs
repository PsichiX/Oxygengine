use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InternalMargin(pub Scalar, pub Scalar);

impl From<Scalar> for InternalMargin {
    fn from(v: Scalar) -> Self {
        Self(v, v)
    }
}

impl From<(Scalar, Scalar)> for InternalMargin {
    fn from((x, y): (Scalar, Scalar)) -> Self {
        Self(x, y)
    }
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct ContainerProps {
    #[serde(default = "ContainerProps::default_variant")]
    pub variant: String,
    #[serde(default)]
    pub internal_margin: InternalMargin,
    #[serde(default = "ContainerProps::default_canvas_color")]
    pub canvas_color: Option<Color>,
    #[serde(default)]
    pub fixed_left: bool,
    #[serde(default)]
    pub fixed_right: bool,
}

impl ContainerProps {
    fn default_variant() -> String {
        "red".to_owned()
    }

    #[allow(clippy::unnecessary_wraps)]
    fn default_canvas_color() -> Option<Color> {
        Some(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
    }
}

impl Default for ContainerProps {
    fn default() -> Self {
        Self {
            variant: Self::default_variant(),
            internal_margin: Default::default(),
            canvas_color: Self::default_canvas_color(),
            fixed_left: false,
            fixed_right: false,
        }
    }
}

pub fn container(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    let container_props = props.read_cloned_or_default::<ContainerProps>();

    let background_material = ImageBoxImage {
        id: format!("ui/ui-bg-{}.svg", container_props.variant.as_str()),
        source_rect: Some(Rect {
            left: if container_props.fixed_left {
                384.0
            } else {
                0.0
            },
            right: if container_props.fixed_right {
                768.0 - 384.0
            } else {
                768.0
            },
            top: 0.0,
            bottom: 512.0,
        }),
        scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
            source: Rect {
                left: if container_props.fixed_left {
                    0.0
                } else {
                    384.0
                },
                right: if container_props.fixed_right {
                    0.0
                } else {
                    384.0
                },
                top: 0.0,
                bottom: 0.0,
            },
            destination: Rect::default(),
            frame_only: false,
            frame_keep_aspect_ratio: true,
        }),
        ..Default::default()
    };

    let background_props = Props::new(ContentBoxItemLayout {
        depth: Scalar::NEG_INFINITY,
        margin: Rect {
            left: -container_props.internal_margin.0,
            right: -container_props.internal_margin.1,
            top: 0.0,
            bottom: 0.0,
        },
        ..Default::default()
    })
    .with(ImageBoxProps {
        material: ImageBoxMaterial::Image(background_material),
        ..Default::default()
    });

    let background = widget! {
        (#{"background"} image_box: {background_props})
    };

    let items = if let Some(color) = container_props.canvas_color {
        let canvas_props = Props::new(ContentBoxItemLayout {
            depth: Scalar::NEG_INFINITY,
            ..Default::default()
        })
        .with(ImageBoxProps {
            material: ImageBoxMaterial::Color(ImageBoxColor {
                color,
                ..Default::default()
            }),
            ..Default::default()
        });

        let canvas = widget! {
            (#{"canvas"} image_box: {canvas_props})
        };

        std::iter::once(canvas)
            .chain(std::iter::once(background))
            .chain(listed_slots.into_iter())
            .collect::<Vec<_>>()
    } else {
        std::iter::once(background)
            .chain(listed_slots.into_iter())
            .collect::<Vec<_>>()
    };

    widget! {
        (#{key} content_box: {props.clone()} |[items]|)
    }
}
