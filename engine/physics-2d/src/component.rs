use core::{
    ecs::Entity,
    prefab::{Prefab, PrefabError, PrefabProxy},
    state::StateToken,
    Scalar,
};
use ncollide2d::{
    pipeline::CollisionGroups,
    shape::{Ball, Capsule, ConvexPolygon, Cuboid, HeightField, Plane, Segment, ShapeHandle},
};
use nphysics2d::{
    math::{Inertia, Isometry, Point, Vector, Velocity},
    object::{
        ActivationStatus, BodyStatus, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle,
        RigidBodyDesc,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(not(feature = "scalar64"))]
use std::f32::{consts::PI as SCALAR_PI, MAX as SCALAR_MAX};
#[cfg(feature = "scalar64")]
use std::f64::{consts::PI as SCALAR_PI, MAX as SCALAR_MAX};

pub(crate) enum RigidBody2dInner {
    None,
    Description(RigidBodyDesc<Scalar>),
    Handle(DefaultBodyHandle),
}

pub struct RigidBody2d(pub(crate) RigidBody2dInner);

impl RigidBody2d {
    pub fn new(desc: RigidBodyDesc<Scalar>) -> Self {
        Self(RigidBody2dInner::Description(desc))
    }

    pub fn is_created(&self) -> bool {
        matches!(&self.0, RigidBody2dInner::Handle(_))
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBody2dPrefabProxyBodyStatus {
    Disabled,
    Static,
    Dynamic,
    Kinematic,
}

impl Default for RigidBody2dPrefabProxyBodyStatus {
    fn default() -> Self {
        Self::Dynamic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody2dPrefabProxy {
    #[serde(default = "RigidBody2dPrefabProxy::default_gravity_enabled")]
    pub gravity_enabled: bool,
    #[serde(default = "RigidBody2dPrefabProxy::default_linear_motion_interpolation_enabled")]
    pub linear_motion_interpolation_enabled: bool,
    #[serde(default = "RigidBody2dPrefabProxy::default_position")]
    pub position: Vector<Scalar>,
    #[serde(default = "RigidBody2dPrefabProxy::default_rotation")]
    pub rotation: Scalar,
    #[serde(default = "RigidBody2dPrefabProxy::default_velocity")]
    pub velocity: (Vector<Scalar>, Scalar),
    #[serde(default = "RigidBody2dPrefabProxy::default_linear_damping")]
    pub linear_damping: Scalar,
    #[serde(default = "RigidBody2dPrefabProxy::default_angular_damping")]
    pub angular_damping: Scalar,
    #[serde(default = "RigidBody2dPrefabProxy::default_max_linear_velocity")]
    pub max_linear_velocity: Scalar,
    #[serde(default = "RigidBody2dPrefabProxy::default_max_angular_velocity")]
    pub max_angular_velocity: Scalar,
    #[serde(default = "RigidBody2dPrefabProxy::default_local_inertia")]
    pub local_inertia: (Scalar, Scalar),
    #[serde(default = "RigidBody2dPrefabProxy::default_local_center_of_mass")]
    pub local_center_of_mass: Point<Scalar>,
    #[serde(default = "RigidBody2dPrefabProxy::default_status")]
    pub status: RigidBody2dPrefabProxyBodyStatus,
    #[serde(default = "RigidBody2dPrefabProxy::default_sleep_threshold")]
    pub sleep_threshold: Option<Scalar>,
    #[serde(default = "RigidBody2dPrefabProxy::default_kinematic_translations")]
    pub kinematic_translations: Vector<bool>,
    #[serde(default = "RigidBody2dPrefabProxy::default_kinematic_rotations")]
    pub kinematic_rotations: bool,
}

impl Default for RigidBody2dPrefabProxy {
    fn default() -> Self {
        Self {
            gravity_enabled: Self::default_gravity_enabled(),
            linear_motion_interpolation_enabled: Self::default_linear_motion_interpolation_enabled(
            ),
            position: Self::default_position(),
            rotation: Self::default_rotation(),
            velocity: Self::default_velocity(),
            linear_damping: Self::default_linear_damping(),
            angular_damping: Self::default_angular_damping(),
            max_linear_velocity: Self::default_max_linear_velocity(),
            max_angular_velocity: Self::default_max_angular_velocity(),
            local_inertia: Self::default_local_inertia(),
            local_center_of_mass: Self::default_local_center_of_mass(),
            status: Self::default_status(),
            sleep_threshold: Self::default_sleep_threshold(),
            kinematic_translations: Self::default_kinematic_translations(),
            kinematic_rotations: Self::default_kinematic_rotations(),
        }
    }
}

impl RigidBody2dPrefabProxy {
    fn default_gravity_enabled() -> bool {
        true
    }

    fn default_linear_motion_interpolation_enabled() -> bool {
        false
    }

    fn default_position() -> Vector<Scalar> {
        Vector::new(0.0, 0.0)
    }

    fn default_rotation() -> Scalar {
        0.0
    }

    fn default_velocity() -> (Vector<Scalar>, Scalar) {
        (Vector::new(0.0, 0.0), 0.0)
    }

    fn default_linear_damping() -> Scalar {
        0.0
    }

    fn default_angular_damping() -> Scalar {
        0.0
    }

    fn default_max_linear_velocity() -> Scalar {
        SCALAR_MAX
    }

    fn default_max_angular_velocity() -> Scalar {
        SCALAR_MAX
    }

    fn default_local_inertia() -> (Scalar, Scalar) {
        (0.0, 0.0)
    }

    fn default_local_center_of_mass() -> Point<Scalar> {
        Point::origin()
    }

    fn default_status() -> RigidBody2dPrefabProxyBodyStatus {
        RigidBody2dPrefabProxyBodyStatus::Dynamic
    }

    #[allow(clippy::unnecessary_wraps)]
    fn default_sleep_threshold() -> Option<Scalar> {
        Some(ActivationStatus::default_threshold())
    }

    fn default_kinematic_translations() -> Vector<bool> {
        Vector::repeat(false)
    }

    fn default_kinematic_rotations() -> bool {
        false
    }
}

impl Prefab for RigidBody2dPrefabProxy {}

impl PrefabProxy<RigidBody2dPrefabProxy> for RigidBody2d {
    fn from_proxy_with_extras(
        proxy: RigidBody2dPrefabProxy,
        _: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        let desc = RigidBodyDesc::new()
            .gravity_enabled(proxy.gravity_enabled)
            .linear_motion_interpolation_enabled(proxy.linear_motion_interpolation_enabled)
            .position(Isometry::new(proxy.position, proxy.rotation))
            .velocity(Velocity::new(proxy.velocity.0, proxy.velocity.1))
            .linear_damping(proxy.linear_damping)
            .angular_damping(proxy.angular_damping)
            .max_linear_velocity(proxy.max_linear_velocity)
            .max_angular_velocity(proxy.max_angular_velocity)
            .local_inertia(Inertia::new(proxy.local_inertia.0, proxy.local_inertia.1))
            .local_center_of_mass(proxy.local_center_of_mass)
            .status(match proxy.status {
                RigidBody2dPrefabProxyBodyStatus::Disabled => BodyStatus::Disabled,
                RigidBody2dPrefabProxyBodyStatus::Static => BodyStatus::Static,
                RigidBody2dPrefabProxyBodyStatus::Dynamic => BodyStatus::Dynamic,
                RigidBody2dPrefabProxyBodyStatus::Kinematic => BodyStatus::Kinematic,
            })
            .sleep_threshold(proxy.sleep_threshold)
            .kinematic_translations(proxy.kinematic_translations)
            .kinematic_rotations(proxy.kinematic_rotations);
        Ok(Self::new(desc))
    }
}

pub(crate) enum Collider2dInner {
    None,
    Description(ColliderDesc<Scalar>),
    Handle(DefaultColliderHandle),
}

pub struct Collider2d(pub(crate) Collider2dInner);

impl Collider2d {
    pub fn new(desc: ColliderDesc<Scalar>) -> Self {
        Self(Collider2dInner::Description(desc))
    }

    pub fn is_created(&self) -> bool {
        matches!(&self.0, Collider2dInner::Handle(_))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Collider2dPrefabProxyShape {
    Ball(Ball<Scalar>),
    Capsule(Capsule<Scalar>),
    ConvexPolygon(ConvexPolygon<Scalar>),
    Cuboid(Cuboid<Scalar>),
    HeightField(HeightField<Scalar>),
    Plane(Plane<Scalar>),
    Segment(Segment<Scalar>),
}

impl Default for Collider2dPrefabProxyShape {
    fn default() -> Self {
        Self::Ball(Ball::new(1.0))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Collider2dPrefabProxyCollisionGroups {
    pub membership: Option<Vec<usize>>,
    pub whitelist: Option<Vec<usize>>,
    pub blacklist: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider2dPrefabProxy {
    #[serde(default = "Collider2dPrefabProxy::default_margin")]
    pub margin: Scalar,
    #[serde(default)]
    pub collision_groups: Collider2dPrefabProxyCollisionGroups,
    #[serde(default)]
    pub shape: Collider2dPrefabProxyShape,
    #[serde(default = "Collider2dPrefabProxy::default_position")]
    pub position: Vector<Scalar>,
    #[serde(default = "Collider2dPrefabProxy::default_rotation")]
    pub rotation: Scalar,
    #[serde(default = "Collider2dPrefabProxy::default_density")]
    pub density: Scalar,
    #[serde(default = "Collider2dPrefabProxy::default_linear_prediction")]
    pub linear_prediction: Scalar,
    #[serde(default = "Collider2dPrefabProxy::default_angular_prediction")]
    pub angular_prediction: Scalar,
    #[serde(default = "Collider2dPrefabProxy::default_is_sensor")]
    pub is_sensor: bool,
    #[serde(default = "Collider2dPrefabProxy::default_ccd_enabled")]
    pub ccd_enabled: bool,
}

impl Default for Collider2dPrefabProxy {
    fn default() -> Self {
        Self {
            margin: Self::default_margin(),
            collision_groups: Collider2dPrefabProxyCollisionGroups::default(),
            shape: Collider2dPrefabProxyShape::default(),
            position: Self::default_position(),
            rotation: Self::default_rotation(),
            density: Self::default_density(),
            linear_prediction: Self::default_linear_prediction(),
            angular_prediction: Self::default_angular_prediction(),
            is_sensor: Self::default_is_sensor(),
            ccd_enabled: Self::default_ccd_enabled(),
        }
    }
}

impl Collider2dPrefabProxy {
    fn default_margin() -> Scalar {
        0.01
    }

    fn default_position() -> Vector<Scalar> {
        Vector::new(0.0, 0.0)
    }

    fn default_rotation() -> Scalar {
        0.0
    }

    fn default_density() -> Scalar {
        0.0
    }

    fn default_linear_prediction() -> Scalar {
        0.001
    }

    fn default_angular_prediction() -> Scalar {
        SCALAR_PI / 180.0 * 5.0
    }

    fn default_is_sensor() -> bool {
        false
    }

    fn default_ccd_enabled() -> bool {
        false
    }
}

impl Prefab for Collider2dPrefabProxy {}

impl PrefabProxy<Collider2dPrefabProxy> for Collider2d {
    fn from_proxy_with_extras(
        proxy: Collider2dPrefabProxy,
        _: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        let shape = match proxy.shape {
            Collider2dPrefabProxyShape::Ball(shape) => ShapeHandle::new(shape),
            Collider2dPrefabProxyShape::Capsule(shape) => ShapeHandle::new(shape),
            Collider2dPrefabProxyShape::ConvexPolygon(shape) => ShapeHandle::new(shape),
            Collider2dPrefabProxyShape::Cuboid(shape) => ShapeHandle::new(shape),
            Collider2dPrefabProxyShape::HeightField(shape) => ShapeHandle::new(shape),
            Collider2dPrefabProxyShape::Plane(shape) => ShapeHandle::new(shape),
            Collider2dPrefabProxyShape::Segment(shape) => ShapeHandle::new(shape),
        };
        let desc = ColliderDesc::new(shape)
            .margin(proxy.margin)
            .collision_groups({
                let mut result = CollisionGroups::new();
                if let Some(list) = proxy.collision_groups.membership {
                    result = result.with_membership(&list)
                }
                if let Some(list) = proxy.collision_groups.whitelist {
                    result = result.with_whitelist(&list)
                }
                if let Some(list) = proxy.collision_groups.blacklist {
                    result = result.with_blacklist(&list)
                }
                result
            })
            .position(Isometry::new(proxy.position, proxy.rotation))
            .density(proxy.density)
            .linear_prediction(proxy.linear_prediction)
            .angular_prediction(proxy.angular_prediction)
            .sensor(proxy.is_sensor)
            .ccd_enabled(proxy.ccd_enabled);
        Ok(Self::new(desc))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Collider2dBody {
    Me,
    Entity(Entity),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Collider2dBodyPrefabProxy {
    Me,
    Entity(String),
}

impl Prefab for Collider2dBodyPrefabProxy {}

impl PrefabProxy<Collider2dBodyPrefabProxy> for Collider2dBody {
    fn from_proxy_with_extras(
        proxy: Collider2dBodyPrefabProxy,
        named_entities: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        match proxy {
            Collider2dBodyPrefabProxy::Me => Ok(Self::Me),
            Collider2dBodyPrefabProxy::Entity(name) => {
                if let Some(entity) = named_entities.get(&name) {
                    Ok(Self::Entity(*entity))
                } else {
                    Err(PrefabError::Custom(format!(
                        "Could not find entity named: {}",
                        name
                    )))
                }
            }
        }
    }
}
