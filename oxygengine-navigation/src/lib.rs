extern crate oxygengine_core as core;

pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{resource::*, system::*};
}

pub type Scalar = f64;

// use core::app::AppBuilder;

// pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>) {}
