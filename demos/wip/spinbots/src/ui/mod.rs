pub mod screens;

use oxygengine::user_interface::raui::{core::prelude::*, material::prelude::*};

pub fn setup(app: &mut Application) {
    app.register_component("battle", screens::battle::battle);
}

pub fn new_theme() -> ThemeProps {
    let mut theme = new_all_white_theme();
    theme.content_backgrounds.insert(
        String::new(),
        ThemedImageMaterial::Image(ImageBoxImage {
            id: "images/ui-dark-bg.svg".to_owned(),
            scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                source: Rect {
                    left: 128.0,
                    right: 128.0,
                    top: 128.0,
                    bottom: 128.0,
                },
                destination: Rect {
                    left: 32.0,
                    right: 32.0,
                    top: 32.0,
                    bottom: 32.0,
                },
                frame_only: false,
                frame_keep_aspect_ratio: false,
            }),
            ..Default::default()
        }),
    );
    theme.button_backgrounds.insert(
        String::new(),
        ThemedButtonMaterial {
            default: ThemedImageMaterial::Image(ImageBoxImage {
                id: "images/ui-dark-bg.svg".to_owned(),
                scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                    source: Rect {
                        left: 128.0,
                        right: 128.0,
                        top: 128.0,
                        bottom: 128.0,
                    },
                    destination: Rect {
                        left: 32.0,
                        right: 32.0,
                        top: 32.0,
                        bottom: 32.0,
                    },
                    frame_only: false,
                    frame_keep_aspect_ratio: false,
                }),
                ..Default::default()
            }),
            selected: ThemedImageMaterial::Image(ImageBoxImage {
                id: "images/ui-light-bg.svg".to_owned(),
                scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                    source: Rect {
                        left: 128.0,
                        right: 128.0,
                        top: 128.0,
                        bottom: 128.0,
                    },
                    destination: Rect {
                        left: 32.0,
                        right: 32.0,
                        top: 32.0,
                        bottom: 32.0,
                    },
                    frame_only: false,
                    frame_keep_aspect_ratio: false,
                }),
                ..Default::default()
            }),
            trigger: ThemedImageMaterial::Image(ImageBoxImage {
                id: "images/ui-light-bg.svg".to_owned(),
                scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                    source: Rect {
                        left: 128.0,
                        right: 128.0,
                        top: 128.0,
                        bottom: 128.0,
                    },
                    destination: Rect {
                        left: 32.0,
                        right: 32.0,
                        top: 32.0,
                        bottom: 32.0,
                    },
                    frame_only: false,
                    frame_keep_aspect_ratio: false,
                }),
                ..Default::default()
            }),
        },
    );
    theme.text_variants.insert(
        String::new(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "fonts/roboto.json".to_owned(),
                size: 18.0,
            },
            ..Default::default()
        },
    );
    theme
}
