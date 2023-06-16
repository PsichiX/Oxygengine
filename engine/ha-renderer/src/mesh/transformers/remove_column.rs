use crate::mesh::{geometry::Geometry, MeshError};

pub fn remove_column(mut geometry: Geometry, name: &str) -> Result<Geometry, MeshError> {
    geometry.vertices.remove_column(name);
    Ok(geometry)
}
