#[cfg(feature = "parallel")]
extern crate rayon;
extern crate serde;

pub mod grid_2d;
pub mod noise_map_generator;

pub mod prelude {
    pub use crate::grid_2d::*;
    pub use crate::noise_map_generator::*;
}

#[cfg(feature = "scalar64")]
pub type Scalar = f64;
#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;
