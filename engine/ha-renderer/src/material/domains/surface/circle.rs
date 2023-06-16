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
pub struct SurfaceCircleFactory {
    pub radius: f32,
    pub level: usize,
    pub color: vek::Vec4<f32>,
}

impl Default for SurfaceCircleFactory {
    fn default() -> Self {
        Self {
            radius: 1.0,
            level: 20,
            color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl SurfaceCircleFactory {
    pub fn geometry(self) -> Result<Geometry, MeshError> {
        if self.level == 0 {
            return Err(MeshError::ZeroSize);
        }
        let edges = 3 * self.level;
        let vertex_count = 1 + edges;
        let tangents = (0..edges)
            .map(|index| {
                let angle = std::f32::consts::PI * index as f32 / edges as f32;
                angle.sin_cos()
            })
            .collect::<Vec<_>>();
        Ok(Geometry::new(
            GeometryVertices::default().with_columns([
                GeometryVerticesColumn::new(
                    "position",
                    std::iter::once(vec2(0.0, 0.0))
                        .chain(
                            tangents
                                .iter()
                                .map(|(x, y)| vec2(*x * self.radius, *y * self.radius)),
                        )
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "textureCoord",
                    std::iter::once(vec2(0.5, 0.5))
                        .chain(
                            tangents
                                .into_iter()
                                .map(|(x, y)| vec2((x + 1.0) * 0.5, (y + 1.0) * 0.5)),
                        )
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "color",
                    std::iter::repeat(self.color).take(vertex_count).collect(),
                ),
            ])?,
            GeometryPrimitives::triangles(
                (0..edges)
                    .map(|index| GeometryTriangle::new([0, 1 + index, 1 + (index + 1) % edges]))
                    .collect::<Vec<_>>(),
            ),
        ))
    }

    pub fn factory<T>(self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        self.geometry()?.factory::<T>()
    }
}
