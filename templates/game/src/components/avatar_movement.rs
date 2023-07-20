use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AvatarMovement {
    #[serde(default = "AvatarMovement::default_step_duration")]
    pub step_duration: Scalar,
}

impl Default for AvatarMovement {
    fn default() -> Self {
        Self {
            step_duration: Self::default_step_duration(),
        }
    }
}

impl AvatarMovement {
    fn default_step_duration() -> Scalar {
        1.0
    }
}

impl Prefab for AvatarMovement {}
impl PrefabComponent for AvatarMovement {}
