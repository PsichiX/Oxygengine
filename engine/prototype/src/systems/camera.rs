use crate::{resources::camera::*, systems::render_prototype_stage::RenderPrototypeStage};
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;

pub type CameraSystemResources<'a> = (
    WorldRef,
    &'a mut Camera,
    &'a CameraCache,
    Comp<&'a mut HaCamera>,
    Comp<&'a mut HaTransform>,
    Comp<&'a HaDefaultCamera>,
);

pub fn camera_system(universe: &mut Universe) {
    let (world, mut settings, cache, ..) = universe.query_resources::<CameraSystemResources>();

    if let Some(info) = cache.default_get_first::<RenderPrototypeStage>() {
        settings.viewport_size = vec2(info.width as _, info.height as _);
        settings.projection_matrix = info.projection_matrix;
        settings.projection_matrix_inv = info.projection_matrix.inverted();
    }

    for (_, (camera, transform)) in world
        .query::<(&mut HaCamera, &mut HaTransform)>()
        .with::<&HaDefaultCamera>()
        .iter()
    {
        *transform = settings.transform().into();
        camera.projection = HaCameraProjection::Orthographic(HaCameraOrthographic {
            scaling: HaCameraOrtographicScaling::FitToView(settings.view_size.into(), false),
            centered: true,
            ignore_depth_planes: false,
        });
    }
}
