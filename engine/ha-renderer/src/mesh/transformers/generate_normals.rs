use crate::mesh::{
    geometry::{Geometry, GeometryValues, GeometryVerticesColumn},
    MeshError,
};
use vek::*;

pub fn generate_normals(mut geometry: Geometry) -> Result<Geometry, MeshError> {
    let positions = geometry.vertices.column("position")?;
    let mut normals = vec![Vec3::<f32>::default(); positions.len()];
    for triangle in &geometry.primitives.as_triangles()?.items {
        let index_a = triangle.indices[0];
        let index_b = triangle.indices[1];
        let index_c = triangle.indices[2];
        let position_a: Vec3<f32> = positions.get(index_a).unwrap().try_into()?;
        let position_b: Vec3<f32> = positions.get(index_b).unwrap().try_into()?;
        let position_c: Vec3<f32> = positions.get(index_c).unwrap().try_into()?;
        let normal = (position_b - position_a).cross(position_c - position_a);
        normals[index_a] += normal;
        normals[index_b] += normal;
        normals[index_c] += normal;
    }
    for normal in &mut normals {
        normal.normalize();
    }
    geometry.vertices.set_column(GeometryVerticesColumn::new(
        "normal",
        GeometryValues::Vec3F(normals),
    ));
    Ok(geometry)
}
