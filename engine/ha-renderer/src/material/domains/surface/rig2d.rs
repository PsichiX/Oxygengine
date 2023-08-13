use crate::{
    components::transform::HaTransform,
    math::*,
    mesh::{
        geometry::{
            Geometry, GeometryPrimitives, GeometryTriangle, GeometryValues, GeometryVertices,
            GeometryVerticesColumn,
        },
        rig::{deformer::Deformer, skeleton::Skeleton},
        transformers::apply_deformer::apply_deformer,
        vertex_factory::{StaticVertexFactory, VertexType},
        MeshError,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRig2dSprite {
    pub size: vek::Vec2<f32>,
    #[serde(default = "SurfaceRig2dSprite::default_pivot")]
    pub pivot: vek::Vec2<f32>,
    #[serde(default = "SurfaceRig2dSprite::default_uvs_rect")]
    pub uvs_rect: vek::Rect<f32, f32>,
    #[serde(default = "SurfaceRig2dSprite::default_color")]
    pub color: vek::Vec4<f32>,
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
}

impl Default for SurfaceRig2dSprite {
    fn default() -> Self {
        Self {
            size: Default::default(),
            pivot: Self::default_pivot(),
            uvs_rect: Self::default_uvs_rect(),
            color: Self::default_color(),
            cols: 0,
            rows: 0,
        }
    }
}

impl SurfaceRig2dSprite {
    fn default_pivot() -> vek::Vec2<f32> {
        0.5.into()
    }

    fn default_uvs_rect() -> vek::Rect<f32, f32> {
        rect(0.0, 0.0, 1.0, 1.0)
    }

    fn default_color() -> vek::Vec4<f32> {
        1.0.into()
    }

    pub fn new(size: vek::Vec2<f32>) -> Self {
        Self::default().size(size)
    }

    pub fn size(mut self, size: vek::Vec2<f32>) -> Self {
        self.size = size;
        self
    }

    pub fn pivot(mut self, pivot: vek::Vec2<f32>) -> Self {
        self.pivot = pivot;
        self
    }

    pub fn uvs_rect(mut self, uvs_rect: vek::Rect<f32, f32>) -> Self {
        self.uvs_rect = uvs_rect;
        self
    }

    pub fn color(mut self, color: vek::Vec4<f32>) -> Self {
        self.color = color;
        self
    }

    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = cols;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    fn vertices_count(&self) -> usize {
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        (cols + 1) * (rows + 1)
    }

    fn positions(&self) -> impl Iterator<Item = vek::Vec2<f32>> + '_ {
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        let offset = self.pivot * self.size;
        (0..=rows).flat_map(move |row| {
            (0..=cols).map(move |col| {
                (vec2(col as f32, row as f32) * self.size) / vec2(cols as f32, rows as f32) - offset
            })
        })
    }

    fn texture_coords(&self) -> impl Iterator<Item = vek::Vec2<f32>> + '_ {
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        (0..=rows).flat_map(move |row| {
            (0..=cols).map(move |col| {
                let fx = col as f32 / cols as f32;
                let fy = row as f32 / rows as f32;
                vec2(
                    self.uvs_rect.x + self.uvs_rect.w * fx,
                    self.uvs_rect.y + self.uvs_rect.h * fy,
                )
            })
        })
    }

    fn triangles(&self) -> impl Iterator<Item = [usize; 3]> {
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        (0..rows).flat_map(move |row| {
            (0..cols).flat_map(move |col| {
                let tl = Self::coord_to_index(col, row, cols + 1);
                let tr = Self::coord_to_index(col + 1, row, cols + 1);
                let br = Self::coord_to_index(col + 1, row + 1, cols + 1);
                let bl = Self::coord_to_index(col, row + 1, cols + 1);
                [[tl, tr, br], [br, bl, tl]]
            })
        })
    }

    fn coord_to_index(col: usize, row: usize, cols: usize) -> usize {
        row * cols + col
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SurfaceRig2dVertex {
    pub position: vek::Vec2<f32>,
    #[serde(default)]
    pub texture_coord: vek::Vec2<f32>,
    #[serde(default = "SurfaceRig2dVertex::default_color")]
    pub color: vek::Vec4<f32>,
}

impl Default for SurfaceRig2dVertex {
    fn default() -> Self {
        Self {
            position: Default::default(),
            texture_coord: Default::default(),
            color: Self::default_color(),
        }
    }
}

impl SurfaceRig2dVertex {
    fn default_color() -> vek::Vec4<f32> {
        vec::Vec4::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn position(mut self, position: vec::Vec2<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn texture_coord(mut self, texture_coord: vec::Vec2<f32>) -> Self {
        self.texture_coord = texture_coord;
        self
    }

    pub fn color(mut self, color: vec::Vec4<f32>) -> Self {
        self.color = color;
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SurfaceRig2dMesh {
    #[serde(default)]
    pub vertices: Vec<SurfaceRig2dVertex>,
    #[serde(default)]
    pub triangles: Vec<[usize; 3]>,
}

impl SurfaceRig2dMesh {
    pub fn vertex(mut self, vertex: SurfaceRig2dVertex) -> Self {
        self.vertices.push(vertex);
        self
    }

    pub fn triangle(mut self, triangle: [usize; 3]) -> Self {
        self.triangles.push(triangle);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceRig2dKind {
    Sprite(SurfaceRig2dSprite),
    Mesh(SurfaceRig2dMesh),
}

impl SurfaceRig2dKind {
    fn vertices_count(&self) -> usize {
        match self {
            Self::Sprite(sprite) => sprite.vertices_count(),
            Self::Mesh(mesh) => mesh.vertices.len(),
        }
    }
}

impl From<SurfaceRig2dSprite> for SurfaceRig2dKind {
    fn from(value: SurfaceRig2dSprite) -> Self {
        Self::Sprite(value)
    }
}

impl From<SurfaceRig2dMesh> for SurfaceRig2dKind {
    fn from(value: SurfaceRig2dMesh) -> Self {
        Self::Mesh(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRig2dNode {
    pub bone: String,
    pub deformer: String,
    pub kind: SurfaceRig2dKind,
    #[serde(default)]
    pub depth: f32,
    #[serde(default)]
    pub attachment_transform: HaTransform,
    /// { bone name: influence range }
    #[serde(default)]
    pub bones_influence: HashMap<String, f32>,
}

impl SurfaceRig2dNode {
    pub fn new(bone_name: impl ToString, kind: impl Into<SurfaceRig2dKind>) -> Self {
        Self {
            bone: bone_name.to_string(),
            deformer: Default::default(),
            kind: kind.into(),
            depth: 0.0,
            attachment_transform: Default::default(),
            bones_influence: Default::default(),
        }
    }

    pub fn bone(mut self, name: impl ToString) -> Self {
        self.bone = name.to_string();
        self
    }

    pub fn deformer(mut self, name: impl ToString) -> Self {
        self.deformer = name.to_string();
        self
    }

    pub fn kind(mut self, kind: SurfaceRig2dKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    pub fn attachment_transform(mut self, attachment_transform: HaTransform) -> Self {
        self.attachment_transform = attachment_transform;
        self
    }

    pub fn bone_influence(mut self, bone_name: &str, range: f32) -> Self {
        self.bones_influence.insert(bone_name.to_owned(), range);
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SurfaceRig2dFactory {
    pub nodes: Vec<SurfaceRig2dNode>,
}

impl SurfaceRig2dFactory {
    pub fn node(mut self, node: SurfaceRig2dNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn geometry(
        &self,
        skeleton: &Skeleton,
        deformer: &Deformer,
        meta: bool,
    ) -> Result<Geometry, MeshError> {
        let mut nodes = self
            .nodes
            .iter()
            .filter_map(|node| {
                skeleton
                    .bone_with_index(&node.bone)
                    .map(|(bone, index)| (node, bone, index))
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|(a, _, _), (b, _, _)| a.depth.partial_cmp(&b.depth).unwrap());
        let vertices_count = nodes
            .iter()
            .map(|(node, _, _)| node.kind.vertices_count())
            .sum::<usize>();
        let positions = nodes
            .iter()
            .flat_map(|(node, bone, _)| {
                let matrix = bone.bind_pose_matrix() * node.attachment_transform.local_matrix();
                match &node.kind {
                    SurfaceRig2dKind::Sprite(sprite) => sprite
                        .positions()
                        .map(|point| matrix.mul_point(point))
                        .collect::<Vec<_>>(),
                    SurfaceRig2dKind::Mesh(mesh) => mesh
                        .vertices
                        .iter()
                        .map(|vertex| matrix.mul_point(vertex.position))
                        .collect(),
                }
            })
            .collect::<Vec<_>>();
        let texture_coords = nodes
            .iter()
            .flat_map(|(node, _, _)| match &node.kind {
                SurfaceRig2dKind::Sprite(sprite) => sprite.texture_coords().collect::<Vec<_>>(),
                SurfaceRig2dKind::Mesh(mesh) => mesh
                    .vertices
                    .iter()
                    .map(|vertex| vertex.texture_coord)
                    .collect(),
            })
            .collect::<Vec<_>>();
        let colors = nodes
            .iter()
            .flat_map(|(node, _, _)| match &node.kind {
                SurfaceRig2dKind::Sprite(sprite) => vec![sprite.color; sprite.vertices_count()],
                SurfaceRig2dKind::Mesh(mesh) => {
                    mesh.vertices.iter().map(|vertex| vertex.color).collect()
                }
            })
            .collect::<Vec<_>>();
        let mut bone_indices = Vec::with_capacity(vertices_count);
        let mut bone_weights = Vec::with_capacity(vertices_count);
        let mut deformer_areas = Vec::with_capacity(vertices_count);
        let mut vertex_offset = 0;
        for (node, _, index) in &nodes {
            let bones = node
                .bones_influence
                .iter()
                .filter_map(|(bone, range)| {
                    if *range <= 0.0 {
                        return None;
                    }
                    skeleton.bone_with_index(bone).map(|(bone, index)| {
                        let start = bone.local_matrix().mul_point(vec2(0.0, 0.0));
                        let end = bone.local_matrix().mul_point(bone.target().into());
                        (index, start, end, *range)
                    })
                })
                .collect::<Vec<_>>();
            deformer_areas.push(node.deformer.to_owned());
            if bones.is_empty() {
                let index = *index as i32 & 0xFF;
                for _ in 0..node.kind.vertices_count() {
                    bone_indices.push(index);
                    bone_weights.push(vec4(1.0, 0.0, 0.0, 0.0));
                }
            } else {
                for shift in 0..node.kind.vertices_count() {
                    let point = positions[vertex_offset + shift];
                    let mut bones = bones
                        .iter()
                        .filter_map(|(index, start, end, range)| {
                            let distance = calculate_bone_distance(point, *start, *end);
                            if distance < *range {
                                Some(((range - distance) / range, *index))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if bones.is_empty() {
                        bone_indices.push((*index & 0xFF) as i32);
                        bone_weights.push(vec4(1.0, 0.0, 0.0, 0.0));
                    } else {
                        bones.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());
                        if bones.len() > 4 {
                            bones.resize_with(4, || unreachable!());
                        }
                        let total_weight = bones.iter().fold(0.0, |accum, bone| accum + bone.0);
                        let mut indices = 0_i32;
                        let mut weights = vec4(0.0, 0.0, 0.0, 0.0);
                        for ((index, bone), weight) in
                            bones.into_iter().enumerate().zip(weights.as_mut_slice())
                        {
                            indices |= ((bone.1 & 0xFF) << (index * 8)) as i32;
                            *weight = bone.0 / total_weight;
                        }
                        bone_indices.push(indices);
                        bone_weights.push(weights);
                    }
                }
            }
            vertex_offset += node.kind.vertices_count();
        }
        vertex_offset = 0;
        let triangles = nodes
            .iter()
            .enumerate()
            .flat_map(|(index, (node, _, _))| {
                let offset = vertex_offset;
                vertex_offset += node.kind.vertices_count();
                match &node.kind {
                    SurfaceRig2dKind::Sprite(sprite) => sprite
                        .triangles()
                        .map(|[a, b, c]| {
                            let mut result =
                                GeometryTriangle::new([offset + a, offset + b, offset + c]);
                            if meta {
                                result.attributes.set("index", index as i32);
                                result.attributes.set("bone", &node.bone);
                            }
                            result
                        })
                        .collect::<Vec<_>>(),
                    SurfaceRig2dKind::Mesh(mesh) => mesh
                        .triangles
                        .iter()
                        .map(|[a, b, c]| {
                            let mut result =
                                GeometryTriangle::new([offset + *a, offset + *b, offset + *c]);
                            if meta {
                                result.attributes.set("index", index as i32);
                                result.attributes.set("bone", &node.bone);
                            }
                            result
                        })
                        .collect(),
                }
            })
            .collect::<Vec<_>>();
        let mut result = Geometry::new(
            GeometryVertices::default().with_columns([
                GeometryVerticesColumn::new("position", GeometryValues::Vec2F(positions)),
                GeometryVerticesColumn::new("textureCoord", GeometryValues::Vec2F(texture_coords)),
                GeometryVerticesColumn::new("color", GeometryValues::Vec4F(colors)),
                GeometryVerticesColumn::new("boneIndices", GeometryValues::Integer(bone_indices)),
                GeometryVerticesColumn::new("boneWeights", GeometryValues::Vec4F(bone_weights)),
                GeometryVerticesColumn::new(
                    "@deformer-area",
                    GeometryValues::String(deformer_areas),
                ),
            ])?,
            GeometryPrimitives::triangles(triangles),
        );
        if !deformer.areas.is_empty() {
            result = apply_deformer(result, deformer)?;
        }
        Ok(result)
    }

    pub fn factory<T>(
        &self,
        skeleton: &Skeleton,
        deformer: &Deformer,
    ) -> Result<StaticVertexFactory, MeshError>
    where
        T: VertexType,
    {
        self.geometry(skeleton, deformer, false)?.factory::<T>()
    }
}

fn calculate_bone_distance(
    point: vek::Vec2<f32>,
    start: vek::Vec2<f32>,
    end: vek::Vec2<f32>,
) -> f32 {
    let direction = point - start;
    let (normal, length) = (end - start).normalized_and_get_magnitude();
    let s = direction.dot(normal);
    if s < 0.0 {
        -s
    } else if s <= length {
        0.0
    } else {
        s - length
    }
}
