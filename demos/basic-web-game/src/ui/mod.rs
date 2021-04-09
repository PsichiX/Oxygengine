pub mod gui;

use oxygengine::user_interface::raui::core::prelude::*;
use oxygengine::user_interface::raui::material::prelude::*;

pub fn setup(app: &mut Application) {
    app.register_component("gui", gui::gui);
}

pub fn new_theme() -> ThemeProps {
    let mut theme = new_all_white_theme();
    theme.content_backgrounds.insert(
        String::new(),
        ThemedImageMaterial::Image(ImageBoxImage {
            id: "ui/ui.svg".to_owned(),
            scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                source: Rect {
                    left: 64.0,
                    right: 64.0,
                    top: 0.0,
                    bottom: 0.0,
                },
                destination: Rect {
                    left: 64.0,
                    right: 64.0,
                    top: 0.0,
                    bottom: 0.0,
                },
                frame_only: false,
                frame_keep_aspect_ratio: true,
            }),
            ..Default::default()
        }),
    );
    theme.text_variants.insert(
        String::new(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "fonts/kato.json".to_owned(),
                size: 18.0,
            },
            ..Default::default()
        },
    );
    theme
}
