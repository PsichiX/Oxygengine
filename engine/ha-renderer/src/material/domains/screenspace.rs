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
            [fragment] uniform color: vec4;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [color -> BaseColor]
    }
}

pub fn default_screenspace_texture_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec2 = {vec2(0.0, 0.0)};

            [fragment] uniform mainImage: sampler2D;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [(texture2d, sampler: mainImage, coord: [TextureCoord => vTexCoord]) -> BaseColor]
    }
}

pub fn screenspace_domain_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [fragment] inout BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
            [fragment] inout VisibilityMask: bool = {true};

            [vertex] in position: vec2 = {vec2(0.0, 0.0)};
        }

        outputs {
            [vertex] inout TextureCoord: vec2;

            [vertex] builtin gl_Position: vec4;
            [fragment] out finalColor: vec4;
        }

        [discarded = (discard_test, condition: (negate, v: VisibilityMask))]
        [position -> TextureCoord]
        [(make_vec4,
            x: (sub_float, a: (mul_float, a: (maskX_vec2, v: position), b: {2.0}), b: {1.0}),
            y: (sub_float, a: (mul_float, a: (maskY_vec2, v: position), b: {2.0}), b: {1.0}),
            z: {0.0},
            w: {1.0}
        ) -> gl_Position]
        [(if_vec4,
            condition: discarded,
            truthy: {vec4(0.0, 0.0, 0.0, 0.0)},
            falsy: BaseColor
        ) -> finalColor]
    }
}

fn default_position() -> vek::Vec2<f32> {
    vec2(0.0, 0.0)
}

pub trait ScreenSpaceDomain: VertexType {}

vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(ScreenSpaceDomain)
    pub struct ScreenSpaceVertex {
        #[serde(default = "default_position")]
        pub position: vec2 = position(0, bounds),
    }
}

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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_screenspace_materials() {
        MaterialLibrary::assert_material_compilation(
            &ScreenSpaceVertex::vertex_layout().unwrap(),
            RenderTargetDescriptor::Main,
            &screenspace_domain_graph(),
            &default_screenspace_color_material_graph(),
        );

        MaterialLibrary::assert_material_compilation(
            &ScreenSpaceVertex::vertex_layout().unwrap(),
            RenderTargetDescriptor::Main,
            &screenspace_domain_graph(),
            &default_screenspace_texture_material_graph(),
        );
    }
}
