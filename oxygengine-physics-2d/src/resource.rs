use crate::Scalar;
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

#[derive(Debug, Copy, Clone)]
pub enum Physics2dWorldSimulationMode {
    FixedTimestepMaxIterations(usize),
    DynamicTimestep,
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

    pub(crate) fn insert_collider(
        &mut self,
        collider: Collider<Scalar, DefaultBodyHandle>,
    ) -> DefaultColliderHandle {
        self.collider_set.insert(collider)
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

    pub fn process(&mut self, mut delta_time: Scalar) {
        match self.simulation_mode {
            Physics2dWorldSimulationMode::FixedTimestepMaxIterations(iterations) => {
                let time_step = self.mechanical_world.timestep();
                delta_time += self.remaining_time_step;
                let mut i = 0;
                while delta_time >= time_step && i < iterations {
                    self.mechanical_world.step(
                        &mut self.geometrical_world,
                        &mut self.body_set,
                        &mut self.collider_set,
                        &mut self.constraint_set,
                        &mut self.force_generator_set,
                    );
                    delta_time -= time_step;
                    i += 1;
                }
                self.remaining_time_step = delta_time % time_step;
            }
            Physics2dWorldSimulationMode::DynamicTimestep => {
                self.mechanical_world.set_timestep(delta_time);
                self.mechanical_world.step(
                    &mut self.geometrical_world,
                    &mut self.body_set,
                    &mut self.collider_set,
                    &mut self.constraint_set,
                    &mut self.force_generator_set,
                );
                self.remaining_time_step = 0.0;
            }
        }
    }
}
