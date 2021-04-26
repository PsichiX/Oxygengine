use crate::utils::rgba_to_raui_color;
use oxygengine::user_interface::raui::core::{implement_props_data, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CollectibleProps {
    pub image: String,
    pub value: usize,
}
implement_props_data!(CollectibleProps);

pub fn collectible(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let CollectibleProps { image, value } = props.read_cloned_or_default();

    let size_props = SizeBoxProps {
        width: SizeBoxSizeValue::Fill,
        height: SizeBoxSizeValue::Exact(64.0),
        ..Default::default()
    };

    let icon_props = ImageBoxProps {
        width: ImageBoxSizeValue::Exact(64.0),
        height: ImageBoxSizeValue::Exact(64.0),
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: image,
            ..Default::default()
        }),
        ..Default::default()
    };

    let value_props = TextBoxProps {
        text: value.to_string(),
        alignment: TextBoxAlignment::Right,
        font: TextBoxFont {
            name: "fonts/aquatico.json".to_owned(),
            size: 48.0,
        },
        color: rgba_to_raui_color(0, 0, 0, 255),
        transform: Transform {
            translation: Vec2 { x: 0.0, y: 12.0 },
            ..Default::default()
        },
        ..Default::default()
    };

    widget! {
        (#{key} size_box: {size_props} {
            content = (#{"list"} horizontal_box [
                (#{"icon"} image_box: {icon_props})
                (#{"value"} text_box: {value_props})
            ])
        })
    }
}
