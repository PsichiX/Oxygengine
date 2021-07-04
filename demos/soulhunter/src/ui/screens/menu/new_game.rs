use crate::components::animal_kind::AnimalKind;
use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(MessageData, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewGameSignal(pub AnimalKind);

fn make_animation() -> Animation {
    Animation::Looped(Box::new(Animation::Value(AnimatedValue {
        name: "phase".to_owned(),
        duration: 1.0,
    })))
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct NewGameButtonProps(pub AnimalKind);

fn use_new_game_button(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.select_start() {
                    drop(context.animator.change("", Some(make_animation())));
                }
                if msg.select_stop() {
                    drop(context.animator.change("", None));
                }
                if msg.trigger_start() {
                    if let Ok(animal) = context.props.read::<NewGameButtonProps>() {
                        context.signals.write(NewGameSignal(animal.0));
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_new_game_button)]
pub fn new_game_button(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        animator,
        ..
    } = context;

    match props.read::<NewGameButtonProps>() {
        Ok(animal) => {
            let value = animator.value_progress_factor_or_zero("", "phase");
            let phase = (value * std::f32::consts::PI * 2.0).sin();
            let scale = 1.0 + 0.1 * phase;
            let image = animal.0.image();

            let button_props =
                Props::new(ButtonNotifyProps(id.to_owned().into())).with(NavItemActive);

            let image_props = ImageBoxProps {
                content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                    horizontal_alignment: 0.5,
                    vertical_alignment: 0.5,
                    outside: false,
                }),
                material: ImageBoxMaterial::Image(ImageBoxImage {
                    id: image.to_owned(),
                    ..Default::default()
                }),
                transform: Transform {
                    pivot: Vec2 { x: 0.5, y: 0.5 },
                    scale: Vec2 { x: scale, y: scale },
                    ..Default::default()
                },
                ..Default::default()
            };

            widget! {
                (#{key} button: {button_props} {
                    content = (#{"image"} image_box: {image_props})
                })
            }
        }
        _ => widget! {()},
    }
}
