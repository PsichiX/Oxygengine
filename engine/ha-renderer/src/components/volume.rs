use crate::math::*;
use core::{
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum HaVolume {
    /// (radius)
    Sphere(Scalar),
    /// (half extents)
    Box(Vec3),
}

impl HaVolume {
    pub fn world_space_contains(&self, matrix: &Mat4, position: Vec3) -> Option<Scalar> {
        match self {
            Self::Sphere(radius) => {
                let origin = matrix.mul_point(Vec3::zero());
                let distance = origin.distance(position);
                if distance > *radius {
                    None
                } else {
                    Some(radius - distance)
                }
            }
            Self::Box(half_extents) => {
                let points = [
                    matrix.mul_point(Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z)),
                    matrix.mul_point(Vec3::new(half_extents.x, -half_extents.y, -half_extents.z)),
                    matrix.mul_point(Vec3::new(half_extents.x, half_extents.y, -half_extents.z)),
                    matrix.mul_point(Vec3::new(-half_extents.x, half_extents.y, -half_extents.z)),
                    matrix.mul_point(Vec3::new(-half_extents.x, -half_extents.y, half_extents.z)),
                    matrix.mul_point(Vec3::new(half_extents.x, -half_extents.y, half_extents.z)),
                    matrix.mul_point(Vec3::new(half_extents.x, half_extents.y, half_extents.z)),
                    matrix.mul_point(Vec3::new(-half_extents.x, half_extents.y, half_extents.z)),
                ];
                let bbox = match BoundsVolume::from_points_cloud(points.into_iter()) {
                    Some(bbox) => bbox,
                    None => return None,
                };
                let diff = position - bbox.origin;
                let half_extents = bbox.half_extents();
                let x = half_extents.x - diff.x.abs();
                let y = half_extents.y - diff.y.abs();
                let z = half_extents.z - diff.z.abs();
                if x >= 0.0 && y >= 0.0 && z >= 0.0 {
                    Some(x.max(y).max(z))
                } else {
                    None
                }
            }
        }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn world_space_overlaps(
        volume_a: &Self,
        matrix_a: &Mat4,
        volume_b: &Self,
        matrix_b: &Mat4,
    ) -> Option<Scalar> {
        match (volume_a, volume_b) {
            (Self::Sphere(a), Self::Sphere(b)) => {
                let origin_a = matrix_a.mul_point(Vec3::zero());
                let origin_b = matrix_b.mul_point(Vec3::zero());
                let distance = origin_a.distance(origin_b);
                let radius = a + b;
                if distance > radius {
                    None
                } else {
                    Some(radius - distance)
                }
            }
            (Self::Box(a), Self::Sphere(b)) => {
                let points = [
                    matrix_a.mul_point(Vec3::new(-a.x, -a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, -a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(-a.x, a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(-a.x, -a.y, a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, -a.y, a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, a.y, a.z)),
                    matrix_a.mul_point(Vec3::new(-a.x, a.y, a.z)),
                ];
                let bbox = match BoundsVolume::from_points_cloud(points.into_iter()) {
                    Some(bbox) => bbox,
                    None => return None,
                };
                let origin = matrix_b.mul_point(Vec3::zero());
                let point = bbox.closest_point_with_box(origin);
                let distance = origin.distance(point);
                if distance > *b {
                    None
                } else {
                    Some(b - distance)
                }
            }
            (Self::Box(a), Self::Box(b)) => {
                let points_a = [
                    matrix_a.mul_point(Vec3::new(-a.x, -a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, -a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(-a.x, a.y, -a.z)),
                    matrix_a.mul_point(Vec3::new(-a.x, -a.y, a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, -a.y, a.z)),
                    matrix_a.mul_point(Vec3::new(a.x, a.y, a.z)),
                    matrix_a.mul_point(Vec3::new(-a.x, a.y, a.z)),
                ];
                let bbox_a = match BoundsVolume::from_points_cloud(points_a.into_iter()) {
                    Some(bbox) => bbox,
                    None => return None,
                };
                let points_b = [
                    matrix_b.mul_point(Vec3::new(-b.x, -b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, -b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(-b.x, b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(-b.x, -b.y, b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, -b.y, b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, b.y, b.z)),
                    matrix_b.mul_point(Vec3::new(-b.x, b.y, b.z)),
                ];
                let bbox_b = match BoundsVolume::from_points_cloud(points_b.into_iter()) {
                    Some(bbox) => bbox,
                    None => return None,
                };
                let diff = bbox_a.origin - bbox_b.origin;
                let extents = bbox_a.half_extents() + bbox_b.half_extents();
                let x = extents.x - diff.x.abs();
                let y = extents.y - diff.y.abs();
                let z = extents.z - diff.z.abs();
                if x >= 0.0 && y >= 0.0 && z >= 0.0 {
                    Some(x.max(y).max(z))
                } else {
                    None
                }
            }
            (Self::Sphere(a), Self::Box(b)) => {
                let points = [
                    matrix_b.mul_point(Vec3::new(-b.x, -b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, -b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(-b.x, b.y, -b.z)),
                    matrix_b.mul_point(Vec3::new(-b.x, -b.y, b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, -b.y, b.z)),
                    matrix_b.mul_point(Vec3::new(b.x, b.y, b.z)),
                    matrix_b.mul_point(Vec3::new(-b.x, b.y, b.z)),
                ];
                let bbox = match BoundsVolume::from_points_cloud(points.into_iter()) {
                    Some(bbox) => bbox,
                    None => return None,
                };
                let origin = matrix_a.mul_point(Vec3::zero());
                let point = bbox.closest_point_with_box(origin);
                let distance = origin.distance(point);
                if distance > *a {
                    None
                } else {
                    Some(a - distance)
                }
            }
        }
    }
}

impl Prefab for HaVolume {}
impl PrefabComponent for HaVolume {}
