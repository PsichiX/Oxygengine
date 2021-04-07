use crate::curve::*;
use core::Scalar;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplinePointDirection<T>
where
    T: Curved,
{
    Single(T),
    InOut(T, T),
}

impl<T> Default for SplinePointDirection<T>
where
    T: Curved,
{
    fn default() -> Self {
        Self::Single(T::zero())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SplinePoint<T>
where
    T: Clone + Curved,
{
    pub point: T,
    #[serde(default)]
    pub direction: SplinePointDirection<T>,
}

impl<T> SplinePoint<T>
where
    T: Clone + Curved,
{
    pub fn point(point: T) -> Self {
        Self {
            point,
            direction: Default::default(),
        }
    }

    pub fn new(point: T, direction: SplinePointDirection<T>) -> Self {
        Self { point, direction }
    }
}

impl<T> From<T> for SplinePoint<T>
where
    T: Clone + Curved,
{
    fn from(value: T) -> Self {
        Self::point(value)
    }
}

impl<T> From<(T, T)> for SplinePoint<T>
where
    T: Clone + Curved,
{
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, SplinePointDirection::Single(value.1))
    }
}

impl<T> From<(T, T, T)> for SplinePoint<T>
where
    T: Clone + Curved,
{
    fn from(value: (T, T, T)) -> Self {
        Self::new(value.0, SplinePointDirection::InOut(value.1, value.2))
    }
}

impl<T> From<[T; 2]> for SplinePoint<T>
where
    T: Clone + Curved,
{
    fn from(value: [T; 2]) -> Self {
        let [a, b] = value;
        Self::new(a, SplinePointDirection::Single(b))
    }
}

impl<T> From<[T; 3]> for SplinePoint<T>
where
    T: Clone + Curved,
{
    fn from(value: [T; 3]) -> Self {
        let [a, b, c] = value;
        Self::new(a, SplinePointDirection::InOut(b, c))
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SplineError {
    EmptyPointsList,
}

impl fmt::Display for SplineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type SplineDef<T> = Vec<SplinePoint<T>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SplineDef<T>")]
#[serde(into = "SplineDef<T>")]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Spline<T>
where
    T: Default + Clone + Curved + CurvedDistance + CurvedOffset + CurvedTangent,
{
    #[serde(skip)]
    points: Vec<SplinePoint<T>>,
    #[serde(skip)]
    cached: Vec<Curve<T>>,
    #[serde(skip)]
    length: Scalar,
    #[serde(skip)]
    parts_times: Vec<Scalar>,
}

impl<T> Default for Spline<T>
where
    T: Default + Clone + Curved + CurvedDistance + CurvedOffset + CurvedTangent,
{
    fn default() -> Self {
        Self::point(T::zero()).unwrap()
    }
}

impl<T> Spline<T>
where
    T: Default + Clone + Curved + CurvedDistance + CurvedOffset + CurvedTangent,
{
    pub fn new(mut points: Vec<SplinePoint<T>>) -> Result<Self, SplineError> {
        if points.is_empty() {
            return Err(SplineError::EmptyPointsList);
        }
        if points.len() == 1 {
            points.push(points[0].clone())
        }
        let cached = points
            .windows(2)
            .map(|pair| {
                let from_direction = match &pair[0].direction {
                    SplinePointDirection::Single(dir) => dir.clone(),
                    SplinePointDirection::InOut(_, dir) => dir.negate(),
                };
                let to_direction = match &pair[1].direction {
                    SplinePointDirection::Single(dir) => dir.negate(),
                    SplinePointDirection::InOut(dir, _) => dir.clone(),
                };
                let from_param = pair[0].point.curved_offset(&from_direction);
                let to_param = pair[1].point.curved_offset(&to_direction);
                Curve::bezier(
                    pair[0].point.clone(),
                    from_param,
                    to_param,
                    pair[1].point.clone(),
                )
            })
            .collect::<Vec<_>>();
        let lengths = cached
            .iter()
            .map(|curve| curve.length())
            .collect::<Vec<_>>();
        let mut time = 0.0;
        let mut parts_times = Vec::with_capacity(points.len());
        parts_times.push(0.0);
        for length in &lengths {
            time += length;
            parts_times.push(time);
        }
        Ok(Self {
            points,
            cached,
            length: time,
            parts_times,
        })
    }

    pub fn linear(from: T, to: T) -> Result<Self, SplineError> {
        Self::new(vec![SplinePoint::point(from), SplinePoint::point(to)])
    }

    pub fn point(point: T) -> Result<Self, SplineError> {
        Self::linear(point.clone(), point)
    }

    pub fn sample(&self, mut factor: Scalar) -> T {
        factor = factor.max(0.0).min(1.0);
        let t = factor * self.length;
        let index = match self
            .parts_times
            .binary_search_by(|time| time.partial_cmp(&t).unwrap())
        {
            Ok(index) | Err(index) => {
                if index > 0 {
                    index - 1
                } else {
                    index
                }
            }
        };
        let index = index.min(self.cached.len() - 1);
        let a = self.parts_times[index];
        let length = self.parts_times[index + 1] - a;
        let f = if length > 0.0 { (t - a) / length } else { 1.0 };
        self.cached[index].sample(f)
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
        let from = self.points.first()?.point.get_axis(axis_index)?;
        let diff = self.points.last()?.point.get_axis(axis_index)? - from;
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

    pub fn length(&self) -> Scalar {
        self.length
    }

    pub fn points(&self) -> &[SplinePoint<T>] {
        &self.points
    }

    pub fn set_points(&mut self, points: Vec<SplinePoint<T>>) {
        if let Ok(result) = Self::new(points) {
            *self = result;
        }
    }

    pub fn curves(&self) -> &[Curve<T>] {
        &self.cached
    }

    pub fn find_time_for_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<Scalar> {
        if (self.points.last().unwrap().point.get_axis(axis_index)?
            - self.points.first().unwrap().point.get_axis(axis_index)?)
        .abs()
            < 1.0e-4
        {
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
            if dv.abs() < 1.0e-4 {
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

impl<T> TryFrom<SplineDef<T>> for Spline<T>
where
    T: Default + Clone + Curved + CurvedDistance + CurvedOffset + CurvedTangent,
{
    type Error = SplineError;

    fn try_from(value: SplineDef<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T> From<Spline<T>> for SplineDef<T>
where
    T: Default + Clone + Curved + CurvedDistance + CurvedOffset + CurvedTangent,
{
    fn from(v: Spline<T>) -> Self {
        v.points
    }
}
