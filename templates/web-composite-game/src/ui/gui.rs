use crate::ui::new_theme;
use oxygengine::user_interface::raui::{core::prelude::*, material::prelude::*};

pub fn gui(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, .. } = context;

    let shared_props = new_theme();

    let info_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 1.0,
            top: 0.85,
            bottom: 1.0,
        },
        margin: Rect {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 6.0,
        },
        ..Default::default()
    })
    .with(PaperProps {
        frame: None,
        ..Default::default()
    });

    let text_props = Props::new(ContentBoxItemLayout {
        margin: Rect {
            left: 32.0,
            right: 32.0,
            top: 6.0,
            bottom: 6.0,
        },
        ..Default::default()
    })
    .with(TextPaperProps {
        text: "Hello, World!".to_owned(),
        width: TextBoxSizeValue::Fill,
        height: TextBoxSizeValue::Fill,
        use_main_color: true,
        ..Default::default()
    });

    widget! {
        (#{key} content_box | {shared_props} [
            (#{"info"} paper: {info_props} [
                (#{"text"} text_paper: {text_props})
            ])
        ])
    }
}
