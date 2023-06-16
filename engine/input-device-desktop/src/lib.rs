extern crate oxygengine_backend_desktop as backend;
extern crate oxygengine_core as core;
extern crate oxygengine_input as input;

pub mod keyboard;
pub mod mouse;

pub mod prelude {
    pub use crate::{keyboard::*, mouse::*};
}
