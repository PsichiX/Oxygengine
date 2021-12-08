use crate::math::*;
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaTransform {
    #[serde(default = "HaTransform::default_translation")]
    translation: Vec3,
    #[serde(default = "HaTransform::default_rotation")]
    rotation: Quat,
    #[serde(default = "HaTransform::default_scale")]
    scale: Vec3,
    #[serde(skip)]
    #[ignite(ignore)]
    cached_local_matrix: Mat4,
    #[serde(skip)]
    #[ignite(ignore)]
    cached_world_matrix: Mat4,
}

impl Default for HaTransform {
    fn default() -> Self {
        Self {
            translation: Self::default_translation(),
            rotation: Self::default_rotation(),
            scale: Self::default_scale(),
            cached_local_matrix: Mat4::identity(),
            cached_world_matrix: Mat4::identity(),
        }
    }
}

impl HaTransform {
    fn default_translation() -> Vec3 {
        Vec3::zero()
    }

    fn default_rotation() -> Quat {
        Quat::identity()
    }

    fn default_scale() -> Vec3 {
        Vec3::one()
    }

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let mut result = Self {
            translation,
            rotation,
            scale,
            cached_local_matrix: Mat4::identity(),
            cached_world_matrix: Mat4::identity(),
        };
        result.rebuild_local_matrix();
        result
    }

    pub fn translation(v: Vec3) -> Self {
        Self::default().with_translation(v)
    }

    pub fn rotation(v: Quat) -> Self {
        Self::default().with_rotation(v)
    }

    pub fn rotation_euler(v: Eulers) -> Self {
        Self::default().with_rotation_euler(v)
    }

    pub fn scale(v: Vec3) -> Self {
        Self::default().with_scale(v)
    }

    pub fn get_translation(&self) -> Vec3 {
        self.translation
    }

    pub fn get_rotation(&self) -> Quat {
        self.rotation
    }

    pub fn get_rotation_euler(&self) -> Eulers {
        Eulers::from(self.rotation)
    }

    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_translation(&mut self, v: Vec3) {
        self.translation = v;
        self.rebuild_local_matrix();
    }

    pub fn set_rotation(&mut self, v: Quat) {
        self.rotation = v;
        self.rebuild_local_matrix();
    }

    pub fn set_rotation_euler(&mut self, v: Eulers) {
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

    pub fn with_rotation(mut self, v: Quat) -> Self {
        self.set_rotation(v);
        self
    }

    pub fn with_rotation_euler(mut self, v: Eulers) -> Self {
        self.set_rotation_euler(v);
        self
    }

    pub fn with_scale(mut self, v: Vec3) -> Self {
        self.set_scale(v);
        self
    }

    pub fn apply(&mut self, translation: Vec3, rotation: Quat, scale: Vec3) {
        self.translation = translation;
        self.rotation = rotation;
        self.scale = scale;
        self.rebuild_local_matrix();
    }

    pub fn local_matrix(&self) -> Mat4 {
        self.cached_local_matrix
    }

    pub fn world_matrix(&self) -> Mat4 {
        self.cached_world_matrix
    }

    pub fn get_world_translation(&self) -> Vec3 {
        self.cached_world_matrix.mul_point(Default::default())
    }

    fn rebuild_local_matrix(&mut self) {
        self.cached_local_matrix = Mat4::from(Transform {
            position: self.translation,
            orientation: self.rotation,
            scale: self.scale,
        });
    }

    pub(crate) fn rebuild_world_matrix(&mut self, parent: Mat4) {
        self.cached_world_matrix = parent * self.cached_local_matrix;
    }
}

impl Prefab for HaTransform {
    fn post_from_prefab(&mut self) {
        self.rebuild_local_matrix();
    }
}

impl PrefabComponent for HaTransform {}
