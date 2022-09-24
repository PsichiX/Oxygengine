use crate::components::transform::HaTransform;
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

    for (entity, transform) in world
        .query::<&mut HaTransform>()
        .without::<&Parent>()
        .iter()
    {
        transform.rebuild_world_matrix(None);
        if let Some(children) = hierarchy.children(entity) {
            for child in children {
                if child != entity {
                    propagate(child, &world, transform, &hierarchy);
                }
            }
        }
    }
}

fn propagate(child: Entity, world: &World, parent_transform: &HaTransform, hierarchy: &Hierarchy) {
    if let Ok(transform) = unsafe { world.get_unchecked::<&mut HaTransform>(child) } {
        transform.rebuild_world_matrix(Some(parent_transform));
        if let Some(children) = hierarchy.children(child) {
            for child in children {
                propagate(child, world, transform, hierarchy);
            }
        }
    }
}
