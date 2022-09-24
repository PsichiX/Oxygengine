use crate::math::*;
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaTransformDef {
    #[serde(default)]
    pub translation: Vec3,
    #[serde(default)]
    pub rotation: Rotator,
    #[serde(default = "HaTransform::default_scale")]
    pub scale: Vec3,
}

impl From<HaTransform> for HaTransformDef {
    fn from(v: HaTransform) -> Self {
        Self {
            translation: v.translation,
            rotation: v.rotation,
            scale: v.scale,
        }
    }
}

impl From<HaTransformDef> for HaTransform {
    fn from(v: HaTransformDef) -> Self {
        Self::new(v.translation, v.rotation, v.scale)
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "HaTransformDef")]
#[serde(into = "HaTransformDef")]
pub struct HaTransform {
    translation: Vec3,
    rotation: Rotator,
    scale: Vec3,
    #[ignite(ignore)]
    cached_local_matrix: Mat4,
    #[ignite(ignore)]
    cached_world_matrix: Mat4,
    #[ignite(ignore)]
    cached_inverse_local_matrix: Mat4,
    #[ignite(ignore)]
    cached_inverse_world_matrix: Mat4,
}

impl Default for HaTransform {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Self::default_scale(),
            cached_local_matrix: Mat4::identity(),
            cached_world_matrix: Mat4::identity(),
            cached_inverse_local_matrix: Mat4::identity(),
            cached_inverse_world_matrix: Mat4::identity(),
        }
    }
}

impl HaTransform {
    fn default_scale() -> Vec3 {
        Vec3::one()
    }

    pub fn new(translation: Vec3, rotation: impl Into<Rotator>, scale: Vec3) -> Self {
        let mut result = Self {
            translation,
            rotation: rotation.into(),
            scale,
            cached_local_matrix: Mat4::identity(),
            cached_world_matrix: Mat4::identity(),
            cached_inverse_local_matrix: Mat4::identity(),
            cached_inverse_world_matrix: Mat4::identity(),
        };
        result.rebuild_local_matrix();
        result
    }

    pub fn translation(v: Vec3) -> Self {
        Self::default().with_translation(v)
    }

    pub fn rotation(v: impl Into<Rotator>) -> Self {
        Self::default().with_rotation(v.into())
    }

    pub fn scale(v: Vec3) -> Self {
        Self::default().with_scale(v)
    }

    pub fn get_translation(&self) -> Vec3 {
        self.translation
    }

    pub fn get_rotation(&self) -> Rotator {
        self.rotation
    }

    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_translation(&mut self, v: Vec3) {
        self.translation = v;
        self.rebuild_local_matrix();
    }

    pub fn set_rotation(&mut self, v: impl Into<Rotator>) {
        self.rotation = v.into();
        self.rebuild_local_matrix();
    }

    pub fn set_scale(&mut self, v: Vec3) {
        self.scale = v;
        self.rebuild_local_matrix();
    }

    pub fn with_translation(mut self, v: Vec3) -> Self {
        self.set_translation(v);
        self
    }

    pub fn with_rotation(mut self, v: impl Into<Rotator>) -> Self {
        self.set_rotation(v.into());
        self
    }

    pub fn with_scale(mut self, v: Vec3) -> Self {
        self.set_scale(v);
        self
    }

    pub fn change_translation<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3),
    {
        f(&mut self.translation);
        self.rebuild_local_matrix();
    }

    pub fn change_rotation<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Rotator),
    {
        f(&mut self.rotation);
        self.rebuild_local_matrix();
    }

    pub fn change_scale<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3),
    {
        f(&mut self.scale);
        self.rebuild_local_matrix();
    }

    pub fn transform_local_point(&self, v: Vec3) -> Vec3 {
        self.cached_local_matrix.mul_point(v)
    }

    pub fn transform_world_point(&self, v: Vec3) -> Vec3 {
        self.cached_world_matrix.mul_point(v)
    }

    pub fn transform_local_direction(&self, v: Vec3) -> Vec3 {
        self.cached_local_matrix.mul_direction(v)
    }

    pub fn transform_world_direction(&self, v: Vec3) -> Vec3 {
        self.cached_world_matrix.mul_direction(v)
    }

    pub fn inverse_transform_local_point(&self, v: Vec3) -> Vec3 {
        self.cached_inverse_local_matrix.mul_point(v)
    }

    pub fn inverse_transform_world_point(&self, v: Vec3) -> Vec3 {
        self.cached_inverse_world_matrix.mul_point(v)
    }

    pub fn inverse_transform_local_direction(&self, v: Vec3) -> Vec3 {
        self.cached_inverse_local_matrix.mul_direction(v)
    }

    pub fn inverse_transform_world_direction(&self, v: Vec3) -> Vec3 {
        self.cached_inverse_world_matrix.mul_direction(v)
    }

    pub fn get_world_origin(&self) -> Vec3 {
        self.transform_world_point(Default::default())
    }

    pub fn get_world_forward(&self) -> Vec3 {
        self.transform_world_direction(Vec3::unit_x())
    }

    pub fn get_world_right(&self) -> Vec3 {
        self.transform_world_direction(Vec3::unit_y())
    }

    pub fn get_world_up(&self) -> Vec3 {
        self.transform_world_direction(Vec3::unit_z())
    }

    pub fn get_world_scale_lossy(&self) -> Vec3 {
        let matrix = &self.cached_world_matrix;
        let x = (matrix.cols.x.x * matrix.cols.x.x
            + matrix.cols.y.x * matrix.cols.y.x
            + matrix.cols.z.x * matrix.cols.z.x)
            .sqrt();
        let y = (matrix.cols.x.y * matrix.cols.x.y
            + matrix.cols.y.y * matrix.cols.y.y
            + matrix.cols.z.y * matrix.cols.z.y)
            .sqrt();
        let z = (matrix.cols.x.z * matrix.cols.x.z
            + matrix.cols.y.z * matrix.cols.y.z
            + matrix.cols.z.z * matrix.cols.z.z)
            .sqrt();
        Vec3::new(x, y, z)
    }

    pub fn interpolate(from: &Self, to: &Self, factor: Scalar) -> Self {
        Self::new(
            Vec3::lerp(from.translation, to.translation, factor),
            Rotator::interpolate(&from.rotation, &to.rotation, factor),
            Vec3::lerp(from.scale, to.scale, factor),
        )
    }

    pub fn interpolate_many(iter: impl Iterator<Item = (Self, Scalar)>) -> Option<Self> {
        let mut result = None;
        for (transform, weight) in iter {
            let translation = transform.get_translation() * weight;
            let rotation = Quat::slerp(Quat::identity(), transform.get_rotation().quat(), weight);
            let scale = transform.get_scale() * weight;
            result = match result {
                Some((t, r, s)) => Some((t + translation, r * rotation, s + scale)),
                None => Some((translation, rotation, scale)),
            }
        }
        result.map(|(t, r, s)| Self::new(t, r, s))
    }

    /// This conversion assumes non-negative scale components - when either of scale components was negative, rotation will be affected.
    pub fn from_matrix(matrix: Mat4) -> Self {
        let translation = Vec3::from(matrix.cols.w);
        let matrix = Mat3::from(matrix);
        let sx = (matrix.cols.x.x * matrix.cols.x.x
            + matrix.cols.y.x * matrix.cols.y.x
            + matrix.cols.z.x * matrix.cols.z.x)
            .sqrt();
        let sy = (matrix.cols.x.y * matrix.cols.x.y
            + matrix.cols.y.y * matrix.cols.y.y
            + matrix.cols.z.y * matrix.cols.z.y)
            .sqrt();
        let sz = (matrix.cols.x.z * matrix.cols.x.z
            + matrix.cols.y.z * matrix.cols.y.z
            + matrix.cols.z.z * matrix.cols.z.z)
            .sqrt();
        let scale = Vec3::new(sx, sy, sz);
        let scale_inv = Mat3::scaling_3d(Vec3::one() / scale);
        let matrix = scale_inv * matrix;
        let rotation = Rotator::from(matrix);
        Self::new(translation, rotation, scale)
    }

    pub fn apply(&mut self, translation: Vec3, rotation: impl Into<Rotator>, scale: Vec3) {
        self.translation = translation;
        self.rotation = rotation.into();
        self.scale = scale;
        self.rebuild_local_matrix();
    }

    pub fn local_matrix(&self) -> Mat4 {
        self.cached_local_matrix
    }

    pub fn inverse_local_matrix(&self) -> Mat4 {
        self.cached_inverse_local_matrix
    }

    pub fn world_matrix(&self) -> Mat4 {
        self.cached_world_matrix
    }

    pub fn inverse_world_matrix(&self) -> Mat4 {
        self.cached_inverse_world_matrix
    }

    fn rebuild_local_matrix(&mut self) {
        self.cached_local_matrix = Mat4::from(Transform {
            position: self.translation,
            orientation: self.rotation.into(),
            scale: self.scale,
        });
        self.cached_inverse_local_matrix = self.cached_local_matrix.inverted();
    }

    pub(crate) fn rebuild_world_matrix(&mut self, parent: Option<&HaTransform>) {
        if let Some(parent) = parent {
            self.cached_world_matrix = parent.world_matrix() * self.cached_local_matrix;
        } else {
            self.cached_world_matrix = self.cached_local_matrix;
        }
        self.cached_inverse_world_matrix = self.cached_world_matrix.inverted();
    }
}

impl Prefab for HaTransform {}
impl PrefabComponent for HaTransform {}
