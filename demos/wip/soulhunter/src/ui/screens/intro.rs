use crate::states::intro::IntroStopSignal;
use oxygengine::{
    animation::{
        phase::Phase,
        spline::{SplinePoint, SplinePointDirection},
    },
    prelude::{filter_box, FilterBoxProps, FilterBoxValues},
    user_interface::raui::core::prelude::*,
};
use serde::{Deserialize, Serialize};

fn make_animation() -> Animation {
    Animation::Sequence(vec![
        Animation::Value(AnimatedValue {
            name: "oxygen-logo".to_owned(),
            duration: 2.0,
        }),
        Animation::Value(AnimatedValue {
            name: "game-logo".to_owned(),
            duration: 2.0,
        }),
        Animation::Message("completed".to_owned()),
    ])
}

fn make_filter_props(phase: Scalar) -> Props {
    Props::new(ContentBoxItemLayout {
        margin: Rect {
            left: 128.0,
            right: 128.0,
            top: 128.0,
            bottom: 128.0,
        },
        ..Default::default()
    })
    .with(FilterBoxProps::Combine(FilterBoxValues {
        blur: lerp(20.0, 0.0, phase),
        ..Default::default()
    }))
}

fn make_image_props(id: &str, scale: Scalar) -> Props {
    ImageBoxProps {
        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
            horizontal_alignment: 0.5,
            vertical_alignment: 0.5,
            outside: false,
        }),
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: id.to_owned(),
            ..Default::default()
        }),
        transform: Transform {
            pivot: Vec2 { x: 0.5, y: 0.5 },
            scale: Vec2 { x: scale, y: scale },
            ..Default::default()
        },
        ..Default::default()
    }
    .into()
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
struct IntroState {
    #[serde(default = "IntroState::default_alpha_phase")]
    pub alpha_phase: Phase,
    #[serde(default = "IntroState::default_scale_phase")]
    pub scale_phase: Phase,
}

impl Default for IntroState {
    fn default() -> Self {
        Self {
            alpha_phase: Self::default_alpha_phase(),
            scale_phase: Self::default_scale_phase(),
        }
    }
}

impl IntroState {
    fn default_alpha_phase() -> Phase {
        Phase::new(vec![
            SplinePoint::point((0.0, 0.0)),
            SplinePoint::new((0.4, 1.0), SplinePointDirection::Single((0.4, 0.0))),
            SplinePoint::new((0.8, 1.0), SplinePointDirection::Single((0.2, 0.0))),
            SplinePoint::point((1.0, 0.0)),
        ])
        .expect("Could not build default alpha phase animation")
    }

    fn default_scale_phase() -> Phase {
        Phase::new(vec![
            SplinePoint::new((0.0, 0.5), SplinePointDirection::Single((0.0, 0.5))),
            SplinePoint::new((0.5, 1.0), SplinePointDirection::Single((0.5, 0.0))),
            SplinePoint::new((0.8, 1.0), SplinePointDirection::Single((0.2, 0.0))),
            SplinePoint::new((1.0, 1.5), SplinePointDirection::Single((0.0, 0.5))),
        ])
        .expect("Could not build default scale phase animation")
    }
}

fn use_intro(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        drop(context.state.write(IntroState::default()));
        drop(context.animator.change("", Some(make_animation())));
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(AnimationMessage(msg)) = msg.as_any().downcast_ref() {
                if msg == "completed" {
                    context.signals.write(IntroStopSignal);
                }
            }
        }
    });
}

#[pre_hooks(use_intro)]
pub fn intro(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        state,
        animator,
        ..
    } = context;

    let IntroState {
        alpha_phase,
        scale_phase,
    } = state.read_cloned_or_default();
    let (oxygen_scale, oxygen_alpha) = {
        let value = animator.value_progress_factor_or_zero("", "oxygen-logo");
        let scale = scale_phase.sample(value);
        let alpha = alpha_phase.sample(value);
        (scale, alpha)
    };
    let (game_scale, game_alpha) = {
        let value = animator.value_progress_factor_or_zero("", "game-logo");
        let scale = scale_phase.sample(value);
        let alpha = alpha_phase.sample(value);
        (scale, alpha)
    };

    let oxygen_filter = make_filter_props(oxygen_alpha);
    let game_filter = make_filter_props(game_alpha);
    let oxygen_logo = make_image_props("images/oxygen-logo.svg", oxygen_scale);
    let game_logo = make_image_props("images/game-logo.svg", game_scale);

    widget! {
        (#{key} content_box [
            (#{"oxygen-logo"} filter_box: {oxygen_filter} {
                content = (image_box: {oxygen_logo} | {WidgetAlpha(oxygen_alpha)})
            })
            (#{"game-logo"} filter_box: {game_filter} {
                content = (image_box: {game_logo} | {WidgetAlpha(game_alpha)})
            })
        ])
    }
}
