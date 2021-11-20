use core::{ecs::Entity, Scalar};
use ncollide2d::{pipeline::narrow_phase::ContactEvent, query::Proximity};
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    math::*,
    object::{
        Collider, DefaultBodyHandle, DefaultBodySet, DefaultColliderHandle, DefaultColliderSet,
        RigidBody,
    },
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone)]
pub enum Physics2dWorldSimulationMode {
    FixedTimestepMaxIterations(usize),
    DynamicTimestep,
}

#[derive(Debug, Clone)]
pub enum Physics2dContact {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}

#[derive(Debug, Clone)]
pub enum Physics2dProximity {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}

pub struct Physics2dWorld {
    geometrical_world: DefaultGeometricalWorld<Scalar>,
    mechanical_world: DefaultMechanicalWorld<Scalar>,
    body_set: DefaultBodySet<Scalar>,
    collider_set: DefaultColliderSet<Scalar>,
    constraint_set: DefaultJointConstraintSet<Scalar>,
    force_generator_set: DefaultForceGeneratorSet<Scalar>,
    remaining_time_step: Scalar,
    simulation_mode: Physics2dWorldSimulationMode,
    collider_map: HashMap<DefaultColliderHandle, Entity>,
    last_contacts: Vec<Physics2dContact>,
    last_proximities: Vec<Physics2dProximity>,
    active_contacts: HashSet<(Entity, Entity)>,
    active_proximities: HashSet<(Entity, Entity)>,
    paused: bool,
}

impl Default for Physics2dWorld {
    fn default() -> Self {
        Self {
            geometrical_world: DefaultGeometricalWorld::new(),
            mechanical_world: DefaultMechanicalWorld::new(Vector::y() * 9.81),
            body_set: DefaultBodySet::new(),
            collider_set: DefaultColliderSet::new(),
            constraint_set: DefaultJointConstraintSet::new(),
            force_generator_set: DefaultForceGeneratorSet::new(),
            remaining_time_step: 0.0,
            simulation_mode: Physics2dWorldSimulationMode::FixedTimestepMaxIterations(3),
            collider_map: Default::default(),
            last_contacts: vec![],
            last_proximities: vec![],
            active_contacts: Default::default(),
            active_proximities: Default::default(),
            paused: false,
        }
    }
}

impl Physics2dWorld {
    pub fn new(gravity: Vector<Scalar>, mode: Physics2dWorldSimulationMode) -> Self {
        let mut result = Self::default();
        result.set_gravity(gravity);
        result.set_simulation_mode(mode);
        result
    }

    pub fn geometrical_world(&self) -> &DefaultGeometricalWorld<Scalar> {
        &self.geometrical_world
    }

    pub fn mechanical_world(&self) -> &DefaultMechanicalWorld<Scalar> {
        &self.mechanical_world
    }

    pub fn gravity(&self) -> Vector<Scalar> {
        self.mechanical_world.gravity
    }

    pub fn set_gravity(&mut self, value: Vector<Scalar>) {
        self.mechanical_world.gravity = value;
    }

    pub fn simulation_mode(&self) -> Physics2dWorldSimulationMode {
        self.simulation_mode
    }

    pub fn set_simulation_mode(&mut self, simulation_mode: Physics2dWorldSimulationMode) {
        self.simulation_mode = simulation_mode;
    }

    pub fn time_step(&self) -> Scalar {
        self.mechanical_world.timestep()
    }

    pub fn set_time_step(&mut self, value: Scalar) {
        self.mechanical_world.set_timestep(value);
        self.remaining_time_step = 0.0;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn set_paused(&mut self, value: bool) {
        self.paused = value;
    }

    pub fn reset_timestep_accumulator(&mut self) {
        self.remaining_time_step = 0.0;
    }

    pub(crate) fn insert_body(&mut self, body: RigidBody<Scalar>) -> DefaultBodyHandle {
        self.body_set.insert(body)
    }

    pub(crate) fn destroy_body(&mut self, handle: DefaultBodyHandle) {
        self.body_set.remove(handle);
    }

    pub fn body(&self, handle: DefaultBodyHandle) -> Option<&RigidBody<Scalar>> {
        self.body_set.rigid_body(handle)
    }

    pub fn body_mut(&mut self, handle: DefaultBodyHandle) -> Option<&mut RigidBody<Scalar>> {
        self.body_set.rigid_body_mut(handle)
    }

    pub fn last_contacts(&self) -> impl Iterator<Item = &Physics2dContact> {
        self.last_contacts.iter()
    }

    pub fn last_proximities(&self) -> impl Iterator<Item = &Physics2dProximity> {
        self.last_proximities.iter()
    }

    pub fn active_contacts(&self) -> impl Iterator<Item = &(Entity, Entity)> {
        self.active_contacts.iter()
    }

    pub fn active_proximities(&self) -> impl Iterator<Item = &(Entity, Entity)> {
        self.active_proximities.iter()
    }

    pub(crate) fn insert_collider(
        &mut self,
        collider: Collider<Scalar, DefaultBodyHandle>,
        entity: Entity,
    ) -> DefaultColliderHandle {
        let handle = self.collider_set.insert(collider);
        self.collider_map.insert(handle, entity);
        handle
    }

    pub(crate) fn destroy_collider(&mut self, handle: DefaultColliderHandle) {
        self.collider_set.remove(handle);
    }

    pub fn collider(
        &self,
        handle: DefaultColliderHandle,
    ) -> Option<&Collider<Scalar, DefaultBodyHandle>> {
        self.collider_set.get(handle)
    }

    pub fn collider_mut(
        &mut self,
        handle: DefaultColliderHandle,
    ) -> Option<&mut Collider<Scalar, DefaultBodyHandle>> {
        self.collider_set.get_mut(handle)
    }

    pub fn are_in_contact(&self, first: Entity, second: Entity) -> bool {
        self.active_contacts.contains(&(first, second))
            || self.active_contacts.contains(&(second, first))
    }

    pub fn are_in_proximity(&self, first: Entity, second: Entity) -> bool {
        self.active_proximities.contains(&(first, second))
            || self.active_proximities.contains(&(second, first))
    }

    pub fn process(&mut self, mut delta_time: Scalar) {
        if self.paused {
            return;
        }
        self.last_contacts.clear();
        self.last_proximities.clear();
        self.active_contacts.clear();
        self.active_proximities.clear();
        match self.simulation_mode {
            Physics2dWorldSimulationMode::FixedTimestepMaxIterations(iterations) => {
                let time_step = self.mechanical_world.timestep();
                delta_time += self.remaining_time_step;
                let mut i = 0;
                while delta_time >= time_step && i < iterations {
                    self.step();
                    delta_time -= time_step;
                    i += 1;
                }
                self.remaining_time_step = delta_time % time_step;
            }
            Physics2dWorldSimulationMode::DynamicTimestep => {
                self.mechanical_world.set_timestep(delta_time);
                self.step();
                self.remaining_time_step = 0.0;
            }
        }
    }

    fn step(&mut self) {
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.body_set,
            &mut self.collider_set,
            &mut self.constraint_set,
            &mut self.force_generator_set,
        );
        for contact in self.geometrical_world.contact_events() {
            match contact {
                ContactEvent::Started(a, b) => {
                    if let Some(a) = self.collider_map.get(a) {
                        if let Some(b) = self.collider_map.get(b) {
                            // TODO: test if test for effective (deep) filtering is needed.
                            self.last_contacts.push(Physics2dContact::Started(*a, *b));
                            self.active_contacts.insert((*a, *b));
                            self.active_contacts.insert((*b, *a));
                        }
                    }
                }
                ContactEvent::Stopped(a, b) => {
                    if let Some(a) = self.collider_map.get(a) {
                        if let Some(b) = self.collider_map.get(b) {
                            // TODO: test if test for effective (deep) filtering is needed.
                            self.last_contacts.push(Physics2dContact::Stopped(*a, *b));
                            self.active_contacts.remove(&(*a, *b));
                            self.active_contacts.remove(&(*b, *a));
                        }
                    }
                }
            }
        }
        for proximity in self.geometrical_world.proximity_events() {
            if let Some(a) = self.collider_map.get(&proximity.collider1) {
                if let Some(b) = self.collider_map.get(&proximity.collider2) {
                    let p = proximity.prev_status == Proximity::Intersecting;
                    let n = proximity.new_status == Proximity::Intersecting;
                    if !p && n {
                        // TODO: test if test for effective (deep) filtering is needed.
                        self.last_proximities
                            .push(Physics2dProximity::Started(*a, *b));
                        self.active_proximities.insert((*a, *b));
                        self.active_proximities.insert((*b, *a));
                    } else if p && !n {
                        // TODO: test if test for effective (deep) filtering is needed.
                        self.last_proximities
                            .push(Physics2dProximity::Stopped(*a, *b));
                        self.active_proximities.remove(&(*a, *b));
                        self.active_proximities.remove(&(*b, *a));
                    }
                }
            }
        }
    }
}
