use crate::{resource::ZERO_TRESHOLD, Scalar};
use serde::{Deserialize, Serialize};
use spade::PointN;
use std::ops::{Add, Div, Mul, Sub};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavVec3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl NavVec3 {
    #[inline]
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn sqr_magnitude(self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn magnitude(self) -> Scalar {
        self.sqr_magnitude().sqrt()
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.magnitude();
        if len < ZERO_TRESHOLD {
            Self::new(0.0, 0.0, 0.0)
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    #[inline]
    pub fn project(self, from: Self, to: Self) -> Scalar {
        let diff = to - from;
        (self - from).dot(diff) / diff.sqr_magnitude()
    }

    #[inline]
    pub fn unproject(from: Self, to: Self, t: Scalar) -> Self {
        let diff = to - from;
        from + Self::new(diff.x * t, diff.y * t, diff.z * t)
    }

    #[inline]
    pub fn is_above_plane(self, origin: Self, normal: Self) -> bool {
        normal.dot(self - origin) > -ZERO_TRESHOLD
    }

    pub fn project_on_plane(self, origin: Self, normal: Self) -> Self {
        let v = self - origin;
        let n = normal.normalize();
        let dot = v.dot(n);
        let d = NavVec3::new(normal.x * dot, normal.y * dot, normal.z * dot);
        self - d
    }

    pub fn is_line_between_points(from: Self, to: Self, a: Self, b: Self, normal: Self) -> bool {
        let n = (to - from).cross(normal);
        let sa = Self::side(n.dot(a - from));
        let sb = Self::side(n.dot(b - from));
        sa != sb
    }

    fn side(v: Scalar) -> i8 {
        if v.abs() < ZERO_TRESHOLD {
            0
        } else {
            v.signum() as i8
        }
    }
}

impl From<(Scalar, Scalar, Scalar)> for NavVec3 {
    fn from(value: (Scalar, Scalar, Scalar)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

impl From<(Scalar, Scalar)> for NavVec3 {
    fn from(value: (Scalar, Scalar)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: 0.0,
        }
    }
}

impl Add for NavVec3 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn add(self, other: Scalar) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
            z: self.z + other,
        }
    }
}

impl Sub for NavVec3 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Scalar) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
            z: self.z - other,
        }
    }
}

impl Mul for NavVec3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Mul<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Scalar) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Div for NavVec3 {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Div<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn div(self, other: Scalar) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl PointN for NavVec3 {
    type Scalar = Scalar;

    fn dimensions() -> usize {
        3
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => unreachable!(),
        }
    }

    fn from_value(value: Self::Scalar) -> Self {
        NavVec3::new(value, value, value)
    }
}
