use core::{
    prefab::{Prefab, PrefabComponent},
    utils::TagFilters,
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaVolumeOverlap {
    #[serde(default)]
    pub filters: TagFilters,
    #[serde(default)]
    pub delay: Scalar,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) time: Scalar,
}

impl Prefab for HaVolumeOverlap {}
impl PrefabComponent for HaVolumeOverlap {}
