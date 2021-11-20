use crate::{
    components::{gizmo::HaGizmo, mesh_instance::HaMeshInstance, transform::HaTransform},
    ha_renderer::HaRenderer,
    resources::gizmos::Gizmos,
};
use core::ecs::{Comp, Universe, WorldRef};

pub type HaMeshBoundsGizmoSystemResources<'a> = (
    WorldRef,
    &'a HaRenderer,
    &'a mut Gizmos,
    Comp<&'a HaTransform>,
    Comp<&'a HaGizmo>,
    Comp<&'a HaMeshInstance>,
);

pub fn ha_mesh_bounds_gizmo_system(universe: &mut Universe) {
    let (world, renderer, mut gizmos, ..) =
        universe.query_resources::<HaMeshBoundsGizmoSystemResources>();

    for (_, (transform, gizmo, instance)) in world
        .query::<(&HaTransform, &HaGizmo, &HaMeshInstance)>()
        .iter()
    {
        if !gizmo.visible {
            continue;
        }
        let mut points = match instance
            .reference
            .id()
            .and_then(|id| renderer.mesh(*id))
            .and_then(|m| m.bounds())
            .map(|b| b.box_vertices())
        {
            Some(points) => points,
            None => continue,
        };
        let matrix = transform.world_matrix();
        for point in &mut points {
            *point = matrix.mul_point(*point);
        }
        // TODO: replace with simpler direct write of points and their indices.
        let vertices = [
            (points[0], points[1]),
            (points[1], points[2]),
            (points[2], points[3]),
            (points[3], points[0]),
            (points[4], points[5]),
            (points[5], points[6]),
            (points[6], points[7]),
            (points[7], points[4]),
            (points[0], points[4]),
            (points[1], points[5]),
            (points[2], points[6]),
            (points[3], points[7]),
        ];
        gizmos
            .factory
            .lines(gizmo.color.into(), vertices.into_iter());
    }
}
