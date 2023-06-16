use crate::{
    components::{rig_animation_instance::*, rig_instance::*, transform::*},
    systems::rig_animation::*,
};
use core::ecs::{Comp, Universe, WorldRef};

pub type HaApplyRigAnimationSystemResources<'a> = (
    WorldRef,
    &'a HaRigAnimationSystemCache,
    Comp<&'a HaRigAnimationInstance>,
    Comp<&'a mut HaRigInstance>,
);

pub fn ha_apply_rig_animation(universe: &mut Universe) {
    let (world, cache, ..) = universe.query_resources::<HaApplyRigAnimationSystemResources>();

    for (_, (animation, rig)) in world
        .query::<(&HaRigAnimationInstance, &mut HaRigInstance)>()
        .iter()
    {
        if let (Some((asset, _)), Some(active)) =
            (cache.map.get(&animation.animation), &animation.active)
        {
            let total_weight = active
                .current_sequences
                .values()
                .chain(active.old_sequences.values())
                .fold(0.0, |a, v| a + v.weight());

            rig.skeleton
                .with_existing_bone_transforms(|name, transform| {
                    let result = HaTransform::interpolate_many(
                        active
                            .current_sequences
                            .iter()
                            .chain(active.old_sequences.iter())
                            .map(|(sequence, data)| {
                                let transform = asset
                                    .sequences
                                    .get(sequence)
                                    .map(|sequence| {
                                        sequence.sample_bone(name, data.time, transform)
                                    })
                                    .unwrap_or_else(|| transform.to_owned());
                                let weight = data.weight() / total_weight;
                                (transform, weight)
                            }),
                    );
                    if let Some(result) = result {
                        *transform = result;
                    }
                });
        }
    }
}
