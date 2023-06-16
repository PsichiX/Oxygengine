extern crate oxygengine_core as core;

pub mod app;
pub mod resource;

pub mod prelude {
    pub use crate::{app::*, resource::*};
}
