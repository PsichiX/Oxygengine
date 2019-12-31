use crate::Scalar;
use core::ecs::{Component, Entity, FlaggedStorage, VecStorage};
use nphysics2d::object::{ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, RigidBodyDesc};

pub(crate) enum RigidBody2dInner {
    None,
    Description(RigidBodyDesc<Scalar>),
    Handle(DefaultBodyHandle),
}

pub struct RigidBody2d(pub(crate) RigidBody2dInner);

impl Component for RigidBody2d {
    type Storage = FlaggedStorage<Self>;
}

impl RigidBody2d {
    pub fn new(desc: RigidBodyDesc<Scalar>) -> Self {
        Self(RigidBody2dInner::Description(desc))
    }

    pub fn is_created(&self) -> bool {
        if let RigidBody2dInner::Handle(_) = &self.0 {
            true
        } else {
            false
        }
    }

    pub fn handle(&self) -> Option<DefaultBodyHandle> {
        if let RigidBody2dInner::Handle(handle) = &self.0 {
            Some(*handle)
        } else {
            None
        }
    }

    pub(crate) fn take_description(&mut self) -> Option<RigidBodyDesc<Scalar>> {
        if self.is_created() {
            return None;
        }
        let inner = std::mem::replace(&mut self.0, RigidBody2dInner::None);
        if let RigidBody2dInner::Description(desc) = inner {
            Some(desc)
        } else {
            None
        }
    }
}

pub(crate) enum Collider2dInner {
    None,
    Description(ColliderDesc<Scalar>),
    Handle(DefaultColliderHandle),
}

pub struct Collider2d(pub(crate) Collider2dInner);

impl Component for Collider2d {
    type Storage = FlaggedStorage<Self>;
}

impl Collider2d {
    pub fn new(desc: ColliderDesc<Scalar>) -> Self {
        Self(Collider2dInner::Description(desc))
    }

    pub fn is_created(&self) -> bool {
        if let Collider2dInner::Handle(_) = &self.0 {
            true
        } else {
            false
        }
    }

    pub fn handle(&self) -> Option<DefaultColliderHandle> {
        if let Collider2dInner::Handle(handle) = &self.0 {
            Some(*handle)
        } else {
            None
        }
    }

    pub(crate) fn take_description(&mut self) -> Option<ColliderDesc<Scalar>> {
        if self.is_created() {
            return None;
        }
        let inner = std::mem::replace(&mut self.0, Collider2dInner::None);
        if let Collider2dInner::Description(desc) = inner {
            Some(desc)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Collider2dBody {
    Me,
    Entity(Entity),
}

impl Component for Collider2dBody {
    type Storage = VecStorage<Self>;
}
