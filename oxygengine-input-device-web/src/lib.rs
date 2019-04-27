extern crate oxygengine_core as core;
extern crate oxygengine_input as input;

pub mod keyboard;
pub mod mouse;
pub mod utils;

pub mod prelude {
    pub use crate::keyboard::*;
    pub use crate::mouse::*;
    pub use crate::utils::*;
}
