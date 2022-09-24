use crate::{curve::*, range_iter};
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
    T: Curved,
{
    pub point: T,
    #[serde(default)]
    pub direction: SplinePointDirection<T>,
}

impl<T> SplinePoint<T>
where
    T: Curved,
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
    T: Curved,
{
    fn from(value: T) -> Self {
        Self::point(value)
    }
}

impl<T> From<(T, T)> for SplinePoint<T>
where
    T: Curved,
{
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, SplinePointDirection::Single(value.1))
    }
}

impl<T> From<(T, T, T)> for SplinePoint<T>
where
    T: Curved,
{
    fn from(value: (T, T, T)) -> Self {
        Self::new(value.0, SplinePointDirection::InOut(value.1, value.2))
    }
}

impl<T> From<[T; 2]> for SplinePoint<T>
where
    T: Curved,
{
    fn from(value: [T; 2]) -> Self {
        let [a, b] = value;
        Self::new(a, SplinePointDirection::Single(b))
    }
}

impl<T> From<[T; 3]> for SplinePoint<T>
where
    T: Curved,
{
    fn from(value: [T; 3]) -> Self {
        let [a, b, c] = value;
        Self::new(a, SplinePointDirection::InOut(b, c))
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SplineError {
    EmptyPointsList,
    /// (points pair index, curve error)
    Curve(usize, CurveError),
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
    T: Default + Clone + Curved + CurvedChange,
{
    points: Vec<SplinePoint<T>>,
    cached: Vec<Curve<T>>,
    length: Scalar,
    parts_times_values: Vec<(Scalar, T)>,
}

impl<T> Default for Spline<T>
where
    T: Default + Clone + Curved + CurvedChange,
{
    fn default() -> Self {
        Self::point(T::zero()).unwrap()
    }
}

impl<T> Spline<T>
where
    T: Default + Clone + Curved + CurvedChange,
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
            .enumerate()
            .map(|(index, pair)| {
                let from_direction = match &pair[0].direction {
                    SplinePointDirection::Single(dir) => dir.clone(),
                    SplinePointDirection::InOut(_, dir) => dir.negate(),
                };
                let to_direction = match &pair[1].direction {
                    SplinePointDirection::Single(dir) => dir.negate(),
                    SplinePointDirection::InOut(dir, _) => dir.clone(),
                };
                let from_param = pair[0].point.offset(&from_direction);
                let to_param = pair[1].point.offset(&to_direction);
                Curve::bezier(
                    pair[0].point.clone(),
                    from_param,
                    to_param,
                    pair[1].point.clone(),
                )
                .map_err(|error| SplineError::Curve(index, error))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let lengths = cached
            .iter()
            .map(|curve| curve.length())
            .collect::<Vec<_>>();
        let mut time = 0.0;
        let mut parts_times_values = Vec::with_capacity(points.len());
        parts_times_values.push((0.0, points[0].point.clone()));
        for (length, point) in lengths.iter().zip(points.iter().skip(1)) {
            time += length;
            parts_times_values.push((time, point.point.clone()));
        }
        Ok(Self {
            points,
            cached,
            length: time,
            parts_times_values,
        })
    }

    pub fn linear(from: T, to: T) -> Result<Self, SplineError> {
        Self::new(vec![SplinePoint::point(from), SplinePoint::point(to)])
    }

    pub fn point(point: T) -> Result<Self, SplineError> {
        Self::linear(point.clone(), point)
    }

    pub fn value_along_axis_iter(
        &self,
        steps: usize,
        axis_index: usize,
    ) -> Option<impl Iterator<Item = Scalar>> {
        let from = self.points.first()?.point.get_axis(axis_index)?;
        let to = self.points.last()?.point.get_axis(axis_index)?;
        Some(range_iter(steps, from, to))
    }

    pub fn sample(&self, factor: Scalar) -> T {
        let (index, factor) = self.find_curve_index_factor(factor);
        self.cached[index].sample(factor)
    }

    pub fn sample_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T> {
        let index = self.find_curve_index_by_axis_value(axis_value, axis_index)?;
        self.cached[index].sample_along_axis(axis_value, axis_index)
    }

    pub fn sample_direction(&self, factor: Scalar) -> T {
        let (index, factor) = self.find_curve_index_factor(factor);
        self.cached[index].sample_direction(factor)
    }

    pub fn sample_direction_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T> {
        let index = self.find_curve_index_by_axis_value(axis_value, axis_index)?;
        self.cached[index].sample_direction_along_axis(axis_value, axis_index)
    }

    pub fn sample_tangent(&self, factor: Scalar) -> T {
        let (index, factor) = self.find_curve_index_factor(factor);
        self.cached[index].sample_tangent(factor)
    }

    pub fn sample_tangent_along_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<T> {
        let index = self.find_curve_index_by_axis_value(axis_value, axis_index)?;
        self.cached[index].sample_tangent_along_axis(axis_value, axis_index)
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

    pub fn find_curve_index_factor(&self, mut factor: Scalar) -> (usize, Scalar) {
        factor = factor.max(0.0).min(1.0);
        let t = factor * self.length;
        let index = match self
            .parts_times_values
            .binary_search_by(|(time, _)| time.partial_cmp(&t).unwrap())
        {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let index = index.min(self.cached.len().saturating_sub(1));
        let start = self.parts_times_values[index].0;
        let length = self.parts_times_values[index + 1].0 - start;
        let factor = if length > 0.0 {
            (t - start) / length
        } else {
            1.0
        };
        (index, factor)
    }

    pub fn find_curve_index_by_axis_value(
        &self,
        mut axis_value: Scalar,
        axis_index: usize,
    ) -> Option<usize> {
        let min = self.points.first().unwrap().point.get_axis(axis_index)?;
        let max = self.points.last().unwrap().point.get_axis(axis_index)?;
        axis_value = axis_value.max(min).min(max);
        let index = match self.parts_times_values.binary_search_by(|(_, value)| {
            value
                .get_axis(axis_index)
                .unwrap()
                .partial_cmp(&axis_value)
                .unwrap()
        }) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        Some(index.min(self.cached.len().saturating_sub(1)))
    }

    pub fn find_time_for_axis(&self, axis_value: Scalar, axis_index: usize) -> Option<Scalar> {
        let index = self.find_curve_index_by_axis_value(axis_value, axis_index)?;
        self.cached[index].find_time_for_axis(axis_value, axis_index)
    }
}

impl<T> TryFrom<SplineDef<T>> for Spline<T>
where
    T: Default + Clone + Curved + CurvedChange,
{
    type Error = SplineError;

    fn try_from(value: SplineDef<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T> From<Spline<T>> for SplineDef<T>
where
    T: Default + Clone + Curved + CurvedChange,
{
    fn from(v: Spline<T>) -> Self {
        v.points
    }
}
