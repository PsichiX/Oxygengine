use crate::ui::components::container::*;
use oxygengine::user_interface::raui::{
    core::{implement_message_data, implement_props_data, prelude::*},
    material::prelude::*,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const DEFAULT_DURATION: Scalar = 2.0;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NotificationShow {
    pub text: String,
    pub side: bool,
    pub height: Option<Scalar>,
    pub duration: Option<Scalar>,
}

#[derive(Debug, Clone)]
pub enum NotificationSignal {
    None,
    Register,
    Unregister,
    Show(NotificationShow),
}
implement_message_data!(NotificationSignal);

impl Default for NotificationSignal {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NotificationsState(pub VecDeque<NotificationShow>);
implement_props_data!(NotificationsState);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsProps {
    #[serde(default)]
    pub side_margin: Scalar,
    #[serde(default)]
    pub side_external_margin: Scalar,
    #[serde(default)]
    pub side_internal_margin: Scalar,
    #[serde(default)]
    pub side_default_height: Scalar,
    #[serde(default)]
    pub external_margin: Scalar,
    #[serde(default)]
    pub internal_margin: Scalar,
    #[serde(default)]
    pub default_height: Scalar,
}
implement_props_data!(NotificationsProps);

impl Default for NotificationsProps {
    fn default() -> Self {
        Self {
            side_margin: 128.0,
            side_external_margin: 16.0,
            side_internal_margin: 4.0,
            side_default_height: 26.0,
            external_margin: 64.0,
            internal_margin: 16.0,
            default_height: 48.0,
        }
    }
}

fn make_animation(duration: Scalar) -> Animation {
    Animation::Sequence(vec![
        Animation::Value(AnimatedValue {
            name: "fade-in".to_owned(),
            duration: 0.25,
        }),
        Animation::Value(AnimatedValue {
            name: "delay".to_owned(),
            duration,
        }),
        Animation::Value(AnimatedValue {
            name: "fade-out".to_owned(),
            duration: 0.25,
        }),
        Animation::Message("next".to_owned()),
    ])
}

fn use_notifications(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        drop(context.state.write(NotificationsState::default()));
        context.signals.write(NotificationSignal::Register);
    });

    context.life_cycle.unmount(|context| {
        context.signals.write(NotificationSignal::Unregister);
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(NotificationSignal::Show(data)) = msg.as_any().downcast_ref() {
                let mut state = context.state.read_cloned_or_default::<NotificationsState>();
                state.0.push_back(data.clone());
                if !context.animator.has("") {
                    let duration = state
                        .0
                        .front()
                        .unwrap()
                        .duration
                        .unwrap_or(DEFAULT_DURATION);
                    drop(context.animator.change("", Some(make_animation(duration))));
                }
                drop(context.state.write(state));
            } else if let Some(AnimationMessage(msg)) = msg.as_any().downcast_ref() {
                if msg == "next" {
                    let mut state = context.state.read_cloned_or_default::<NotificationsState>();
                    state.0.pop_front();
                    if !state.0.is_empty() {
                        let duration = state
                            .0
                            .front()
                            .unwrap()
                            .duration
                            .unwrap_or(DEFAULT_DURATION);
                        drop(context.animator.change("", Some(make_animation(duration))));
                    }
                    drop(context.state.write(state));
                }
            }
        }
    });
}

#[pre_hooks(use_notifications)]
pub fn notifications(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        state,
        animator,
        ..
    } = context;

    let NotificationsProps {
        side_margin,
        side_external_margin,
        side_internal_margin,
        side_default_height,
        external_margin,
        internal_margin,
        default_height,
    } = props.read_cloned_or_default();

    let phase = {
        let a = animator.value_progress_or_zero("", "fade-in");
        let b = animator.value_progress_or_zero("", "fade-out");
        a - b
    };

    let item = if let Ok(state) = state.read::<NotificationsState>() {
        if let Some(state) = state.0.front() {
            let height = state.height.unwrap_or_else(|| {
                if state.side {
                    side_default_height
                } else {
                    default_height
                }
            });
            let mut props = Props::new(ContainerProps {
                variant: "dark".to_owned(),
                canvas_color: None,
                internal_margin: if state.side {
                    (0.0, 40.0).into()
                } else {
                    0.0.into()
                },
                ..Default::default()
            });
            if state.side {
                props.write(ContentBoxItemLayout {
                    anchors: Rect {
                        left: lerp(1.0, 0.0, phase),
                        right: 1.0,
                        top: 0.0,
                        bottom: 0.0,
                    },
                    offset: Vec2 {
                        x: 0.0,
                        y: side_external_margin,
                    },
                    align: Vec2 { x: 1.0, y: 0.0 },
                    margin: Rect {
                        left: side_margin,
                        right: 0.0,
                        top: 0.0,
                        bottom: -height,
                    },
                    ..Default::default()
                });
            } else {
                props.write(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.0,
                        right: 1.0,
                        top: 1.0,
                        bottom: 1.0,
                    },
                    offset: Vec2 {
                        x: 0.0,
                        y: -external_margin * phase,
                    },
                    align: Vec2 { x: 0.5, y: 1.0 },
                    margin: Rect {
                        left: external_margin,
                        right: external_margin,
                        top: -height,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            let size_props = SizeBoxProps {
                width: SizeBoxSizeValue::Fill,
                height: SizeBoxSizeValue::Fill,
                margin: if state.side {
                    Rect {
                        left: side_internal_margin,
                        right: side_internal_margin,
                        top: side_internal_margin,
                        bottom: side_internal_margin,
                    }
                } else {
                    Rect {
                        left: internal_margin,
                        right: internal_margin,
                        top: internal_margin,
                        bottom: internal_margin,
                    }
                },
                ..Default::default()
            };
            let text_props = TextPaperProps {
                text: state.text.to_owned(),
                variant: "3".to_owned(),
                use_main_color: true,
                ..Default::default()
            };
            widget! {
                (#{"item"} container: {props} | {WidgetAlpha(phase)} [
                    (#{"wrapper"} size_box: {size_props} {
                        content = (#{"text"} text_paper: {text_props})
                    })
                ])
            }
        } else {
            widget! {()}
        }
    } else {
        widget! {()}
    };

    widget! {
        (#{key} content_box: {props.clone()} [
            {item}
        ])
    }
}
