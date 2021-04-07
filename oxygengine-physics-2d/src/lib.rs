extern crate oxygengine_core as core;

pub mod component;
pub mod resource;
pub mod system;

pub use ncollide2d as collide;
pub use nphysics2d as physics;

pub mod prelude {
    pub use crate::{component::*, physics::*, resource::*, system::*};
    pub use ncollide2d::shape::*;
    pub use nphysics2d::{math::*, object::*};
}

use crate::{
    component::{
        Collider2d, Collider2dBody, Collider2dBodyPrefabProxy, Collider2dPrefabProxy, RigidBody2d,
        RigidBody2dPrefabProxy,
    },
    resource::{Physics2dWorld, Physics2dWorldSimulationMode},
    system::Physics2dSystem,
};
use core::{app::AppBuilder, prefab::PrefabManager, Scalar};
use nphysics2d::math::Vector;

pub fn bundle_installer(
    builder: &mut AppBuilder,
    (gravity, simulation_mode): (Vector<Scalar>, Physics2dWorldSimulationMode),
) {
    builder.install_resource(Physics2dWorld::new(gravity, simulation_mode));
    builder.install_system(Physics2dSystem::default(), "physics-2d", &[]);
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory_proxy::<RigidBody2d, RigidBody2dPrefabProxy>("RigidBody2d");
    prefabs.register_component_factory_proxy::<Collider2d, Collider2dPrefabProxy>("Collider2d");
    prefabs.register_component_factory_proxy::<Collider2dBody, Collider2dBodyPrefabProxy>(
        "Collider2dBody",
    );
}
