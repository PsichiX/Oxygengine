use oxygengine_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaCameraFollowConstraints {
    None,
    Chunk,
    Region,
}

impl Default for HaCameraFollowConstraints {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct HaCameraFollowBoardEntity {
    pub name: Option<String>,
    #[serde(default = "HaCameraFollowBoardEntity::default_strength_factor")]
    pub strength_factor: Scalar,
    #[serde(default)]
    pub constraints: HaCameraFollowConstraints,
}

impl Default for HaCameraFollowBoardEntity {
    fn default() -> Self {
        Self {
            name: None,
            strength_factor: Self::default_strength_factor(),
            constraints: Default::default(),
        }
    }
}

impl HaCameraFollowBoardEntity {
    fn default_strength_factor() -> Scalar {
        1.0
    }
}

impl Prefab for HaCameraFollowBoardEntity {}

impl PrefabComponent for HaCameraFollowBoardEntity {}
