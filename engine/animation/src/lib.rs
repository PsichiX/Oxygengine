extern crate oxygengine_core as core;

pub mod animation;
pub mod curve;
pub mod phase;
pub mod spline;
pub mod transition;

pub mod prelude {
    pub use crate::{animation::*, curve::*, phase::*, spline::*, transition::*};
}

use core::Scalar;

pub fn factor_iter(steps: usize) -> impl Iterator<Item = Scalar> {
    (0..=steps).map(move |index| index as Scalar / steps as Scalar)
}

pub fn range_iter(steps: usize, from: Scalar, to: Scalar) -> impl Iterator<Item = Scalar> {
    let diff = to - from;
    (0..=steps).map(move |index| from + diff * index as Scalar / steps as Scalar)
}
