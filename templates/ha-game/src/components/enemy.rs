use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enemy {
    Regular,
    Boss,
}

impl Default for Enemy {
    fn default() -> Self {
        Self::Regular
    }
}

impl Prefab for Enemy {}
impl PrefabComponent for Enemy {}
