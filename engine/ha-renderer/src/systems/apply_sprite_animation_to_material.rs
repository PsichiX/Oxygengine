use crate::components::{sprite_animation_instance::*, virtual_image_uniforms::*};
use core::ecs::{Comp, Universe, WorldRef};

pub type HaApplySpriteAnimationToMaterialSystemResources<'a> = (
    WorldRef,
    Comp<&'a HaSpriteAnimationInstance>,
    Comp<&'a mut HaVirtualImageUniforms>,
);

pub fn ha_apply_sprite_animation_to_material(universe: &mut Universe) {
    let (world, ..) = universe.query_resources::<HaApplySpriteAnimationToMaterialSystemResources>();

    for (_, (sprite, uniforms)) in world
        .query::<(&HaSpriteAnimationInstance, &mut HaVirtualImageUniforms)>()
        .iter()
    {
        if let (true, Some(name)) = (sprite.frame_lately_changed(), sprite.active_frame()) {
            uniforms.set("mainImage", name);
        }
    }
}
