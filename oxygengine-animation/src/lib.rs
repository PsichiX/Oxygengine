extern crate oxygengine_core as core;

pub mod animation;
pub mod curve;
pub mod phase;
pub mod spline;
pub mod transition;

pub mod prelude {
    pub use crate::animation::*;
    pub use crate::curve::*;
    pub use crate::phase::*;
    pub use crate::spline::*;
    pub use crate::transition::*;
}
