use crate::{
    material::domains::surface::SurfaceDomain,
    math::*,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
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
    pub fn factory<T>(self) -> Result<StaticVertexFactory, MeshError>
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
        let mut result = StaticVertexFactory::new(vertex_layout, 4, 2, MeshDrawMode::Triangles);
        let ox = -self.size.x * self.align.x;
        let oy = -self.size.y * self.align.y;
        result.vertices_vec3f(
            "position",
            &[
                vec3(ox, oy, 0.0),
                vec3(ox + self.size.x, oy, 0.0),
                vec3(ox + self.size.x, oy + self.size.y, 0.0),
                vec3(ox, oy + self.size.y, 0.0),
            ],
            None,
        )?;
        if T::has_attribute("normal") {
            result.vertices_vec3f(
                "normal",
                &[
                    vec3(0.0, 0.0, 1.0),
                    vec3(0.0, 0.0, 1.0),
                    vec3(0.0, 0.0, 1.0),
                    vec3(0.0, 0.0, 1.0),
                ],
                None,
            )?;
        }
        if T::has_attribute("textureCoord") {
            result.vertices_vec3f(
                "textureCoord",
                &[
                    vec3(0.0, 0.0, 0.0),
                    vec3(1.0, 0.0, 0.0),
                    vec3(1.0, 1.0, 0.0),
                    vec3(0.0, 1.0, 0.0),
                ],
                None,
            )?;
        }
        if T::has_attribute("color") {
            result.vertices_vec4f(
                "color",
                &[self.color, self.color, self.color, self.color],
                None,
            )?;
        }
        result.triangles(&[(0, 1, 2), (2, 3, 0)], None)?;
        Ok(result)
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
