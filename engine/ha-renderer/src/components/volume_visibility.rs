use core::{
    prefab::{Prefab, PrefabComponent},
    utils::TagFilters,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaVolumeVisibilityMode {
    Sphere,
    Box,
}

impl Default for HaVolumeVisibilityMode {
    fn default() -> Self {
        Self::Sphere
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaVolumeVisibility {
    #[serde(default)]
    pub filters: TagFilters,
    #[serde(default)]
    pub mode: HaVolumeVisibilityMode,
}

impl Prefab for HaVolumeVisibility {}
impl PrefabComponent for HaVolumeVisibility {}
