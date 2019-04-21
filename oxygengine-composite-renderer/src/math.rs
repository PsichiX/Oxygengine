use std::ops::{Add, Div, Mul, Neg, Sub};

pub type Scalar = f32;

pub fn mul_mat(a: [Scalar; 6], b: [Scalar; 6]) -> [Scalar; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
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

    pub fn r(&self, value: u8) -> Self {
        let mut result = *self;
        result.r = value;
        result
    }

    pub fn g(&self, value: u8) -> Self {
        let mut result = *self;
        result.g = value;
        result
    }

    pub fn b(&self, value: u8) -> Self {
        let mut result = *self;
        result.b = value;
        result
    }

    pub fn a(&self, value: u8) -> Self {
        let mut result = *self;
        result.a = value;
        result
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
            self.a as f32 / 255.0
        )
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vec2 {
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub fn sqr_magnitude(&self) -> Scalar {
        self.x * self.x + self.y * self.y
    }

    pub fn magnitude(&self) -> Scalar {
        self.sqr_magnitude().sqrt()
    }

    pub fn normalized(&self) -> Self {
        *self / self.magnitude()
    }

    pub fn dot(&self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<Scalar> for Vec2 {
    type Output = Self;

    fn add(self, other: Scalar) -> Self {
        Vec2 {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<Scalar> for Vec2 {
    type Output = Self;

    fn sub(self, other: Scalar) -> Self {
        Vec2 {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<Scalar> for Vec2 {
    type Output = Self;

    fn mul(self, other: Scalar) -> Self {
        Vec2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Vec2 {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Div<Scalar> for Vec2 {
    type Output = Self;

    fn div(self, other: Scalar) -> Self {
        Vec2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Vec2 {
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

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Rect {
    pub x: Scalar,
    pub y: Scalar,
    pub w: Scalar,
    pub h: Scalar,
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
