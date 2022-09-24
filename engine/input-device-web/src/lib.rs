extern crate oxygengine_backend_web as backend;
extern crate oxygengine_core as core;
extern crate oxygengine_input as input;

pub mod keyboard;
pub mod mouse;
pub mod touch;
pub mod utils;

pub mod prelude {
    pub use crate::{keyboard::*, mouse::*, touch::*, utils::*};
}
