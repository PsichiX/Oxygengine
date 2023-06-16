pub mod circle;
pub mod grid;
pub mod immediate;
pub mod quad;
pub mod rig2d;
pub mod text;
pub mod tilemap;
pub mod triangles2d;

use crate::{
    compound_vertex_type, material::graph::MaterialGraph, material_graph, math::*,
    mesh::vertex_factory::VertexType, vertex_type,
};
use serde::{Deserialize, Serialize};

pub fn default_surface_flat_material_graph() -> MaterialGraph {
    Default::default()
}

pub fn default_surface_flat_color_material_graph() -> MaterialGraph {
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

pub fn default_surface_flat_texture_2d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [color = (texture2d, sampler: mainImage, coord: coord)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_texture_2d_array_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [color = (texture2dArray, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_texture_3d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler3D;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [color = (texture3d, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_sdf_texture_2d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [sdf = (texture2d, sampler: mainImage, coord: coord)]
        [distance = (maskX_vec4, v: sdf)]
        // [density = (maskY_vec4, v: sdf)]
        // [sharpness = (maskZ_vec4, v: sdf)]
        // [alpha = (maskW_vec4, v: sdf)]
        [value = (roundEven_float, v: distance)]
        [color = (make_vec4, x: {1.0}, y: {1.0}, z: {1.0}, w: value)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_sdf_texture_2d_array_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [sdf = (texture2dArray, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [distance = (maskX_vec4, v: sdf)]
        // [density = (maskY_vec4, v: sdf)]
        // [sharpness = (maskZ_vec4, v: sdf)]
        // [alpha = (maskW_vec4, v: sdf)]
        [value = (roundEven_float, v: distance)]
        [color = (make_vec4, x: {1.0}, y: {1.0}, z: {1.0}, w: value)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_sdf_texture_3d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [sdf = (texture3d, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [distance = (maskX_vec4, v: sdf)]
        // [density = (maskY_vec4, v: sdf)]
        // [sharpness = (maskZ_vec4, v: sdf)]
        // [alpha = (maskW_vec4, v: sdf)]
        [value = (roundEven_float, v: distance)]
        [color = (make_vec4, x: {1.0}, y: {1.0}, z: {1.0}, w: value)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_virtual_uniform_texture_2d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
            [fragment] uniform mainImageOffset: vec2;
            [fragment] uniform mainImageSize: vec2;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [color = (virtualTexture2d,
            sampler: mainImage,
            coord: coord,
            offset: mainImageOffset,
            size: mainImageSize
        )]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_virtual_uniform_texture_2d_array_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2DArray;
            [fragment] uniform mainImageOffset: vec3;
            [fragment] uniform mainImageSize: vec3;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [color = (virtualTexture2dArray,
            sampler: mainImage,
            coord: [TextureCoord => vTexCoord],
            offset: mainImageOffset,
            size: mainImageSize
        )]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_virtual_uniform_texture_3d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler3D;
            [fragment] uniform mainImageOffset: vec3;
            [fragment] uniform mainImageSize: vec3;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [color = (virtualTexture3d,
            sampler: mainImage,
            coord: [TextureCoord => vTexCoord],
            offset: mainImageOffset,
            size: mainImageSize
        )]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_text_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [col = (texture2dArray, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [value = (maskW_vec4, v: col)]
        [color = (make_vec4, x: {1.0}, y: {1.0}, z: {1.0}, w: value)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_sdf_text_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // (red, green, blue, thickness)
            // [vertex] in outline: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [sdf = (texture2dArray, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [distance = (maskX_vec4, v: sdf)]
        // [density = (maskY_vec4, v: sdf)]
        // [sharpness = (maskZ_vec4, v: sdf)]
        // [alpha = (maskW_vec4, v: sdf)]
        [value = (roundEven_float, v: distance)]
        [color = (make_vec4, x: {1.0}, y: {1.0}, z: {1.0}, w: value)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn surface_flat_domain_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout WorldPositionOffset: vec3 = {vec3(0.0, 0.0, 0.0)};
            [fragment] inout BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
            [fragment] inout ScreenDepthOffset: float = {0.0};
            [fragment] inout VisibilityMask: bool = {true};

            [vertex] uniform model: mat4;
            [vertex] uniform view: mat4;
            [vertex] uniform projection: mat4;
            [vertex] uniform time: vec4;

            [vertex] in position: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] in normal: vec3 = {vec3(0.0, 0.0, 1.0)};
            [vertex] in textureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] in color: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
        }

        outputs {
            [vertex] inout Model: mat4;
            [vertex] inout View: mat4;
            [vertex] inout Projection: mat4;
            [vertex] inout ViewProjection: mat4;
            [vertex] inout ModelViewProjection: mat4;
            [vertex] inout LocalPosition: vec3;
            [vertex] inout WorldPosition: vec3;
            [vertex] inout ScreenPosition: vec3;
            [vertex] inout LocalNormal: vec3;
            [vertex] inout WorldNormal: vec3;
            [vertex] inout ScreenNormal: vec3;
            [vertex] inout TextureCoord: vec3;
            [vertex] inout TintColor: vec4;
            [vertex] inout Time: float;
            [vertex] inout DeltaTime: float;
            [vertex] inout TimeFraction: float;

            [vertex] builtin gl_Position: vec4;
            [fragment] builtin gl_FragDepth: float;
            [fragment] out finalColor: vec4;
        }

        [discarded = (discard_test, condition: (negate, v: VisibilityMask))]
        [local_position = position]
        [model_dir = (cast_mat4_mat3, v: model)]
        [view_projection = (mul_mat4, a: projection, b: view)]
        [model_view_projection = (mul_mat4, a: view_projection, b: model)]
        [model_view_projection_dir = (cast_mat4_mat3, v: model_view_projection)]
        [pos = (append_vec4, a: local_position, b: {1.0})]
        [world_position = (truncate_vec4, v: (mul_mat4_vec4, a: model, b: pos))]
        [world_position := (add_vec3, a: world_position, b: WorldPositionOffset)]
        [pos := (append_vec4, a: world_position, b: {1.0})]
        [screen_position = (truncate_vec4, v: (mul_mat4_vec4, a: view_projection, b: pos))]
        [world_normal = (mul_mat3_vec3, a: model_dir, b: normal)]
        [screen_normal = (mul_mat3_vec3, a: model_view_projection_dir, b: normal)]

        [model -> Model]
        [view -> View]
        [projection -> Projection]
        [view_projection -> ViewProjection]
        [model_view_projection -> ModelViewProjection]
        [local_position -> LocalPosition]
        [world_position -> WorldPosition]
        [screen_position -> ScreenPosition]
        [normal -> LocalNormal]
        [world_normal -> WorldNormal]
        [screen_normal -> ScreenNormal]
        [textureCoord -> TextureCoord]
        [color -> TintColor]
        [(maskX_vec4, v: time) -> Time]
        [(maskY_vec4, v: time) -> DeltaTime]
        [(maskZ_vec4, v: time) -> TimeFraction]
        [(append_vec4, a: screen_position, b: {1.0}) -> gl_Position]
        [(if_vec4,
            condition: discarded,
            truthy: {vec4(0.0, 0.0, 0.0, 0.0)},
            falsy: BaseColor
        ) -> finalColor]
        [(add_float, a: gl_FragDepth, b: ScreenDepthOffset) -> gl_FragDepth]
    }
}

fn default_position() -> vek::Vec3<f32> {
    vec3(0.0, 0.0, 0.0)
}

fn default_normal() -> vek::Vec3<f32> {
    vec3(0.0, 0.0, 1.0)
}

fn default_texture_coord() -> vek::Vec3<f32> {
    vec3(0.0, 0.0, 0.0)
}

fn default_color() -> vek::Vec4<f32> {
    vec4(1.0, 1.0, 1.0, 1.0)
}

fn default_outline() -> vek::Vec4<f32> {
    vec4(0.0, 0.0, 0.0, 0.0)
}

fn default_thickness() -> f32 {
    0.0
}

fn default_animation_column() -> f32 {
    0.0
}

fn default_curves_index() -> i32 {
    0
}

fn default_bone_indices() -> i32 {
    0
}

fn default_bone_weights() -> vek::Vec4<f32> {
    vec4(0.0, 0.0, 0.0, 0.0)
}

pub trait SurfaceDomain: VertexType {}
pub trait SurfaceColoredDomain: SurfaceDomain {}
pub trait SurfaceTexturedDomain: SurfaceDomain {}
pub trait SurfaceVertAnimDomain: SurfaceDomain {}
pub trait SurfaceDeformerDomain: SurfaceDomain {}
pub trait SurfaceSkinnedDomain: SurfaceDomain {}
pub trait SurfaceTextDomain: SurfaceColoredDomain + SurfaceTexturedDomain {}
pub trait SurfaceCompleteDomain: SurfaceColoredDomain + SurfaceTexturedDomain {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @middlewares(vertanim)
    pub struct SurfaceVertAnimFragment {
        #[serde(default = "default_animation_column")]
        pub animation_column: float = animationColumn(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @middlewares(deformer)
    pub struct SurfaceDeformerFragment {
        #[serde(default = "default_curves_index")]
        pub curves_index: int = curvesIndex(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @middlewares(skinning)
    pub struct SurfaceSkinningFragment {
        #[serde(default = "default_bone_indices")]
        pub bone_indices: int = boneIndices(0),
        #[serde(default = "default_bone_weights")]
        pub bone_weights: vec4 = boneWeights(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain)
    pub struct SurfaceVertexP {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain)
    pub struct SurfaceVertexPN {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexPT {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexPNT {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexPC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexPNC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexPTC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexPNTC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain)
    pub struct SurfaceVertexAP {
        #[serde(default)]
        pub vertex: SurfaceVertexP,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain)
    pub struct SurfaceVertexAPN {
        #[serde(default)]
        pub vertex: SurfaceVertexPN,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexAPT {
        #[serde(default)]
        pub vertex: SurfaceVertexPT,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexAPNT {
        #[serde(default)]
        pub vertex: SurfaceVertexPNT,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexAPC {
        #[serde(default)]
        pub vertex: SurfaceVertexPC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexAPNC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexAPTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPTC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexAPNTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNTC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain)
    pub struct SurfaceVertexSP {
        #[serde(default)]
        pub vertex: SurfaceVertexP,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain)
    pub struct SurfaceVertexSPN {
        #[serde(default)]
        pub vertex: SurfaceVertexPN,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexSPT {
        #[serde(default)]
        pub vertex: SurfaceVertexPT,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexSPNT {
        #[serde(default)]
        pub vertex: SurfaceVertexPNT,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexSPC {
        #[serde(default)]
        pub vertex: SurfaceVertexPC,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexSPNC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNC,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexSPTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPTC,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexSPNTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNTC,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain)
    pub struct SurfaceVertexASP {
        #[serde(default)]
        pub vertex: SurfaceVertexP,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain)
    pub struct SurfaceVertexASPN {
        #[serde(default)]
        pub vertex: SurfaceVertexPN,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexASPT {
        #[serde(default)]
        pub vertex: SurfaceVertexPT,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexASPNT {
        #[serde(default)]
        pub vertex: SurfaceVertexPNT,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexASPC {
        #[serde(default)]
        pub vertex: SurfaceVertexPC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexASPNC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexASPTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPTC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceVertAnimDomain, SurfaceSkinnedDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexASPNTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNTC,
        #[serde(default)]
        pub vert_anim: SurfaceVertAnimFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain)
    pub struct SurfaceVertexDP {
        #[serde(default)]
        pub vertex: SurfaceVertexP,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain)
    pub struct SurfaceVertexDPN {
        #[serde(default)]
        pub vertex: SurfaceVertexPN,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexDPT {
        #[serde(default)]
        pub vertex: SurfaceVertexPT,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexDPNT {
        #[serde(default)]
        pub vertex: SurfaceVertexPNT,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexDPC {
        #[serde(default)]
        pub vertex: SurfaceVertexPC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexDPNC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexDPTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPTC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexDPNTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNTC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain)
    pub struct SurfaceVertexDSP {
        #[serde(default)]
        pub vertex: SurfaceVertexP,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain)
    pub struct SurfaceVertexDSPN {
        #[serde(default)]
        pub vertex: SurfaceVertexPN,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexDSPT {
        #[serde(default)]
        pub vertex: SurfaceVertexPT,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexDSPNT {
        #[serde(default)]
        pub vertex: SurfaceVertexPNT,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexDSPC {
        #[serde(default)]
        pub vertex: SurfaceVertexPC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain, SurfaceColoredDomain)
    pub struct SurfaceVertexDSPNC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexDSPTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPTC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceDeformerDomain, SurfaceSkinnedDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexDSPNTC {
        #[serde(default)]
        pub vertex: SurfaceVertexPNTC,
        #[serde(default)]
        pub deformer: SurfaceDeformerFragment,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceColoredDomain, SurfaceTexturedDomain, SurfaceTextDomain, SurfaceCompleteDomain)
    pub struct SurfaceVertexText {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
        /// (red, green, blue, outline thickness)
        #[serde(default = "default_outline")]
        pub outline: vec4 = outline(0),
        #[serde(default = "default_thickness")]
        pub thickness: float = thickness(0),
    }
}
