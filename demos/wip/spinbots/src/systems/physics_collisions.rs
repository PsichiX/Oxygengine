use crate::{components::spinbot::SpinBot, utils::physics::Arena};
use oxygengine::prelude::*;
use std::collections::{HashMap, HashSet};

pub type PhysicsCollisionsSystemResources<'a> = (
    WorldMut,
    &'a Arena,
    Comp<&'a mut CompositeTransform>,
    Comp<&'a mut SpinBot>,
);

pub fn physics_collisions_system(universe: &mut Universe) {
    // let (mut world, arena, ..) = universe.query_resources::<PhysicsCollisionsSystemResources>();
    //
    // let mut meta = world
    //     .query_mut::<(&mut SpinBot, &mut CompositeTransform)>()
    //     .into_iter()
    //     .collect::<HashMap<_, _>>();
    //
    // let pairs = meta
    //     .iter()
    //     .flat_map(|(entity_a, (spinbot_a, transform_a))| {
    //         meta.iter()
    //             .filter_map(move |(entity_b, (spinbot_b, transform_b))| {
    //                 if entity_a == entity_b {
    //                     return None;
    //                 }
    //                 let position_a = transform_a.get_translation();
    //                 let position_b = transform_b.get_translation();
    //                 let radius_a = spinbot_a.config.radius();
    //                 let radius_b = spinbot_b.config.radius();
    //                 let radius_sqr = radius_a * radius_a + radius_b * radius_b;
    //                 let distance_sqr = (position_b - position_a).sqr_magnitude();
    //                 if distance_sqr > radius_sqr {
    //                     return None;
    //                 }
    //                 CollisionPair::new(Collider::SpinBot(*entity_a), Collider::SpinBot(*entity_b))
    //             })
    //     })
    //     .chain(meta.iter().filter_map(|(entity, (spinbot, transform))| {
    //         let radius = spinbot.config.radius();
    //         let position = transform.get_translation();
    //         if arena.does_object_collide(radius, position) {
    //             return CollisionPair::new(Collider::SpinBot(*entity), Collider::Arena);
    //         }
    //         None
    //     }))
    //     .collect::<HashSet<_>>();
    //
    // for CollisionPair(a, b) in pairs {
    //     match (a, b) {
    //         (Collider::SpinBot(entity_a), Collider::SpinBot(entity_b)) => {
    //             let hit_a = {
    //                 let (spinbot_a, transform_a) = meta.get(&entity_a).unwrap();
    //                 build_hit_object_spinbot(spinbot_a, transform_a.get_translation())
    //             };
    //             let hit_b = {
    //                 let (spinbot_b, transform_b) = meta.get(&entity_b).unwrap();
    //                 build_hit_object_spinbot(spinbot_b, transform_b.get_translation())
    //             };
    //             let (velocity_a, velocity_b) = hit_a.solve(&hit_b);
    //             meta.get_mut(&entity_a).unwrap().0.velocity = velocity_a;
    //             meta.get_mut(&entity_b).unwrap().0.velocity = velocity_b;
    //         }
    //         (Collider::SpinBot(entity), Collider::Arena) => {
    //             let (spinbot, transform) = meta.get_mut(&entity).unwrap();
    //             let radius = spinbot.config.radius();
    //             let position = transform.get_translation();
    //             if let Some(hit_point) = arena.walls_contact_point(radius, position) {
    //                 let hit_a = build_hit_object_spinbot(spinbot, position);
    //                 let hit_b = build_hit_object_arena(&arena, hit_point);
    //                 let (velocity, _) = hit_a.solve(&hit_b);
    //                 spinbot.velocity = velocity;
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}
