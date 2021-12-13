use core::{Ignite, Scalar};
use serde::{Deserialize, Serialize};

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

pub fn mat2<T>(a: [[T; 2]; 2]) -> vek::Mat2<T> {
    vek::Mat2::<T>::from_col_arrays(a)
}

pub fn mat3<T>(a: [[T; 3]; 3]) -> vek::Mat3<T> {
    vek::Mat3::<T>::from_col_arrays(a)
}

pub fn mat4<T>(a: [[T; 4]; 4]) -> vek::Mat4<T> {
    vek::Mat4::<T>::from_col_arrays(a)
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Eulers {
    /// Z radians
    #[serde(default)]
    pub yaw: Scalar,
    /// Y radians
    #[serde(default)]
    pub pitch: Scalar,
    /// X radians
    #[serde(default)]
    pub roll: Scalar,
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
        let roll = sinr_cosp.atan2(cosr_cosp);
        let sinp = 2.0 * (q.w * q.y - q.z * q.x);
        let pitch = if sinp.abs() >= 1.0 {
            PI * 0.5 * sinp.signum()
        } else {
            sinp.asin()
        };
        let siny_cosp = 2.0 * (q.w * q.z + q.x * q.y);
        let cosy_cosp = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
        let yaw = siny_cosp.atan2(cosy_cosp);
        Eulers { yaw, pitch, roll }
    }
}

impl From<Eulers> for Quat {
    #[allow(clippy::many_single_char_names)]
    fn from(value: Eulers) -> Self {
        let v = Vec3::new(value.roll, value.pitch, value.yaw) * 0.5;
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

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
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

    pub fn overlap_point(&self, position: Vec3) -> bool {
        let diff = position - self.origin;
        diff.x.abs() <= self.half_extents.x
            && diff.y.abs() <= self.half_extents.y
            && diff.z.abs() <= self.half_extents.z
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
        if self.overlap_point(position) {
            -dist
        } else {
            dist
        }
    }
}
