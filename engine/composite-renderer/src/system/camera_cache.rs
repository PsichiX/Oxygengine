use crate::{
    component::{CompositeCamera, CompositeTransform},
    composite_renderer::CompositeRenderer,
    resource::CompositeCameraCache,
};
use core::ecs::{Comp, Universe, WorldRef};
use std::collections::HashMap;

pub type CompositeCameraCacheSystemResources<'a, CR> = (
    WorldRef,
    &'a CR,
    &'a mut CompositeCameraCache,
    Comp<&'a CompositeCamera>,
    Comp<&'a CompositeTransform>,
);

pub fn composite_camera_cache_system<CR>(universe: &mut Universe)
where
    CR: CompositeRenderer + 'static,
{
    let (world, renderer, mut cache, ..) =
        universe.query_resources::<CompositeCameraCacheSystemResources<CR>>();

    let view_size = renderer.view_size();
    cache.last_view_size = view_size;
    cache.world_transforms = world
        .query::<(&CompositeCamera, &CompositeTransform)>()
        .iter()
        .map(|(entity, (camera, transform))| (entity, camera.view_matrix(transform, view_size)))
        .collect::<HashMap<_, _>>();
    cache.world_inverse_transforms = cache
        .world_transforms
        .iter()
        .filter_map(|(k, v)| v.inverse().map(|v| (*k, v)))
        .collect::<HashMap<_, _>>();
}
