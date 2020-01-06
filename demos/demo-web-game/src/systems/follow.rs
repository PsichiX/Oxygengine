#![allow(clippy::type_complexity)]

use crate::components::follow::{Follow, FollowMode};
use oxygengine::prelude::*;

pub struct FollowSystem;

impl<'s> System<'s> for FollowSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, AppLifeCycle>,
        ReadStorage<'s, Follow>,
        WriteStorage<'s, CompositeTransform>,
    );

    fn run(&mut self, (entities, lifecycle, follows, mut transforms): Self::SystemData) {
        let dt = lifecycle.delta_time_seconds() as f32;
        let to_follow = (&entities, &follows, &transforms)
            .join()
            .filter_map(|(entity, follow, transform)| {
                if let Some(follow_entity) = follow.0 {
                    let pos = transform.get_translation();
                    if let Some(transform) = transforms.get(follow_entity) {
                        return Some((entity, follow.1, pos, transform.get_translation()));
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        for (entity, follow_mode, from, to) in to_follow {
            let transform = transforms.get_mut(entity).unwrap();
            match follow_mode {
                FollowMode::Instant => transform.set_translation(to),
                FollowMode::Delayed(f) => {
                    transform.set_translation(from.lerp(to, (f * dt).max(0.0).min(1.0)))
                }
            }
        }
    }
}
