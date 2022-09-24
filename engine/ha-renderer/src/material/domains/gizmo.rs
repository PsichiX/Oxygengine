use crate::{
    material::graph::MaterialGraph,
    material_graph,
    math::*,
    mesh::{
        vertex_factory::{StaticVertexFactory, VertexType},
        MeshDrawMode, MeshError,
    },
    vertex_type,
};
use serde::{Deserialize, Serialize};

pub fn default_gizmo_color_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [[TintColor => vColor] -> BaseColor]
    }
}

pub fn gizmo_domain_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [fragment] inout BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
            [fragment] inout VisibilityMask: bool = {true};

            [vertex] uniform model: mat4;
            [vertex] uniform view: mat4;
            [vertex] uniform projection: mat4;

            [vertex] in position: vec3 = vec3(0.0, 0.0, 0.0);
            [vertex] in phase: float = 0.0;
            [vertex] in color: vec4 = vec4(1.0, 1.0, 1.0, 1.0);
        }

        outputs {
            [vertex] inout LocalPosition: vec3;
            [vertex] inout WorldPosition: vec3;
            [vertex] inout ScreenPosition: vec3;
            [vertex] inout Phase: float;
            [vertex] inout TintColor: vec4;

            [vertex] builtin gl_Position: vec4;
            [fragment] out finalColor: vec4;
        }

        [discarded = (discard_test, condition: (negate, v: VisibilityMask))]
        [view_projection = (mul_mat4, a: projection, b: view)]
        [pos = (append_vec4, a: position, b: {1.0})]
        [world_position = (truncate_vec4, v: (mul_mat4_vec4, a: model, b: pos))]
        [(append_vec4, a: world_position, b: {1.0}) -> pos]
        [screen_position = (truncate_vec4, v: (mul_mat4_vec4, a: view_projection, b: pos))]

        [position -> LocalPosition]
        [world_position -> WorldPosition]
        [screen_position -> ScreenPosition]
        [phase -> Phase]
        [color -> TintColor]
        [(append_vec4, a: screen_position, b: {1.0}) -> gl_Position]
        [(if_vec4,
            condition: discarded,
            truthy: {vec4(0.0, 0.0, 0.0, 0.0)},
            falsy: BaseColor
        ) -> finalColor]
    }
}

fn default_position() -> vek::Vec3<f32> {
    vec3(0.0, 0.0, 0.0)
}

fn default_color() -> vek::Vec4<f32> {
    vec4(1.0, 1.0, 1.0, 1.0)
}

pub trait GizmoDomain: VertexType {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(GizmoDomain)
    pub struct GizmoVertex {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default)]
        pub phase: float = phase(0),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GizmoFactory {
    vertices: Vec<GizmoVertex>,
    indices: Vec<u32>,
}

impl GizmoFactory {
    pub fn with_capacity(vertex_capacity: usize, index_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            indices: Vec::with_capacity(index_capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn line(&mut self, color: vek::Vec4<f32>, from: vek::Vec3<f32>, to: vek::Vec3<f32>) {
        self.vertices.reserve(2);
        self.indices.reserve(2);
        let offset = self.vertices.len() as u32;
        self.vertices.push(GizmoVertex {
            position: from,
            phase: 0.0,
            color,
        });
        self.vertices.push(GizmoVertex {
            position: to,
            phase: 1.0,
            color,
        });
        self.indices.push(offset);
        self.indices.push(offset + 1);
    }

    pub fn lines(
        &mut self,
        color: vek::Vec4<f32>,
        iter: impl Iterator<Item = (vek::Vec3<f32>, vek::Vec3<f32>)>,
    ) {
        let size = iter.size_hint();
        let size = size.1.unwrap_or(size.0);
        self.vertices.reserve(size * 2);
        self.indices.reserve(size * 2);
        let mut phase = false;
        let mut offset = self.vertices.len() as u32;
        for (from, to) in iter {
            let (p1, p2) = if phase { (1.0, 0.0) } else { (0.0, 1.0) };
            self.vertices.push(GizmoVertex {
                position: from,
                phase: p1,
                color,
            });
            self.vertices.push(GizmoVertex {
                position: to,
                phase: p2,
                color,
            });
            self.indices.push(offset);
            self.indices.push(offset + 1);
            offset += 2;
            phase = !phase;
        }
    }

    pub fn line_string(
        &mut self,
        color: vek::Vec4<f32>,
        mut iter: impl Iterator<Item = vek::Vec3<f32>>,
    ) -> bool {
        let size = iter.size_hint();
        let size = size.1.unwrap_or(size.0);
        self.vertices.reserve(size);
        self.indices.reserve(size.saturating_sub(1) * 2);
        let mut phase = false;
        let mut offset = self.vertices.len() as u32;
        let position = match iter.next() {
            Some(position) => position,
            None => return false,
        };
        self.vertices.push(GizmoVertex {
            position,
            phase: 0.0,
            color,
        });
        for position in iter {
            phase = !phase;
            let p = if phase { 1.0 } else { 0.0 };
            self.vertices.push(GizmoVertex {
                position,
                phase: p,
                color,
            });
            self.indices.push(offset);
            self.indices.push(offset + 1);
            offset += 1;
        }
        true
    }

    pub fn polygon(
        &mut self,
        color: vek::Vec4<f32>,
        mut iter: impl Iterator<Item = vek::Vec3<f32>>,
    ) -> bool {
        let size = iter.size_hint();
        let size = size.1.unwrap_or(size.0);
        self.vertices.reserve(size);
        self.indices.reserve(size * 2);
        let mut phase = false;
        let first = self.vertices.len() as u32;
        let mut offset = first;
        let position = match iter.next() {
            Some(position) => position,
            None => return false,
        };
        self.vertices.push(GizmoVertex {
            position,
            phase: 0.0,
            color,
        });
        for position in iter {
            phase = !phase;
            let p = if phase { 1.0 } else { 0.0 };
            self.vertices.push(GizmoVertex {
                position,
                phase: p,
                color,
            });
            self.indices.push(offset);
            self.indices.push(offset + 1);
            offset += 1;
        }
        self.indices.push(offset);
        self.indices.push(first);
        true
    }

    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        let mut result = StaticVertexFactory::new(
            GizmoVertex::vertex_layout()?,
            self.vertices.len(),
            self.indices.len(),
            MeshDrawMode::Lines,
        );
        let positions = self.vertices.iter().map(|v| v.position).collect::<Vec<_>>();
        result.vertices_vec3f("position", &positions, None)?;
        let phases = self.vertices.iter().map(|v| v.phase).collect::<Vec<_>>();
        result.vertices_scalar("phase", &phases, None)?;
        let colors = self.vertices.iter().map(|v| v.color).collect::<Vec<_>>();
        result.vertices_vec4f("color", &colors, None)?;
        let indices = self
            .indices
            .chunks_exact(2)
            .map(|c| (c[0], c[1]))
            .collect::<Vec<_>>();
        result.lines(&indices, None)?;
        Ok(result)
    }
}
