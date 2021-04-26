use crate::spline::*;
use core::Scalar;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Ease {
    InSine,
    OutSine,
    InOutSine,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBack,
    OutBack,
    InOutBack,
}

impl Ease {
    pub fn bezier(self) -> (Scalar, Scalar, Scalar, Scalar) {
        match self {
            Self::InSine => (0.47, 0.0, 0.745, 0.715),
            Self::OutSine => (0.39, 0.575, 0.565, 1.0),
            Self::InOutSine => (0.445, 0.05, 0.55, 0.95),
            Self::InQuad => (0.55, 0.085, 0.68, 0.53),
            Self::OutQuad => (0.25, 0.46, 0.45, 0.94),
            Self::InOutQuad => (0.455, 0.03, 0.515, 0.955),
            Self::InCubic => (0.55, 0.055, 0.675, 0.19),
            Self::OutCubic => (0.215, 0.61, 0.355, 1.0),
            Self::InOutCubic => (0.645, 0.045, 0.355, 1.0),
            Self::InQuart => (0.895, 0.03, 0.685, 0.22),
            Self::OutQuart => (0.165, 0.84, 0.44, 1.0),
            Self::InOutQuart => (0.77, 0.0, 0.175, 1.0),
            Self::InQuint => (0.755, 0.05, 0.855, 0.06),
            Self::OutQuint => (0.23, 1.0, 0.32, 1.0),
            Self::InOutQuint => (0.86, 0.0, 0.07, 1.0),
            Self::InExpo => (0.95, 0.05, 0.795, 0.035),
            Self::OutExpo => (0.19, 1.0, 0.22, 1.0),
            Self::InOutExpo => (1.0, 0.0, 0.0, 1.0),
            Self::InCirc => (0.6, 0.04, 0.98, 0.335),
            Self::OutCirc => (0.075, 0.82, 0.165, 1.0),
            Self::InOutCirc => (0.785, 0.135, 0.15, 0.86),
            Self::InBack => (0.6, -0.28, 0.735, 0.045),
            Self::OutBack => (0.175, 0.885, 0.32, 1.275),
            Self::InOutBack => (0.68, -0.55, 0.265, 1.55),
        }
    }
}

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

    pub fn bezier(
        (mut x1, mut y1, mut x2, mut y2): (Scalar, Scalar, Scalar, Scalar),
        from: Scalar,
        to: Scalar,
        duration: Scalar,
    ) -> Result<Self, SplineError> {
        let distance = (to - from).abs();
        x1 *= duration;
        y1 *= distance;
        x2 = (1.0 - x2) * -duration;
        y2 = (1.0 - y2) * -distance;
        Self::new(vec![
            SplinePoint::new((0.0, from), SplinePointDirection::Single((x1, y1))),
            SplinePoint::new((duration, to), SplinePointDirection::Single((x2, y2))),
        ])
    }

    pub fn ease(
        ease: Ease,
        from: Scalar,
        to: Scalar,
        duration: Scalar,
    ) -> Result<Self, SplineError> {
        Self::bezier(ease.bezier(), from, to, duration)
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
