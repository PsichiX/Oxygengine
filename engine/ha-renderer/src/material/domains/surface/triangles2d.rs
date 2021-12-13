use crate::{
    material::domains::surface::SurfaceDomain,
    math::*,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
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
    pub fn factory<T>(&self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        let vertex_layout = T::vertex_layout()?;
        if !T::has_attribute("position") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "position".to_owned(),
            ));
        }
        let mut result = StaticVertexFactory::new(
            vertex_layout,
            self.triangles.len() * 3,
            self.triangles.len(),
            MeshDrawMode::Triangles,
        );
        let mut positions = Vec::with_capacity(self.triangles.len() * 3);
        for [a, b, c] in &self.triangles {
            positions.push(a.position.into());
            positions.push(b.position.into());
            positions.push(c.position.into());
        }
        result.vertices_vec3f("position", &positions, None)?;
        if T::has_attribute("normal") {
            let normals = std::iter::repeat(vec3(0.0, 0.0, 1.0))
                .take(self.triangles.len() * 3)
                .collect::<Vec<_>>();
            result.vertices_vec3f("normal", &normals, None)?;
        }
        if T::has_attribute("textureCoord") {
            let mut texture_coords = Vec::with_capacity(self.triangles.len() * 3);
            for [a, b, c] in &self.triangles {
                texture_coords.push(a.texture_coord.with_z(0.0));
                texture_coords.push(b.texture_coord.with_z(0.0));
                texture_coords.push(c.texture_coord.with_z(0.0));
            }
            result.vertices_vec3f("textureCoord", &texture_coords, None)?;
        }
        if T::has_attribute("color") {
            let mut colors = Vec::with_capacity(self.triangles.len() * 3);
            for [a, b, c] in &self.triangles {
                colors.push(a.color);
                colors.push(b.color);
                colors.push(c.color);
            }
            result.vertices_vec4f("color", &colors, None)?;
        }
        let indices = (0..self.triangles.len())
            .map(|i| {
                let i = i as u32 * 3;
                (i, i + 1, i + 2)
            })
            .collect::<Vec<_>>();
        result.triangles(&indices, None)?;
        Ok(result)
    }
}
