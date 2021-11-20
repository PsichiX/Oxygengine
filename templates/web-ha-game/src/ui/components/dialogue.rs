use oxygengine::user_interface::raui::core::prelude::*;

pub fn dialogue(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;
    let message = props.read_cloned_or_default::<String>();

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
                            a: 0.75,
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .with_props(ContentBoxItemLayout {
                    margin: Rect {
                        left: 1.0,
                        right: 1.0,
                        top: 1.0,
                        bottom: 1.0,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(make_widget!(image_box).with_props(ImageBoxProps {
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: "atlases/ui.yaml@frame".to_owned(),
                scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                    source: 5.0.into(),
                    destination: 5.0.into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .listed_slot(
            make_widget!(text_box)
                .key("message")
                .with_props(TextBoxProps {
                    text: message,
                    font: TextBoxFont {
                        name: "fonts/pixel.yaml".to_owned(),
                        size: 12.0,
                    },
                    color: Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    vertical_align: TextBoxVerticalAlign::Middle,
                    ..Default::default()
                })
                .with_props(ContentBoxItemLayout {
                    margin: Rect {
                        left: 6.0,
                        right: 6.0,
                        top: 6.0,
                        bottom: 6.0,
                    },
                    ..Default::default()
                }),
        )
        .into()
}
