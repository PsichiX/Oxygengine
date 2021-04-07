pub mod components;
pub mod gui;
pub mod screens;

use oxygengine::user_interface::raui::core::prelude::*;
use oxygengine::user_interface::raui::material::prelude::*;
use std::collections::HashMap;

pub fn setup(app: &mut Application) {
    app.register_props::<components::container::ContainerProps>("ContainerProps");
    app.register_props::<components::main_menu_button::MainMenuButtonProps>("MainMenuButtonProps");
    app.register_props::<components::tip::TipProps>("TipProps");
    app.register_props::<screens::menu::state::MenuState>("MenuState");
    app.register_props::<screens::notifications::NotificationsState>("NotificationsState");
    app.register_props::<screens::notifications::NotificationsProps>("NotificationsProps");
    app.register_component("container", components::container::container);
    app.register_component(
        "main_menu_button",
        components::main_menu_button::main_menu_button,
    );
    app.register_component("tip", components::tip::tip);
    app.register_component("gui", gui::gui);
    app.register_component("hud", screens::hud::hud);
    app.register_component("menu", screens::menu::menu);
    app.register_component("notifications", screens::notifications::notifications);
}

pub fn new_theme() -> ThemeProps {
    let mut theme = new_all_white_theme();
    make_text_variants(
        "",
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "fonts/thraex.json".to_owned(),
                size: 16.0,
            },
            alignment: TextBoxAlignment::Center,
            ..Default::default()
        },
        &mut theme.text_variants,
    );
    make_text_variants(
        "roboto",
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "fonts/roboto.json".to_owned(),
                size: 16.0,
            },
            alignment: TextBoxAlignment::Center,
            ..Default::default()
        },
        &mut theme.text_variants,
    );
    theme
}

fn make_text_variants(
    base_id: &str,
    base_material: ThemedTextMaterial,
    text_variants: &mut HashMap<String, ThemedTextMaterial>,
) {
    {
        let mut material = base_material.clone();
        material.font.size *= 2.0;
        text_variants.insert(format!("{}1", base_id), material);
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 1.5;
        text_variants.insert(format!("{}2", base_id), material);
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 1.17;
        text_variants.insert(format!("{}3", base_id), material);
    }
    {
        text_variants.insert(format!("{}4", base_id), base_material.clone());
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 0.83;
        text_variants.insert(format!("{}5", base_id), material);
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 0.67;
        text_variants.insert(format!("{}6", base_id), material);
    }
    text_variants.insert(base_id.to_owned(), base_material);
}
