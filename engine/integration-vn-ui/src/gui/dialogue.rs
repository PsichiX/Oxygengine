use crate::{
    VisualNovelAction, VisualNovelDialogueCharacterLayout, VisualNovelDialogueCharacterThemed,
    VisualNovelDialogueMessageLayout, VisualNovelDialogueMessageProps,
    VisualNovelDialogueMessageThemed, VisualNovelDialogueOptionProps,
    VisualNovelDialogueOptionThemed, VisualNovelDialogueOptionsLayout,
    VisualNovelDialogueOptionsList, VisualNovelDialogueOptionsProps, VisualNovelDialogueTextLayout,
    VisualNovelSignal, VisualNovelStoryUsed, VisualNovelTextTransition,
};
use oxygengine_core::ecs::ResRead;
use oxygengine_user_interface::raui::{core::prelude::*, material::prelude::*};
use oxygengine_visual_novel::{dialogue::ActiveDialogue, resource::VnStoryManager};

#[pre_hooks(use_nav_container_active)]
pub fn visual_novel_dialogue_container(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        shared_props,
        process_context,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {message, options});

    if message.is_none() {
        message = make_widget!(visual_novel_dialogue_message).into();
    }
    if options.is_none() {
        options = make_widget!(visual_novel_dialogue_options).into();
    }

    let name = shared_props
        .read_cloned_or_default::<VisualNovelStoryUsed>()
        .0;
    let story = match process_context.owned_ref::<ResRead<VnStoryManager>>() {
        Some(story) => match story.get(&name) {
            Some(story) => story,
            None => return Default::default(),
        },
        None => return Default::default(),
    };

    if let Some(p) = message.props_mut() {
        let message_props = match make_dialogue_message_props(
            story.active_dialogue(),
            &shared_props.read_cloned_or_default(),
        ) {
            Some(p) => p,
            None => return Default::default(),
        };
        p.write(message_props);
        p.write(match shared_props.read_cloned_or_default() {
            VisualNovelDialogueMessageLayout(Some(layout)) => layout,
            _ => ContentBoxItemLayout {
                anchors: Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.7,
                    bottom: 1.0,
                },
                margin: Rect {
                    left: 16.0,
                    right: 16.0,
                    top: 0.0,
                    bottom: 16.0,
                },
                align: Vec2 { x: 0.0, y: 1.0 },
                ..Default::default()
            },
        });
    }
    if let Some(p) = options.props_mut() {
        let options_props = match make_dialogue_options_props(
            story.active_dialogue(),
            &shared_props.read_cloned_or_default(),
        ) {
            Some(p) => p,
            None => return Default::default(),
        };
        p.write(options_props);
        p.write(match shared_props.read_cloned_or_default() {
            VisualNovelDialogueOptionsLayout(Some(layout)) => layout,
            _ => ContentBoxItemLayout {
                anchors: Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: 0.7,
                },
                margin: Rect {
                    left: 64.0,
                    right: 64.0,
                    top: 32.0,
                    bottom: 32.0,
                },
                align: Vec2 { x: 0.0, y: 1.0 },
                ..Default::default()
            },
        });
    }

    widget! {
        (#{key} content_box [
            {message}
            {options}
        ])
    }
}

pub fn visual_novel_dialogue_message(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        ..
    } = context;

    if let Ok(message) = props.read::<VisualNovelDialogueMessageProps>() {
        let message_themed =
            shared_props.read_cloned_or_default::<VisualNovelDialogueMessageThemed>();
        let character_themed =
            shared_props.read_cloned_or_default::<VisualNovelDialogueCharacterThemed>();

        let message_text_props = Props::new(TextPaperProps {
            text: message.text.to_owned(),
            variant: message_themed
                .text_variant
                .clone()
                .unwrap_or_else(|| "dialogue-message".to_owned()),
            use_main_color: message_themed.use_main_color,
            ..Default::default()
        })
        .with(ThemedWidgetProps {
            color: message_themed.color,
            ..Default::default()
        });

        let message_container_props = Props::new(WrapBoxProps {
            margin: message_themed.margin,
            fill: true,
        })
        .with(PaperProps {
            variant: message_themed
                .background_variant
                .unwrap_or_else(|| "dialogue-message".to_owned()),
            ..Default::default()
        })
        .with(match shared_props.read_cloned_or_default() {
            VisualNovelDialogueTextLayout(Some(layout)) => layout,
            _ => ContentBoxItemLayout {
                margin: Rect {
                    left: 0.0,
                    right: 0.0,
                    top: 64.0,
                    bottom: 0.0,
                },
                ..Default::default()
            },
        });

        let character_text_props = Props::new(TextPaperProps {
            text: message.character.to_owned(),
            variant: character_themed
                .text_variant
                .clone()
                .unwrap_or_else(|| "dialogue-character".to_owned()),
            use_main_color: character_themed.use_main_color,
            ..Default::default()
        })
        .with(ThemedWidgetProps {
            color: character_themed.color,
            ..Default::default()
        });

        let character_container_props = Props::new(WrapBoxProps {
            margin: character_themed.margin,
            fill: true,
        })
        .with(PaperProps {
            variant: character_themed
                .background_variant
                .unwrap_or_else(|| "dialogue-character".to_owned()),
            ..Default::default()
        })
        .with(match shared_props.read_cloned_or_default() {
            VisualNovelDialogueCharacterLayout(Some(layout)) => layout,
            _ => ContentBoxItemLayout {
                anchors: Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: 0.0,
                },
                margin: Rect {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: -56.0,
                },
                align: Vec2 { x: 0.0, y: 1.0 },
                ..Default::default()
            },
        });

        widget! {
            (#{key} content_box [
                (#{"text-wrap"} wrap_paper: {message_container_props} | {WidgetAlpha(message.container_alpha)} {
                    content = (#{"text"} text_paper: {message_text_props} | {WidgetAlpha(message.message_alpha)})
                })
                (#{"character-wrap"} wrap_paper: {character_container_props} | {WidgetAlpha(message.container_alpha)} {
                    content = (#{"name"} text_paper: {character_text_props} | {WidgetAlpha(message.message_alpha)})
                })
            ])
        }
    } else {
        Default::default()
    }
}

pub fn visual_novel_dialogue_options(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        named_slots,
        ..
    } = context;

    unpack_named_slots!(named_slots => item);

    if item.is_none() {
        item = make_widget!(visual_novel_dialogue_option).into();
    }

    if let Ok(options) = props.read::<VisualNovelDialogueOptionsProps>() {
        if options.alpha > 0.0 && !options.options.is_empty() {
            let items = options
                .options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    let mut item = item.clone();
                    if let Some(p) = item.props_mut() {
                        p.write(index);
                        p.write(FlexBoxItemLayout {
                            fill: 1.0,
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        });
                        p.write(VisualNovelDialogueOptionProps {
                            index,
                            text: option.text.to_owned(),
                            alpha: options.alpha,
                            focused: option.focused.value(),
                            focus_phase: option.focused.phase(),
                        });
                    }
                    item
                })
                .collect::<Vec<_>>();
            let VisualNovelDialogueOptionsList(list_props) = shared_props.read_cloned_or_default();
            let list_props = Props::new(list_props)
                .with(NavJumpActive(NavJumpMode::StepVertical))
                .with(NavJumpLooped)
                .with(NavItemActive);
            return widget! { (#{key} nav_vertical_box: {list_props} |[ items ]|) };
        }
    }
    Default::default()
}

pub fn use_visual_novel_dialogue_option(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        let story = context
            .shared_props
            .read_cloned_or_default::<VisualNovelStoryUsed>()
            .0;
        let index = context.props.read_cloned_or_default::<usize>();

        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.select_start() {
                    context.signals.write(VisualNovelSignal {
                        story: story.to_owned(),
                        action: VisualNovelAction::FocusDialogueOption(Some(index)),
                    });
                }
                if msg.select_stop() {
                    context.signals.write(VisualNovelSignal {
                        story: story.to_owned(),
                        action: VisualNovelAction::FocusDialogueOption(None),
                    });
                }
                if msg.trigger_start() {
                    context.signals.write(VisualNovelSignal {
                        story: story.to_owned(),
                        action: VisualNovelAction::SelectDialogueOption(Some(index)),
                    });
                }
            }
        }
    });
}

#[pre_hooks(use_visual_novel_dialogue_option)]
pub fn visual_novel_dialogue_option(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        shared_props,
        ..
    } = context;

    if let Ok(option) = props.read::<VisualNovelDialogueOptionProps>() {
        let themed = shared_props.read_cloned_or_default::<VisualNovelDialogueOptionThemed>();

        let size_props = SizeBoxProps {
            width: SizeBoxSizeValue::Fill,
            height: themed.height,
            ..Default::default()
        };

        let props = Props::new(TextPaperProps {
            text: option.text.to_owned(),
            use_main_color: themed.use_main_color,
            variant: if option.focused {
                themed
                    .text_variant_focused
                    .clone()
                    .unwrap_or_else(|| "dialogue-option-focused".to_owned())
            } else {
                themed
                    .text_variant_default
                    .clone()
                    .unwrap_or_else(|| "dialogue-option".to_owned())
            },
            ..Default::default()
        })
        .with(WrapBoxProps {
            margin: themed.margin,
            fill: true,
        })
        .with(ThemedWidgetProps {
            color: if option.focused {
                themed.color_focused
            } else {
                themed.color_default
            },
            ..Default::default()
        })
        .with(if option.focused {
            ButtonPaperOverrideStyle::Selected
        } else {
            ButtonPaperOverrideStyle::Default
        })
        .with(PaperProps {
            variant: themed
                .button_variant
                .unwrap_or_else(|| "dialogue-option".to_owned()),
            ..Default::default()
        })
        .with(ButtonNotifyProps(id.to_owned().into()))
        .with(NavItemActive);

        widget! {
            (#{key} size_box: {size_props} {
                content = (#{"button"} text_button_paper: {props} | {WidgetAlpha(option.alpha)})
            })
        }
    } else {
        Default::default()
    }
}

pub fn make_dialogue_options_props(
    dialogue: &ActiveDialogue,
    text_transition: &VisualNovelTextTransition,
) -> Option<VisualNovelDialogueOptionsProps> {
    let phase = dialogue.phase();
    let (options, alpha) = match (dialogue.from().as_ref(), dialogue.to().as_ref()) {
        (None, None) => return None,
        (None, Some(to)) => match text_transition {
            VisualNovelTextTransition::Instant | VisualNovelTextTransition::Unfold => {
                (to.options.to_owned(), 1.0)
            }
            VisualNovelTextTransition::Fade => (to.options.to_owned(), phase),
        },
        (Some(from), Some(to)) => match text_transition {
            VisualNovelTextTransition::Instant | VisualNovelTextTransition::Unfold => {
                if phase < 0.5 {
                    (from.options.to_owned(), 1.0)
                } else {
                    (to.options.to_owned(), 1.0)
                }
            }
            VisualNovelTextTransition::Fade => {
                if phase < 0.5 {
                    (from.options.to_owned(), (1.0 - phase * 2.0).max(0.0))
                } else {
                    (to.options.to_owned(), (phase - 0.5) * 2.0)
                }
            }
        },
        (Some(from), None) => match text_transition {
            VisualNovelTextTransition::Instant | VisualNovelTextTransition::Unfold => {
                (from.options.to_owned(), 1.0)
            }
            VisualNovelTextTransition::Fade => (from.options.to_owned(), 1.0 - phase),
        },
    };
    Some(VisualNovelDialogueOptionsProps { options, alpha })
}

pub fn make_dialogue_message_props(
    dialogue: &ActiveDialogue,
    text_transition: &VisualNovelTextTransition,
) -> Option<VisualNovelDialogueMessageProps> {
    let phase = dialogue.phase();
    let (c, t, ma, ca) = match (dialogue.from().as_ref(), dialogue.to().as_ref()) {
        (None, None) => return None,
        (None, Some(to)) => match text_transition {
            VisualNovelTextTransition::Instant => {
                (to.character.to_owned(), to.text.to_owned(), 1.0, 1.0)
            }
            VisualNovelTextTransition::Fade => {
                (to.character.to_owned(), to.text.to_owned(), phase, phase)
            }
            VisualNovelTextTransition::Unfold => (
                to.character.to_owned(),
                text_unfold(&to.text, phase).to_owned(),
                1.0,
                1.0,
            ),
        },
        (Some(from), Some(to)) => match text_transition {
            VisualNovelTextTransition::Instant => {
                if phase < 0.5 {
                    (from.character.to_owned(), from.text.to_owned(), 1.0, 1.0)
                } else {
                    (to.character.to_owned(), to.text.to_owned(), 1.0, 1.0)
                }
            }
            VisualNovelTextTransition::Fade => {
                if phase < 0.5 {
                    (
                        from.character.to_owned(),
                        from.text.to_owned(),
                        (1.0 - phase * 2.0).max(0.0),
                        1.0,
                    )
                } else {
                    (
                        to.character.to_owned(),
                        to.text.to_owned(),
                        (phase - 0.5) * 2.0,
                        1.0,
                    )
                }
            }
            VisualNovelTextTransition::Unfold => {
                if phase < 0.5 {
                    (
                        from.character.to_owned(),
                        text_unfold(&from.text, 1.0 - phase * 2.0).to_owned(),
                        1.0,
                        1.0,
                    )
                } else {
                    (
                        to.character.to_owned(),
                        text_unfold(&to.text, (phase - 0.5) * 2.0).to_owned(),
                        1.0,
                        1.0,
                    )
                }
            }
        },
        (Some(from), None) => match text_transition {
            VisualNovelTextTransition::Instant => {
                (from.character.to_owned(), from.text.to_owned(), 1.0, 1.0)
            }
            VisualNovelTextTransition::Fade => (
                from.character.to_owned(),
                from.text.to_owned(),
                1.0 - phase,
                1.0 - phase,
            ),
            VisualNovelTextTransition::Unfold => (
                from.character.to_owned(),
                text_unfold(&from.text, 1.0 - phase).to_owned(),
                1.0,
                1.0,
            ),
        },
    };
    Some(VisualNovelDialogueMessageProps {
        character: c,
        text: t,
        container_alpha: ca,
        message_alpha: ma,
    })
}

pub fn text_unfold(text: &str, factor: Scalar) -> &str {
    let n = ((text.len() as Scalar * factor.min(1.0).max(0.0)) as usize).min(text.len());
    &text[..n]
}
