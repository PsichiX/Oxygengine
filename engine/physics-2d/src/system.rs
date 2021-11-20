use crate::{
    component::{Collider2d, Collider2dBody, Collider2dInner, RigidBody2d, RigidBody2dInner},
    resource::Physics2dWorld,
};
use core::{
    app::AppLifeCycle,
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
};
use nphysics2d::object::{BodyPartHandle, DefaultBodyHandle, DefaultColliderHandle};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Physics2dSystemCache {
    cached_bodies: HashMap<Entity, DefaultBodyHandle>,
    cached_colliders: HashMap<Entity, DefaultColliderHandle>,
}

pub type Physics2dSystemResources<'a> = (
    WorldRef,
    &'a EntityChanges,
    &'a AppLifeCycle,
    &'a mut Physics2dWorld,
    &'a mut Physics2dSystemCache,
    Comp<&'a mut RigidBody2d>,
    Comp<&'a mut Collider2d>,
    Comp<&'a Collider2dBody>,
);

pub fn physics_2d_system(universe: &mut Universe) {
    let (world, changes, lifecycle, mut physics, mut cache, ..) =
        universe.query_resources::<Physics2dSystemResources>();

    for entity in changes.despawned() {
        if let Some(handle) = cache.cached_bodies.remove(&entity) {
            physics.destroy_body(handle);
        }
        if let Some(handle) = cache.cached_colliders.remove(&entity) {
            physics.destroy_collider(handle);
        }
    }

    for (entity, body) in world.query::<&mut RigidBody2d>().iter() {
        if !body.is_created() {
            let b = body.take_description().unwrap().build();
            let h = physics.insert_body(b);
            body.0 = RigidBody2dInner::Handle(h);
            cache.cached_bodies.insert(entity, h);
        }
    }

    for (entity, (collider, collider_body)) in
        world.query::<(&mut Collider2d, &Collider2dBody)>().iter()
    {
        if !collider.is_created() {
            let other = match collider_body {
                Collider2dBody::Me => entity,
                Collider2dBody::Entity(other) => *other,
            };
            if let Ok(body) = unsafe { world.get_unchecked::<RigidBody2d>(other) } {
                if let Some(h) = body.handle() {
                    let c = collider
                        .take_description()
                        .unwrap()
                        .build(BodyPartHandle(h, 0));
                    let h = physics.insert_collider(c, entity);
                    collider.0 = Collider2dInner::Handle(h);
                    cache.cached_colliders.insert(entity, h);
                }
            }
        }
    }

    physics.process(lifecycle.delta_time_seconds());
}
