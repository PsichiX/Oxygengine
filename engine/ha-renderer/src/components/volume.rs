use crate::math::Vec3;
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum HaVolume {
    /// (radius)
    Sphere(Scalar),
    /// (half extents)
    Box(Vec3),
}

impl Prefab for HaVolume {}
impl PrefabComponent for HaVolume {}
