#![allow(clippy::type_complexity)]

use crate::{
    component::{Collider2d, Collider2dBody, Collider2dInner, RigidBody2d, RigidBody2dInner},
    resource::Physics2dWorld,
};
use core::{
    app::AppLifeCycle,
    ecs::{
        storage::ComponentEvent, Entities, Entity, Join, ReadExpect, ReadStorage, ReaderId,
        Resources, System, Write, WriteStorage,
    },
};
use nphysics2d::object::{BodyPartHandle, DefaultBodyHandle, DefaultColliderHandle};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Physics2dSystem {
    cached_bodies: HashMap<Entity, DefaultBodyHandle>,
    cached_colliders: HashMap<Entity, DefaultColliderHandle>,
    bodies_reader_id: Option<ReaderId<ComponentEvent>>,
    colliders_reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'s> System<'s> for Physics2dSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, AppLifeCycle>,
        Option<Write<'s, Physics2dWorld>>,
        WriteStorage<'s, RigidBody2d>,
        WriteStorage<'s, Collider2d>,
        ReadStorage<'s, Collider2dBody>,
    );

    fn setup(&mut self, res: &mut Resources) {
        use core::ecs::SystemData;
        Self::SystemData::setup(res);
        self.bodies_reader_id = Some(WriteStorage::<RigidBody2d>::fetch(&res).register_reader());
        self.colliders_reader_id = Some(WriteStorage::<Collider2d>::fetch(&res).register_reader());
    }

    fn run(
        &mut self,
        (entities, lifecycle, world, mut bodies, mut colliders, colliders_body): Self::SystemData,
    ) {
        if world.is_none() {
            return;
        }

        let world: &mut Physics2dWorld = &mut world.unwrap();

        let events = bodies
            .channel()
            .read(self.bodies_reader_id.as_mut().unwrap());
        for event in events {
            if let ComponentEvent::Removed(index) = event {
                let found = self.cached_bodies.iter().find_map(|(entity, handle)| {
                    if entity.id() == *index {
                        Some((*entity, *handle))
                    } else {
                        None
                    }
                });
                if let Some((entity, handle)) = found {
                    self.cached_bodies.remove(&entity);
                    world.destroy_body(handle);
                }
            }
        }

        let events = colliders
            .channel()
            .read(self.colliders_reader_id.as_mut().unwrap());
        for event in events {
            if let ComponentEvent::Removed(index) = event {
                let found = self.cached_colliders.iter().find_map(|(entity, handle)| {
                    if entity.id() == *index {
                        Some((*entity, *handle))
                    } else {
                        None
                    }
                });
                if let Some((entity, handle)) = found {
                    self.cached_colliders.remove(&entity);
                    world.destroy_collider(handle);
                }
            }
        }

        for (entity, body) in (&entities, &mut bodies).join() {
            if !body.is_created() {
                let b = body.take_description().unwrap().build();
                let h = world.insert_body(b);
                body.0 = RigidBody2dInner::Handle(h);
                self.cached_bodies.insert(entity, h);
            }
        }
        for (entity, collider, collider_body) in (&entities, &mut colliders, &colliders_body).join()
        {
            if !collider.is_created() {
                let e = match collider_body {
                    Collider2dBody::Me => entity,
                    Collider2dBody::Entity(e) => *e,
                };
                if let Some(body) = bodies.get(e) {
                    if let Some(h) = body.handle() {
                        let c = collider
                            .take_description()
                            .unwrap()
                            .build(BodyPartHandle(h, 0));
                        let h = world.insert_collider(c, entity);
                        collider.0 = Collider2dInner::Handle(h);
                        self.cached_colliders.insert(entity, h);
                    }
                }
            }
        }

        world.process(lifecycle.delta_time_seconds());
    }
}
