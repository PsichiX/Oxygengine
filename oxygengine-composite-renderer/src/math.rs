use core::{Ignite, Scalar};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

#[inline]
pub fn lerp(a: Scalar, b: Scalar, f: Scalar) -> Scalar {
    (b - a) * f + a
}

#[inline]
pub fn lerp_clamped(a: Scalar, b: Scalar, f: Scalar) -> Scalar {
    lerp(a, b, f.max(0.0).min(1.0))
}

#[inline]
pub fn unlerp(a: Scalar, b: Scalar, v: Scalar) -> Scalar {
    (v - a) / (b - a)
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mat2d(pub [Scalar; 6]);

impl Default for Mat2d {
    fn default() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
}

impl Mat2d {
    pub fn new(cells: [Scalar; 6]) -> Self {
        Self(cells)
    }

    pub fn translation(value: Vec2) -> Self {
        Self([1.0, 0.0, 0.0, 1.0, value.x, value.y])
    }

    pub fn rotation(value: Scalar) -> Self {
        let (sin, cos) = value.sin_cos();
        Self([cos, sin, -sin, cos, 0.0, 0.0])
    }

    pub fn scale(value: Vec2) -> Self {
        Self([value.x, 0.0, 0.0, value.y, 0.0, 0.0])
    }

    pub fn inverse(self) -> Option<Self> {
        !self
    }
}

impl Mul for Mat2d {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self([
            self.0[0] * other.0[0] + self.0[2] * other.0[1],
            self.0[1] * other.0[0] + self.0[3] * other.0[1],
            self.0[0] * other.0[2] + self.0[2] * other.0[3],
            self.0[1] * other.0[2] + self.0[3] * other.0[3],
            self.0[0] * other.0[4] + self.0[2] * other.0[5] + self.0[4],
            self.0[1] * other.0[4] + self.0[3] * other.0[5] + self.0[5],
        ])
    }
}

impl Mul<Vec2> for Mat2d {
    type Output = Vec2;

    fn mul(self, other: Vec2) -> Vec2 {
        other * self
    }
}

impl Not for Mat2d {
    type Output = Option<Self>;

    fn not(self) -> Option<Self> {
        let det = self.0[0] * self.0[3] - self.0[1] * self.0[2];
        if det == 0.0 {
            return None;
        }
        let det = 1.0 / det;
        Some(Self([
            self.0[3] * det,
            -self.0[1] * det,
            -self.0[2] * det,
            self.0[0] * det,
            (self.0[2] * self.0[5] - self.0[3] * self.0[4]) * det,
            (self.0[1] * self.0[4] - self.0[0] * self.0[5]) * det,
        ]))
    }
}

impl From<[Scalar; 6]> for Mat2d {
    fn from(value: [Scalar; 6]) -> Self {
        Self(value)
    }
}

impl From<(Scalar, Scalar, Scalar, Scalar, Scalar, Scalar)> for Mat2d {
    #[allow(clippy::many_single_char_names)]
    fn from((a, b, c, d, e, f): (Scalar, Scalar, Scalar, Scalar, Scalar, Scalar)) -> Self {
        Self([a, b, c, d, e, f])
    }
}

impl Into<[Scalar; 6]> for Mat2d {
    fn into(self) -> [Scalar; 6] {
        self.0
    }
}

impl Into<(Scalar, Scalar, Scalar, Scalar, Scalar, Scalar)> for Mat2d {
    fn into(self) -> (Scalar, Scalar, Scalar, Scalar, Scalar, Scalar) {
        (
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn transparent() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    pub fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }

    pub fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn red() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn green() -> Self {
        Self {
            r: 0,
            g: 255,
            b: 0,
            a: 255,
        }
    }

    pub fn blue() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        }
    }

    pub fn yellow() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        }
    }

    pub fn cyan() -> Self {
        Self {
            r: 0,
            g: 255,
            b: 255,
            a: 255,
        }
    }

    pub fn magenta() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
            a: 255,
        }
    }

    pub fn r(mut self, value: u8) -> Self {
        self.r = value;
        self
    }

    pub fn g(mut self, value: u8) -> Self {
        self.g = value;
        self
    }

    pub fn b(mut self, value: u8) -> Self {
        self.b = value;
        self
    }

    pub fn a(mut self, value: u8) -> Self {
        self.a = value;
        self
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            Scalar::from(self.a) / 255.0
        )
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vec2 {
    #[inline]
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    #[inline]
    pub fn sqr_magnitude(self) -> Scalar {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn magnitude(self) -> Scalar {
        self.sqr_magnitude().sqrt()
    }

    #[inline]
    pub fn normalized(self) -> Self {
        self / self.magnitude()
    }

    #[inline]
    pub fn dot(self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn right(self) -> Vec2 {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    #[inline]
    pub fn is_clockwise(a: Self, b: Self) -> bool {
        a.x * b.y - a.y * b.x > 0.0
    }

    #[inline]
    pub fn lerp(self, other: Vec2, factor: Scalar) -> Self {
        Self {
            x: lerp(self.x, other.x, factor),
            y: lerp(self.y, other.y, factor),
        }
    }

    #[inline]
    pub fn lerp_clamped(self, other: Vec2, factor: Scalar) -> Self {
        Self {
            x: lerp_clamped(self.x, other.x, factor),
            y: lerp_clamped(self.y, other.y, factor),
        }
    }

    #[inline]
    pub fn project_distance(self, from: Vec2, to: Vec2) -> Scalar {
        let u = self - from;
        let v = to - from;
        u.dot(v) / v.sqr_magnitude()
    }

    #[inline]
    pub fn project(self, from: Vec2, to: Vec2) -> Vec2 {
        (to - from) * self.project_distance(from, to)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<Scalar> for Vec2 {
    type Output = Self;

    fn add(self, other: Scalar) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<Scalar> for Vec2 {
    type Output = Self;

    fn sub(self, other: Scalar) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<Scalar> for Vec2 {
    type Output = Self;

    fn mul(self, other: Scalar) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<Mat2d> for Vec2 {
    type Output = Self;

    fn mul(self, other: Mat2d) -> Self {
        Self {
            x: other.0[0] * self.x + other.0[2] * self.y + other.0[4],
            y: other.0[1] * self.x + other.0[3] * self.y + other.0[5],
        }
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Div<Scalar> for Vec2 {
    type Output = Self;

    fn div(self, other: Scalar) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl From<Scalar> for Vec2 {
    fn from(value: Scalar) -> Self {
        Self { x: value, y: value }
    }
}

impl From<(Scalar, Scalar)> for Vec2 {
    fn from(value: (Scalar, Scalar)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<[Scalar; 2]> for Vec2 {
    fn from(value: [Scalar; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: Scalar,
    pub y: Scalar,
    pub w: Scalar,
    pub h: Scalar,
}

impl Rect {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            x: position.x,
            y: position.y,
            w: size.x,
            h: size.y,
        }
    }

    pub fn with_size(size: Vec2) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: size.x,
            h: size.y,
        }
    }

    pub fn bounding(points: &[Vec2]) -> Option<Self> {
        points.iter().fold(None, |a, v| {
            if let Some(a) = a {
                Some(a.include(*v))
            } else {
                Some(Rect::new(*v, Vec2::zero()))
            }
        })
    }

    pub fn align(&self, factor: Vec2) -> Self {
        Self {
            x: self.x - self.w * factor.x,
            y: self.y - self.h * factor.y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn expand(&self, thickness: Scalar) -> Self {
        let tt = thickness * 2.0;
        let mut result = Self {
            x: self.x - thickness,
            y: self.y - thickness,
            w: self.w + tt,
            h: self.h + tt,
        };
        result.validate();
        result
    }

    pub fn include(&self, point: Vec2) -> Self {
        let (x, w) = if point.x < self.x {
            (point.x, self.w + self.x - point.x)
        } else if point.x > self.x + self.w {
            (self.x, point.x - self.x)
        } else {
            (self.x, self.w)
        };
        let (y, h) = if point.y < self.y {
            (point.y, self.h + self.y - point.y)
        } else if point.y > self.y + self.h {
            (self.y, point.y - self.y)
        } else {
            (self.y, self.h)
        };
        Self { x, y, w, h }
    }

    pub fn validate(&mut self) {
        if self.w < 0.0 {
            self.x += self.w;
            self.w = -self.w;
        }
        if self.h < 0.0 {
            self.y += self.h;
            self.h = -self.h;
        }
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x > self.x
            && point.x < self.x + self.w
            && point.y > self.y
            && point.y < self.y + self.h
    }
}

impl From<(Scalar, Scalar)> for Rect {
    fn from(value: (Scalar, Scalar)) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: value.0,
            h: value.1,
        }
    }
}

impl From<[Scalar; 2]> for Rect {
    fn from(value: [Scalar; 2]) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: value[0],
            h: value[1],
        }
    }
}

impl From<(Scalar, Scalar, Scalar, Scalar)> for Rect {
    fn from(value: (Scalar, Scalar, Scalar, Scalar)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            w: value.2,
            h: value.3,
        }
    }
}

impl From<[Scalar; 4]> for Rect {
    fn from(value: [Scalar; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            w: value[2],
            h: value[3],
        }
    }
}
