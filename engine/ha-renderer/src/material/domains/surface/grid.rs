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
    pub fn geometry(self, meta: bool) -> Result<Geometry, MeshError> {
        if self.cols == 0 || self.rows == 0 {
            return Err(MeshError::ZeroSize);
        }
        let cols = self.cols + 1;
        let rows = self.rows + 1;
        let vertex_count = cols * rows;
        Ok(Geometry::new(
            GeometryVertices::default().with_columns([
                GeometryVerticesColumn::new(
                    "position",
                    (0..rows)
                        .flat_map(|row| {
                            (0..cols).map(move |col| {
                                vec2(self.cell_size.x * col as f32, self.cell_size.y * row as f32)
                            })
                        })
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "textureCoord",
                    (0..rows)
                        .flat_map(|row| {
                            (0..cols).map(move |col| {
                                vec2(
                                    col as f32 / (cols - 1) as f32,
                                    row as f32 / (rows - 1) as f32,
                                )
                            })
                        })
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "color",
                    std::iter::repeat(self.color).take(vertex_count).collect(),
                ),
            ])?,
            GeometryPrimitives::triangles(
                (0..self.rows)
                    .flat_map(|row| {
                        (0..self.cols).flat_map(move |col| {
                            let tl = Self::coord_to_index(col, row, cols);
                            let tr = Self::coord_to_index(col + 1, row, cols);
                            let br = Self::coord_to_index(col + 1, row + 1, cols);
                            let bl = Self::coord_to_index(col, row + 1, cols);
                            let mut a = GeometryTriangle::new([tl, tr, br]);
                            let mut b = GeometryTriangle::new([br, bl, tl]);
                            if meta {
                                a.attributes.set("col", col as i32);
                                b.attributes.set("col", col as i32);
                                a.attributes.set("row", row as i32);
                                b.attributes.set("row", row as i32);
                            }
                            [a, b]
                        })
                    })
                    .collect::<Vec<_>>(),
            ),
        ))
    }

    pub fn factory<T>(self) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceDomain,
    {
        self.geometry(false)?.factory::<T>()
    }

    fn coord_to_index(col: usize, row: usize, cols: usize) -> usize {
        row * cols + col
    }
}
