use crate::{
    component::{
        CompositeCamera, CompositeCameraAlignment, CompositeEffect, CompositeRenderAlpha,
        CompositeRenderDepth, CompositeRenderLayer, CompositeRenderable, CompositeRenderableStroke,
        CompositeTransform, CompositeVisibility,
    },
    composite_renderer::{Command, CompositeRenderer, Renderable, Stats},
    math::{Mat2d, Vec2},
    resource::CompositeTransformCache,
};
use core::{
    app::AppLifeCycle,
    assets::database::AssetsDatabase,
    ecs::{components::Tag, Comp, Universe, UnsafeRef, UnsafeScope, WorldRef},
};

pub type CompositeRendererSystemResources<'a, CR> = (
    WorldRef,
    &'a mut CR,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a CompositeTransformCache,
    (
        Comp<&'a CompositeCamera>,
        Comp<&'a CompositeVisibility>,
        Comp<&'a CompositeRenderable>,
        Comp<&'a CompositeTransform>,
        Comp<&'a CompositeRenderLayer>,
        Comp<&'a CompositeRenderDepth>,
        Comp<&'a CompositeRenderAlpha>,
        Comp<&'a CompositeCameraAlignment>,
        Comp<&'a CompositeRenderableStroke>,
        Comp<&'a CompositeEffect>,
        Comp<&'a Tag>,
    ),
);

#[allow(clippy::many_single_char_names)]
pub fn composite_renderer_system<CR>(universe: &mut Universe)
where
    CR: CompositeRenderer + 'static,
{
    let (world, mut renderer, lifecycle, assets, cache, ..) =
        universe.query_resources::<CompositeRendererSystemResources<CR>>();

    renderer.update_cache(&assets);
    renderer.update_state();
    let (width, height) = {
        let r = renderer.view_size();
        (r.x, r.y)
    };
    let mut stats = Stats {
        view_size: renderer.view_size(),
        images_count: renderer.images_count(),
        fontfaces_count: renderer.fontfaces_count(),
        surfaces_count: renderer.surfaces_count(),
        ..Default::default()
    };

    if let Some(color) = renderer.state().clear_color {
        let result = renderer.execute(vec![Command::Draw(Renderable::FullscreenRectangle(color))]);
        if let Ok((render_ops, renderables)) = result {
            stats.render_ops += render_ops;
            stats.renderables += renderables;
        }
    }

    let unsafe_scope = UnsafeScope;
    let mut sorted_cameras = world
        .query::<(
            &CompositeCamera,
            &CompositeTransform,
            Option<&CompositeVisibility>,
            Option<&CompositeRenderAlpha>,
            Option<&CompositeRenderLayer>,
            Option<&CompositeRenderDepth>,
        )>()
        .iter()
        .filter_map(
            |(_, (camera, transform, visibility, alpha, layer, depth))| {
                let visible = visibility.map(|v| v.0).unwrap_or(true);
                let alpha = alpha.map(|v| v.0);
                let alpha_visible = alpha.map(|v| v > 0.0).unwrap_or(true);
                if visible && alpha_visible {
                    let layer = layer.map(|v| v.0).unwrap_or(0);
                    let depth = depth.map(|v| v.0).unwrap_or(0.0);
                    unsafe {
                        Some((
                            layer,
                            depth,
                            alpha,
                            UnsafeRef::upgrade(&unsafe_scope, camera),
                            UnsafeRef::upgrade(&unsafe_scope, transform),
                        ))
                    }
                } else {
                    None
                }
            },
        )
        .collect::<Vec<_>>();
    sorted_cameras.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then_with(|| a.1.partial_cmp(&b.1).unwrap())
    });

    for (_, _, camera_alpha, camera, camera_transform) in sorted_cameras {
        let unsafe_scope = UnsafeScope;
        let mut sorted = world
            .query::<(
                &CompositeRenderable,
                Option<&Tag>,
                Option<&CompositeVisibility>,
                Option<&CompositeRenderAlpha>,
                Option<&CompositeRenderLayer>,
                Option<&CompositeRenderDepth>,
                Option<&CompositeCameraAlignment>,
                Option<&CompositeEffect>,
                Option<&CompositeRenderableStroke>,
            )>()
            .with::<&CompositeTransform>()
            .iter()
            .filter(|(_, (_, tag, ..))| unsafe {
                camera.read().tags.is_empty()
                    || tag
                        .map(|tag| camera.read().tags.contains(&tag.0))
                        .unwrap_or(false)
            })
            .filter_map(
                |(
                    entity,
                    (renderable, _, visibility, alpha, layer, depth, alignment, effect, stroke),
                )| {
                    let visible = visibility.map(|v| v.0).unwrap_or(true);
                    let alpha_visible = alpha.map(|v| v.0 > 0.0).unwrap_or(true);
                    if visible && alpha_visible {
                        let layer = layer.map(|v| v.0).unwrap_or(0);
                        let depth = depth.map(|v| v.0).unwrap_or(0.0);
                        unsafe {
                            Some((
                                layer,
                                depth,
                                cache.matrix(entity).unwrap_or_default(),
                                UnsafeRef::upgrade(&unsafe_scope, renderable),
                                alignment
                                    .map(|alignment| UnsafeRef::upgrade(&unsafe_scope, alignment)),
                                effect.map(|effect| UnsafeRef::upgrade(&unsafe_scope, effect)),
                                alpha.map(|alpha| UnsafeRef::upgrade(&unsafe_scope, alpha)),
                                stroke.map(|stroke| UnsafeRef::upgrade(&unsafe_scope, stroke)),
                            ))
                        }
                    } else {
                        None
                    }
                },
            )
            .collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap()
                .then_with(|| a.1.partial_cmp(&b.1).unwrap())
        });

        let camera_matrix = unsafe {
            camera
                .read()
                .view_matrix(camera_transform.read(), [width, height].into())
        };
        let commands = std::iter::once(Command::Store)
            .chain(std::iter::once({
                let [a, b, c, d, e, f] = camera_matrix.0;
                Command::Transform(a, b, c, d, e, f)
            }))
            .chain(std::iter::once(
                camera_alpha.map(Command::Alpha).unwrap_or(Command::None),
            ))
            .chain(sorted.iter().flat_map(
                |(_, _, matrix, renderable, alignment, effect, alpha, stroke)| {
                    let [a, b, c, d, e, f] = matrix.0;
                    vec![
                        Command::Store,
                        alignment
                            .as_ref()
                            .map(|alignment| unsafe {
                                let alignment = alignment.read();
                                let p = Vec2::new(alignment.0.x * width, alignment.0.y * height);
                                let [a, b, c, d, e, f] =
                                    ((!camera_matrix).unwrap() * Mat2d::translation(p)).0;
                                Command::Transform(a, b, c, d, e, f)
                            })
                            .unwrap_or(Command::None),
                        Command::Transform(a, b, c, d, e, f),
                        effect
                            .as_ref()
                            .map(|effect| unsafe { Command::Effect(effect.read().0) })
                            .unwrap_or(Command::None),
                        alpha
                            .as_ref()
                            .map(|alpha| unsafe { Command::Alpha(alpha.read().0) })
                            .unwrap_or(Command::None),
                        stroke
                            .as_ref()
                            .map(|stroke| unsafe {
                                Command::Stroke(stroke.read().0, renderable.read().0.clone())
                            })
                            .unwrap_or_else(|| unsafe {
                                Command::Draw(renderable.read().0.clone())
                            }),
                        Command::Restore,
                    ]
                },
            ))
            .chain(std::iter::once(Command::Restore));

        if let Ok((render_ops, renderables)) = renderer.execute(commands) {
            stats.render_ops += render_ops;
            stats.renderables += renderables;
        }
    }
    stats.delta_time = lifecycle.delta_time_seconds();
    stats.fps = if stats.delta_time > 0.0 {
        1.0 / stats.delta_time
    } else {
        0.0
    };
    renderer.state_mut().set_stats(stats);
}
