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

use crate::{resource::Physics2dWorld, system::Physics2dSystem};
use core::app::AppBuilder;
use nphysics2d::math::Vector;

type Scalar = f64;

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>, gravity: Vector<Scalar>) {
    builder.install_resource(Physics2dWorld::new(gravity));
    builder.install_system(Physics2dSystem::default(), "physics-2d", &[]);
}
