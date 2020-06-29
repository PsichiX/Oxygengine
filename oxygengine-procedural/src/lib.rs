#[cfg(feature = "parallel")]
extern crate rayon;

pub mod wave_function_collapse;
pub mod world_2d;
pub mod world_2d_climate_simulation;

pub mod prelude {
    pub use crate::wave_function_collapse::*;
    pub use crate::world_2d::*;
    pub use crate::world_2d_climate_simulation::*;
    pub use oxygengine_utils::Scalar;
}
