use crate::{VisualNovelOverlayMaterial, VisualNovelOverlayPhase, VisualNovelStoryUsed};
use oxygengine_core::ecs::ResRead;
use oxygengine_user_interface::raui::core::prelude::*;
use oxygengine_visual_novel::resource::VnStoryManager;
#[cfg(not(feature = "scalar64"))]
use std::f32::consts::PI;
#[cfg(feature = "scalar64")]
use std::f64::consts::PI;

#[pre_hooks(use_nav_container_active)]
pub fn visual_novel_overlay_container(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        shared_props,
        process_context,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if content.is_none() {
        content = make_widget!(visual_novel_overlay).into();
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
    let phase = story.active_scene().phase();
    if phase > 0.0 && phase < 1.0 {
        if let Some(p) = content.props_mut() {
            p.write(phase);
        }

        widget! {
            (#{key} button: {NavItemActive} {
                content = {content}
            })
        }
    } else {
        Default::default()
    }
}

pub fn visual_novel_overlay(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        ..
    } = context;

    let phase = props.read_cloned_or_default::<Scalar>();
    let alpha = match shared_props.read() {
        Ok(VisualNovelOverlayPhase(p)) => p.sample(phase),
        _ => (phase * PI).sin(),
    };
    let image_props = Props::new(ImageBoxProps {
        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
            horizontal_alignment: 0.5,
            vertical_alignment: 0.5,
            outside: true,
        }),
        material: props
            .read_cloned_or_default::<VisualNovelOverlayMaterial>()
            .0,
        ..Default::default()
    });

    widget! { (#{key} image_box: {image_props} | {WidgetAlpha(alpha)}) }
}
