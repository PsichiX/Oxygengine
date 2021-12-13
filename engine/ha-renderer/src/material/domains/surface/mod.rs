pub mod circle;
pub mod grid;
pub mod immediate;
pub mod quad;
pub mod text;
pub mod tilemap;
pub mod triangles2d;

use crate::{
    material::graph::MaterialGraph, material_graph, math::*, mesh::vertex_factory::VertexType,
    vertex_type,
};
use serde::{Deserialize, Serialize};

pub fn default_surface_flat_color_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
        }

        outputs {
            [fragment] domain BaseColor: vec4;
        }

        [[TintColor => vColor] -> BaseColor]
    }
}

pub fn default_surface_flat_texture_2d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [color = (texture2d, sampler: mainImage, coord: coord)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_texture_2d_array_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
        }

        [color = (texture2dArray, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_texture_3d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler3D;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
        }

        [color = (texture3d, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}

pub fn default_surface_flat_sdf_texture_2d_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
            [fragment] uniform mainImageOffset: vec2;
            [fragment] uniform mainImageSize: vec2;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2DArray;
            [fragment] uniform mainImageOffset: vec3;
            [fragment] uniform mainImageSize: vec3;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler3D;
            [fragment] uniform mainImageOffset: vec3;
            [fragment] uniform mainImageSize: vec3;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            // (red, green, blue, thickness)
            // [vertex] in outline: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
            // [vertex] in thickness: float = {0.0};

            [fragment] uniform mainImage: sampler2DArray;
        }

        outputs {
            [fragment] domain BaseColor: vec4;
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
            [vertex] domain WorldPositionOffset: vec3 = {vec3(0.0, 0.0, 0.0)};
            [fragment] domain BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
            [fragment] domain ScreenDepthOffset: float = {0.0};

            [vertex] uniform model: mat4;
            [vertex] uniform view: mat4;
            [vertex] uniform projection: mat4;

            [vertex] in position: vec3 = vec3(0.0, 0.0, 0.0);
            [vertex] in normal: vec3 = vec3(0.0, 0.0, 1.0);
            [vertex] in textureCoord: vec3 = vec3(0.0, 0.0, 0.0);
            [vertex] in color: vec4 = vec4(1.0, 1.0, 1.0, 1.0);
        }

        outputs {
            [vertex] domain Model: mat4;
            [vertex] domain View: mat4;
            [vertex] domain Projection: mat4;
            [vertex] domain ViewProjection: mat4;
            [vertex] domain ModelViewProjection: mat4;
            [vertex] domain LocalPosition: vec3;
            [vertex] domain WorldPosition: vec3;
            [vertex] domain ScreenPosition: vec3;
            [vertex] domain LocalNormal: vec3;
            [vertex] domain WorldNormal: vec3;
            [vertex] domain ScreenNormal: vec3;
            [vertex] domain TextureCoord: vec3;
            [vertex] domain TintColor: vec4;

            [vertex] builtin gl_Position: vec4;
            [fragment] builtin gl_FragDepth: float;
            [fragment] out finalColor: vec4;
        }

        [model_dir = (cast_mat4_mat3, v: model)]
        [view_projection = (mul_mat4, a: projection, b: view)]
        [model_view_projection = (mul_mat4, a: view_projection, b: model)]
        [model_view_projection_dir = (cast_mat4_mat3, v: model_view_projection)]
        [pos = (append_vec4, a: position, b: {1.0})]
        [world_position = (truncate_vec4, v: (mul_mat4_vec4, a: model, b: pos))]
        [world_position = (add_vec3, a: world_position, b: WorldPositionOffset)]
        [pos = (append_vec4, a: world_position, b: {1.0})]
        [screen_position = (truncate_vec4, v: (mul_mat4_vec4, a: view_projection, b: pos))]
        [world_normal = (mul_mat3_vec3, a: model_dir, b: normal)]
        [screen_normal = (mul_mat3_vec3, a: model_view_projection_dir, b: normal)]

        [model -> Model]
        [view -> View]
        [projection -> Projection]
        [view_projection -> ViewProjection]
        [model_view_projection -> ModelViewProjection]
        [position -> LocalPosition]
        [world_position -> WorldPosition]
        [screen_position -> ScreenPosition]
        [normal -> LocalNormal]
        [world_normal -> WorldNormal]
        [screen_normal -> ScreenNormal]
        [textureCoord -> TextureCoord]
        [color -> TintColor]
        [(append_vec4, a: screen_position, b: {1.0}) -> gl_Position]
        [BaseColor -> finalColor]
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

pub trait SurfaceDomain: VertexType {}
pub trait SurfaceColoredDomain: SurfaceDomain {}
pub trait SurfaceTexturedDomain: SurfaceDomain {}
pub trait SurfaceTextDomain: SurfaceColoredDomain + SurfaceTexturedDomain {}
pub trait SurfaceCompleteDomain: SurfaceColoredDomain + SurfaceTexturedDomain {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexP {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
    }
}

impl SurfaceDomain for SurfaceVertexP {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexPN {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
    }
}

impl SurfaceDomain for SurfaceVertexPN {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexPT {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
    }
}

impl SurfaceDomain for SurfaceVertexPT {}
impl SurfaceTexturedDomain for SurfaceVertexPT {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexPNT {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
    }
}

impl SurfaceDomain for SurfaceVertexPNT {}
impl SurfaceTexturedDomain for SurfaceVertexPNT {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexPC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

impl SurfaceDomain for SurfaceVertexPC {}
impl SurfaceColoredDomain for SurfaceVertexPC {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexPNC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_normal")]
        pub normal: vec3 = normal(0, normalized),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

impl SurfaceDomain for SurfaceVertexPNC {}
impl SurfaceColoredDomain for SurfaceVertexPNC {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct SurfaceVertexPTC {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
        #[serde(default = "default_color")]
        pub color: vec4 = color(0),
    }
}

impl SurfaceDomain for SurfaceVertexPTC {}
impl SurfaceColoredDomain for SurfaceVertexPTC {}
impl SurfaceTexturedDomain for SurfaceVertexPTC {}
impl SurfaceCompleteDomain for SurfaceVertexPTC {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
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

impl SurfaceDomain for SurfaceVertexPNTC {}
impl SurfaceColoredDomain for SurfaceVertexPNTC {}
impl SurfaceTexturedDomain for SurfaceVertexPNTC {}
impl SurfaceCompleteDomain for SurfaceVertexPNTC {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
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

impl SurfaceDomain for SurfaceVertexText {}
impl SurfaceColoredDomain for SurfaceVertexText {}
impl SurfaceTexturedDomain for SurfaceVertexText {}
impl SurfaceTextDomain for SurfaceVertexText {}
impl SurfaceCompleteDomain for SurfaceVertexText {}
