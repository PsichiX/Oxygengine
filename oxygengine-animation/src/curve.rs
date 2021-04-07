use core::Scalar;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

const CALCULATE_LENGTH_SAMPLES: usize = 100;

pub trait Curved {
    fn zero() -> Self;
    fn one() -> Self;
    fn negate(&self) -> Self;
    fn get_axis(&self, index: usize) -> Option<Scalar>;
    fn interpolate(&self, other: &Self, factor: Scalar) -> Self;
}

impl Curved for Scalar {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn negate(&self) -> Self {
        -self
    }

    fn get_axis(&self, index: usize) -> Option<Scalar> {
        match index {
            0 => Some(*self),
            _ => None,
        }
    }

    fn interpolate(&self, other: &Self, factor: Scalar) -> Self {
        let diff = other - self;
        diff * factor + self
    }
}

impl Curved for (Scalar, Scalar) {
    fn zero() -> Self {
        (0.0, 0.0)
    }

    fn one() -> Self {
        (1.0, 1.0)
    }

    fn negate(&self) -> Self {
        (-self.0, -self.1)
    }

    fn get_axis(&self, index: usize) -> Option<Scalar> {
        match index {
            0 => Some(self.0),
            1 => Some(self.1),
            _ => None,
        }
    }

    fn interpolate(&self, other: &Self, factor: Scalar) -> Self {
        let diff0 = other.0 - self.0;
        let diff1 = other.1 - self.1;
        (diff0 * factor + self.0, diff1 * factor + self.1)
    }
}

impl<T> Curved for Arc<RwLock<T>>
where
    T: Curved,
{
    fn zero() -> Self {
        Arc::new(RwLock::new(T::zero()))
    }

    fn one() -> Self {
        Arc::new(RwLock::new(T::one()))
    }

    fn negate(&self) -> Self {
        Arc::new(RwLock::new(self.read().unwrap().negate()))
    }

    fn get_axis(&self, index: usize) -> Option<Scalar> {
        self.read().unwrap().get_axis(index)
    }

    fn interpolate(&self, other: &Self, factor: Scalar) -> Self {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        let value = from.interpolate(to, factor);
        Arc::new(RwLock::new(value))
    }
}

pub trait CurvedDistance {
    fn curved_distance(&self, other: &Self) -> Scalar;
}

impl CurvedDistance for (Scalar, Scalar) {
    fn curved_distance(&self, other: &Self) -> Scalar {
        let diff0 = other.0 - self.0;
        let diff1 = other.1 - self.1;
        (diff0 * diff0 + diff1 * diff1).sqrt()
    }
}

impl<T> CurvedDistance for Arc<RwLock<T>>
where
    T: CurvedDistance,
{
    fn curved_distance(&self, other: &Self) -> Scalar {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        from.curved_distance(to)
    }
}

pub trait CurvedOffset {
    fn curved_offset(&self, other: &Self) -> Self;
}

impl CurvedOffset for (Scalar, Scalar) {
    fn curved_offset(&self, other: &Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }
}

impl<T> CurvedOffset for Arc<RwLock<T>>
where
    T: CurvedOffset,
{
    fn curved_offset(&self, other: &Self) -> Self {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        Arc::new(RwLock::new(from.curved_offset(to)))
    }
}

pub trait CurvedDirection {
    fn curved_direction(&self, other: &Self) -> Self;
}

impl CurvedDirection for (Scalar, Scalar) {
    fn curved_direction(&self, other: &Self) -> Self {
        (other.0 - self.0, other.1 - self.1)
    }
}

impl<T> CurvedDirection for Arc<RwLock<T>>
where
    T: CurvedDirection,
{
    fn curved_direction(&self, other: &Self) -> Self {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        Arc::new(RwLock::new(from.curved_direction(to)))
    }
}

pub trait CurvedTangent {
    fn curved_tangent(&self, other: &Self) -> Self;
    fn curved_slope(&self, other: &Self) -> Scalar;
}

impl CurvedTangent for (Scalar, Scalar) {
    fn curved_tangent(&self, other: &Self) -> Self {
        let diff0 = other.0 - self.0;
        let diff1 = other.1 - self.1;
        let len = (diff0 * diff0 + diff1 * diff1).sqrt();
        if len > 0.0 {
            (diff0 / len, diff1 / len)
        } else {
            (0.0, 0.0)
        }
    }

    fn curved_slope(&self, other: &Self) -> Scalar {
        self.0 * other.0 + self.1 * other.1
    }
}

impl<T> CurvedTangent for Arc<RwLock<T>>
where
    T: CurvedTangent,
{
    fn curved_tangent(&self, other: &Self) -> Self {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        Arc::new(RwLock::new(from.curved_tangent(to)))
    }

    fn curved_slope(&self, other: &Self) -> Scalar {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        from.curved_slope(to)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveDef<T>(pub T, pub T, pub T, pub T)
where
    T: Curved;

impl<T> Default for CurveDef<T>
where
    T: Curved,
{
    fn default() -> Self {
        Self(T::zero(), T::zero(), T::one(), T::one())
    }
}

impl<T> From<CurveDef<T>> for Curve<T>
where
    T: Clone + Curved + CurvedDistance + CurvedTangent,
{
    fn from(value: CurveDef<T>) -> Self {
        Self::bezier(value.0, value.1, value.2, value.3)
    }
}

impl<T> From<Curve<T>> for CurveDef<T>
where
    T: Clone + Curved + CurvedDistance + CurvedTangent,
{
    fn from(v: Curve<T>) -> Self {
        Self(v.from, v.from_param, v.to_param, v.to)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "CurveDef<T>")]
#[serde(into = "CurveDef<T>")]
pub struct Curve<T>
where
    T: Clone + Curved + CurvedDistance + CurvedTangent,
{
    from: T,
    from_param: T,
    to_param: T,
    to: T,
    length: Scalar,
}

impl<T> Default for Curve<T>
where
    T: Clone + Curved + CurvedDistance + CurvedTangent,
{
    fn default() -> Self {
        Self::linear(T::zero(), T::one())
    }
}

impl<T> Curve<T>
where
    T: Clone + Curved + CurvedDistance + CurvedTangent,
{
    pub fn linear(from: T, to: T) -> Self {
        let mut result = Self {
            from: from.clone(),
            from_param: from,
            to_param: to.clone(),
            to,
            length: 0.0,
        };
        result.length = result.calculate_length(CALCULATE_LENGTH_SAMPLES);
        result
    }

    pub fn bezier(from: T, from_param: T, to_param: T, to: T) -> Self {
        let mut result = Self {
            from,
            from_param,
            to_param,
            to,
            length: 0.0,
        };
        result.length = result.calculate_length(CALCULATE_LENGTH_SAMPLES);
        result
    }

    pub fn from(&self) -> &T {
        &self.from
    }

    pub fn set_from(&mut self, value: T) {
        self.from = value;
        self.length = self.calculate_length(CALCULATE_LENGTH_SAMPLES);
    }

    pub fn from_param(&self) -> &T {
        &self.from_param
    }

    pub fn set_from_param(&mut self, value: T) {
        self.from_param = value;
        self.length = self.calculate_length(CALCULATE_LENGTH_SAMPLES);
    }

    pub fn to_param(&self) -> &T {
        &self.to_param
    }

    pub fn set_to_param(&mut self, value: T) {
        self.to_param = value;
        self.length = self.calculate_length(CALCULATE_LENGTH_SAMPLES);
    }

    pub fn to(&self) -> &T {
        &self.to
    }

    pub fn set_to(&mut self, value: T) {
        self.to = value;
        self.length = self.calculate_length(CALCULATE_LENGTH_SAMPLES);
    }

    pub fn set(&mut self, from: T, from_param: T, to_param: T, to: T) {
        self.from = from;
        self.from_param = from_param;
        self.to_param = to_param;
        self.to = to;
        self.length = self.calculate_length(CALCULATE_LENGTH_SAMPLES);
    }

    pub fn length(&self) -> Scalar {
        self.length
    }

    #[allow(clippy::many_single_char_names)]
    pub fn sample(&self, mut factor: Scalar) -> T {
        factor = factor.max(0.0).min(1.0);
        let a = self.from.interpolate(&self.from_param, factor);
        let b = self.from_param.interpolate(&self.to_param, factor);
        let c = self.to_param.interpolate(&self.to, factor);
        let d = a.interpolate(&b, factor);
        let e = b.interpolate(&c, factor);
        d.interpolate(&e, factor)
    }

    pub fn sample_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T> {
        let factor = self.find_time_for_axis(axis_value, axis_index)?;
        Some(self.sample(factor))
    }

    pub fn calculate_samples(&self, count: usize) -> impl Iterator<Item = T> + '_ {
        (0..=count).map(move |i| self.sample(i as Scalar / count as Scalar))
    }

    pub fn calculate_samples_along_axis(
        &self,
        count: usize,
        axis_index: usize,
    ) -> Option<impl Iterator<Item = T> + '_> {
        let from = self.from.get_axis(axis_index)?;
        let diff = self.to.get_axis(axis_index)? - from;
        Some((0..=count).filter_map(move |i| {
            self.sample_along_axis(from + diff * i as Scalar / count as Scalar, axis_index)
        }))
    }

    pub fn sample_direction_with_sensitivity(&self, factor: Scalar, sensitivity: Scalar) -> T
    where
        T: CurvedDirection,
    {
        if self.length > 0.0 {
            let s = sensitivity / self.length;
            let a = self.sample(factor - s);
            let b = self.sample(factor + s);
            a.curved_direction(&b)
        } else {
            T::zero()
        }
    }

    pub fn sample_direction_with_sensitivity_along_axis(
        &self,
        axis_value: Scalar,
        sensitivity: Scalar,
        axis_index: usize,
    ) -> Option<T>
    where
        T: CurvedDirection,
    {
        if self.length > 0.0 {
            let factor = self.find_time_for_axis(axis_value, axis_index)?;
            let s = sensitivity / self.length;
            let a = self.sample(factor - s);
            let b = self.sample(factor + s);
            Some(a.curved_direction(&b))
        } else {
            Some(T::zero())
        }
    }

    pub fn sample_direction(&self, factor: Scalar) -> T
    where
        T: CurvedDirection,
    {
        self.sample_direction_with_sensitivity(factor, 1.0e-2)
    }

    pub fn sample_direction_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T>
    where
        T: CurvedDirection,
    {
        self.sample_direction_with_sensitivity_along_axis(axis_value, 1.0e-2, axis_index)
    }

    pub fn sample_tangent_with_sensitivity(&self, factor: Scalar, sensitivity: Scalar) -> T
    where
        T: CurvedTangent,
    {
        if self.length > 0.0 {
            let s = sensitivity / self.length;
            let a = self.sample(factor - s);
            let b = self.sample(factor + s);
            a.curved_tangent(&b)
        } else {
            T::zero()
        }
    }

    pub fn sample_tangent(&self, factor: Scalar) -> T
    where
        T: CurvedTangent,
    {
        self.sample_tangent_with_sensitivity(factor, 1.0e-2)
    }

    pub fn calculate_length(&self, mut samples: usize) -> Scalar {
        samples = samples.max(1);
        let count = samples as Scalar;
        let mut length = 0.0;
        let mut last = self.sample(0.0);
        for i in 1..=samples {
            let current = self.sample(i as Scalar / count);
            length += last.curved_distance(&current);
            last = current;
        }
        length
    }

    pub fn find_time_for_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<Scalar> {
        if (self.to.get_axis(axis_index)? - self.from.get_axis(axis_index)?).abs() < 1.0e-6 {
            return Some(1.0);
        }
        let mut guess = if self.length > 0.0 {
            axis_value / self.length
        } else {
            1.0
        };
        let mut last_tangent = None;
        for _ in 0..5 {
            let dv = self.sample(guess).get_axis(axis_index)? - axis_value;
            if dv.abs() < 1.0e-6 {
                return Some(guess);
            }
            let dv = if self.length > 0.0 {
                dv / self.length
            } else {
                0.0
            };
            let tangent = self.sample_tangent(guess);
            let slope = if let Some(last_tangent) = last_tangent {
                tangent.curved_slope(&last_tangent)
            } else {
                1.0
            };
            last_tangent = Some(tangent);
            guess -= dv * slope;
        }
        Some(guess)
    }
}
