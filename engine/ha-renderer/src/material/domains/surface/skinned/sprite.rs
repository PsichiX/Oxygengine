use crate::{
    components::transform::HaTransform,
    material::domains::surface::SurfaceSkinnedDomain,
    math::*,
    mesh::{skeleton::Skeleton, vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceSkinnedSprite {
    pub bone: String,
    pub size: vek::Vec2<f32>,
    #[serde(default = "SurfaceSkinnedSprite::default_pivot")]
    pub pivot: vek::Vec2<f32>,
    #[serde(default = "SurfaceSkinnedSprite::default_uvs_rect")]
    pub uvs_rect: vek::Rect<f32, f32>,
    #[serde(default = "SurfaceSkinnedSprite::default_color")]
    pub color: vek::Vec4<f32>,
    #[serde(default)]
    pub depth: f32,
    #[serde(default)]
    pub attachment_transform: HaTransform,
    /// { bone name: influence range }
    #[serde(default)]
    pub bones_influence: HashMap<String, f32>,
}

impl Default for SurfaceSkinnedSprite {
    fn default() -> Self {
        Self {
            bone: Default::default(),
            size: Default::default(),
            pivot: Self::default_pivot(),
            uvs_rect: Self::default_uvs_rect(),
            color: Self::default_color(),
            depth: 0.0,
            attachment_transform: Default::default(),
            bones_influence: Default::default(),
        }
    }
}

impl SurfaceSkinnedSprite {
    fn default_pivot() -> vek::Vec2<f32> {
        0.5.into()
    }

    fn default_uvs_rect() -> vek::Rect<f32, f32> {
        rect(0.0, 0.0, 1.0, 1.0)
    }

    fn default_color() -> vek::Vec4<f32> {
        1.0.into()
    }

    pub fn new(bone_name: &str, size: vek::Vec2<f32>) -> Self {
        Self::default().bone(bone_name).size(size)
    }

    pub fn bone(mut self, name: &str) -> Self {
        self.bone = name.to_owned();
        self
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
pub struct SurfaceSkinnedSpriteFactory {
    pub sprites: Vec<SurfaceSkinnedSprite>,
}

impl SurfaceSkinnedSpriteFactory {
    pub fn sprite(mut self, sprite: SurfaceSkinnedSprite) -> Self {
        self.sprites.push(sprite);
        self
    }

    pub fn factory<T>(&self, skeleton: &Skeleton) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceSkinnedDomain,
    {
        let vertex_layout = T::vertex_layout()?;
        if !T::has_attribute("position") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "position".to_owned(),
            ));
        }
        if !T::has_attribute("boneIndices") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "boneIndices".to_owned(),
            ));
        }
        if !T::has_attribute("boneWeights") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "boneWeights".to_owned(),
            ));
        }
        let has_texture_coords = T::has_attribute("textureCoord");
        let has_color = T::has_attribute("color");
        let mut sprites = self
            .sprites
            .iter()
            .filter_map(|sprite| {
                skeleton
                    .bone_with_index(&sprite.bone)
                    .map(|(bone, index)| (sprite, bone, index))
            })
            .collect::<Vec<_>>();
        sprites.sort_by(|(a, _, _), (b, _, _)| a.depth.partial_cmp(&b.depth).unwrap());
        let vertices_count = sprites.len() * 4;
        let triangles_count = sprites.len() * 2;
        let mut result = StaticVertexFactory::new(
            vertex_layout,
            vertices_count,
            triangles_count,
            MeshDrawMode::Triangles,
        );
        let mut positions = Vec::with_capacity(vertices_count);
        for (sprite, bone, _) in &sprites {
            let matrix = bone.bind_pose_matrix() * sprite.attachment_transform.local_matrix();
            let offset = sprite.pivot * sprite.size;
            positions.push(matrix.mul_point(vec3(-offset.x, -offset.y, 0.0)));
            positions.push(matrix.mul_point(vec3(sprite.size.x - offset.x, -offset.y, 0.0)));
            positions.push(matrix.mul_point(vec3(
                sprite.size.x - offset.x,
                sprite.size.y - offset.y,
                0.0,
            )));
            positions.push(matrix.mul_point(vec3(-offset.x, sprite.size.y - offset.y, 0.0)));
        }
        result.vertices_vec3f("position", &positions, None)?;
        let mut bone_indices = Vec::with_capacity(vertices_count);
        let mut bone_weights = Vec::with_capacity(vertices_count);
        for (sprite, _, index) in &sprites {
            let bones = sprite
                .bones_influence
                .iter()
                .filter_map(|(bone, range)| {
                    if *range <= 0.0 {
                        return None;
                    }
                    skeleton.bone_with_index(bone).map(|(bone, index)| {
                        let start = bone.local_matrix().mul_point(vec3(0.0, 0.0, 0.0));
                        let end = bone.local_matrix().mul_point(bone.target());
                        (index, start, end, *range)
                    })
                })
                .collect::<Vec<_>>();
            if bones.is_empty() {
                let index = *index as i32 & 0xFF;
                for _ in 0..4 {
                    bone_indices.push(index);
                    bone_weights.push(vec4(1.0, 0.0, 0.0, 0.0));
                }
            } else {
                let offset = index * 4;
                for shift in 0..4 {
                    let point = positions[offset + shift];
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
        }
        result.vertices_integer("boneIndices", &bone_indices, None)?;
        result.vertices_vec4f("boneWeights", &bone_weights, None)?;
        if T::has_attribute("normal") {
            let normals = std::iter::repeat(vec3(0.0, 0.0, 1.0))
                .take(vertices_count)
                .collect::<Vec<_>>();
            result.vertices_vec3f("normal", &normals, None)?;
        }
        if has_texture_coords {
            let mut texture_coords = Vec::with_capacity(vertices_count);
            for (sprite, _, _) in &sprites {
                texture_coords.push(vec3(sprite.uvs_rect.x, sprite.uvs_rect.y, 0.0));
                texture_coords.push(vec3(
                    sprite.uvs_rect.x + sprite.uvs_rect.w,
                    sprite.uvs_rect.y,
                    0.0,
                ));
                texture_coords.push(vec3(
                    sprite.uvs_rect.x + sprite.uvs_rect.w,
                    sprite.uvs_rect.y + sprite.uvs_rect.h,
                    0.0,
                ));
                texture_coords.push(vec3(
                    sprite.uvs_rect.x,
                    sprite.uvs_rect.y + sprite.uvs_rect.h,
                    0.0,
                ));
            }
            result.vertices_vec3f("textureCoord", &texture_coords, None)?;
        }
        if has_color {
            let mut colors = Vec::with_capacity(vertices_count);
            for (sprite, _, _) in &sprites {
                colors.push(sprite.color);
                colors.push(sprite.color);
                colors.push(sprite.color);
                colors.push(sprite.color);
            }
            result.vertices_vec4f("color", &colors, None)?;
        }
        let indices = (0..sprites.len())
            .flat_map(|i| {
                let i = i as u32 * 4;
                [(i, i + 1, i + 2), (i + 2, i + 3, i)]
            })
            .collect::<Vec<_>>();
        result.triangles(&indices, None)?;
        Ok(result)
    }
}

fn calculate_bone_distance(
    point: vek::Vec3<f32>,
    start: vek::Vec3<f32>,
    end: vek::Vec3<f32>,
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
