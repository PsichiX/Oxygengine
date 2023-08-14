use oxygengine_ha_renderer::prelude::*;

pub fn default_prototype_sprite_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
            [fragment] uniform mainImageOffset: vec2;
            [fragment] uniform mainImageSize: vec2;
            [fragment] uniform mainImageTiling: vec2;
            [fragment] uniform tint: vec4;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [color = (virtualTexture2d,
            sampler: mainImage,
            coord: (fract_vec2, v: (mul_vec2, a: coord, b: mainImageTiling)),
            offset: mainImageOffset,
            size: mainImageSize
        )]
        [color := (mul_vec4, a: color, b: tint)]
        [(mul_vec4, a: color, b: [TintColor => vColor]) -> BaseColor]
    }
}
