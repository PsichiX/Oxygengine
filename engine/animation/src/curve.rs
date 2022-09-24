use crate::range_iter;
use core::Scalar;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

const EPSILON: Scalar = Scalar::EPSILON * 10.0;

pub trait Curved {
    fn zero() -> Self;
    fn one() -> Self;
    fn negate(&self) -> Self;
    fn scale(&self, value: Scalar) -> Self;
    fn inverse_scale(&self, value: Scalar) -> Self;
    fn length(&self) -> Scalar;
    fn get_axis(&self, index: usize) -> Option<Scalar>;
    fn interpolate(&self, other: &Self, factor: Scalar) -> Self;
    fn is_valid(&self) -> bool;
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

    fn scale(&self, value: Scalar) -> Self {
        self * value
    }

    fn inverse_scale(&self, value: Scalar) -> Self {
        self / value
    }

    fn length(&self) -> Scalar {
        self.abs()
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

    fn is_valid(&self) -> bool {
        self.is_finite()
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

    fn scale(&self, value: Scalar) -> Self {
        (self.0 * value, self.1 * value)
    }

    fn inverse_scale(&self, value: Scalar) -> Self {
        (self.0 / value, self.1 / value)
    }

    fn length(&self) -> Scalar {
        (self.0 * self.0 + self.1 * self.1).sqrt()
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

    fn is_valid(&self) -> bool {
        self.0.is_valid() && self.1.is_valid()
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

    fn scale(&self, value: Scalar) -> Self {
        Arc::new(RwLock::new(self.read().unwrap().scale(value)))
    }

    fn inverse_scale(&self, value: Scalar) -> Self {
        Arc::new(RwLock::new(self.read().unwrap().inverse_scale(value)))
    }

    fn length(&self) -> Scalar {
        self.read().unwrap().length()
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

    fn is_valid(&self) -> bool {
        self.read().unwrap().is_valid()
    }
}

pub trait CurvedChange {
    fn offset(&self, other: &Self) -> Self;
    fn delta(&self, other: &Self) -> Self;
    fn slope(&self, other: &Self) -> Scalar;
}

impl CurvedChange for (Scalar, Scalar) {
    fn offset(&self, other: &Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }

    fn delta(&self, other: &Self) -> Self {
        (other.0 - self.0, other.1 - self.1)
    }

    fn slope(&self, other: &Self) -> Scalar {
        self.0 * other.0 + self.1 * other.1
    }
}

impl<T> CurvedChange for Arc<RwLock<T>>
where
    T: CurvedChange,
{
    fn offset(&self, other: &Self) -> Self {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        Arc::new(RwLock::new(from.offset(to)))
    }

    fn delta(&self, other: &Self) -> Self {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        Arc::new(RwLock::new(from.delta(to)))
    }

    fn slope(&self, other: &Self) -> Scalar {
        let from: &T = &self.read().unwrap();
        let to: &T = &other.read().unwrap();
        from.slope(to)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveDef<T>(pub T, pub T, pub T, pub T);

impl<T> Default for CurveDef<T>
where
    T: Curved,
{
    fn default() -> Self {
        Self(T::zero(), T::zero(), T::one(), T::one())
    }
}

impl<T> TryFrom<CurveDef<T>> for Curve<T>
where
    T: Clone + Curved + CurvedChange,
{
    type Error = CurveError;

    fn try_from(value: CurveDef<T>) -> Result<Self, Self::Error> {
        Self::bezier(value.0, value.1, value.2, value.3)
    }
}

impl<T> From<Curve<T>> for CurveDef<T>
where
    T: Clone + Curved + CurvedChange,
{
    fn from(v: Curve<T>) -> Self {
        Self(v.from, v.from_param, v.to_param, v.to)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CurveError {
    InvalidFromValue,
    InvalidFromParamValue,
    InvalidToParamValue,
    InvalidToValue,
    CannotSplit,
}

impl std::fmt::Display for CurveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "CurveDef<T>")]
#[serde(into = "CurveDef<T>")]
pub struct Curve<T>
where
    T: Clone + Curved + CurvedChange,
{
    from: T,
    from_param: T,
    to_param: T,
    to: T,
    length: Scalar,
}

impl<T> Default for Curve<T>
where
    T: Clone + Curved + CurvedChange,
{
    fn default() -> Self {
        Self::linear(T::zero(), T::one()).unwrap()
    }
}

impl<T> Curve<T>
where
    T: Clone + Curved + CurvedChange,
{
    fn new_uninitialized(from: T, from_param: T, to_param: T, to: T) -> Result<Self, CurveError> {
        if !from.is_valid() {
            return Err(CurveError::InvalidFromValue);
        }
        if !from_param.is_valid() {
            return Err(CurveError::InvalidFromParamValue);
        }
        if !to_param.is_valid() {
            return Err(CurveError::InvalidToParamValue);
        }
        if !to.is_valid() {
            return Err(CurveError::InvalidToValue);
        }
        Ok(Self {
            from,
            from_param,
            to_param,
            to,
            length: 0.0,
        })
    }

    pub fn linear(from: T, to: T) -> Result<Self, CurveError> {
        let mut result = Self::new_uninitialized(from.clone(), to.clone(), from, to)?;
        result.recalculate_length();
        Ok(result)
    }

    pub fn bezier(from: T, from_param: T, to_param: T, to: T) -> Result<Self, CurveError> {
        let mut result = Self::new_uninitialized(from, from_param, to_param, to)?;
        result.recalculate_length();
        Ok(result)
    }

    fn recalculate_length(&mut self) {
        self.length = self.arc_length(EPSILON);
    }

    pub fn from(&self) -> &T {
        &self.from
    }

    pub fn set_from(&mut self, value: T) {
        self.from = value;
        self.recalculate_length();
    }

    pub fn from_param(&self) -> &T {
        &self.from_param
    }

    pub fn set_from_param(&mut self, value: T) {
        self.from_param = value;
        self.recalculate_length();
    }

    pub fn to_param(&self) -> &T {
        &self.to_param
    }

    pub fn set_to_param(&mut self, value: T) {
        self.to_param = value;
        self.recalculate_length();
    }

    pub fn to(&self) -> &T {
        &self.to
    }

    pub fn set_to(&mut self, value: T) {
        self.to = value;
        self.recalculate_length();
    }

    pub fn set(&mut self, from: T, from_param: T, to_param: T, to: T) {
        self.from = from;
        self.from_param = from_param;
        self.to_param = to_param;
        self.to = to;
        self.recalculate_length();
    }

    pub fn length(&self) -> Scalar {
        self.length
    }

    pub fn value_along_axis_iter(
        &self,
        steps: usize,
        axis_index: usize,
    ) -> Option<impl Iterator<Item = Scalar>> {
        let from = self.from.get_axis(axis_index)?;
        let to = self.to.get_axis(axis_index)?;
        Some(range_iter(steps, from, to))
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

    /// Velocity of change along the curve.
    pub fn sample_first_derivative(&self, factor: Scalar) -> T {
        let a = self.from.delta(&self.from_param);
        let b = self.from_param.delta(&self.to_param);
        let c = self.to_param.delta(&self.to);
        let d = a.interpolate(&b, factor);
        let e = b.interpolate(&c, factor);
        d.interpolate(&e, factor)
    }

    /// Velocity of change along the curve axis.
    pub fn sample_first_derivative_along_axis(
        &self,
        axis_value: Scalar,
        axis_index: usize,
    ) -> Option<T> {
        let factor = self.find_time_for_axis(axis_value, axis_index)?;
        Some(self.sample_first_derivative(factor))
    }

    /// Acceleration of change along the curve.
    pub fn sample_second_derivative(&self, factor: Scalar) -> T {
        let a = self.from.delta(&self.from_param);
        let b = self.from_param.delta(&self.to_param);
        let c = self.to_param.delta(&self.to);
        let d = a.delta(&b);
        let e = b.delta(&c);
        d.interpolate(&e, factor)
    }

    /// Acceleration of change along the curve axis.
    pub fn sample_second_derivative_along_axis(
        &self,
        axis_value: Scalar,
        axis_index: usize,
    ) -> Option<T> {
        let factor = self.find_time_for_axis(axis_value, axis_index)?;
        Some(self.sample_second_derivative(factor))
    }

    pub fn sample_direction(&self, mut factor: Scalar) -> T {
        factor = factor.max(EPSILON).min(1.0 - EPSILON);
        self.sample_first_derivative(factor)
    }

    pub fn sample_direction_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T> {
        let factor = self.find_time_for_axis(axis_value, axis_index)?;
        Some(self.sample_direction(factor))
    }

    pub fn sample_tangent(&self, factor: Scalar) -> T {
        let direction = self.sample_direction(factor);
        let length = direction.length();
        direction.inverse_scale(length)
    }

    pub fn sample_tangent_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T> {
        let factor = self.find_time_for_axis(axis_value, axis_index)?;
        Some(self.sample_tangent(factor))
    }

    fn split_uninitialized(&self, mut factor: Scalar) -> Result<(Self, Self), CurveError> {
        factor = factor.max(0.0).min(1.0);
        #[allow(clippy::manual_range_contains)]
        if factor < EPSILON || factor > 1.0 - EPSILON {
            return Err(CurveError::CannotSplit);
        }
        let a = self.from.interpolate(&self.from_param, factor);
        let b = self.from_param.interpolate(&self.to_param, factor);
        let c = self.to_param.interpolate(&self.to, factor);
        let d = a.interpolate(&b, factor);
        let e = b.interpolate(&c, factor);
        let f = d.interpolate(&e, factor);
        let first = Self::new_uninitialized(self.from.clone(), a, d, f.clone())?;
        let second = Self::new_uninitialized(f, e, c, self.to.clone())?;
        Ok((first, second))
    }

    pub fn split(&self, factor: Scalar) -> Result<(Self, Self), CurveError> {
        self.split_uninitialized(factor).map(|(mut a, mut b)| {
            a.recalculate_length();
            b.recalculate_length();
            (a, b)
        })
    }

    fn estimate_arc_length(&self) -> Scalar {
        let a = self.from.delta(&self.from_param).length();
        let b = self.from_param.delta(&self.to_param).length();
        let c = self.to_param.delta(&self.to).length();
        let d = self.to.delta(&self.from).length();
        (a + b + c + d) * 0.5
    }

    fn arc_length(&self, threshold: Scalar) -> Scalar {
        self.arc_length_inner(self.estimate_arc_length(), threshold, 5)
    }

    fn arc_length_inner(&self, estimation: Scalar, threshold: Scalar, levels: usize) -> Scalar {
        let (a, b) = match self.split_uninitialized(0.5) {
            Ok((a, b)) => (a, b),
            Err(_) => return estimation,
        };
        let ra = a.estimate_arc_length();
        let rb = b.estimate_arc_length();
        let result = ra + rb;
        if (estimation - result).abs() < threshold || levels == 0 {
            return result;
        }
        let levels = levels - 1;
        let a = a.arc_length_inner(ra, threshold, levels);
        let b = b.arc_length_inner(rb, threshold, levels);
        a + b
    }

    pub fn find_time_for_axis(&self, mut axis_value: Scalar, axis_index: usize) -> Option<Scalar> {
        let min = self.from.get_axis(axis_index)?;
        let max = self.to.get_axis(axis_index)?;
        let dist = max - min;
        if dist.abs() < EPSILON {
            return Some(1.0);
        }
        axis_value = axis_value.max(min).min(max);
        let mut guess = (axis_value - min) / dist;
        let mut last_tangent = None;
        for _ in 0..5 {
            let dv = self.sample(guess).get_axis(axis_index)? - axis_value;
            if dv.abs() < EPSILON {
                return Some(guess);
            }
            let tangent = self.sample_tangent(guess);
            let slope = if let Some(last_tangent) = last_tangent {
                tangent.slope(&last_tangent)
            } else {
                1.0
            };
            last_tangent = Some(tangent);
            guess -= dv * slope;
        }
        Some(guess)
    }
}
