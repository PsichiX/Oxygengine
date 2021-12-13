use crate::{
    material::domains::surface::SurfaceDomain,
    math::*,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SurfaceGridFactory {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: vek::Vec2<f32>,
    pub color: vek::Vec4<f32>,
}

impl Default for SurfaceGridFactory {
    fn default() -> Self {
        Self {
            cols: 1,
            rows: 1,
            cell_size: vec2(1.0, 1.0),
            color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl SurfaceGridFactory {
    pub fn factory<T>(self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        if self.cols == 0 || self.rows == 0 {
            return Err(MeshError::ZeroSize);
        }
        let vertex_layout = T::vertex_layout()?;
        if !T::has_attribute("position") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "position".to_owned(),
            ));
        }
        let cols = self.cols + 1;
        let rows = self.rows + 1;
        let vertex_count = cols * rows;
        let triangles_count = self.cols * self.rows * 2;
        let mut result = StaticVertexFactory::new(
            vertex_layout,
            vertex_count,
            triangles_count,
            MeshDrawMode::Triangles,
        );
        let mut position = Vec::with_capacity(vertex_count);
        for row in 0..rows {
            for col in 0..cols {
                position.push(vec3(
                    self.cell_size.x * col as f32,
                    self.cell_size.y * row as f32,
                    0.0,
                ));
            }
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
            let mut texture_coord = Vec::with_capacity(vertex_count);
            for row in 0..rows {
                for col in 0..cols {
                    texture_coord.push(vec3(
                        col as f32 / cols as f32,
                        row as f32 / rows as f32,
                        0.0,
                    ));
                }
            }
            result.vertices_vec3f("textureCoord", &texture_coord, None)?;
        }
        if T::has_attribute("color") {
            result.vertices_vec4f(
                "color",
                &(0..vertex_count).map(|_| self.color).collect::<Vec<_>>(),
                None,
            )?;
        }
        let mut triangles = Vec::with_capacity(triangles_count);
        for row in 0..self.rows {
            for col in 0..self.cols {
                let tl = Self::coord_to_index(col, row, cols) as u32;
                let tr = Self::coord_to_index(col + 1, row, cols) as u32;
                let br = Self::coord_to_index(col + 1, row + 1, cols) as u32;
                let bl = Self::coord_to_index(col, row + 1, cols) as u32;
                triangles.push((tl, tr, br));
                triangles.push((br, bl, tl));
            }
        }
        result.triangles(&triangles, None)?;
        Ok(result)
    }

    fn coord_to_index(col: usize, row: usize, cols: usize) -> usize {
        row * cols + col
    }
}
