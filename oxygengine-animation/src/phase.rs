use crate::spline::*;
use core::Scalar;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub type PhaseDef = Vec<SplinePoint<(Scalar, Scalar)>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "PhaseDef")]
#[serde(into = "PhaseDef")]
pub struct Phase {
    spline: Spline<(Scalar, Scalar)>,
    duration: Scalar,
}

impl Default for Phase {
    fn default() -> Self {
        Self::point(1.0).expect("Could not create default point phase")
    }
}

impl Phase {
    pub fn new(mut points: Vec<SplinePoint<(Scalar, Scalar)>>) -> Result<Self, SplineError> {
        let mut last = 0.0;
        for point in &mut points {
            match &mut point.direction {
                SplinePointDirection::Single(dir) => dir.0 = dir.0.max(0.0),
                SplinePointDirection::InOut(from, to) => {
                    from.0 = from.0.min(0.0);
                    to.0 = to.0.max(0.0);
                }
            }
            point.point.0 = point.point.0.max(last);
            last = point.point.0;
        }
        Ok(Self {
            spline: Spline::new(points)?,
            duration: last,
        })
    }

    pub fn linear(from: Scalar, to: Scalar, duration: Scalar) -> Result<Self, SplineError> {
        Self::new(vec![
            SplinePoint::point((0.0, from)),
            SplinePoint::point((duration, to)),
        ])
    }

    pub fn point(point: Scalar) -> Result<Self, SplineError> {
        Self::linear(point, point, 0.0)
    }

    pub fn duration(&self) -> Scalar {
        self.duration
    }

    pub fn points(&self) -> &[SplinePoint<(Scalar, Scalar)>] {
        self.spline.points()
    }

    pub fn set_points(&mut self, points: Vec<SplinePoint<(Scalar, Scalar)>>) {
        if let Ok(result) = Self::new(points) {
            *self = result;
        }
    }

    pub fn spline(&self) -> &Spline<(Scalar, Scalar)> {
        &self.spline
    }

    pub fn sample(&self, mut time: Scalar) -> Scalar {
        time = time.max(0.0).min(self.duration);
        self.spline.sample_along_axis(time, 0).unwrap().1
    }

    pub fn calculate_samples(&self, count: usize) -> impl Iterator<Item = Scalar> + '_ {
        (0..=count).map(move |i| self.sample(self.duration * i as Scalar / count as Scalar))
    }
}

impl TryFrom<PhaseDef> for Phase {
    type Error = SplineError;

    fn try_from(value: PhaseDef) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Phase> for PhaseDef {
    fn from(v: Phase) -> Self {
        v.spline.into()
    }
}
