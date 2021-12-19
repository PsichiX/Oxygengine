use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub level: usize,
}

impl Player {
    pub fn health_capacity(&self) -> usize {
        1 << self.level
    }

    pub fn weapons_capacity(&self) -> usize {
        1 << self.level
    }
}

impl Prefab for Player {}
impl PrefabComponent for Player {}
