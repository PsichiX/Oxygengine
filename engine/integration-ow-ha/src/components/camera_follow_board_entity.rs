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
    #[serde(default)]
    pub strength_factor: Option<Scalar>,
    #[serde(default)]
    pub constraints: HaCameraFollowConstraints,
    #[serde(default)]
    pub nth: usize,
}

impl Default for HaCameraFollowBoardEntity {
    fn default() -> Self {
        Self {
            name: None,
            strength_factor: None,
            constraints: Default::default(),
            nth: 0,
        }
    }
}

impl Prefab for HaCameraFollowBoardEntity {}

impl PrefabComponent for HaCameraFollowBoardEntity {}
