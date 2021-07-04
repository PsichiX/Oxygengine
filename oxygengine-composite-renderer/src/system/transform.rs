use crate::{component::CompositeTransform, math::Mat2d, resource::CompositeTransformCache};
use core::ecs::{
    hierarchy::{Hierarchy, Parent},
    Comp, Entity, Universe, World, WorldRef,
};

pub type CompositeTransformSystemResources<'a> = (
    WorldRef,
    &'a Hierarchy,
    &'a mut CompositeTransformCache,
    Comp<&'a Parent>,
    Comp<&'a CompositeTransform>,
);

pub fn composite_transform_system(universe: &mut Universe) {
    let (world, hierarchy, mut cache, ..) =
        universe.query_resources::<CompositeTransformSystemResources>();

    cache.clear();
    for (entity, transform) in world
        .query::<&CompositeTransform>()
        .without::<Parent>()
        .iter()
    {
        let mat = transform.matrix();
        cache.insert(entity, mat);
        if let Some(children) = hierarchy.children(entity) {
            for child in children {
                add_matrix(child, &world, mat, &hierarchy, &mut cache);
            }
        }
    }
}

fn add_matrix(
    child: Entity,
    world: &World,
    parent_matrix: Mat2d,
    hierarchy: &Hierarchy,
    cache: &mut CompositeTransformCache,
) {
    if let Ok(transform) = unsafe { world.get_unchecked::<CompositeTransform>(child) } {
        let mat = parent_matrix * transform.matrix();
        cache.insert(child, mat);
        if let Some(children) = hierarchy.children(child) {
            for child in children {
                add_matrix(child, world, mat, hierarchy, cache);
            }
        }
    }
}
