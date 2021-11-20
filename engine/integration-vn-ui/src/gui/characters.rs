use crate::{
    VisualNovelCharacterContentAlignment, VisualNovelCharacterProps, VisualNovelStoryUsed,
};
use oxygengine_core::ecs::ResRead;
use oxygengine_user_interface::raui::core::prelude::*;
use oxygengine_visual_novel::{resource::VnStoryManager, Position};

pub fn visual_novel_characters_container(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        shared_props,
        process_context,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => item);

    if item.is_none() {
        item = make_widget!(visual_novel_character).into();
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

    let items = story
        .characters()
        .filter_map(|(_, character)| {
            if character.visibility() > 0.0 {
                let mut item = item.clone();
                if let Some(p) = item.props_mut() {
                    p.write(VisualNovelCharacterProps(character.clone()));
                }
                Some(item)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    widget! { (#{key} content_box|[ items ]|) }
}

pub fn visual_novel_character(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        props,
        shared_props,
        ..
    } = context;

    if let Ok(VisualNovelCharacterProps(character)) = props.read() {
        let (from, phase, to) = character.style();
        let Position(ax, ay) = character.alignment();
        let (cx, cy) = if let VisualNovelCharacterContentAlignment(Some(Position(cx, cy))) =
            shared_props.read_cloned_or_default()
        {
            (cx, cy)
        } else {
            (ax, ay)
        };
        let Position(px, py) = character.position();
        let Position(sx, sy) = character.scale();
        let r = character.rotation();
        let alpha = character.visibility();

        let from = if phase < 1.0 {
            if let Some(image) = character.styles.get(from) {
                let props = Props::new(ImageBoxProps {
                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                        horizontal_alignment: cx,
                        vertical_alignment: cy,
                        outside: false,
                    }),
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: image.to_owned(),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
                widget! { (#{"from"} image_box: {props} | {WidgetAlpha((1.0 - phase) * alpha)}) }
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        let to = if phase > 0.0 {
            if let Some(image) = character.styles.get(to) {
                let props = Props::new(ImageBoxProps {
                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                        horizontal_alignment: cx,
                        vertical_alignment: cy,
                        outside: false,
                    }),
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: image.to_owned(),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
                widget! { (#{"to"} image_box: {props} | {WidgetAlpha(phase * alpha)}) }
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        let props = ContentBoxProps {
            transform: Transform {
                pivot: (ax, ay).into(),
                align: (px, py).into(),
                rotation: r,
                scale: (sx, sy).into(),
                ..Default::default()
            },
            ..Default::default()
        };

        widget! {
            (#{character.name()} content_box: {props} [
                {from}
                {to}
            ])
        }
    } else {
        Default::default()
    }
}
