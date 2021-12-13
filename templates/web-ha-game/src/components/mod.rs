pub mod avatar_combat;
pub mod avatar_movement;
pub mod health;
pub mod weapon;

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

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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