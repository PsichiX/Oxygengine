use crate::{
    material::domains::surface::SurfaceDomain,
    math::*,
    mesh::{
        geometry::{Geometry, GeometryPrimitives, GeometryVertices, GeometryVerticesColumn},
        vertex_factory::StaticVertexFactory,
        MeshError,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SurfaceQuadFactory {
    pub size: vek::Vec2<f32>,
    pub align: vek::Vec2<f32>,
    pub color: vek::Vec4<f32>,
}

impl Default for SurfaceQuadFactory {
    fn default() -> Self {
        Self {
            size: vec2(1.0, 1.0),
            align: vec2(0.5, 0.5),
            color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl SurfaceQuadFactory {
    pub fn geometry(self) -> Result<Geometry, MeshError> {
        let ox = -self.size.x * self.align.x;
        let oy = -self.size.y * self.align.y;
        Ok(Geometry::new(
            GeometryVertices::default().with_columns([
                GeometryVerticesColumn::new(
                    "position",
                    [
                        vec2(ox, oy),
                        vec2(ox + self.size.x, oy),
                        vec2(ox + self.size.x, oy + self.size.y),
                        vec2(ox, oy + self.size.y),
                    ]
                    .into_iter()
                    .collect(),
                ),
                GeometryVerticesColumn::new(
                    "textureCoord",
                    [
                        vec2(0.0, 0.0),
                        vec2(1.0, 0.0),
                        vec2(1.0, 1.0),
                        vec2(0.0, 1.0),
                    ]
                    .into_iter()
                    .collect(),
                ),
                GeometryVerticesColumn::new(
                    "color",
                    std::iter::repeat(self.color).take(4).collect(),
                ),
            ])?,
            GeometryPrimitives::triangles(vec![[0, 1, 2].into(), [2, 3, 0].into()]),
        ))
    }

    pub fn factory<T>(self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        self.geometry()?.factory::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::domains::surface::*;

    #[test]
    fn test_surface_quad_factory() {
        let factory = SurfaceQuadFactory {
            size: vek::Vec2::new(1.0, 1.0),
            align: vek::Vec2::new(0.5, 0.5),
            color: vek::Vec4::new(1.0, 0.0, 0.0, 1.0),
        };
        let factory = factory.factory::<SurfaceVertexP>().unwrap();
        println!("* Factory: {:#?}", factory);
        let (_, buffers, _, _, _) = factory.into_inner();
        let buffer = &buffers[0];
        let data = unsafe { buffer.align_to::<f32>().1 };
        let data = data.chunks(3).collect::<Vec<_>>();
        println!("* Data: {:#?}", data);
    }
}
