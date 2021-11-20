use crate::{math::Vec3, TagFilters};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct HaVisibility(pub bool);

impl Default for HaVisibility {
    fn default() -> Self {
        Self(true)
    }
}

impl Prefab for HaVisibility {}
impl PrefabComponent for HaVisibility {}

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum HaVolume {
    /// (radius)
    Sphere(Scalar),
    /// (half extents)
    Box(Vec3),
}

impl Prefab for HaVolume {}
impl PrefabComponent for HaVolume {}

/// (tag name, overlap test as box)
#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaVolumeVisibility(pub TagFilters, pub bool);

impl Prefab for HaVolumeVisibility {}
impl PrefabComponent for HaVolumeVisibility {}
