use oxygengine::user_interface::raui::core::prelude::*;

pub fn gui(_context: WidgetContext) -> WidgetNode {
    make_widget!(text_box)
        .with_props(TextBoxProps {
            text: "Hello World!".to_owned(),
            font: TextBoxFont {
                name: "fonts/roboto.yaml".to_owned(),
                size: 64.0,
            },
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Bottom,
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.5,
                a: 1.0,
            },
            ..Default::default()
        })
        .into()
}
