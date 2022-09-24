use oxygengine::prelude::*;

pub fn avatar_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
            // alpha is used to lerp between original color and blink color.
            [fragment] uniform blinkColor: vec4;
        }

        outputs {
            [fragment] out BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [textureColor = (texture2d, sampler: mainImage, coord: coord)]
        [originalColor = (mul_vec4, a: textureColor, b: [TintColor => vColor])]
        [originalRgb = (truncate_vec4, v: originalColor)]
        [alpha = (maskW_vec4, v: originalColor)]
        [blinkRgb = (truncate_vec4, v: blinkColor)]
        [weight = (maskW_vec4, v: blinkColor)]
        [mergedColor = (mix_vec3, x: originalRgb, y: blinkRgb, alpha: (fill_vec3, v: weight))]
        [(append_vec4, a: mergedColor, b: alpha) -> BaseColor]
    }
}

pub fn virtual_uniform_avatar_material_graph() -> MaterialGraph {
    material_graph! {
        inputs {
            [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
            [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

            [fragment] uniform mainImage: sampler2D;
            [fragment] uniform mainImageOffset: vec2;
            [fragment] uniform mainImageSize: vec2;
            // alpha is used to lerp between original color and blink color.
            [fragment] uniform blinkColor: vec4;
        }

        outputs {
            [fragment] inout BaseColor: vec4;
        }

        [coord = (truncate_vec3, v: [TextureCoord => vTexCoord])]
        [textureColor = (virtualTexture2d,
            sampler: mainImage,
            coord: coord,
            offset: mainImageOffset,
            size: mainImageSize
        )]
        [originalColor = (mul_vec4, a: textureColor, b: [TintColor => vColor])]
        [originalRgb = (truncate_vec4, v: originalColor)]
        [alpha = (maskW_vec4, v: originalColor)]
        [blinkRgb = (truncate_vec4, v: blinkColor)]
        [weight = (maskW_vec4, v: blinkColor)]
        [mergedColor = (mix_vec3, x: originalRgb, y: blinkRgb, alpha: (fill_vec3, v: weight))]
        [(append_vec4, a: mergedColor, b: alpha) -> BaseColor]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_material() {
        MaterialLibrary::assert_material_compilation(
            &SurfaceVertexPT::vertex_layout().unwrap(),
            RenderTargetDescriptor::Main,
            &surface_flat_domain_graph(),
            &avatar_material_graph(),
        );
    }
}
