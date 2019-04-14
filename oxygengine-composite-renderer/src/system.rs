use crate::{
    component::{
        CompositeRenderDepth, CompositeRenderable, CompositeRenderableStroke, CompositeTransform,
    },
    composite_renderer::{Command, CompositeRenderer, Rectangle, Renderable, Transformation},
};
use core::ecs::{Entities, Join, ReadStorage, System, Write};
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
        ReadStorage<'s, CompositeRenderable>,
        ReadStorage<'s, CompositeTransform>,
        ReadStorage<'s, CompositeRenderDepth>,
        ReadStorage<'s, CompositeRenderableStroke>,
    );

    fn run(
        &mut self,
        (renderer, entities, renderables, transforms, depths, strokes): Self::SystemData,
    ) {
        if renderer.is_none() {
            return;
        }

        let renderer: &mut CR = &mut renderer.unwrap();
        renderer.update_state();
        let viewport = renderer.viewport();
        let mut sorted = (&entities, &renderables, &transforms)
            .join()
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
        let commands = std::iter::once(if let Some(color) = renderer.state().clear_color {
            Command::Draw(Renderable::Rectangle(Rectangle {
                color,
                rect: viewport,
            }))
        } else {
            Command::None
        })
        .chain(
            sorted
                .iter()
                .flat_map(|(_, renderable, transform, entity)| {
                    let renderable = if let Some(stroke) = strokes.get(*entity) {
                        Command::Stroke(stroke.0, renderable.0.clone())
                    } else {
                        Command::Draw(renderable.0.clone())
                    };
                    vec![
                        Command::Store,
                        Command::Transform(Transformation::Translate(transform.translation)),
                        Command::Transform(Transformation::Rotate(transform.rotation)),
                        Command::Transform(Transformation::Scale(transform.scale)),
                        renderable,
                        Command::Restore,
                    ]
                }),
        );

        drop(renderer.execute(commands));
    }
}
