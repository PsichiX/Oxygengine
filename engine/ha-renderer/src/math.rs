use core::Scalar;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Deref, Sub};

#[cfg(not(feature = "scalar64"))]
use std::f32::consts::PI;
#[cfg(feature = "scalar64")]
use std::f64::consts::PI;

pub use vek;
pub use vek::*;

pub type Rect = vek::Rect<Scalar, Scalar>;
pub type Rect3 = vek::Rect3<Scalar, Scalar>;
pub type Vec2 = vek::Vec2<Scalar>;
pub type Vec3 = vek::Vec3<Scalar>;
pub type Vec4 = vek::Vec4<Scalar>;
pub type Rgba = vek::Rgba<Scalar>;
pub type Quat = vek::Quaternion<Scalar>;
pub type Mat2 = vek::Mat2<Scalar>;
pub type Mat3 = vek::Mat3<Scalar>;
pub type Mat4 = vek::Mat4<Scalar>;
pub type Transform = vek::Transform<Scalar, Scalar, Scalar>;

pub fn rect<T>(x: T, y: T, width: T, height: T) -> vek::Rect<T, T> {
    vek::Rect::new(x, y, width, height)
}

pub fn rect3<T>(x: T, y: T, z: T, width: T, height: T, depth: T) -> vek::Rect3<T, T> {
    vek::Rect3::new(x, y, z, width, height, depth)
}

pub fn vec2<T>(x: T, y: T) -> vek::Vec2<T> {
    vek::Vec2::new(x, y)
}

pub fn vec3<T>(x: T, y: T, z: T) -> vek::Vec3<T> {
    vek::Vec3::new(x, y, z)
}

pub fn vec4<T>(x: T, y: T, z: T, w: T) -> vek::Vec4<T> {
    vek::Vec4::new(x, y, z, w)
}

pub fn mat2<T>(v: [[T; 2]; 2]) -> vek::Mat2<T> {
    vek::Mat2::<T>::from_col_arrays(v)
}

pub fn mat3<T>(v: [[T; 3]; 3]) -> vek::Mat3<T> {
    vek::Mat3::<T>::from_col_arrays(v)
}

pub fn mat4<T>(v: [[T; 4]; 4]) -> vek::Mat4<T> {
    vek::Mat4::<T>::from_col_arrays(v)
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Eulers {
    /// Z degrees
    #[serde(default)]
    pub yaw: Scalar,
    /// Y degrees
    #[serde(default)]
    pub pitch: Scalar,
    /// X degrees
    #[serde(default)]
    pub roll: Scalar,
}

impl Eulers {
    pub fn new(yaw: Scalar, pitch: Scalar, roll: Scalar) -> Self {
        Self { yaw, pitch, roll }
    }

    pub fn yaw(yaw: Scalar) -> Self {
        Self {
            yaw,
            pitch: 0.0,
            roll: 0.0,
        }
    }

    pub fn pitch(pitch: Scalar) -> Self {
        Self {
            yaw: 0.0,
            pitch,
            roll: 0.0,
        }
    }

    pub fn roll(roll: Scalar) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll,
        }
    }

    pub fn with_yaw(mut self, degrees: Scalar) -> Self {
        self.yaw = degrees;
        self
    }

    pub fn with_pitch(mut self, degrees: Scalar) -> Self {
        self.pitch = degrees;
        self
    }

    pub fn with_roll(mut self, degrees: Scalar) -> Self {
        self.roll = degrees;
        self
    }
}

impl Add for Eulers {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            yaw: self.yaw + other.yaw,
            pitch: self.pitch + other.pitch,
            roll: self.roll + other.roll,
        }
    }
}

impl Sub for Eulers {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            yaw: self.yaw - other.yaw,
            pitch: self.pitch - other.pitch,
            roll: self.roll - other.roll,
        }
    }
}

impl From<Vec3> for Eulers {
    fn from(v: Vec3) -> Self {
        Eulers {
            yaw: v.z,
            pitch: v.y,
            roll: v.x,
        }
    }
}

impl From<Eulers> for Vec3 {
    fn from(value: Eulers) -> Self {
        Self::new(value.roll, value.pitch, value.yaw)
    }
}

impl From<Quat> for Eulers {
    fn from(q: Quat) -> Self {
        let q = q.normalized();
        let sinr_cosp = 2.0 * (q.w * q.x + q.y * q.z);
        let cosr_cosp = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
        let roll = sinr_cosp.atan2(cosr_cosp).to_degrees();
        let sinp = 2.0 * (q.w * q.y - q.z * q.x);
        let pitch = if sinp.abs() >= 1.0 {
            PI * 0.5 * sinp.signum()
        } else {
            sinp.asin()
        }
        .to_degrees();
        let siny_cosp = 2.0 * (q.w * q.z + q.x * q.y);
        let cosy_cosp = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
        let yaw = siny_cosp.atan2(cosy_cosp).to_degrees();
        Eulers { yaw, pitch, roll }
    }
}

impl From<Eulers> for Quat {
    #[allow(clippy::many_single_char_names)]
    fn from(value: Eulers) -> Self {
        let v = Vec3::new(
            value.roll.to_radians(),
            value.pitch.to_radians(),
            value.yaw.to_radians(),
        ) * 0.5;
        let (sy, cy) = v.z.sin_cos();
        let (sp, cp) = v.y.sin_cos();
        let (sr, cr) = v.x.sin_cos();
        let w = cr * cp * cy + sr * sp * sy;
        let x = sr * cp * cy - cr * sp * sy;
        let y = cr * sp * cy + sr * cp * sy;
        let z = cr * cp * sy - sr * sp * cy;
        Self::from_xyzw(x, y, z, w)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RotatorDef(pub Eulers);

impl From<Rotator> for RotatorDef {
    fn from(v: Rotator) -> Self {
        Self(v.eulers())
    }
}

impl From<RotatorDef> for Rotator {
    fn from(v: RotatorDef) -> Self {
        v.0.into()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RotatorDef")]
#[serde(into = "RotatorDef")]
pub struct Rotator {
    quat: Quat,
    eulers: Eulers,
}

impl Rotator {
    pub fn quat(&self) -> Quat {
        self.quat
    }

    pub fn set_quat(&mut self, value: Quat) {
        self.quat = value;
        self.eulers = value.into();
    }

    pub fn with_quat<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Quat),
    {
        f(&mut self.quat);
        self.eulers = self.quat.into();
    }

    pub fn eulers(&self) -> Eulers {
        self.eulers
    }

    pub fn set_eulers(&mut self, value: Eulers) {
        self.quat = value.into();
        self.eulers = value;
    }

    pub fn with_eulers<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Eulers),
    {
        f(&mut self.eulers);
        self.quat = self.eulers.into();
    }

    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.quat * direction
    }

    pub fn interpolate(from: &Self, to: &Self, factor: Scalar) -> Self {
        Quat::slerp(from.quat, to.quat, factor).into()
    }

    pub fn interpolate_many(iter: impl Iterator<Item = (Self, Scalar)>) -> Option<Self> {
        let mut result = None;
        for (value, weight) in iter {
            let quat = Quat::slerp(Quat::identity(), value.quat(), weight);
            result = match result {
                Some(result) => Some(result * quat),
                None => Some(quat),
            }
        }
        result.map(|result| result.into())
    }

    pub fn look_at(mut forward: Vec3, mut up: Vec3) -> Self {
        forward = forward.normalized();
        up = up.normalized();
        let right = up.cross(forward).normalized();
        up = forward.cross(right);
        let result = Mat3::new(
            forward.x, right.x, up.x, forward.y, right.y, up.y, forward.z, right.z, up.z,
        );
        result.into()
    }
}

impl Add for Rotator {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        (self.eulers() + other.eulers()).into()
    }
}

impl Sub for Rotator {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        (self.eulers() - other.eulers()).into()
    }
}

impl Deref for Rotator {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.quat
    }
}

impl From<Mat3> for Rotator {
    fn from(m: Mat3) -> Self {
        let right = m.cols.x;
        let up = m.cols.y;
        let forward = m.cols.z;
        let trace = m.trace();
        if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            let x = (up.z - forward.y) * s;
            let y = (forward.x - right.z) * s;
            let z = (right.y - up.x) * s;
            let w = 0.25 / s;
            Quat::from_xyzw(x, y, z, w)
        } else if right.x > up.y && right.x > forward.z {
            let s = 2.0 * (1.0 + right.x - up.y - forward.z).sqrt();
            let x = 0.25 * s;
            let y = (up.x + right.y) / s;
            let z = (forward.x + right.z) / s;
            let w = (up.z - forward.y) / s;
            Quat::from_xyzw(x, y, z, w)
        } else if up.y > forward.z {
            let s = 2.0 * (1.0 + up.y - right.x - forward.z).sqrt();
            let x = (up.x + right.y) / s;
            let y = 0.25 * s;
            let z = (forward.y + up.z) / s;
            let w = (forward.x - right.z) / s;
            Quat::from_xyzw(x, y, z, w)
        } else {
            let s = 2.0 * (1.0 + forward.z - right.x - up.y).sqrt();
            let x = (forward.x + right.z) / s;
            let y = (forward.y + up.z) / s;
            let z = 0.25 * s;
            let w = (right.y - up.x) / s;
            Quat::from_xyzw(x, y, z, w)
        }
        .normalized()
        .into()
    }
}

impl From<Rotator> for Mat3 {
    fn from(value: Rotator) -> Self {
        value.quat().into()
    }
}

impl From<Quat> for Rotator {
    fn from(value: Quat) -> Self {
        Self {
            quat: value,
            eulers: value.into(),
        }
    }
}

impl From<Rotator> for Quat {
    fn from(value: Rotator) -> Self {
        value.quat()
    }
}

impl From<Eulers> for Rotator {
    fn from(value: Eulers) -> Self {
        Self {
            quat: value.into(),
            eulers: value,
        }
    }
}

impl From<Rotator> for Eulers {
    fn from(value: Rotator) -> Self {
        value.eulers()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundsVolume {
    pub origin: Vec3,
    radius: Scalar,
    half_extents: Vec3,
}

impl BoundsVolume {
    pub fn from_sphere(origin: Vec3, radius: Scalar) -> Self {
        let size = ((radius * radius) / 3.0).sqrt();
        let half_extents = Vec3::new(size, size, size);
        Self {
            origin,
            radius,
            half_extents,
        }
    }

    pub fn from_box(origin: Vec3, mut half_extents: Vec3) -> Self {
        half_extents.x = half_extents.x.abs();
        half_extents.y = half_extents.y.abs();
        half_extents.z = half_extents.z.abs();
        let radius = half_extents.magnitude();
        Self {
            origin,
            radius,
            half_extents,
        }
    }

    pub fn from_points_cloud(iter: impl Iterator<Item = Vec3>) -> Option<Self> {
        let mut limits = None;
        for point in iter {
            limits = Some(match limits {
                Some((from, to)) => (Vec3::partial_min(from, point), Vec3::partial_max(to, point)),
                None => (point, point),
            });
        }
        limits.map(|(from, to)| Self::from_box((from + to) * 0.5, (to - from) * 0.5))
    }

    pub fn radius(&self) -> Scalar {
        self.radius
    }

    pub fn half_extents(&self) -> Vec3 {
        self.half_extents
    }

    pub fn closest_point_with_box(&self, position: Vec3) -> Vec3 {
        Vec3::partial_max(
            self.origin - self.half_extents,
            Vec3::partial_min(self.origin + self.half_extents, position),
        )
    }

    pub fn closest_point_with_sphere(&self, position: Vec3) -> Vec3 {
        let diff = position - self.origin;
        if diff.magnitude() > self.radius {
            self.origin + diff.normalized() * self.radius
        } else {
            position
        }
    }

    pub fn overlap_point_with_box(&self, position: Vec3) -> bool {
        let diff = position - self.origin;
        diff.x.abs() <= self.half_extents.x
            && diff.y.abs() <= self.half_extents.y
            && diff.z.abs() <= self.half_extents.z
    }

    pub fn overlap_point_with_sphere(&self, position: Vec3) -> bool {
        let distance = Vec3::distance(position, self.origin);
        distance <= self.radius
    }

    pub fn overlap_spheres(&self, other: &Self) -> bool {
        let distance = (self.origin - other.origin).magnitude_squared();
        let threshold = self.radius * self.radius + other.radius * other.radius;
        distance <= threshold
    }

    pub fn overlap_boxes(&self, other: &Self) -> bool {
        let from_a = self.origin - self.half_extents;
        let to_a = self.origin + self.half_extents;
        let from_b = other.origin - other.half_extents;
        let to_b = other.origin + other.half_extents;
        to_a.x > from_b.x
            && from_a.x < to_b.x
            && to_a.y > from_b.y
            && from_a.y < to_b.y
            && to_a.z > from_b.z
            && from_a.z < to_b.z
    }

    pub fn box_vertices(&self) -> [Vec3; 8] {
        let he = self.half_extents;
        [
            self.origin + Vec3::new(-he.x, -he.y, -he.z),
            self.origin + Vec3::new(he.x, -he.y, -he.z),
            self.origin + Vec3::new(he.x, he.y, -he.z),
            self.origin + Vec3::new(-he.x, he.y, -he.z),
            self.origin + Vec3::new(-he.x, -he.y, he.z),
            self.origin + Vec3::new(he.x, -he.y, he.z),
            self.origin + Vec3::new(he.x, he.y, he.z),
            self.origin + Vec3::new(-he.x, he.y, he.z),
        ]
    }

    pub fn transformed(&self, matrix: Mat4) -> Option<Self> {
        Self::from_points_cloud(
            self.box_vertices()
                .into_iter()
                .map(|p| Vec3::from(matrix * Vec4::from(p))),
        )
    }

    pub fn distance_sphere(&self, position: Vec3) -> Scalar {
        (position - self.origin).magnitude() - self.radius
    }

    pub fn distance_box(&self, position: Vec3) -> Vec3 {
        let diff = position - self.origin;
        let x = diff.x.abs() - self.half_extents.x;
        let y = diff.y.abs() - self.half_extents.y;
        let z = diff.z.abs() - self.half_extents.z;
        Vec3::new(x, y, z)
    }

    pub fn distance_box_single(&self, position: Vec3) -> Scalar {
        let dist = self.distance_box(position).magnitude();
        if self.overlap_point_with_box(position) {
            -dist
        } else {
            dist
        }
    }
}
