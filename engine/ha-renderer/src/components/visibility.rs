use core::prefab::{Prefab, PrefabComponent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct HaVisibility(pub bool);

impl Default for HaVisibility {
    fn default() -> Self {
        Self(true)
    }
}

impl Prefab for HaVisibility {}
impl PrefabComponent for HaVisibility {}
