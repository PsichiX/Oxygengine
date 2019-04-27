use crate::{
    component::{
        CompositeCamera, CompositeRenderDepth, CompositeRenderable, CompositeRenderableStroke,
        CompositeTransform,
    },
    composite_renderer::{Command, CompositeRenderer, Rectangle, Renderable, Stats},
    math::Mat2d,
    resource::CompositeTransformRes,
};
use core::{
    app::AppLifeCycle,
    assets::database::AssetsDatabase,
    ecs::{Entities, Entity, Join, Read, ReadExpect, ReadStorage, System, Write},
    hierarchy::{HierarchyRes, Parent, Tag},
};
use std::marker::PhantomData;

pub struct CompositeTransformSystem;

impl<'s> System<'s> for CompositeTransformSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, CompositeTransform>,
        ReadExpect<'s, HierarchyRes>,
        Write<'s, CompositeTransformRes>,
    );

    fn run(
        &mut self,
        (entities, parents, transforms, hierarchy, mut transform_res): Self::SystemData,
    ) {
        let hierarchy = &hierarchy;
        let mut transform_res = &mut transform_res;
        transform_res.clear();
        for (entity, transform, _) in (&entities, &transforms, !&parents).join() {
            transform_res.add(entity, transform.matrix());
            for child in hierarchy.children(entity) {
                add_matrix(
                    *child,
                    &transforms,
                    transform.matrix(),
                    hierarchy,
                    &mut transform_res,
                );
            }
        }
    }
}

fn add_matrix<'s>(
    child: Entity,
    transforms: &ReadStorage<'s, CompositeTransform>,
    root_matrix: Mat2d,
    hierarchy: &HierarchyRes,
    result: &mut CompositeTransformRes,
) {
    if let Some(transform) = transforms.get(child) {
        let mat = root_matrix * transform.matrix();
        result.add(child, mat);
        for child in hierarchy.children(child) {
            add_matrix(*child, transforms, mat, hierarchy, result);
        }
    }
}

pub struct CompositeRendererSystem<CR>
where
    CR: CompositeRenderer,
{
    _phantom: PhantomData<CR>,
}

impl<CR> Default for CompositeRendererSystem<CR>
where
    CR: CompositeRenderer,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'s, CR> System<'s> for CompositeRendererSystem<CR>
where
    CR: CompositeRenderer + Send + Sync + 'static,
{
    type SystemData = (
        Option<Write<'s, CR>>,
        Entities<'s>,
        ReadExpect<'s, AppLifeCycle>,
        Option<Read<'s, AssetsDatabase>>,
        Read<'s, CompositeTransformRes>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, CompositeRenderable>,
        ReadStorage<'s, CompositeTransform>,
        ReadStorage<'s, CompositeRenderDepth>,
        ReadStorage<'s, CompositeRenderableStroke>,
        ReadStorage<'s, Tag>,
    );

    fn run(
        &mut self,
        (
            renderer,
            entities,
            lifecycle,
            assets,
            transform_res,
            cameras,
            renderables,
            transforms,
            depths,
            strokes,
            tags,
        ): Self::SystemData,
    ) {
        if renderer.is_none() {
            return;
        }

        let renderer: &mut CR = &mut renderer.unwrap();
        if let Some(assets) = &assets {
            renderer.update_cache(assets);
        }
        renderer.update_state();
        let (w, h) = {
            let r = renderer.view_size();
            (r.x, r.y)
        };
        let mut stats = Stats::default();

        if let Some(color) = renderer.state().clear_color {
            let result = renderer.execute(vec![Command::Draw(Renderable::Rectangle(Rectangle {
                color,
                rect: [0.0, 0.0, w, h].into(),
            }))]);
            if let Ok((render_ops, renderables)) = result {
                stats.render_ops += render_ops;
                stats.renderables += renderables;
            }
        }

        let mut sorted_cameras = (&entities, &cameras, &transforms)
            .join()
            .map(|(entity, camera, transform)| {
                let depth = if let Some(depth) = depths.get(entity) {
                    depth.0
                } else {
                    0.0
                };
                (depth, camera, transform)
            })
            .collect::<Vec<_>>();
        sorted_cameras.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (_, camera, camera_transform) in sorted_cameras {
            let mut sorted = (&entities, &renderables, transform_res.read())
                .join()
                .filter(|(entity, _, _)| {
                    camera.tags.is_empty()
                        || tags
                            .get(*entity)
                            .map_or(false, |tag| camera.tags.contains(&tag.0))
                })
                .map(|(entity, renderable, transform)| {
                    let depth = if let Some(depth) = depths.get(entity) {
                        depth.0
                    } else {
                        0.0
                    };
                    (depth, renderable, transform, entity)
                })
                .collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let camera_matrix = camera.view_matrix(&camera_transform, [w, h].into());
            let commands = std::iter::once(Command::Store)
                .chain(std::iter::once({
                    let [a, b, c, d, e, f] = camera_matrix.0;
                    Command::Transform(a, b, c, d, e, f)
                }))
                .chain(
                    sorted
                        .iter()
                        .flat_map(|(_, renderable, transform, entity)| {
                            let [a, b, c, d, e, f] = transform.0;
                            vec![
                                Command::Store,
                                Command::Transform(a, b, c, d, e, f),
                                if let Some(stroke) = strokes.get(*entity) {
                                    Command::Stroke(stroke.0, renderable.0.clone())
                                } else {
                                    Command::Draw(renderable.0.clone())
                                },
                                Command::Restore,
                            ]
                        }),
                )
                .chain(std::iter::once(Command::Restore));

            if let Ok((render_ops, renderables)) = renderer.execute(commands) {
                stats.render_ops += render_ops;
                stats.renderables += renderables;
            }
        }
        stats.delta_time = lifecycle.delta_time_seconds();
        stats.fps = 1.0 / stats.delta_time;
        renderer.state_mut().set_stats(stats);
    }
}
