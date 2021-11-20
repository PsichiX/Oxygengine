use crate::{components::transform::HaTransform, math::Mat4};
use core::ecs::{
    hierarchy::{Hierarchy, Parent},
    Comp, Entity, Universe, World, WorldRef,
};

pub type HaTransformSystemResources<'a> = (
    WorldRef,
    &'a Hierarchy,
    Comp<&'a Parent>,
    Comp<&'a mut HaTransform>,
);

pub fn ha_transform_system(universe: &mut Universe) {
    let (world, hierarchy, ..) = universe.query_resources::<HaTransformSystemResources>();

    let identity = Mat4::identity();
    for (entity, transform) in world.query::<&mut HaTransform>().without::<Parent>().iter() {
        transform.rebuild_world_matrix(identity);
        let mat = transform.local_matrix();
        if let Some(children) = hierarchy.children(entity) {
            for child in children {
                if child != entity {
                    propagate(child, &world, mat, &hierarchy);
                }
            }
        }
    }
}

fn propagate(child: Entity, world: &World, parent_matrix: Mat4, hierarchy: &Hierarchy) {
    if let Ok(transform) = unsafe { world.get_unchecked_mut::<HaTransform>(child) } {
        let mat = parent_matrix * transform.local_matrix();
        transform.rebuild_world_matrix(mat);
        if let Some(children) = hierarchy.children(child) {
            for child in children {
                propagate(child, world, mat, hierarchy);
            }
        }
    }
}
