use crate::{
    components::{skeletal_animation_instance::*, skeleton_instance::*, transform::*},
    systems::skeletal_animation::*,
};
use core::ecs::{Comp, Universe, WorldRef};

pub type HaApplySkeletalAnimationSystemResources<'a> = (
    WorldRef,
    &'a HaSkeletalAnimationSystemCache,
    Comp<&'a HaSkeletalAnimationInstance>,
    Comp<&'a mut HaSkeletonInstance>,
);

pub fn ha_apply_skeletal_animation(universe: &mut Universe) {
    let (world, cache, ..) = universe.query_resources::<HaApplySkeletalAnimationSystemResources>();

    for (_, (animation, skeleton)) in world
        .query::<(&HaSkeletalAnimationInstance, &mut HaSkeletonInstance)>()
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

            skeleton.with_existing_bone_transforms(|name, transform| {
                let result = HaTransform::interpolate_many(
                    active
                        .current_sequences
                        .iter()
                        .chain(active.old_sequences.iter())
                        .map(|(sequence, data)| {
                            let transform = asset
                                .sequences
                                .get(sequence)
                                .map(|sequence| sequence.sample_bone(name, data.time, transform))
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
