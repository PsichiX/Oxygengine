use crate::{
    material::domains::surface::SurfaceDomain,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceImmediateFactory<V>
where
    V: SurfaceDomain,
{
    vertices: Vec<V>,
    triangles: Vec<(u32, u32, u32)>,
}

impl<V> Default for SurfaceImmediateFactory<V>
where
    V: SurfaceDomain,
{
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            triangles: Default::default(),
        }
    }
}

impl<V> SurfaceImmediateFactory<V>
where
    V: SurfaceDomain + Copy,
{
    pub fn with_capacity(vertex_capacity: usize, triangle_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            triangles: Vec::with_capacity(triangle_capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.triangles.clear();
    }

    pub fn triangles(&mut self, vertices: &[V], triangles: &[(u32, u32, u32)]) {
        self.vertices.reserve(vertices.len());
        self.vertices.extend(vertices.iter().copied());
        self.triangles.reserve(triangles.len());
        self.triangles.extend(triangles.iter().copied());
    }

    pub fn triangle(&mut self, vertices: [V; 3]) {
        self.triangles(&vertices, &[(0, 1, 2)]);
    }

    pub fn quad(&mut self, vertices: [V; 4]) {
        self.triangles(&vertices, &[(0, 1, 2), (2, 3, 0)]);
    }

    pub fn polygon(&mut self, vertices: &[V]) -> bool {
        if vertices.len() < 3 {
            return false;
        }
        let triangles = vertices.len() - 2;
        self.vertices.reserve(vertices.len());
        self.vertices.extend(vertices.iter().copied());
        self.triangles.reserve(triangles);
        for i in 0..(triangles as u32) {
            self.triangles.push((0, i + 1, i + 2));
        }
        true
    }

    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        let mut result = StaticVertexFactory::new(
            V::vertex_layout()?,
            self.vertices.len(),
            self.triangles.len(),
            MeshDrawMode::Triangles,
        );
        result.vertices(&self.vertices, None)?;
        result.triangles(&self.triangles, None)?;
        Ok(result)
    }
}
