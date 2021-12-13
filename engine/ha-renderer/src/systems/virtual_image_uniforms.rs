use crate::{
    components::{material_instance::HaMaterialInstance, virtual_image_uniforms::*},
    ha_renderer::HaRenderer,
    image::{ImageInstanceReference, ImageResourceMapping},
    material::common::MaterialValue,
    math::*,
};
use core::ecs::{Comp, Universe, WorldRef};

pub type HaVirtualImageUniformsSystemResources<'a> = (
    WorldRef,
    &'a HaRenderer,
    &'a ImageResourceMapping,
    Comp<&'a mut HaVirtualImageUniforms>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_virtual_image_uniforms(universe: &mut Universe) {
    let (world, renderer, image_mapping, ..) =
        universe.query_resources::<HaVirtualImageUniformsSystemResources>();

    for (_, (uniforms, material)) in world
        .query::<(&mut HaVirtualImageUniforms, &mut HaMaterialInstance)>()
        .iter()
    {
        if uniforms.dirty {
            let mut changed = 0;
            for (key, data) in uniforms.iter() {
                if let Some((owner, image)) =
                    image_mapping.virtual_resource_by_name(&data.virtual_asset_name)
                {
                    if let Some(virtual_image) = renderer.virtual_images.get(owner) {
                        if let Some((rect, _)) = virtual_image.image_uvs(image) {
                            material.values.insert(
                                key.to_owned(),
                                MaterialValue::Sampler2d {
                                    reference: ImageInstanceReference::VirtualId {
                                        owner,
                                        id: image,
                                    },
                                    filtering: data.filtering,
                                },
                            );
                            material.values.insert(
                                format!("{}Offset", key),
                                MaterialValue::Vec2F(vec2(rect.x, rect.y)),
                            );
                            material.values.insert(
                                format!("{}Size", key),
                                MaterialValue::Vec2F(vec2(rect.w, rect.h)),
                            );
                            changed += 1;
                        }
                    }
                }
            }
            if changed == uniforms.len() {
                uniforms.dirty = false;
            }
        }
    }
}
