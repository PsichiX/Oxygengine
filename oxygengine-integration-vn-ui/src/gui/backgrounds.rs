use crate::{VisualNovelBackgroundsProps, VisualNovelStoryUsed};
use oxygengine_core::ecs::ResRead;
use oxygengine_user_interface::raui::core::prelude::*;
use oxygengine_visual_novel::{resource::VnStoryManager, Position};

pub fn visual_novel_backgrounds_container(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        shared_props,
        process_context,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if content.is_none() {
        content = make_widget!(visual_novel_backgrounds).into();
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
    let scene = if story.active_scene().phase() < 0.5 {
        match story.active_scene().from() {
            Some(name) => story.scene(name),
            None => None,
        }
    } else {
        match story.active_scene().to() {
            Some(name) => story.scene(name),
            None => None,
        }
    };
    let scene = match scene {
        Some(scene) => scene,
        None => return Default::default(),
    };
    let phase = scene.background_style.phase();

    if let Some(p) = content.props_mut() {
        p.write(VisualNovelBackgroundsProps {
            phase,
            from: story.background(scene.background_style.from()).cloned(),
            to: story.background(scene.background_style.to()).cloned(),
        });
    }

    let Position(tx, ty) = scene.camera_position.value();
    let r = scene.camera_rotation.value();
    let container_props = ContentBoxProps {
        transform: Transform {
            pivot: (0.5, 0.5).into(),
            translation: (tx, ty).into(),
            rotation: r,
            ..Default::default()
        },
        ..Default::default()
    };

    widget! {
        (#{key} content_box: {container_props} [
            {content}
        ])
    }
}

pub fn visual_novel_backgrounds(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let VisualNovelBackgroundsProps { phase, from, to } = props.read_cloned_or_default();

    let from = if phase < 1.0 {
        match from {
            Some(from) => {
                let props = ImageBoxProps {
                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                        horizontal_alignment: 0.5,
                        vertical_alignment: 0.5,
                        outside: true,
                    }),
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: from.image.to_owned(),
                        ..Default::default()
                    }),
                    transform: Transform {
                        pivot: (0.5, 0.5).into(),
                        scale: from.scale.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                widget! { (#{"from"} image_box: {props} | {WidgetAlpha(1.0 - phase)}) }
            }
            None => Default::default(),
        }
    } else {
        Default::default()
    };
    let to = if phase > 0.0 {
        match to {
            Some(to) => {
                let props = ImageBoxProps {
                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                        horizontal_alignment: 0.5,
                        vertical_alignment: 0.5,
                        outside: true,
                    }),
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: to.image.to_owned(),
                        ..Default::default()
                    }),
                    transform: Transform {
                        pivot: (0.5, 0.5).into(),
                        scale: to.scale.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                widget! { (#{"to"} image_box: {props} | {WidgetAlpha(phase)}) }
            }
            None => Default::default(),
        }
    } else {
        Default::default()
    };

    widget! {
        (#{key} content_box [
            {from}
            {to}
        ])
    }
}
