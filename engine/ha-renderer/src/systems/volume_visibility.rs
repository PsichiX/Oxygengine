use crate::{
    components::{camera::*, mesh_instance::*, transform::*, visibility::*, volume::*},
    ha_renderer::HaRenderer,
    math::*,
};
use core::ecs::{components::Tag, Comp, Universe, WorldRef};

#[derive(Debug, Default)]
pub struct HaVolumeVisibilitySystemCache {
    // We assume static lifetime only to make borow checker happy, but in the system we use this
    // collection in-place and then clear it so we do not actually store here any data with lifetime
    // extended to static.
    temp_tag_volumes: Vec<(&'static str, BoundsVolume)>,
}

pub type HaVolumeVisibilitySystemResources<'a> = (
    WorldRef,
    &'a HaRenderer,
    &'a mut HaVolumeVisibilitySystemCache,
    Comp<&'a Tag>,
    Comp<&'a HaTransform>,
    Comp<&'a HaVolume>,
    Comp<&'a HaVolumeVisibility>,
    Comp<&'a mut HaVisibility>,
    Comp<&'a HaMeshInstance>,
    Comp<&'a HaCamera>,
);

pub fn ha_volume_visibility_system(universe: &mut Universe) {
    let (world, renderer, mut cache, ..) =
        universe.query_resources::<HaVolumeVisibilitySystemResources>();

    cache.temp_tag_volumes.clear();
    cache.temp_tag_volumes.extend(
        world
            .query::<(&Tag, &HaTransform, &HaVolume)>()
            .iter()
            .filter_map(|(_, (tag, transform, volume))| {
                let bounds = match volume {
                    HaVolume::Sphere(r) => BoundsVolume::from_sphere(Default::default(), *r),
                    HaVolume::Box(he) => BoundsVolume::from_box(Default::default(), *he),
                };
                let bounds = bounds.transformed(transform.world_matrix())?;
                // we extend the lifetime to static ONLY for the duration of the system run.
                let tag = unsafe { std::mem::transmute(tag.0.as_ref()) };
                Some((tag, bounds))
            }),
    );

    for (_, (mut visibility, transform, volume, mesh)) in world
        .query::<(
            &mut HaVisibility,
            &HaTransform,
            &HaVolumeVisibility,
            &HaMeshInstance,
        )>()
        .iter()
    {
        let mesh_id = match mesh.reference.id() {
            Some(mesh_id) => *mesh_id,
            None => continue,
        };
        let mesh = match renderer.mesh(mesh_id) {
            Some(mesh) => mesh,
            None => continue,
        };
        let bounds = match mesh.bounds() {
            Some(bounds) => bounds,
            None => continue,
        };
        let bounds = match bounds.transformed(transform.world_matrix()) {
            Some(bounds) => bounds,
            None => continue,
        };
        visibility.0 = cache.temp_tag_volumes.iter().any(|(t, b)| {
            if volume.0.validate_tag(t) {
                if volume.1 {
                    return bounds.overlap_boxes(b);
                } else {
                    return bounds.overlap_spheres(b);
                }
            }
            false
        });
    }

    cache.temp_tag_volumes.clear();
}
