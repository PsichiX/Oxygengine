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
            for (key, name) in uniforms.iter() {
                if let Some((owner, image)) = image_mapping.virtual_resource_by_name(name) {
                    if let Some(virtual_image) = renderer.virtual_images.get(owner) {
                        if let Some(rect) = virtual_image.image_uvs(image) {
                            material.values.insert(
                                key.to_owned(),
                                MaterialValue::Sampler2D(ImageInstanceReference::VirtualId {
                                    owner,
                                    id: image,
                                }),
                            );
                            material.values.insert(
                                format!("{}Region", key),
                                MaterialValue::Vec4F(vec4(rect.x, rect.y, rect.w, rect.h)),
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
