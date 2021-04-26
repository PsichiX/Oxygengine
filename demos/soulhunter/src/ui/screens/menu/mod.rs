pub mod new_game;

use self::new_game::*;
use crate::{components::animal_kind::AnimalKind, utils::rgba_to_raui_color};
use oxygengine::user_interface::raui::core::prelude::*;

#[pre_hooks(use_nav_container_active)]
pub fn menu(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, .. } = context;

    let background_props = ImageBoxProps {
        material: ImageBoxMaterial::Color(ImageBoxColor {
            color: rgba_to_raui_color(101, 119, 255, 255),
            ..Default::default()
        }),
        ..Default::default()
    };

    let content_props = ContentBoxItemLayout {
        margin: Rect {
            left: 200.0,
            right: 200.0,
            top: 100.0,
            bottom: 100.0,
        },
        ..Default::default()
    };

    let game_logo_props = ImageBoxProps {
        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
            horizontal_alignment: 0.5,
            vertical_alignment: 0.5,
        }),
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "images/game-logo.svg".to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let text_props = TextBoxProps {
        text: "Select your starting soul:".to_owned(),
        height: TextBoxSizeValue::Exact(20.0),
        alignment: TextBoxAlignment::Center,
        font: TextBoxFont {
            name: "fonts/aquatico.json".to_owned(),
            size: 40.0,
        },
        color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        ..Default::default()
    };

    widget! {
        (#{key} content_box [
            (#{"background"} image_box: {background_props})
            (#{"content"} vertical_box: {content_props} [
                (#{"game-logo"} image_box: {game_logo_props})
                (space_box: {SpaceBoxProps::vertical(60.0)})
                (#{"text"} text_box: {text_props})
                (#{"animals"} nav_horizontal_box: {NavJumpLooped} [
                    (#{"ground-water"} new_game_button: {NewGameButtonProps(AnimalKind::GroundWater)})
                    (#{"water-air"} new_game_button: {NewGameButtonProps(AnimalKind::WaterAir)})
                    (#{"air-ground"} new_game_button: {NewGameButtonProps(AnimalKind::AirGround)})
                ])
            ])
        ])
    }
}
