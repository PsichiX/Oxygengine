use crate::{
    material::domains::surface::SurfaceDomain,
    math::*,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
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
    pub fn factory<T>(self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        if self.level == 0 {
            return Err(MeshError::ZeroSize);
        }
        let vertex_layout = T::vertex_layout()?;
        if !T::has_attribute("position") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "position".to_owned(),
            ));
        }
        let edges = 3 * self.level;
        let vertex_count = 1 + edges;
        let mut result =
            StaticVertexFactory::new(vertex_layout, vertex_count, edges, MeshDrawMode::Triangles);
        let tangents = (0..edges)
            .map(|index| {
                let angle = std::f32::consts::PI * index as f32 / edges as f32;
                angle.sin_cos()
            })
            .collect::<Vec<_>>();
        let mut position = Vec::with_capacity(vertex_count);
        position.push(vec3(0.0, 0.0, 0.0));
        for (x, y) in &tangents {
            position.push(vec3(*x * self.radius, *y * self.radius, 0.0));
        }
        result.vertices_vec3f("position", &position, None)?;
        if T::has_attribute("normal") {
            result.vertices_vec3f(
                "normal",
                &(0..vertex_count)
                    .map(|_| vec3(0.0, 0.0, 1.0))
                    .collect::<Vec<_>>(),
                None,
            )?;
        }
        if T::has_attribute("textureCoord") {
            result.vertices_vec3f(
                "textureCoord",
                &tangents
                    .into_iter()
                    .map(|(x, y)| vec3((x + 1.0) * 0.5, (y + 1.0) * 0.5, 0.0))
                    .collect::<Vec<_>>(),
                None,
            )?;
        }
        if T::has_attribute("color") {
            result.vertices_vec4f(
                "color",
                &(0..vertex_count).map(|_| self.color).collect::<Vec<_>>(),
                None,
            )?;
        }
        result.triangles(
            &(0..edges)
                .map(|index| (0, (1 + index) as u32, (1 + (index + 1) % edges) as u32))
                .collect::<Vec<_>>(),
            None,
        )?;
        Ok(result)
    }
}
