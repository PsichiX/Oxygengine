pub mod wave_function_collapse;
pub mod world_2d;
pub mod world_2d_climate_simulation;
pub use oxygengine_utils::{grid_2d::*, noise_map_generator::*, Scalar};

pub mod prelude {
    pub use crate::wave_function_collapse::*;
    pub use crate::world_2d::*;
    pub use crate::world_2d_climate_simulation::*;
    pub use oxygengine_utils::{grid_2d::*, noise_map_generator::*, Scalar};
}
