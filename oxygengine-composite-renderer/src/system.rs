use crate::{
    component::{
        CompositeCamera, CompositeRenderDepth, CompositeRenderable, CompositeRenderableStroke,
        CompositeScalingMode, CompositeTag, CompositeTransform,
    },
    composite_renderer::{Command, CompositeRenderer, Rectangle, Renderable, Transformation},
    math::Vec2,
};
use core::{
    assets::database::AssetsDatabase,
    ecs::{Entities, Join, Read, ReadStorage, System, Write},
};
use std::marker::PhantomData;

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
        Option<Read<'s, AssetsDatabase>>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, CompositeRenderable>,
        ReadStorage<'s, CompositeTransform>,
        ReadStorage<'s, CompositeRenderDepth>,
        ReadStorage<'s, CompositeRenderableStroke>,
        ReadStorage<'s, CompositeTag>,
    );

    fn run(
        &mut self,
        (
            renderer,
            entities,
            assets,
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
            let r = renderer.viewport();
            (r.x, r.y)
        };
        let wh = w * 0.5;
        let hh = h * 0.5;
        let s = if w > h { h } else { w };

        if let Some(color) = renderer.state().clear_color {
            drop(
                renderer.execute(vec![Command::Draw(Renderable::Rectangle(Rectangle {
                    color,
                    rect: [0.0, 0.0, w, h].into(),
                }))]),
            );
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
            let mut sorted = (&entities, &renderables, &transforms)
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

            let camera_transforms = match camera.scaling {
                CompositeScalingMode::None => vec![
                    Command::Transform(Transformation::Scale(
                        Vec2::one() / camera_transform.get_scale(),
                    )),
                    Command::Transform(Transformation::Rotate(-camera_transform.get_rotation())),
                    Command::Transform(Transformation::Translate(
                        -camera_transform.get_translation(),
                    )),
                ],
                CompositeScalingMode::Center => vec![
                    Command::Transform(Transformation::Translate([wh, hh].into())),
                    Command::Transform(Transformation::Scale(
                        Vec2::one() / camera_transform.get_scale(),
                    )),
                    Command::Transform(Transformation::Rotate(-camera_transform.get_rotation())),
                    Command::Transform(Transformation::Translate(
                        -camera_transform.get_translation(),
                    )),
                ],
                CompositeScalingMode::Aspect => vec![
                    Command::Transform(Transformation::Scale(
                        Vec2::new(s, s) / camera_transform.get_scale(),
                    )),
                    Command::Transform(Transformation::Rotate(-camera_transform.get_rotation())),
                    Command::Transform(Transformation::Translate(
                        -camera_transform.get_translation(),
                    )),
                ],
                CompositeScalingMode::CenterAspect => vec![
                    Command::Transform(Transformation::Translate([wh, hh].into())),
                    Command::Transform(Transformation::Scale(
                        Vec2::new(s, s) / camera_transform.get_scale(),
                    )),
                    Command::Transform(Transformation::Rotate(-camera_transform.get_rotation())),
                    Command::Transform(Transformation::Translate(
                        -camera_transform.get_translation(),
                    )),
                ],
            };
            let commands = std::iter::once(Command::Store)
                .chain(camera_transforms.into_iter())
                .chain(
                    sorted
                        .iter()
                        .flat_map(|(_, renderable, transform, entity)| {
                            let [a, b, c, d, e, f] = transform.matrix();
                            vec![
                                Command::Store,
                                Command::Transform(Transformation::Transform(a, b, c, d, e, f)),
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

            drop(renderer.execute(commands));
        }
    }
}
