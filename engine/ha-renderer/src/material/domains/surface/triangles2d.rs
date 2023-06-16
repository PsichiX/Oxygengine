use crate::{
    material::domains::surface::SurfaceDomain,
    math::*,
    mesh::{
        geometry::{
            Geometry, GeometryPrimitives, GeometryTriangle, GeometryVertices,
            GeometryVerticesColumn,
        },
        vertex_factory::StaticVertexFactory,
        MeshError,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SurfaceTriangle2dVertex {
    pub position: vek::Vec2<f32>,
    pub texture_coord: vek::Vec2<f32>,
    pub color: vek::Vec4<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceTriangles2dFactory {
    pub triangles: Vec<[SurfaceTriangle2dVertex; 3]>,
}

impl SurfaceTriangles2dFactory {
    pub fn geometry(&self) -> Result<Geometry, MeshError> {
        Ok(Geometry::new(
            GeometryVertices::default().with_columns([
                GeometryVerticesColumn::new(
                    "position",
                    self.triangles
                        .iter()
                        .flat_map(|[a, b, c]| [a.position, b.position, c.position])
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "textureCoord",
                    self.triangles
                        .iter()
                        .flat_map(|[a, b, c]| [a.texture_coord, b.texture_coord, c.texture_coord])
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "color",
                    self.triangles
                        .iter()
                        .flat_map(|[a, b, c]| [a.color, b.color, c.color])
                        .collect(),
                ),
            ])?,
            GeometryPrimitives::triangles(
                (0..self.triangles.len())
                    .map(|i| {
                        let i = i * 3;
                        GeometryTriangle::new([i, i + 1, i + 2])
                    })
                    .collect::<Vec<_>>(),
            ),
        ))
    }

    pub fn factory<T>(&self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        self.geometry()?.factory::<T>()
    }
}
