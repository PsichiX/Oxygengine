use crate::{
    components::{
        camera::{HaCamera, HaDefaultCamera},
        transform::HaTransform,
    },
    ha_renderer::HaRenderer,
    resources::camera_cache::CameraCache,
};
use core::ecs::{components::Name, Comp, Universe, WorldRef};

pub type HaCameraCacheSystemResources<'a> = (
    WorldRef,
    &'a HaRenderer,
    &'a mut CameraCache,
    Comp<&'a HaTransform>,
    Comp<&'a HaCamera>,
    Comp<&'a HaDefaultCamera>,
    Comp<&'a Name>,
);

pub fn ha_camera_cache_system(universe: &mut Universe) {
    let (world, renderer, mut cache, ..) =
        universe.query_resources::<HaCameraCacheSystemResources>();

    cache.default_entity = None;
    cache.info.clear();

    for (entity, (transform, camera, is_default, name)) in world
        .query::<(
            &HaTransform,
            &HaCamera,
            Option<&HaDefaultCamera>,
            Option<&Name>,
        )>()
        .iter()
    {
        if is_default.is_some() {
            cache.default_entity = Some(entity);
        }
        if let Some(iter) = camera.pipeline_stage_info_raw(None, &renderer, transform) {
            cache.info.extend(iter.map(|(type_id, info)| {
                (
                    entity,
                    type_id,
                    name.map(|name| name.0.clone().into()),
                    info,
                )
            }));
        }
    }
}
