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

pub fn default_screenspace_color_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain LocalPosition: vec2;

            [fragment] uniform color: vec4;
        }

        outputs {
            [vertex] domain ScreenPosition: vec2;

            [fragment] domain BaseColor: vec4;
        }

        [(mul_vec2, a: LocalPosition, b: {vec2(2.0, 2.0)}) -> ScreenPosition]
        [color -> BaseColor]
    }
}

pub fn default_screenspace_texture_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain LocalPosition: vec2;
            [vertex] domain TextureCoord: vec2 = {vec2(0.0, 0.0)};

            [fragment] uniform color: vec4;
            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [vertex] domain ScreenPosition: vec2;

            [fragment] domain BaseColor: vec4;
        }

        [(mul_vec2, a: LocalPosition, b: {vec2(2.0, 2.0)}) -> ScreenPosition]
        [col = (texture, sampler: mainImage, coord: [TextureCoord => vTexCoord])]
        [(mul_vec4, a: col, b: color) -> BaseColor]
    }
}

pub fn screenspace_domain_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] domain ScreenPosition: vec2;
            [fragment] domain BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [vertex] in position: vec3 = vec3(0.0, 0.0, 0.0);
            [vertex] in textureCoord: vec2 = vec2(0.0, 0.0);
        }

        outputs {
            [vertex] domain LocalPosition: vec2;
            [vertex] domain TextureCoord: vec2;

            [vertex] builtin gl_Position: vec4;
            [fragment] out finalColor: vec4;
        }

        [position -> LocalPosition]
        [textureCoord -> TextureCoord]
        [(make_vec4,
            x: (maskX_vec2, v: ScreenPosition),
            y: (maskY_vec2, v: ScreenPosition),
            z: {0.0},
            w: {1.0}
        ) -> gl_Position]
        [BaseColor -> finalColor]
    }
}

fn default_position() -> vek::Vec2<f32> {
    vec2(0.0, 0.0)
}

fn default_texture_coord() -> vek::Vec2<f32> {
    vec2(0.0, 0.0)
}

pub trait ScreenSpaceDomain: VertexType {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub struct ScreenSpaceVertex {
        #[serde(default = "default_position")]
        pub position: vec2 = position(0),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec2 = textureCoord(0),
    }
}

impl ScreenSpaceDomain for ScreenSpaceVertex {}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ScreenSpaceQuadFactory;

impl ScreenSpaceQuadFactory {
    pub fn factory(self) -> Result<StaticVertexFactory, MeshError> {
        let mut result = StaticVertexFactory::new(
            ScreenSpaceVertex::vertex_layout()?,
            4,
            2,
            MeshDrawMode::Triangles,
        );
        result.vertices_vec2f(
            "position",
            &[
                vec2(-1.0, -1.0),
                vec2(1.0, -1.0),
                vec2(1.0, 1.0),
                vec2(-1.0, 1.0),
            ],
            None,
        )?;
        result.vertices_vec2f(
            "textureCoord",
            &[
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                vec2(1.0, 1.0),
                vec2(0.0, 1.0),
            ],
            None,
        )?;
        result.triangles(&[(0, 1, 2), (2, 3, 0)], None)?;
        Ok(result)
    }
}
