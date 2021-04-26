pub mod hook;
pub mod state;

use crate::ui::{
    components::{container::*, main_menu_button::*, tip::*},
    screens::menu::{hook::use_menu, state::*},
};
use oxygengine::user_interface::raui::core::prelude::*;

#[pre_hooks(use_nav_container_active, use_menu)]
pub fn menu(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        state,
        animator,
        ..
    } = context;

    let state = state.read_cloned_or_default::<MenuState>();
    if !state.opened && animator.is_done() {
        return widget! {()};
    }

    let d = if state.opened { 1.0 } else { 0.0 };
    let phase = animator.value_progress_or("", "phase", d);
    let phase = if state.opened { phase } else { 1.0 - phase };

    let background_props = ImageBoxProps {
        material: ImageBoxMaterial::Color(ImageBoxColor {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.65 * phase,
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let container_props = Props::new(ContentBoxProps {
        transform: Transform {
            align: Vec2 {
                x: phase - 1.0,
                y: 0.0,
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .with(ContainerProps {
        internal_margin: 64.0.into(),
        canvas_color: None,
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        margin: Rect {
            bottom: 20.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let buttons_props = Props::new(FlexBoxProps {
        separation: 32.0,
        wrap: true,
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.5,
            right: 0.5,
            top: 0.45,
            bottom: 0.45,
        },
        margin: Rect {
            left: -200.0,
            right: -200.0,
            top: -128.0,
            bottom: -128.0,
        },
        align: Vec2 { x: 0.5, y: 0.5 },
        ..Default::default()
    })
    .with(NavJumpActive(NavJumpMode::StepHorizontal))
    .with(NavJumpLooped)
    .with(NavItemActive);

    let bar_props = Props::new(ContentBoxProps {
        transform: Transform {
            align: Vec2 {
                x: 0.0,
                y: 1.0 - phase,
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .with(ContainerProps {
        variant: "black".to_owned(),
        canvas_color: Some(Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        }),
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
            top: -20.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let tips_props = Props::new(HorizontalBoxProps {
        separation: 16.0,
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        margin: Rect {
            left: 32.0,
            right: 32.0,
            top: 4.0,
            bottom: 0.0,
        },
        ..Default::default()
    });

    let buttons_list = [
        ("pokedex", "POKEDEX"),
        ("pokemon", "POKEMON"),
        ("bag", "BAG"),
        ("save", "SAVE"),
        ("map", "TOWN MAP"),
        ("options", "OPTIONS"),
    ]
    .iter()
    .map(|(name, label)| {
        widget! {
            (#{name} main_menu_button: {MainMenuButtonProps {
                id: name.to_string(),
                label: label.to_string(),
            }})
        }
    })
    .collect::<Vec<_>>();

    let tips_list = vec![
        widget! { (#{"confirm"} confirm_tip) },
        widget! { (#{"save"} save_tip) },
        widget! { (#{"quit"} quit_tip) },
    ];

    widget! {
        (#{key} content_box [
            (#{"background"} image_box: {background_props})
            (#{"content"} container: {container_props} | {WidgetAlpha(phase)} [
                (#{"buttons"} nav_flex_box: {buttons_props} |[ buttons_list ]|)
            ])
            (#{"bar"} container: {bar_props} [
                (#{"tips"} horizontal_box: {tips_props} |[ tips_list ]|)
            ])
        ])
    }
}
